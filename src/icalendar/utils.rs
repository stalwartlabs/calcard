/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

use std::borrow::Cow;

use super::{
    ICalendar, ICalendarComponent, ICalendarComponentType, ICalendarDuration, ICalendarEntry,
    ICalendarParameterName, ICalendarProperty, ICalendarRecurrenceRule, ICalendarStatus,
    ICalendarTransparency, ICalendarValue, Uri,
};
use crate::{
    common::{IanaString, IanaType, PartialDateTime},
    icalendar::{ICalendarParameterValue, ICalendarValueType},
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

    pub fn component_by_id(&self, id: u32) -> Option<&ICalendarComponent> {
        self.components.get(id as usize)
    }

    pub fn alarms_for_id(&self, id: u32) -> impl Iterator<Item = &ICalendarComponent> {
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

    pub fn jsid(&self) -> Option<&str> {
        self.property(&ICalendarProperty::Jsid)
            .and_then(|e| e.values.first())
            .and_then(|v| v.as_text())
    }

    pub fn property(&self, prop: &ICalendarProperty) -> Option<&ICalendarEntry> {
        self.entries.iter().find(|entry| &entry.name == prop)
    }

    pub fn property_mut(&mut self, prop: &ICalendarProperty) -> Option<&mut ICalendarEntry> {
        self.entries.iter_mut().find(|entry| &entry.name == prop)
    }

    pub fn has_property(&self, prop: &ICalendarProperty) -> bool {
        self.entries.iter().any(|entry| &entry.name == prop)
    }

    pub fn properties<'x, 'y: 'x>(
        &'x self,
        prop: &'y ICalendarProperty,
    ) -> impl Iterator<Item = &'x ICalendarEntry> + 'x {
        self.entries.iter().filter(move |entry| &entry.name == prop)
    }

    pub fn properties_mut<'x, 'y: 'x>(
        &'x mut self,
        prop: &'y ICalendarProperty,
    ) -> impl Iterator<Item = &'x mut ICalendarEntry> + 'x {
        self.entries
            .iter_mut()
            .filter(move |entry| &entry.name == prop)
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

    pub fn into_text(self) -> Option<Cow<'static, str>> {
        match self {
            ICalendarValue::Text(s) => Some(Cow::Owned(s)),
            ICalendarValue::Uri(v) => Some(Cow::Owned(v.into_unwrapped_string())),
            ICalendarValue::CalendarScale(v) => Some(Cow::Borrowed(v.as_str())),
            ICalendarValue::Method(v) => Some(Cow::Borrowed(v.as_str())),
            ICalendarValue::Classification(v) => Some(Cow::Borrowed(v.as_str())),
            ICalendarValue::Status(v) => Some(Cow::Borrowed(v.as_str())),
            ICalendarValue::Transparency(v) => Some(Cow::Borrowed(v.as_str())),
            ICalendarValue::Action(v) => Some(Cow::Borrowed(v.as_str())),
            ICalendarValue::BusyType(v) => Some(Cow::Borrowed(v.as_str())),
            ICalendarValue::ParticipantType(v) => Some(Cow::Borrowed(v.as_str())),
            ICalendarValue::ResourceType(v) => Some(Cow::Borrowed(v.as_str())),
            ICalendarValue::Proximity(v) => Some(Cow::Borrowed(v.as_str())),
            ICalendarValue::Integer(i) => Some(Cow::Owned(i.to_string())),
            ICalendarValue::Float(f) => Some(Cow::Owned(f.to_string())),
            ICalendarValue::Boolean(b) => Some(Cow::Borrowed(if b { "TRUE" } else { "FALSE" })),
            ICalendarValue::PartialDateTime(dt) => dt.to_rfc3339().map(Cow::Owned),
            ICalendarValue::RecurrenceRule(rrule) => Some(Cow::Owned(rrule.to_string())),
            ICalendarValue::Binary(value) => String::from_utf8(value).ok().map(Cow::Owned),
            ICalendarValue::Duration(value) => Some(Cow::Owned(value.to_string())),
            ICalendarValue::Period(value) => Some(Cow::Owned(value.to_string())),
        }
    }

    pub fn into_partial_date_time(self) -> Option<Box<PartialDateTime>> {
        match self {
            ICalendarValue::PartialDateTime(dt) => Some(dt),
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

    #[inline]
    pub fn parameter(&self, prop: &ICalendarParameterName) -> Option<&ICalendarParameterValue> {
        self.params.iter().find_map(move |param| {
            if &param.name == prop {
                Some(&param.value)
            } else {
                None
            }
        })
    }

    #[inline]
    pub fn has_parameter(&self, prop: &ICalendarParameterName) -> bool {
        self.params.iter().any(|param| &param.name == prop)
    }

    pub fn jsid(&self) -> Option<&str> {
        self.parameter(&ICalendarParameterName::Jsid)
            .and_then(|v| v.as_text())
    }

    pub fn is_derived(&self) -> bool {
        self.parameter(&ICalendarParameterName::Derived)
            .is_some_and(|v| matches!(v, ICalendarParameterValue::Bool(true)))
    }

    pub fn calendar_address(&self) -> Option<&str> {
        self.values
            .first()
            .and_then(|v| v.as_text())
            .map(|v| v.strip_prefix("mailto:").unwrap_or(v))
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

    pub fn into_text(self) -> Option<Cow<'static, str>> {
        match self {
            ICalendarParameterValue::Text(v) => Some(Cow::Owned(v)),
            ICalendarParameterValue::Bool(v) => {
                Some(Cow::Borrowed(if v { "TRUE" } else { "FALSE" }))
            }
            ICalendarParameterValue::Uri(uri) => Some(Cow::Owned(uri.into_unwrapped_string())),
            ICalendarParameterValue::Cutype(v) => Some(Cow::Borrowed(v.as_str())),
            ICalendarParameterValue::Fbtype(v) => Some(Cow::Borrowed(v.as_str())),
            ICalendarParameterValue::Partstat(v) => Some(Cow::Borrowed(v.as_str())),
            ICalendarParameterValue::Related(v) => Some(Cow::Borrowed(v.as_str())),
            ICalendarParameterValue::Reltype(v) => Some(Cow::Borrowed(v.as_str())),
            ICalendarParameterValue::Role(v) => Some(Cow::Borrowed(v.as_str())),
            ICalendarParameterValue::ScheduleAgent(v) => Some(Cow::Borrowed(v.as_str())),
            ICalendarParameterValue::ScheduleForceSend(v) => Some(Cow::Borrowed(v.as_str())),
            ICalendarParameterValue::Value(v) => Some(Cow::Borrowed(v.as_str())),
            ICalendarParameterValue::Display(v) => Some(Cow::Borrowed(v.as_str())),
            ICalendarParameterValue::Feature(v) => Some(Cow::Borrowed(v.as_str())),
            ICalendarParameterValue::Linkrel(v) => Some(Cow::Borrowed(v.as_str())),
            ICalendarParameterValue::Duration(v) => Some(Cow::Owned(v.to_string())),
            ICalendarParameterValue::Integer(v) => Some(Cow::Owned(v.to_string())),
            ICalendarParameterValue::Null => None,
        }
    }

    pub fn into_value_type(self) -> Option<IanaType<ICalendarValueType, String>> {
        match self {
            ICalendarParameterValue::Value(v) => Some(IanaType::Iana(v)),
            ICalendarParameterValue::Text(v) => Some(IanaType::Other(v)),
            _ => None,
        }
    }

    pub fn as_integer(&self) -> Option<u64> {
        match self {
            ICalendarParameterValue::Integer(v) => Some(*v),
            ICalendarParameterValue::Text(v) => v.parse().ok(),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            ICalendarParameterValue::Bool(v) => Some(*v),
            _ => None,
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

impl ICalendarComponentType {
    pub fn has_time_ranges(&self) -> bool {
        matches!(
            self,
            ICalendarComponentType::VEvent
                | ICalendarComponentType::VTodo
                | ICalendarComponentType::VJournal
                | ICalendarComponentType::VFreebusy
        )
    }

    pub fn is_scheduling_object(&self) -> bool {
        matches!(
            self,
            ICalendarComponentType::VEvent
                | ICalendarComponentType::VTodo
                | ICalendarComponentType::VJournal
                | ICalendarComponentType::VFreebusy
        )
    }

    pub fn is_event(&self) -> bool {
        matches!(self, ICalendarComponentType::VEvent)
    }

    pub fn is_todo(&self) -> bool {
        matches!(self, ICalendarComponentType::VTodo)
    }

    pub fn is_event_or_todo(&self) -> bool {
        matches!(
            self,
            ICalendarComponentType::VEvent | ICalendarComponentType::VTodo
        )
    }

    pub fn is_journal(&self) -> bool {
        matches!(self, ICalendarComponentType::VJournal)
    }

    pub fn is_freebusy(&self) -> bool {
        matches!(self, ICalendarComponentType::VFreebusy)
    }

    pub fn is_location(&self) -> bool {
        matches!(self, ICalendarComponentType::VLocation)
    }

    pub fn is_alarm(&self) -> bool {
        matches!(self, ICalendarComponentType::VAlarm)
    }

    pub fn is_participant(&self) -> bool {
        matches!(self, ICalendarComponentType::Participant)
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
    pub fn from_seconds(seconds: i64) -> Self {
        let mut secs = seconds;
        let neg = secs < 0;
        if neg {
            secs = -secs;
        }
        let weeks = (secs / 604800) as u32;
        secs %= 604800;
        let days = (secs / 86400) as u32;
        secs %= 86400;
        let hours = (secs / 3600) as u32;
        secs %= 3600;
        let minutes = (secs / 60) as u32;
        secs %= 60;
        let seconds = secs as u32;

        Self {
            weeks,
            days,
            hours,
            minutes,
            seconds,
            neg,
        }
    }

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

    pub fn is_empty(&self) -> bool {
        self.weeks == 0
            && self.days == 0
            && self.hours == 0
            && self.minutes == 0
            && self.seconds == 0
    }
}
