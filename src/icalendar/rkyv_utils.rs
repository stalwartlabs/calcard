/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

use super::*;
use crate::common::{ArchivedPartialDateTime, timezone::Tz};
use chrono::DateTime;

impl ArchivedICalendar {
    pub fn uids(&self) -> impl Iterator<Item = &str> {
        self.components
            .iter()
            .filter_map(|component| component.uid())
    }

    pub fn size(&self) -> usize {
        self.components
            .iter()
            .map(|component| component.size())
            .sum()
    }
}

impl ArchivedICalendarComponent {
    pub fn uid(&self) -> Option<&str> {
        self.property(&ICalendarProperty::Uid)
            .and_then(|e| e.values.first())
            .and_then(|v| v.as_text())
    }

    pub fn property(&self, prop: &ICalendarProperty) -> Option<&ArchivedICalendarEntry> {
        self.entries.iter().find(|entry| &entry.name == prop)
    }

    pub fn properties<'x, 'y: 'x>(
        &'x self,
        prop: &'y ICalendarProperty,
    ) -> impl Iterator<Item = &'x ArchivedICalendarEntry> + 'x {
        self.entries.iter().filter(move |entry| &entry.name == prop)
    }

    pub fn size(&self) -> usize {
        self.entries.iter().map(|entry| entry.size()).sum()
    }

    pub fn is_recurrent(&self) -> bool {
        self.entries.iter().any(|entry| {
            matches!(
                entry.name,
                ArchivedICalendarProperty::Rrule | ArchivedICalendarProperty::Rdate
            )
        })
    }

    pub fn is_recurrence_override(&self) -> bool {
        self.entries
            .iter()
            .any(|entry| matches!(entry.name, ArchivedICalendarProperty::RecurrenceId))
    }

    pub fn is_recurrent_or_override(&self) -> bool {
        self.entries.iter().any(|entry| {
            matches!(
                entry.name,
                ArchivedICalendarProperty::Rrule
                    | ArchivedICalendarProperty::Rdate
                    | ArchivedICalendarProperty::RecurrenceId
            )
        })
    }

    pub fn status(&self) -> Option<&ArchivedICalendarStatus> {
        self.entries
            .iter()
            .find_map(|entry| match (&entry.name, entry.values.first()) {
                (
                    ArchivedICalendarProperty::Status,
                    Some(ArchivedICalendarValue::Status(status)),
                ) => Some(status),
                _ => None,
            })
    }

    pub fn transparency(&self) -> Option<&ArchivedICalendarTransparency> {
        self.entries
            .iter()
            .find_map(|entry| match (&entry.name, entry.values.first()) {
                (
                    ArchivedICalendarProperty::Transp,
                    Some(ArchivedICalendarValue::Transparency(trans)),
                ) => Some(trans),
                _ => None,
            })
    }
}

impl ArchivedICalendarValue {
    pub fn size(&self) -> usize {
        match self {
            ArchivedICalendarValue::Binary(value) => value.len(),
            ArchivedICalendarValue::Text(value) => value.len(),
            ArchivedICalendarValue::PartialDateTime(_) => std::mem::size_of::<PartialDateTime>(),
            ArchivedICalendarValue::RecurrenceRule(_) => {
                std::mem::size_of::<ICalendarRecurrenceRule>()
            }
            _ => std::mem::size_of::<ICalendarValue>(),
        }
    }

    pub fn as_text(&self) -> Option<&str> {
        match self {
            ArchivedICalendarValue::Text(s) => Some(s.as_str()),
            ArchivedICalendarValue::Uri(v) => v.as_str(),
            ArchivedICalendarValue::CalendarScale(v) => Some(v.as_str()),
            ArchivedICalendarValue::Method(v) => Some(v.as_str()),
            ArchivedICalendarValue::Classification(v) => Some(v.as_str()),
            ArchivedICalendarValue::Status(v) => Some(v.as_str()),
            ArchivedICalendarValue::Transparency(v) => Some(v.as_str()),
            ArchivedICalendarValue::Action(v) => Some(v.as_str()),
            ArchivedICalendarValue::BusyType(v) => Some(v.as_str()),
            ArchivedICalendarValue::ParticipantType(v) => Some(v.as_str()),
            ArchivedICalendarValue::ResourceType(v) => Some(v.as_str()),
            ArchivedICalendarValue::Proximity(v) => Some(v.as_str()),
            _ => None,
        }
    }

    pub fn as_integer(&self) -> Option<i64> {
        match self {
            ArchivedICalendarValue::Integer(i) => Some(i.to_native()),
            _ => None,
        }
    }

    pub fn as_float(&self) -> Option<f64> {
        match self {
            ArchivedICalendarValue::Float(f) => Some(f.to_native()),
            _ => None,
        }
    }

    pub fn as_boolean(&self) -> Option<bool> {
        match self {
            ArchivedICalendarValue::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_partial_date_time(&self) -> Option<&ArchivedPartialDateTime> {
        match self {
            ArchivedICalendarValue::PartialDateTime(dt) => Some(dt),
            _ => None,
        }
    }

    pub fn as_binary(&self) -> Option<&[u8]> {
        match self {
            ArchivedICalendarValue::Binary(d) => Some(d.as_slice()),
            _ => None,
        }
    }
}

impl ArchivedICalendarParameterName {
    pub fn as_str(&self) -> &str {
        match self {
            ArchivedICalendarParameterName::Altrep => "ALTREP",
            ArchivedICalendarParameterName::Cn => "CN",
            ArchivedICalendarParameterName::Cutype => "CUTYPE",
            ArchivedICalendarParameterName::DelegatedFrom => "DELEGATED-FROM",
            ArchivedICalendarParameterName::DelegatedTo => "DELEGATED-TO",
            ArchivedICalendarParameterName::Dir => "DIR",
            ArchivedICalendarParameterName::Fmttype => "FMTTYPE",
            ArchivedICalendarParameterName::Fbtype => "FBTYPE",
            ArchivedICalendarParameterName::Language => "LANGUAGE",
            ArchivedICalendarParameterName::Member => "MEMBER",
            ArchivedICalendarParameterName::Partstat => "PARTSTAT",
            ArchivedICalendarParameterName::Range => "RANGE",
            ArchivedICalendarParameterName::Related => "RELATED",
            ArchivedICalendarParameterName::Reltype => "RELTYPE",
            ArchivedICalendarParameterName::Role => "ROLE",
            ArchivedICalendarParameterName::Rsvp => "RSVP",
            ArchivedICalendarParameterName::ScheduleAgent => "SCHEDULE-AGENT",
            ArchivedICalendarParameterName::ScheduleForceSend => "SCHEDULE-FORCE-SEND",
            ArchivedICalendarParameterName::ScheduleStatus => "SCHEDULE-STATUS",
            ArchivedICalendarParameterName::SentBy => "SENT-BY",
            ArchivedICalendarParameterName::Tzid => "TZID",
            ArchivedICalendarParameterName::Value => "VALUE",
            ArchivedICalendarParameterName::Display => "DISPLAY",
            ArchivedICalendarParameterName::Email => "EMAIL",
            ArchivedICalendarParameterName::Feature => "FEATURE",
            ArchivedICalendarParameterName::Label => "LABEL",
            ArchivedICalendarParameterName::Size => "SIZE",
            ArchivedICalendarParameterName::Filename => "FILENAME",
            ArchivedICalendarParameterName::ManagedId => "MANAGED-ID",
            ArchivedICalendarParameterName::Order => "ORDER",
            ArchivedICalendarParameterName::Schema => "SCHEMA",
            ArchivedICalendarParameterName::Derived => "DERIVED",
            ArchivedICalendarParameterName::Gap => "GAP",
            ArchivedICalendarParameterName::Linkrel => "LINKREL",
            ArchivedICalendarParameterName::Jsptr => "JSPTR",
            ArchivedICalendarParameterName::Jsid => "JSID",
            ArchivedICalendarParameterName::Other(name) => name.as_str(),
        }
    }
}

impl ArchivedICalendarEntry {
    #[inline]
    pub fn parameters(
        &self,
        prop: &ICalendarParameterName,
    ) -> impl Iterator<Item = &ArchivedICalendarParameterValue> {
        self.params.iter().filter_map(move |param| {
            if &param.name == prop {
                Some(&param.value)
            } else {
                None
            }
        })
    }

    pub fn size(&self) -> usize {
        self.values.iter().map(|value| value.size()).sum::<usize>()
            + self
                .params
                .iter()
                .map(|param| std::mem::size_of::<ICalendarParameterName>() + param.value.size())
                .sum::<usize>()
            + self.name.as_str().len()
    }
}

impl ArchivedICalendarParameterValue {
    pub fn as_text(&self) -> Option<&str> {
        match self {
            ArchivedICalendarParameterValue::Text(v) => Some(v.as_str()),
            ArchivedICalendarParameterValue::Bool(v) => Some(if *v { "TRUE" } else { "FALSE" }),
            ArchivedICalendarParameterValue::Uri(uri) => uri.as_str(),
            ArchivedICalendarParameterValue::Cutype(v) => Some(v.as_str()),
            ArchivedICalendarParameterValue::Fbtype(v) => Some(v.as_str()),
            ArchivedICalendarParameterValue::Partstat(v) => Some(v.as_str()),
            ArchivedICalendarParameterValue::Related(v) => Some(v.as_str()),
            ArchivedICalendarParameterValue::Reltype(v) => Some(v.as_str()),
            ArchivedICalendarParameterValue::Role(v) => Some(v.as_str()),
            ArchivedICalendarParameterValue::ScheduleAgent(v) => Some(v.as_str()),
            ArchivedICalendarParameterValue::ScheduleForceSend(v) => Some(v.as_str()),
            ArchivedICalendarParameterValue::Value(v) => Some(v.as_str()),
            ArchivedICalendarParameterValue::Display(v) => Some(v.as_str()),
            ArchivedICalendarParameterValue::Feature(v) => Some(v.as_str()),
            ArchivedICalendarParameterValue::Linkrel(v) => Some(v.as_str()),
            ArchivedICalendarParameterValue::Duration(_)
            | ArchivedICalendarParameterValue::Integer(_)
            | ArchivedICalendarParameterValue::Null => None,
        }
    }

    pub fn size(&self) -> usize {
        match self {
            ArchivedICalendarParameterValue::Text(v) => v.len(),
            ArchivedICalendarParameterValue::Uri(v) => v.size(),
            _ => std::mem::size_of::<ICalendarParameterValue>(),
        }
    }
}

impl ArchivedUri {
    pub fn as_str(&self) -> Option<&str> {
        match self {
            ArchivedUri::Data(data) => data.content_type.as_deref(),
            ArchivedUri::Location(loc) => loc.as_str().into(),
        }
    }

    pub fn size(&self) -> usize {
        match self {
            ArchivedUri::Data(data) => {
                data.data.len() + data.content_type.as_ref().map(|s| s.len()).unwrap_or(0)
            }
            ArchivedUri::Location(loc) => loc.len(),
        }
    }
}

impl ArchivedICalendarPeriod {
    pub fn time_range(&self, tz: Tz) -> Option<(DateTime<Tz>, DateTime<Tz>)> {
        match self {
            ArchivedICalendarPeriod::Range { start, end } => {
                if let (Some(start), Some(end)) = (
                    start
                        .to_date_time()
                        .and_then(|start| start.to_date_time_with_tz(tz)),
                    end.to_date_time()
                        .and_then(|end| end.to_date_time_with_tz(tz)),
                ) {
                    Some((start, end))
                } else {
                    None
                }
            }
            ArchivedICalendarPeriod::Duration { start, duration } => start
                .to_date_time()
                .and_then(|start| start.to_date_time_with_tz(tz))
                .and_then(|start| {
                    duration
                        .to_time_delta()
                        .and_then(|duration| start.checked_add_signed(duration))
                        .map(|end| (start, end))
                }),
        }
    }
}

impl ArchivedPartialDateTime {
    pub fn to_date_time_with_tz(&self, tz: Tz) -> Option<DateTime<Tz>> {
        self.to_date_time()
            .and_then(|dt| dt.to_date_time_with_tz(tz))
    }

    #[inline(always)]
    pub fn has_date(&self) -> bool {
        self.year.is_some() && self.month.is_some() && self.day.is_some()
    }

    #[inline(always)]
    pub fn has_time(&self) -> bool {
        self.hour.is_some() && self.minute.is_some()
    }

    #[inline(always)]
    pub fn has_zone(&self) -> bool {
        self.tz_hour.is_some()
    }

    #[inline(always)]
    pub fn has_date_and_time(&self) -> bool {
        self.has_date() && self.has_time()
    }
}

impl ArchivedICalendarDuration {
    pub fn to_time_delta(&self) -> Option<chrono::TimeDelta> {
        chrono::TimeDelta::new(self.as_seconds(), 0)
    }

    pub fn as_seconds(&self) -> i64 {
        let secs = self.seconds.to_native() as i64
            + self.minutes.to_native() as i64 * 60
            + self.hours.to_native() as i64 * 3600
            + self.days.to_native() as i64 * 86400
            + self.weeks.to_native() as i64 * 604800;

        if self.neg { -secs } else { secs }
    }
}

impl ArchivedICalendarMonth {
    pub fn is_leap(&self) -> bool {
        self.0 < 0
    }

    pub fn month(&self) -> u8 {
        self.0.unsigned_abs()
    }
}
