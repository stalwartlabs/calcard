use ahash::{AHashMap, AHashSet};
use chrono::{DateTime, TimeDelta, TimeZone, Timelike};

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
    MissingDtStart,
    InvalidDtStart,
    InvalidDtEnd,
    InvalidDuration,
    RRule(RRuleError),
}

pub struct CalendarExpandOpts<'x> {
    tz_resolver: TzResolver<'x>,
    limit: usize,
    overridden: AHashMap<(i64, DateTime<Tz>), bool>,
}

impl ICalendar {
    pub fn expand_dates(
        &self,
        default_tz: impl Into<Tz>,
        limit: usize,
    ) -> impl Iterator<Item = (u16, Result<CalendarExpand, CalendarExpandError>)> + '_ {
        let mut opts = self.build_calendar_expand_opts(default_tz, limit);
        self.components
            .iter()
            .enumerate()
            .filter_map(move |(idx, comp)| {
                if matches!(
                    comp.component_type,
                    ICalendarComponentType::VEvent
                        | ICalendarComponentType::VTodo
                        | ICalendarComponentType::VJournal
                        | ICalendarComponentType::VFreebusy
                ) {
                    Some((idx as u16, comp.expand_dates(&mut opts)))
                } else {
                    None
                }
            })
    }

    pub fn build_calendar_expand_opts(
        &self,
        default_tz: impl Into<Tz>,
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
                let mut sequence = i64::MAX;
                let mut this_and_future = false;
                let mut recur = None;

                for entry in &component.entries {
                    match (&entry.name, entry.values.first()) {
                        (
                            ICalendarProperty::RecurrenceId,
                            Some(ICalendarValue::PartialDateTime(dt)),
                        ) => {
                            let mut tz_id = None;

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

                            recur = dt.to_date_time_with_tz(tz_resolver.resolve(tz_id));
                        }
                        (ICalendarProperty::Sequence, Some(ICalendarValue::Integer(seq))) => {
                            sequence = *seq;
                        }

                        _ => (),
                    }
                }

                if let Some(recur) = recur {
                    overridden.insert((sequence, recur), this_and_future);
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
        opts: &mut CalendarExpandOpts,
    ) -> Result<CalendarExpand, CalendarExpandError> {
        let mut dt_start = None;
        let mut dt_start_tzid = None;
        let mut dt_start_has_time = false;
        let mut dt_end: Option<DateTimeResult> = None;
        let mut dt_end_tzid = None;
        let mut due: Option<DateTimeResult> = None;
        let mut due_tzid = None;
        let mut rid: Option<DateTimeResult> = None;
        let mut rid_tzid = None;
        let mut duration = None;
        let mut rrule = None;
        let mut rrule_seq = i64::MAX;
        let mut rdates = vec![];
        let mut rdates_periods = vec![];
        let mut exdates = vec![];
        let mut events = vec![];

        for entry in &self.entries {
            match (&entry.name, entry.values.first()) {
                (ICalendarProperty::Dtstart, Some(ICalendarValue::PartialDateTime(dt))) => {
                    dt_start = dt.to_date_time();
                    dt_start_tzid = entry.tz_id();
                    dt_start_has_time = dt.has_time();
                }
                (ICalendarProperty::Dtend, Some(ICalendarValue::PartialDateTime(dt))) => {
                    if let Some(dt) = dt.to_date_time() {
                        dt_end = Some(dt);
                        dt_end_tzid = entry.tz_id();
                    }
                }
                (ICalendarProperty::Due, Some(ICalendarValue::PartialDateTime(dt))) => {
                    if let Some(dt) = dt.to_date_time() {
                        due = Some(dt);
                        due_tzid = entry.tz_id();
                    }
                }
                (ICalendarProperty::RecurrenceId, Some(ICalendarValue::PartialDateTime(dt))) => {
                    if let Some(dt) = dt.to_date_time() {
                        rid = Some(dt);
                        rid_tzid = entry.tz_id();
                    }
                }
                (ICalendarProperty::Duration, Some(ICalendarValue::Duration(dur))) => {
                    duration = Some(dur);
                }
                (ICalendarProperty::Rrule, Some(ICalendarValue::RecurrenceRule(rule))) => {
                    rrule = RRule::from_floating_ical(rule);
                }
                (ICalendarProperty::Sequence, Some(ICalendarValue::Integer(seq))) => {
                    rrule_seq = *seq;
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

        let dt_start = match dt_start {
            Some(dt_start) => dt_start,
            None => match rid {
                Some(rid) => {
                    dt_start_tzid = rid_tzid;
                    rid
                }
                None => match self.component_type {
                    ICalendarComponentType::VEvent => {
                        return Err(CalendarExpandError::MissingDtStart);
                    }
                    ICalendarComponentType::VTodo if due.is_some() => {
                        dt_start_has_time = true;
                        dt_start_tzid = due_tzid;
                        due.unwrap()
                    }
                    _ => {
                        return Ok(CalendarExpand {
                            default_duration: TimeDelta::zero(),
                            events: vec![],
                        });
                    }
                },
            },
        };
        let start_tz = opts.tz_resolver.resolve(dt_start_tzid);
        let dt_start_tz = dt_start
            .to_date_time_with_tz(start_tz)
            .ok_or(CalendarExpandError::InvalidDtStart)?;
        let default_duration = if let Some(dt_end) = dt_end {
            let end = dt_end
                .to_date_time_with_tz(opts.tz_resolver.resolve(dt_end_tzid.or(dt_start_tzid)))
                .ok_or(CalendarExpandError::InvalidDtEnd)?;

            if rrule.is_none() {
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
            if rrule.is_none() {
                events.push(CalendarEventRange {
                    start: dt_start_tz,
                    end: TimeOrDelta::Delta(duration),
                });
            }
            duration
        } else {
            /*
               For cases where a "VEVENT" calendar component
               specifies a "DTSTART" property with a DATE value type but no
               "DTEND" nor "DURATION" property, the event's duration is taken to
               be one day.  For cases where a "VEVENT" calendar component
               specifies a "DTSTART" property with a DATE-TIME value type but no
               "DTEND" property, the event ends on the same calendar date and
               time of day specified by the "DTSTART" property.
            */

            let duration = if dt_start_has_time {
                // If the start has time, we use the same time for the end
                dt_start
                    .date_time
                    .with_hour(23)
                    .and_then(|dt| dt.with_minute(59))
                    .and_then(|dt| dt.with_second(59))
                    .map(|dt| dt - dt_start.date_time)
                    .unwrap_or_else(|| TimeDelta::days(1))
            } else {
                TimeDelta::days(1)
            };
            if rrule.is_none() {
                events.push(CalendarEventRange {
                    start: dt_start_tz,
                    end: TimeOrDelta::Delta(duration),
                });
            }
            duration
        };

        if let Some(rrule) = rrule {
            let floating_start = Tz::Floating
                .from_local_datetime(&dt_start.date_time)
                .single()
                .ok_or(CalendarExpandError::InvalidDtStart)?;
            let rrule = rrule
                .validate(floating_start)
                .map_err(CalendarExpandError::RRule)?;
            let exdates = exdates
                .into_iter()
                .filter_map(|(tz_id, dt)| {
                    dt.to_date_time_with_tz(opts.tz_resolver.resolve(tz_id.or(dt_start_tzid)))
                })
                .collect::<AHashSet<_>>();

            for date in RRuleIter::new(&rrule, &floating_start, true) {
                if opts.limit != 0 {
                    opts.limit -= 1;
                } else {
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
                match opts.overridden.get(&(rrule_seq, date)) {
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

        events.sort_by(|a, b| a.start.cmp(&b.start));
        Ok(CalendarExpand {
            default_duration,
            events,
        })
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

impl CalendarEventRange {
    pub fn into_date_time(self, default_delta: TimeDelta) -> Option<(DateTime<Tz>, DateTime<Tz>)> {
        match self.end {
            TimeOrDelta::Time(time) => Some(time),
            TimeOrDelta::Delta(delta) => self
                .start
                .naive_local()
                .checked_add_signed(delta)
                .and_then(|end| end.and_local_timezone(self.start.timezone()).single()),
            TimeOrDelta::Default => self
                .start
                .naive_local()
                .checked_add_signed(default_delta)
                .and_then(|end| end.and_local_timezone(self.start.timezone()).single()),
        }
        .map(|end| (self.start, end))
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

    pub fn overridden(mut self, seq: i64, dt: DateTime<Tz>, this_and_future: bool) -> Self {
        self.overridden.insert((seq, dt), this_and_future);
        self
    }
}
