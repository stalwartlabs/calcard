/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

use super::{
    ICalendar, ICalendarComponent, ICalendarComponentType, ICalendarDuration, ICalendarEntry,
    ICalendarParameter, ICalendarParameterName, ICalendarProperty, ICalendarRecurrenceRule,
    ICalendarStatus, ICalendarTransparency, ICalendarValue, Uri,
};
use crate::common::PartialDateTime;

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
            ICalendarValue::Integer(ref i) => Some(*i),
            _ => None,
        }
    }

    pub fn as_float(&self) -> Option<f64> {
        match self {
            ICalendarValue::Float(ref f) => Some(*f),
            _ => None,
        }
    }

    pub fn as_boolean(&self) -> Option<bool> {
        match self {
            ICalendarValue::Boolean(ref b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_partial_date_time(&self) -> Option<&PartialDateTime> {
        match self {
            ICalendarValue::PartialDateTime(ref dt) => Some(dt),
            _ => None,
        }
    }

    pub fn as_binary(&self) -> Option<&[u8]> {
        match self {
            ICalendarValue::Binary(ref d) => Some(d.as_slice()),
            _ => None,
        }
    }
}

impl ICalendarEntry {
    pub fn size(&self) -> usize {
        self.values.iter().map(|value| value.size()).sum::<usize>()
            + self.params.iter().map(|param| param.size()).sum::<usize>()
            + self.name.as_str().len()
    }
}

impl ICalendarParameter {
    pub fn matches_name(&self, name: &ICalendarParameterName) -> bool {
        match name {
            ICalendarParameterName::Altrep => matches!(self, ICalendarParameter::Altrep(_)),
            ICalendarParameterName::Cn => matches!(self, ICalendarParameter::Cn(_)),
            ICalendarParameterName::Cutype => matches!(self, ICalendarParameter::Cutype(_)),
            ICalendarParameterName::DelegatedFrom => {
                matches!(self, ICalendarParameter::DelegatedFrom(_))
            }
            ICalendarParameterName::DelegatedTo => {
                matches!(self, ICalendarParameter::DelegatedTo(_))
            }
            ICalendarParameterName::Dir => matches!(self, ICalendarParameter::Dir(_)),
            ICalendarParameterName::Fmttype => matches!(self, ICalendarParameter::Fmttype(_)),
            ICalendarParameterName::Fbtype => matches!(self, ICalendarParameter::Fbtype(_)),
            ICalendarParameterName::Language => matches!(self, ICalendarParameter::Language(_)),
            ICalendarParameterName::Member => matches!(self, ICalendarParameter::Member(_)),
            ICalendarParameterName::Partstat => matches!(self, ICalendarParameter::Partstat(_)),
            ICalendarParameterName::Range => matches!(self, ICalendarParameter::Range),
            ICalendarParameterName::Related => matches!(self, ICalendarParameter::Related(_)),
            ICalendarParameterName::Reltype => matches!(self, ICalendarParameter::Reltype(_)),
            ICalendarParameterName::Role => matches!(self, ICalendarParameter::Role(_)),
            ICalendarParameterName::Rsvp => matches!(self, ICalendarParameter::Rsvp(_)),
            ICalendarParameterName::ScheduleAgent => {
                matches!(self, ICalendarParameter::ScheduleAgent(_))
            }
            ICalendarParameterName::ScheduleForceSend => {
                matches!(self, ICalendarParameter::ScheduleForceSend(_))
            }
            ICalendarParameterName::ScheduleStatus => {
                matches!(self, ICalendarParameter::ScheduleStatus(_))
            }
            ICalendarParameterName::SentBy => matches!(self, ICalendarParameter::SentBy(_)),
            ICalendarParameterName::Tzid => matches!(self, ICalendarParameter::Tzid(_)),
            ICalendarParameterName::Value => matches!(self, ICalendarParameter::Value(_)),
            ICalendarParameterName::Display => matches!(self, ICalendarParameter::Display(_)),
            ICalendarParameterName::Email => matches!(self, ICalendarParameter::Email(_)),
            ICalendarParameterName::Feature => matches!(self, ICalendarParameter::Feature(_)),
            ICalendarParameterName::Label => matches!(self, ICalendarParameter::Label(_)),
            ICalendarParameterName::Size => matches!(self, ICalendarParameter::Size(_)),
            ICalendarParameterName::Filename => matches!(self, ICalendarParameter::Filename(_)),
            ICalendarParameterName::ManagedId => {
                matches!(self, ICalendarParameter::ManagedId(_))
            }
            ICalendarParameterName::Order => matches!(self, ICalendarParameter::Order(_)),
            ICalendarParameterName::Schema => matches!(self, ICalendarParameter::Schema(_)),
            ICalendarParameterName::Derived => matches!(self, ICalendarParameter::Derived(_)),
            ICalendarParameterName::Gap => matches!(self, ICalendarParameter::Gap(_)),
            ICalendarParameterName::Linkrel => matches!(self, ICalendarParameter::Linkrel(_)),
            ICalendarParameterName::Other(name) => {
                if let ICalendarParameter::Other(ref o) = self {
                    o.iter().any(|s| s == name)
                } else {
                    false
                }
            }
        }
    }

    pub fn as_text(&self) -> Option<&str> {
        match self {
            ICalendarParameter::Altrep(uri) => uri.as_str(),
            ICalendarParameter::Cn(v) => v.as_str().into(),
            ICalendarParameter::Cutype(v) => v.as_str().into(),
            ICalendarParameter::DelegatedFrom(uris) => uris.first().and_then(|u| u.as_str()),
            ICalendarParameter::DelegatedTo(uris) => uris.first().and_then(|u| u.as_str()),
            ICalendarParameter::Dir(v) => v.as_str(),
            ICalendarParameter::Fmttype(v) => v.as_str().into(),
            ICalendarParameter::Fbtype(v) => v.as_str().into(),
            ICalendarParameter::Language(v) => v.as_str().into(),
            ICalendarParameter::Member(uris) => uris.first().and_then(|u| u.as_str()),
            ICalendarParameter::Partstat(v) => v.as_str().into(),
            ICalendarParameter::Range => "THISANDFUTURE".into(),
            ICalendarParameter::Related(v) => v.as_str().into(),
            ICalendarParameter::Reltype(v) => v.as_str().into(),
            ICalendarParameter::Role(v) => v.as_str().into(),
            ICalendarParameter::Rsvp(v) => (if *v { "true" } else { "false" }).into(),
            ICalendarParameter::ScheduleAgent(v) => v.as_str().into(),
            ICalendarParameter::ScheduleForceSend(v) => v.as_str().into(),
            ICalendarParameter::ScheduleStatus(v) => v.as_str().into(),
            ICalendarParameter::SentBy(v) => v.as_str(),
            ICalendarParameter::Tzid(v) => v.as_str().into(),
            ICalendarParameter::Value(v) => v.as_str().into(),
            ICalendarParameter::Display(items) => items.first().map(|s| s.as_str()),
            ICalendarParameter::Email(v) => v.as_str().into(),
            ICalendarParameter::Feature(items) => items.first().map(|s| s.as_str()),
            ICalendarParameter::Label(v) => v.as_str().into(),
            ICalendarParameter::Size(_) => None,
            ICalendarParameter::Filename(v) => v.as_str().into(),
            ICalendarParameter::ManagedId(v) => v.as_str().into(),
            ICalendarParameter::Order(_) => None,
            ICalendarParameter::Schema(v) => v.as_str(),
            ICalendarParameter::Derived(v) => (if *v { "true" } else { "false" }).into(),
            ICalendarParameter::Gap(_) => None,
            ICalendarParameter::Linkrel(v) => v.as_str(),
            ICalendarParameter::Other(items) => items.first().map(|s| s.as_str()),
        }
    }

    pub fn size(&self) -> usize {
        match self {
            ICalendarParameter::Altrep(s) => s.size(),
            ICalendarParameter::Cn(s) => s.len(),
            ICalendarParameter::Cutype(s) => s.as_str().len(),
            ICalendarParameter::DelegatedFrom(v) => v.iter().map(|s| s.size()).sum(),
            ICalendarParameter::DelegatedTo(v) => v.iter().map(|s| s.size()).sum(),
            ICalendarParameter::Dir(s) => s.size(),
            ICalendarParameter::Fmttype(s) => s.len(),
            ICalendarParameter::Fbtype(s) => s.as_str().len(),
            ICalendarParameter::Language(s) => s.len(),
            ICalendarParameter::Member(v) => v.iter().map(|s| s.size()).sum(),
            ICalendarParameter::Partstat(s) => s.as_str().len(),
            ICalendarParameter::Related(ref r) => r.as_str().len(),
            ICalendarParameter::Reltype(ref r) => r.as_str().len(),
            ICalendarParameter::Role(ref r) => r.as_str().len(),
            ICalendarParameter::ScheduleAgent(ref a) => a.as_str().len(),
            ICalendarParameter::ScheduleForceSend(ref a) => a.as_str().len(),
            ICalendarParameter::ScheduleStatus(ref a) => a.len(),
            ICalendarParameter::SentBy(ref u) => u.size(),
            ICalendarParameter::Tzid(ref t) => t.len(),
            ICalendarParameter::Value(ref t) => t.as_str().len(),
            ICalendarParameter::Display(ref d) => d.iter().map(|s| s.as_str().len()).sum(),
            ICalendarParameter::Email(ref e) => e.len(),
            ICalendarParameter::Feature(ref f) => f.iter().map(|s| s.as_str().len()).sum(),
            ICalendarParameter::Label(ref l) => l.len(),
            ICalendarParameter::Filename(s) => s.as_str().len(),
            ICalendarParameter::ManagedId(s) => s.as_str().len(),
            ICalendarParameter::Schema(s) => s.size(),
            ICalendarParameter::Linkrel(ref l) => l.size(),
            ICalendarParameter::Other(ref o) => o.iter().map(|s| s.len()).sum(),
            _ => std::mem::size_of::<ICalendarParameter>(),
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

        if self.neg {
            -secs
        } else {
            secs
        }
    }
}
