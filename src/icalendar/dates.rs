/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

use super::{
    ICalendar, ICalendarComponent, ICalendarComponentType, ICalendarPeriod, ICalendarProperty,
    ICalendarValue, timezone::TzResolver,
};
use crate::{
    common::{DateTimeResult, timezone::Tz},
    datecalc::{RRuleIter, error::RRuleError, rrule::RRule},
    icalendar::ICalendarParameterName,
};
use ahash::{AHashMap, AHashSet};
use chrono::{DateTime, TimeDelta, TimeZone, Timelike};
use std::fmt::{Display, Formatter};

#[allow(clippy::type_complexity)]
#[derive(Debug, Default, Clone, PartialEq, Eq)]
#[cfg_attr(any(test, feature = "serde"), derive(serde::Serialize))]
pub struct CalendarExpand {
    pub events: Vec<CalendarEvent<DateTime<Tz>, TimeOrDelta<DateTime<Tz>, TimeDelta>>>,
    pub errors: Vec<CalendarError>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(any(test, feature = "serde"), derive(serde::Serialize))]
pub struct CalendarEvent<S, E> {
    pub comp_id: u32,
    pub start: S,
    pub end: E,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    any(test, feature = "serde"),
    derive(serde::Serialize, serde::Deserialize)
)]
#[cfg_attr(any(test, feature = "serde"), serde(tag = "type", content = "data"))]
pub enum TimeOrDelta<T, D> {
    Time(T),
    Delta(D),
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(any(test, feature = "serde"), derive(serde::Serialize))]
pub struct CalendarError {
    pub comp_id: u32,
    pub error: CalendarErrorType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(any(test, feature = "serde"), derive(serde::Serialize))]
pub enum CalendarErrorType {
    MissingDtStart,
    InvalidDtStart,
    InvalidDtEnd,
    InvalidDuration,
    RRule(RRuleError),
}

impl ICalendar {
    pub fn expand_dates(&self, default_tz: impl Into<Tz>, mut limit: usize) -> CalendarExpand {
        let tz_resolver = self.build_tz_resolver().with_default(default_tz);
        let mut expand = CalendarExpand::default();
        let mut rrules = Vec::new();
        let mut overridden = AHashMap::new();

        for (comp_id, comp) in self.components.iter().enumerate() {
            if comp.component_type.has_time_ranges() {
                match comp.build_calendar_date(comp_id as u32, &tz_resolver, &mut expand.events) {
                    Ok(Some(event)) => {
                        if event.rrule.is_some() {
                            rrules.push((comp_id as u32, event));
                        } else if let Some(rid) = event.rid {
                            overridden.insert((event.rrule_seq, rid), (comp_id as u32, event));
                        } else if let Some(cal_event) = event.event {
                            expand.events.push(cal_event);
                        }
                    }
                    Ok(None) => {}
                    Err(error) => {
                        expand.errors.push(CalendarError {
                            comp_id: comp_id as u32,
                            error,
                        });
                    }
                }
            }
        }

        // Expand recurrences
        for (mut comp_id, event) in rrules {
            let rrule = event.rrule.unwrap();
            let floating_start = if let Some(floating_start) = Tz::Floating
                .from_local_datetime(&event.dt_start.date_time)
                .single()
            {
                floating_start
            } else {
                expand.errors.push(CalendarError {
                    comp_id,
                    error: CalendarErrorType::InvalidDtStart,
                });
                continue;
            };
            let rrule = match rrule.validate(floating_start) {
                Ok(rrule) => rrule,
                Err(err) => {
                    expand.errors.push(CalendarError {
                        comp_id,
                        error: CalendarErrorType::RRule(err),
                    });
                    continue;
                }
            };
            let exdates = event
                .exdates
                .into_iter()
                .filter_map(|(tz_id, dt)| {
                    dt.to_date_time_with_tz(
                        tz_resolver.resolve_or_default(tz_id.or(event.dt_start_tzid)),
                    )
                })
                .collect::<AHashSet<_>>();
            let mut override_offset = None;

            for date in RRuleIter::new(&rrule, &floating_start, true) {
                if limit != 0 {
                    limit -= 1;
                } else {
                    break;
                }
                let mut date = if date.timezone().is_floating() {
                    event
                        .start_tz
                        .from_local_datetime(&date.naive_local())
                        .single()
                        .unwrap_or(date)
                } else {
                    date
                };
                if let Some(override_offset) = override_offset {
                    date += override_offset;
                }
                match overridden.remove(&(event.rrule_seq, date)) {
                    Some((new_comp_id, overridden_event)) => {
                        if let Some(new_event) = overridden_event.event {
                            if overridden_event.rid_this_and_future {
                                comp_id = new_comp_id;
                                override_offset = Some(new_event.start - date);
                            }
                            expand.events.push(new_event);
                        }
                    }
                    None if !exdates.contains(&date) => {
                        expand.events.push(CalendarEvent {
                            start: date,
                            end: TimeOrDelta::Delta(event.default_duration),
                            comp_id,
                        });
                    }
                    _ => {}
                }
            }
        }

        // Add missing overridden events (this should not occur unless the iCalendar is malformed)
        for (_, event) in overridden.into_values() {
            if let Some(cal_event) = event.event {
                expand.events.push(cal_event);
            }
        }

        expand
    }
}

#[allow(clippy::type_complexity)]
struct CalendarEventBuilder<'x> {
    event: Option<CalendarEvent<DateTime<Tz>, TimeOrDelta<DateTime<Tz>, TimeDelta>>>,
    dt_start: DateTimeResult,
    dt_start_tzid: Option<&'x str>,
    start_tz: Tz,
    default_duration: TimeDelta,
    rrule: Option<RRule>,
    rrule_seq: i64,
    exdates: Vec<(Option<&'x str>, DateTimeResult)>,
    rid: Option<DateTime<Tz>>,
    rid_this_and_future: bool,
}

impl ICalendarComponent {
    #[allow(clippy::type_complexity)]
    fn build_calendar_date(
        &self,
        comp_id: u32,
        tz_resolver: &TzResolver<&'_ str>,
        events: &mut Vec<CalendarEvent<DateTime<Tz>, TimeOrDelta<DateTime<Tz>, TimeDelta>>>,
    ) -> Result<Option<CalendarEventBuilder<'_>>, CalendarErrorType> {
        let mut dt_start = None;
        let mut dt_start_tzid = None;
        let mut dt_start_has_time = false;
        let mut dt_end: Option<DateTimeResult> = None;
        let mut dt_end_tzid = None;
        let mut todo_dates = vec![];
        let mut rid: Option<DateTimeResult> = None;
        let mut rid_tzid = None;
        let mut rid_this_and_future = false;
        let mut duration = None;
        let mut rrule = None;
        let mut rrule_seq = i64::MAX;
        let mut rdates = vec![];
        let mut rdates_periods = vec![];
        let mut exdates = vec![];

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
                (
                    ICalendarProperty::Due
                    | ICalendarProperty::Completed
                    | ICalendarProperty::Created,
                    Some(ICalendarValue::PartialDateTime(dt)),
                ) if self.component_type == ICalendarComponentType::VTodo => {
                    todo_dates.push((&entry.name, dt.to_date_time(), entry.tz_id()));
                }
                (ICalendarProperty::RecurrenceId, Some(ICalendarValue::PartialDateTime(dt))) => {
                    if let Some(dt) = dt.to_date_time() {
                        for param in &entry.params {
                            match &param.name {
                                ICalendarParameterName::Tzid => {
                                    rid_tzid = param.value.as_text();
                                }
                                ICalendarParameterName::Range => {
                                    rid_this_and_future = true;
                                }
                                _ => (),
                            }
                        }

                        rid = Some(dt);
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
                        if let ICalendarValue::PartialDateTime(dt) = value
                            && let Some(dt) = dt.to_date_time()
                        {
                            exdates.push((tz_id, dt));
                        }
                    }
                }
                _ => (),
            }
        }

        let dt_start = match dt_start {
            Some(dt_start) => dt_start,
            None => match &rid {
                Some(rid) => {
                    dt_start_tzid = rid_tzid;
                    rid.clone()
                }
                None => match self.component_type {
                    ICalendarComponentType::VEvent => {
                        return Err(CalendarErrorType::MissingDtStart);
                    }
                    ICalendarComponentType::VTodo => {
                        let mut due_idx = None;
                        let mut completed_idx = None;
                        let mut created_idx = None;

                        for (idx, (prop, dt, _)) in todo_dates.iter().enumerate() {
                            if dt.is_some() {
                                match prop {
                                    ICalendarProperty::Due => due_idx = Some(idx),
                                    ICalendarProperty::Completed => completed_idx = Some(idx),
                                    ICalendarProperty::Created => created_idx = Some(idx),
                                    _ => (),
                                }
                            }
                        }

                        match (due_idx, completed_idx, created_idx) {
                            (Some(due_idx), _, _) => {
                                let due = &mut todo_dates[due_idx];
                                dt_start_tzid = due.2;
                                dt_start_has_time = true;
                                due.1.take().unwrap()
                            }
                            (_, Some(completed_idx), Some(created_idx)) => {
                                let completed = &mut todo_dates[completed_idx];
                                dt_end = completed.1.take();
                                dt_end_tzid = completed.2;

                                let created = &mut todo_dates[created_idx];
                                dt_start_tzid = created.2;
                                created.1.take().unwrap()
                            }
                            (_, Some(date_idx), _) | (_, _, Some(date_idx)) => {
                                let date = &mut todo_dates[date_idx];
                                dt_start_tzid = date.2;
                                dt_start_has_time = true;
                                date.1.take().unwrap()
                            }
                            _ => {
                                return Ok(None);
                            }
                        }
                    }
                    _ => {
                        return Ok(None);
                    }
                },
            },
        };
        let mut event = None;
        let start_tz = tz_resolver.resolve_or_default(dt_start_tzid);
        let dt_start_tz = dt_start
            .to_date_time_with_tz(start_tz)
            .ok_or(CalendarErrorType::InvalidDtStart)?;
        let default_duration = if let Some(dt_end) = dt_end {
            let end = dt_end
                .to_date_time_with_tz(tz_resolver.resolve_or_default(dt_end_tzid.or(dt_start_tzid)))
                .ok_or(CalendarErrorType::InvalidDtEnd)?;

            if rrule.is_none() {
                event = Some(CalendarEvent {
                    start: dt_start_tz,
                    end: TimeOrDelta::Time(end),
                    comp_id,
                });
            }
            dt_end.date_time - dt_start.date_time
        } else if let Some(duration) = duration {
            let duration = duration
                .to_time_delta()
                .ok_or(CalendarErrorType::InvalidDuration)?;
            if rrule.is_none() {
                event = Some(CalendarEvent {
                    start: dt_start_tz,
                    end: TimeOrDelta::Delta(duration),
                    comp_id,
                });
            }
            duration
        } else if let Some((due, due_tzid)) = todo_dates
            .into_iter()
            .filter_map(|(prop, dt, tz_id)| {
                if prop == &ICalendarProperty::Due {
                    dt.map(|dt| (dt, tz_id))
                } else {
                    None
                }
            })
            .next()
        {
            let end = due
                .to_date_time_with_tz(tz_resolver.resolve_or_default(due_tzid.or(dt_start_tzid)))
                .ok_or(CalendarErrorType::InvalidDtEnd)?;

            if rrule.is_none() {
                event = Some(CalendarEvent {
                    start: dt_start_tz,
                    end: TimeOrDelta::Time(end),
                    comp_id,
                });
            }
            due.date_time - dt_start.date_time
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
                event = Some(CalendarEvent {
                    start: dt_start_tz,
                    end: TimeOrDelta::Delta(duration),
                    comp_id,
                });
            }
            duration
        };
        let rid = if let Some(rid) = rid {
            rid.to_date_time_with_tz(tz_resolver.resolve_or_default(rid_tzid.or(dt_start_tzid)))
        } else {
            None
        };

        // Add rdates
        for (tz_id, rdate) in rdates {
            if let Some(date_start) =
                rdate.to_date_time_with_tz(tz_resolver.resolve_or_default(tz_id.or(dt_start_tzid)))
            {
                events.push(CalendarEvent {
                    start: date_start,
                    end: TimeOrDelta::Delta(default_duration),
                    comp_id,
                });
            }
        }
        for (tz_id, start, end) in rdates_periods {
            let tz = tz_resolver.resolve_or_default(tz_id.or(dt_start_tzid));
            if let (Some(date_start), Some(date_end)) = (
                start.to_date_time_with_tz(tz),
                end.into_date_time_with_tz(tz),
            ) {
                events.push(CalendarEvent {
                    start: date_start,
                    end: date_end,
                    comp_id,
                });
            }
        }

        Ok(Some(CalendarEventBuilder {
            event,
            dt_start_tzid,
            default_duration,
            rrule_seq,
            rrule,
            exdates,
            start_tz,
            dt_start,
            rid,
            rid_this_and_future,
        }))
    }
}

impl TimeOrDelta<DateTimeResult, TimeDelta> {
    pub fn into_date_time_with_tz(self, tz: Tz) -> Option<TimeOrDelta<DateTime<Tz>, TimeDelta>> {
        match self {
            TimeOrDelta::Time(time) => time.to_date_time_with_tz(tz).map(TimeOrDelta::Time),
            TimeOrDelta::Delta(delta) => Some(TimeOrDelta::Delta(delta)),
        }
    }
}

impl CalendarEvent<DateTime<Tz>, TimeOrDelta<DateTime<Tz>, TimeDelta>> {
    pub fn timestamps(&self) -> (i64, i64) {
        let timestamp = self.start.timestamp();
        let end_timestamp = match self.end {
            TimeOrDelta::Time(time) => time.timestamp(),
            TimeOrDelta::Delta(delta) => timestamp + delta.num_seconds(),
        };

        (timestamp, end_timestamp)
    }

    pub fn try_into_date_time(self) -> Option<CalendarEvent<DateTime<Tz>, DateTime<Tz>>> {
        match self.end {
            TimeOrDelta::Time(time) => Some(time),
            TimeOrDelta::Delta(delta) => self
                .start
                .naive_local()
                .checked_add_signed(delta)
                .and_then(|end| end.and_local_timezone(self.start.timezone()).single()),
        }
        .map(|end| CalendarEvent {
            start: self.start,
            end,
            comp_id: self.comp_id,
        })
    }
}

impl Display for CalendarErrorType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            CalendarErrorType::MissingDtStart => write!(f, "Missing DTSTART property"),
            CalendarErrorType::InvalidDtStart => write!(f, "Invalid DTSTART property"),
            CalendarErrorType::InvalidDtEnd => write!(f, "Invalid DTEND property"),
            CalendarErrorType::InvalidDuration => write!(f, "Invalid DURATION property"),
            CalendarErrorType::RRule(err) => write!(f, "RRule error: {err}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        Entry, Parser,
        common::timezone::Tz,
        icalendar::dates::{CalendarError, CalendarEvent},
    };
    use chrono::DateTime;
    use serde::Serialize;
    use std::{io::Write, time::Instant};

    #[test]
    fn expand_rrule() {
        // Read all .ics files in the test directory
        for entry in std::fs::read_dir("resources/ical").unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "ics") {
                let input = match String::from_utf8(std::fs::read(&path).unwrap()) {
                    Ok(input) => input,
                    Err(err) => {
                        // ISO-8859-1
                        err.as_bytes()
                            .iter()
                            .map(|&b| b as char)
                            .collect::<String>()
                    }
                };
                let mut parser = Parser::new(&input);
                let mut output = None;
                //let mut output_debug =
                //    std::fs::File::create(path.with_extension("ics.debug")).unwrap();
                let file_name = path.as_path().to_str().unwrap();

                /*if file_name != "resources/ical/246.ics" {
                    continue;
                }*/

                #[derive(Serialize)]
                struct TestResult {
                    errors: Vec<CalendarError>,
                    events: Vec<CalendarEvent<DateTime<Tz>, DateTime<Tz>>>,
                }

                print!("Expanding recurrences for {file_name}... ");
                let now = Instant::now();
                loop {
                    match parser.entry() {
                        Entry::ICalendar(ical) => {
                            let expanded = ical.expand_dates(chrono_tz::Tz::Pacific__Auckland, 100);
                            let mut events = expanded
                                .events
                                .into_iter()
                                .filter_map(|event| event.try_into_date_time())
                                .collect::<Vec<_>>();
                            events.sort_by_key(|a| (a.start, a.comp_id));

                            for err in &expanded.errors {
                                print!("[{}: {:?}] ", err.comp_id, err.error);
                            }

                            if !events.is_empty() || !expanded.errors.is_empty() {
                                writeln!(
                                    output.get_or_insert_with(|| std::fs::File::create(
                                        path.with_extension("json")
                                    )
                                    .unwrap()),
                                    "{}",
                                    serde_json::to_string_pretty(&TestResult {
                                        errors: expanded.errors,
                                        events,
                                    })
                                    .unwrap()
                                )
                                .unwrap();
                            }
                        }
                        Entry::InvalidLine(_) => {}
                        Entry::Eof => {
                            println!(" (done in {:?})", now.elapsed());
                            break;
                        }
                        other => {
                            panic!("Expected iCal, got {other:?} for {file_name}");
                        }
                    }
                }
            }
        }
    }
}
