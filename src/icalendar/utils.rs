/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

use super::{
    ICalendar, ICalendarComponent, ICalendarComponentType, ICalendarDuration, ICalendarEntry,
    ICalendarParameterName, ICalendarProperty, ICalendarRecurrenceRule, ICalendarStatus,
    ICalendarTransparency, ICalendarValue, Uri,
};
use crate::{
    common::{IanaString, PartialDateTime},
    icalendar::ICalendarParameterValue,
};

impl ICalendar {
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

    pub fn component_by_id(&self, id: u16) -> Option<&ICalendarComponent> {
        self.components.get(id as usize)
    }

    pub fn alarms_for_id(&self, id: u16) -> impl Iterator<Item = &ICalendarComponent> {
        self.component_by_id(id)
            .map_or(&[][..], |c| c.component_ids.as_slice())
            .iter()
            .filter_map(|id| {
                self.component_by_id(*id)
                    .filter(|c| c.component_type == ICalendarComponentType::VAlarm)
            })
    }
}

impl ICalendarComponent {
    pub fn uid(&self) -> Option<&str> {
        self.property(&ICalendarProperty::Uid)
            .and_then(|e| e.values.first())
            .and_then(|v| v.as_text())
    }

    pub fn property(&self, prop: &ICalendarProperty) -> Option<&ICalendarEntry> {
        self.entries.iter().find(|entry| &entry.name == prop)
    }

    pub fn properties<'x, 'y: 'x>(
        &'x self,
        prop: &'y ICalendarProperty,
    ) -> impl Iterator<Item = &'x ICalendarEntry> + 'x {
        self.entries.iter().filter(move |entry| &entry.name == prop)
    }

    pub fn size(&self) -> usize {
        self.entries.iter().map(|entry| entry.size()).sum()
    }

    pub fn is_recurrent(&self) -> bool {
        self.entries.iter().any(|entry| {
            matches!(
                entry.name,
                ICalendarProperty::Rrule | ICalendarProperty::Rdate
            )
        })
    }

    pub fn is_recurrence_override(&self) -> bool {
        self.entries
            .iter()
            .any(|entry| matches!(entry.name, ICalendarProperty::RecurrenceId))
    }

    pub fn is_recurrent_or_override(&self) -> bool {
        self.entries.iter().any(|entry| {
            matches!(
                entry.name,
                ICalendarProperty::Rrule
                    | ICalendarProperty::Rdate
                    | ICalendarProperty::RecurrenceId
            )
        })
    }

    pub fn status(&self) -> Option<&ICalendarStatus> {
        self.entries
            .iter()
            .find_map(|entry| match (&entry.name, entry.values.first()) {
                (ICalendarProperty::Status, Some(ICalendarValue::Status(status))) => Some(status),
                _ => None,
            })
    }

    pub fn transparency(&self) -> Option<&ICalendarTransparency> {
        self.entries
            .iter()
            .find_map(|entry| match (&entry.name, entry.values.first()) {
                (ICalendarProperty::Transp, Some(ICalendarValue::Transparency(trans))) => {
                    Some(trans)
                }
                _ => None,
            })
    }
}

impl ICalendarValue {
    pub fn size(&self) -> usize {
        match self {
            ICalendarValue::Binary(value) => value.len(),
            ICalendarValue::Text(value) => value.len(),
            ICalendarValue::PartialDateTime(_) => std::mem::size_of::<PartialDateTime>(),
            ICalendarValue::RecurrenceRule(_) => std::mem::size_of::<ICalendarRecurrenceRule>(),
            _ => std::mem::size_of::<ICalendarValue>(),
        }
    }

    pub fn as_text(&self) -> Option<&str> {
        match self {
            ICalendarValue::Text(s) => Some(s.as_str()),
            ICalendarValue::Uri(v) => v.as_str(),
            ICalendarValue::CalendarScale(v) => Some(v.as_str()),
            ICalendarValue::Method(v) => Some(v.as_str()),
            ICalendarValue::Classification(v) => Some(v.as_str()),
            ICalendarValue::Status(v) => Some(v.as_str()),
            ICalendarValue::Transparency(v) => Some(v.as_str()),
            ICalendarValue::Action(v) => Some(v.as_str()),
            ICalendarValue::BusyType(v) => Some(v.as_str()),
            ICalendarValue::ParticipantType(v) => Some(v.as_str()),
            ICalendarValue::ResourceType(v) => Some(v.as_str()),
            ICalendarValue::Proximity(v) => Some(v.as_str()),
            _ => None,
        }
    }

    pub fn as_integer(&self) -> Option<i64> {
        match self {
            ICalendarValue::Integer(i) => Some(*i),
            _ => None,
        }
    }

    pub fn as_float(&self) -> Option<f64> {
        match self {
            ICalendarValue::Float(f) => Some(*f),
            _ => None,
        }
    }

    pub fn as_boolean(&self) -> Option<bool> {
        match self {
            ICalendarValue::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_partial_date_time(&self) -> Option<&PartialDateTime> {
        match self {
            ICalendarValue::PartialDateTime(dt) => Some(dt),
            _ => None,
        }
    }

    pub fn as_binary(&self) -> Option<&[u8]> {
        match self {
            ICalendarValue::Binary(d) => Some(d.as_slice()),
            _ => None,
        }
    }
}

impl ICalendarEntry {
    #[inline]
    pub fn parameters(
        &self,
        prop: &ICalendarParameterName,
    ) -> impl Iterator<Item = &ICalendarParameterValue> {
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

impl ICalendarParameterValue {
    pub fn as_text(&self) -> Option<&str> {
        match self {
            ICalendarParameterValue::Text(v) => Some(v.as_str()),
            ICalendarParameterValue::Bool(v) => Some(if *v { "TRUE" } else { "FALSE" }),
            ICalendarParameterValue::Uri(uri) => uri.as_str(),
            ICalendarParameterValue::Cutype(v) => Some(v.as_str()),
            ICalendarParameterValue::Fbtype(v) => Some(v.as_str()),
            ICalendarParameterValue::Partstat(v) => Some(v.as_str()),
            ICalendarParameterValue::Related(v) => Some(v.as_str()),
            ICalendarParameterValue::Reltype(v) => Some(v.as_str()),
            ICalendarParameterValue::Role(v) => Some(v.as_str()),
            ICalendarParameterValue::ScheduleAgent(v) => Some(v.as_str()),
            ICalendarParameterValue::ScheduleForceSend(v) => Some(v.as_str()),
            ICalendarParameterValue::Value(v) => Some(v.as_str()),
            ICalendarParameterValue::Display(v) => Some(v.as_str()),
            ICalendarParameterValue::Feature(v) => Some(v.as_str()),
            ICalendarParameterValue::Linkrel(v) => Some(v.as_str()),
            ICalendarParameterValue::Duration(_)
            | ICalendarParameterValue::Integer(_)
            | ICalendarParameterValue::Null => None,
        }
    }

    pub fn size(&self) -> usize {
        match self {
            ICalendarParameterValue::Text(v) => v.len(),
            ICalendarParameterValue::Uri(v) => v.size(),
            _ => std::mem::size_of::<ICalendarParameterValue>(),
        }
    }
}

impl Uri {
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Uri::Data(data) => data.content_type.as_deref(),
            Uri::Location(loc) => loc.as_str().into(),
        }
    }

    pub fn size(&self) -> usize {
        match self {
            Uri::Data(data) => {
                data.data.len() + data.content_type.as_ref().map(|s| s.len()).unwrap_or(0)
            }
            Uri::Location(loc) => loc.len(),
        }
    }
}

impl ICalendarDuration {
    pub fn to_time_delta(&self) -> Option<chrono::TimeDelta> {
        chrono::TimeDelta::new(self.as_seconds(), 0)
    }

    pub fn as_seconds(&self) -> i64 {
        let secs = self.seconds as i64
            + self.minutes as i64 * 60
            + self.hours as i64 * 3600
            + self.days as i64 * 86400
            + self.weeks as i64 * 604800;

        if self.neg { -secs } else { secs }
    }
}
