use ahash::{AHashMap, AHashSet};
use chrono::{DateTime, TimeDelta, TimeZone};

use crate::{
    common::{timezone::Tz, DateTimeResult},
    datecalc::{error::RRuleError, rrule::RRule, RRuleIter},
    icalendar::ICalendarParameter,
};

use super::{
    timezone::TzResolver, ICalendar, ICalendarComponent, ICalendarComponentType, ICalendarPeriod,
    ICalendarProperty, ICalendarValue,
};

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct CalendarExpand {
    pub default_duration: TimeDelta,
    pub events: Vec<CalendarEventRange>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct CalendarEventRange {
    pub start: DateTime<Tz>,
    pub end: TimeOrDelta<DateTime<Tz>, TimeDelta>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize)
)]
#[cfg_attr(feature = "serde", serde(tag = "type", content = "data"))]
pub enum TimeOrDelta<T, D> {
    Time(T),
    Delta(D),
    Default,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CalendarExpandError {
    InvalidDtStart,
    InvalidDtEnd,
    InvalidDuration,
    NoDatesFound,
    RRule(RRuleError),
}

pub struct CalendarExpandOpts<'x> {
    tz_resolver: TzResolver<'x>,
    limit: usize,
    overridden: AHashMap<DateTime<Tz>, bool>,
}

impl ICalendar {
    pub fn build_calendar_expand_opts(
        &self,
        default_tz: Tz,
        limit: usize,
    ) -> CalendarExpandOpts<'_> {
        let tz_resolver = self.build_tz_resolver().with_default(default_tz);

        let mut overridden = AHashMap::new();

        for component in &self.components {
            if matches!(
                component.component_type,
                ICalendarComponentType::VEvent
                    | ICalendarComponentType::VTodo
                    | ICalendarComponentType::VJournal
                    | ICalendarComponentType::VFreebusy
            ) {
                for entry in &component.entries {
                    if let (
                        ICalendarProperty::RecurrenceId,
                        Some(ICalendarValue::PartialDateTime(dt)),
                    ) = (&entry.name, entry.values.first())
                    {
                        let mut tz_id = None;
                        let mut this_and_future = false;

                        for param in &entry.params {
                            match param {
                                ICalendarParameter::Tzid(id) => {
                                    tz_id = Some(id.as_str());
                                }
                                ICalendarParameter::Range => {
                                    this_and_future = true;
                                }
                                _ => (),
                            }
                        }

                        if let Some(dt) = dt.to_date_time_with_tz(tz_resolver.resolve(tz_id)) {
                            overridden.insert(dt, this_and_future);
                        }
                    }
                }
            }
        }

        CalendarExpandOpts {
            tz_resolver,
            limit,
            overridden,
        }
    }
}

impl ICalendarComponent {
    pub fn expand_dates(
        &self,
        opts: &CalendarExpandOpts,
    ) -> Result<CalendarExpand, CalendarExpandError> {
        let mut dt_start = None;
        let mut dt_start_tzid = None;
        let mut dt_end: Option<DateTimeResult> = None;
        let mut dt_end_tzid = None;
        let mut duration = None;
        let mut rrule = None;
        let mut rdates = vec![];
        let mut rdates_periods = vec![];
        let mut exdates = vec![];
        let mut events = vec![];

        for entry in &self.entries {
            match (&entry.name, entry.values.first()) {
                (ICalendarProperty::Dtstart, Some(ICalendarValue::PartialDateTime(dt))) => {
                    dt_start = dt.to_date_time();
                    dt_start_tzid = entry.tz_id();
                }
                (
                    ICalendarProperty::Dtend | ICalendarProperty::Due,
                    Some(ICalendarValue::PartialDateTime(dt)),
                ) => {
                    if let Some(dt) = dt.to_date_time() {
                        if dt_end.as_ref().is_none_or(|v| dt.date_time > v.date_time) {
                            dt_end = Some(dt);
                            dt_end_tzid = entry.tz_id();
                        }
                    }
                }
                (ICalendarProperty::Duration, Some(ICalendarValue::Duration(dur))) => {
                    duration = Some(dur);
                }
                (ICalendarProperty::Rrule, Some(ICalendarValue::RecurrenceRule(rule))) => {
                    rrule = RRule::from_floating_ical(rule);
                }
                (ICalendarProperty::Rdate, _) => {
                    let tz_id = entry.tz_id();
                    for value in &entry.values {
                        match value {
                            ICalendarValue::PartialDateTime(dt) => {
                                if let Some(dt) = dt.to_date_time() {
                                    rdates.push((tz_id, dt));
                                }
                            }
                            ICalendarValue::Period(period) => match period {
                                ICalendarPeriod::Range { start, end } => {
                                    if let (Some(start), Some(end)) =
                                        (start.to_date_time(), end.to_date_time())
                                    {
                                        rdates_periods.push((tz_id, start, TimeOrDelta::Time(end)));
                                    }
                                }
                                ICalendarPeriod::Duration { start, duration } => {
                                    if let (Some(start), Some(duration)) =
                                        (start.to_date_time(), duration.to_time_delta())
                                    {
                                        rdates_periods.push((
                                            tz_id,
                                            start,
                                            TimeOrDelta::Delta(duration),
                                        ));
                                    }
                                }
                            },
                            _ => (),
                        }
                    }
                }
                (ICalendarProperty::Exdate, _) => {
                    let tz_id = entry.tz_id();
                    for value in &entry.values {
                        if let ICalendarValue::PartialDateTime(dt) = value {
                            if let Some(dt) = dt.to_date_time() {
                                exdates.push((tz_id, dt));
                            }
                        }
                    }
                }
                _ => (),
            }
        }

        let dt_start = dt_start.ok_or(CalendarExpandError::InvalidDtStart)?;
        let start_tz = opts.tz_resolver.resolve(dt_start_tzid);
        let dt_start_tz = dt_start
            .to_date_time_with_tz(start_tz)
            .ok_or(CalendarExpandError::InvalidDtStart)?;
        let exdates = exdates
            .into_iter()
            .filter_map(|(tz_id, dt)| {
                dt.to_date_time_with_tz(opts.tz_resolver.resolve(tz_id.or(dt_start_tzid)))
            })
            .collect::<AHashSet<_>>();
        let default_duration = if let Some(dt_end) = dt_end {
            let end = dt_end
                .to_date_time_with_tz(opts.tz_resolver.resolve(dt_end_tzid.or(dt_start_tzid)))
                .ok_or(CalendarExpandError::InvalidDtEnd)?;

            if !exdates.contains(&dt_start_tz) && !opts.overridden.contains_key(&dt_start_tz) {
                events.push(CalendarEventRange {
                    start: dt_start_tz,
                    end: TimeOrDelta::Time(end),
                });
            }
            dt_end.date_time - dt_start.date_time
        } else if let Some(duration) = duration {
            let duration = duration
                .to_time_delta()
                .ok_or(CalendarExpandError::InvalidDuration)?;
            if !exdates.contains(&dt_start_tz) && !opts.overridden.contains_key(&dt_start_tz) {
                events.push(CalendarEventRange {
                    start: dt_start_tz,
                    end: TimeOrDelta::Delta(duration),
                });
            }
            duration
        } else {
            return Err(CalendarExpandError::NoDatesFound);
        };

        if let Some(rrule) = rrule {
            let floating_start = Tz::Floating
                .from_local_datetime(&dt_start.date_time)
                .single()
                .ok_or(CalendarExpandError::InvalidDtStart)?;
            let rrule = rrule
                .validate(floating_start)
                .map_err(CalendarExpandError::RRule)?;

            for (idx, date) in RRuleIter::new(&rrule, &floating_start, true).enumerate() {
                if idx >= opts.limit {
                    break;
                }
                let date = if date.timezone().is_floating() {
                    start_tz
                        .from_local_datetime(&date.naive_local())
                        .single()
                        .unwrap_or(date)
                } else {
                    date
                };
                match opts.overridden.get(&date) {
                    Some(true) => {
                        // THISANDFUTURE is set
                        break;
                    }
                    None if !exdates.contains(&date) => {
                        events.push(CalendarEventRange {
                            start: date,
                            end: TimeOrDelta::Default,
                        });
                    }
                    _ => {}
                }
            }
        }

        // Add rdates
        for (tz_id, rdate) in rdates {
            if let Some(date_start) =
                rdate.to_date_time_with_tz(opts.tz_resolver.resolve(tz_id.or(dt_start_tzid)))
            {
                events.push(CalendarEventRange {
                    start: date_start,
                    end: TimeOrDelta::Default,
                });
            }
        }
        for (tz_id, start, end) in rdates_periods {
            let tz = opts.tz_resolver.resolve(tz_id.or(dt_start_tzid));
            if let (Some(date_start), Some(date_end)) = (
                start.to_date_time_with_tz(tz),
                end.into_date_time_with_tz(tz),
            ) {
                events.push(CalendarEventRange {
                    start: date_start,
                    end: date_end,
                });
            }
        }

        if !events.is_empty() {
            events.sort_by(|a, b| a.start.cmp(&b.start));
            Ok(CalendarExpand {
                default_duration,
                events,
            })
        } else {
            Err(CalendarExpandError::NoDatesFound)
        }
    }
}

impl TimeOrDelta<DateTimeResult, TimeDelta> {
    fn into_date_time_with_tz(self, tz: Tz) -> Option<TimeOrDelta<DateTime<Tz>, TimeDelta>> {
        match self {
            TimeOrDelta::Time(time) => time.to_date_time_with_tz(tz).map(TimeOrDelta::Time),
            TimeOrDelta::Delta(delta) => Some(TimeOrDelta::Delta(delta)),
            TimeOrDelta::Default => Some(TimeOrDelta::Default),
        }
    }
}

impl<'x> CalendarExpandOpts<'x> {
    pub fn new(tz_resolver: TzResolver<'x>) -> Self {
        Self {
            tz_resolver,
            limit: 5000,
            overridden: AHashMap::new(),
        }
    }

    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = limit;
        self
    }

    pub fn overridden(mut self, dt: DateTime<Tz>, this_and_future: bool) -> Self {
        self.overridden.insert(dt, this_and_future);
        self
    }
}
