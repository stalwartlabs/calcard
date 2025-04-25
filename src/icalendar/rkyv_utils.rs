use crate::common::ArchivedPartialDateTime;

use super::*;

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
            ArchivedICalendarValue::Text(ref s) => Some(s),
            _ => None,
        }
    }

    pub fn as_integer(&self) -> Option<i64> {
        match self {
            ArchivedICalendarValue::Integer(ref i) => Some(i.to_native()),
            _ => None,
        }
    }

    pub fn as_float(&self) -> Option<f64> {
        match self {
            ArchivedICalendarValue::Float(ref f) => Some(f.to_native()),
            _ => None,
        }
    }

    pub fn as_boolean(&self) -> Option<bool> {
        match self {
            ArchivedICalendarValue::Boolean(ref b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_partial_date_time(&self) -> Option<&ArchivedPartialDateTime> {
        match self {
            ArchivedICalendarValue::PartialDateTime(ref dt) => Some(dt),
            _ => None,
        }
    }

    pub fn as_binary(&self) -> Option<&[u8]> {
        match self {
            ArchivedICalendarValue::Binary(ref d) => Some(d.as_slice()),
            _ => None,
        }
    }
}

impl ArchivedICalendarEntry {
    pub fn size(&self) -> usize {
        self.values.iter().map(|value| value.size()).sum::<usize>()
            + self.params.iter().map(|param| param.size()).sum::<usize>()
            + self.name.as_str().len()
    }
}

impl ArchivedICalendarParameter {
    pub fn matches_name(&self, name: &ICalendarParameterName) -> bool {
        match name {
            ICalendarParameterName::Altrep => matches!(self, ArchivedICalendarParameter::Altrep(_)),
            ICalendarParameterName::Cn => matches!(self, ArchivedICalendarParameter::Cn(_)),
            ICalendarParameterName::Cutype => matches!(self, ArchivedICalendarParameter::Cutype(_)),
            ICalendarParameterName::DelegatedFrom => {
                matches!(self, ArchivedICalendarParameter::DelegatedFrom(_))
            }
            ICalendarParameterName::DelegatedTo => {
                matches!(self, ArchivedICalendarParameter::DelegatedTo(_))
            }
            ICalendarParameterName::Dir => matches!(self, ArchivedICalendarParameter::Dir(_)),
            ICalendarParameterName::Fmttype => {
                matches!(self, ArchivedICalendarParameter::Fmttype(_))
            }
            ICalendarParameterName::Fbtype => matches!(self, ArchivedICalendarParameter::Fbtype(_)),
            ICalendarParameterName::Language => {
                matches!(self, ArchivedICalendarParameter::Language(_))
            }
            ICalendarParameterName::Member => matches!(self, ArchivedICalendarParameter::Member(_)),
            ICalendarParameterName::Partstat => {
                matches!(self, ArchivedICalendarParameter::Partstat(_))
            }
            ICalendarParameterName::Range => matches!(self, ArchivedICalendarParameter::Range),
            ICalendarParameterName::Related => {
                matches!(self, ArchivedICalendarParameter::Related(_))
            }
            ICalendarParameterName::Reltype => {
                matches!(self, ArchivedICalendarParameter::Reltype(_))
            }
            ICalendarParameterName::Role => matches!(self, ArchivedICalendarParameter::Role(_)),
            ICalendarParameterName::Rsvp => matches!(self, ArchivedICalendarParameter::Rsvp(_)),
            ICalendarParameterName::ScheduleAgent => {
                matches!(self, ArchivedICalendarParameter::ScheduleAgent(_))
            }
            ICalendarParameterName::ScheduleForceSend => {
                matches!(self, ArchivedICalendarParameter::ScheduleForceSend(_))
            }
            ICalendarParameterName::ScheduleStatus => {
                matches!(self, ArchivedICalendarParameter::ScheduleStatus(_))
            }
            ICalendarParameterName::SentBy => matches!(self, ArchivedICalendarParameter::SentBy(_)),
            ICalendarParameterName::Tzid => matches!(self, ArchivedICalendarParameter::Tzid(_)),
            ICalendarParameterName::Value => matches!(self, ArchivedICalendarParameter::Value(_)),
            ICalendarParameterName::Display => {
                matches!(self, ArchivedICalendarParameter::Display(_))
            }
            ICalendarParameterName::Email => matches!(self, ArchivedICalendarParameter::Email(_)),
            ICalendarParameterName::Feature => {
                matches!(self, ArchivedICalendarParameter::Feature(_))
            }
            ICalendarParameterName::Label => matches!(self, ArchivedICalendarParameter::Label(_)),
            ICalendarParameterName::Size => matches!(self, ArchivedICalendarParameter::Size(_)),
            ICalendarParameterName::Filename => {
                matches!(self, ArchivedICalendarParameter::Filename(_))
            }
            ICalendarParameterName::ManagedId => {
                matches!(self, ArchivedICalendarParameter::ManagedId(_))
            }
            ICalendarParameterName::Order => matches!(self, ArchivedICalendarParameter::Order(_)),
            ICalendarParameterName::Schema => matches!(self, ArchivedICalendarParameter::Schema(_)),
            ICalendarParameterName::Derived => {
                matches!(self, ArchivedICalendarParameter::Derived(_))
            }
            ICalendarParameterName::Gap => matches!(self, ArchivedICalendarParameter::Gap(_)),
            ICalendarParameterName::Linkrel => {
                matches!(self, ArchivedICalendarParameter::Linkrel(_))
            }
            ICalendarParameterName::Other(name) => {
                if let ArchivedICalendarParameter::Other(ref o) = self {
                    o.iter().any(|s| s == name)
                } else {
                    false
                }
            }
        }
    }

    pub fn as_text(&self) -> Option<&str> {
        match self {
            ArchivedICalendarParameter::Altrep(uri) => uri.as_str(),
            ArchivedICalendarParameter::Cn(v) => v.as_str().into(),
            ArchivedICalendarParameter::Cutype(v) => v.as_str().into(),
            ArchivedICalendarParameter::DelegatedFrom(uris) => {
                uris.first().and_then(|u| u.as_str())
            }
            ArchivedICalendarParameter::DelegatedTo(uris) => uris.first().and_then(|u| u.as_str()),
            ArchivedICalendarParameter::Dir(v) => v.as_str(),
            ArchivedICalendarParameter::Fmttype(v) => v.as_str().into(),
            ArchivedICalendarParameter::Fbtype(v) => v.as_str().into(),
            ArchivedICalendarParameter::Language(v) => v.as_str().into(),
            ArchivedICalendarParameter::Member(uris) => uris.first().and_then(|u| u.as_str()),
            ArchivedICalendarParameter::Partstat(v) => v.as_str().into(),
            ArchivedICalendarParameter::Range => "THISANDFUTURE".into(),
            ArchivedICalendarParameter::Related(v) => v.as_str().into(),
            ArchivedICalendarParameter::Reltype(v) => v.as_str().into(),
            ArchivedICalendarParameter::Role(v) => v.as_str().into(),
            ArchivedICalendarParameter::Rsvp(v) => (if *v { "true" } else { "false" }).into(),
            ArchivedICalendarParameter::ScheduleAgent(v) => v.as_str().into(),
            ArchivedICalendarParameter::ScheduleForceSend(v) => v.as_str().into(),
            ArchivedICalendarParameter::ScheduleStatus(v) => v.as_str().into(),
            ArchivedICalendarParameter::SentBy(v) => v.as_str(),
            ArchivedICalendarParameter::Tzid(v) => v.as_str().into(),
            ArchivedICalendarParameter::Value(v) => v.as_str().into(),
            ArchivedICalendarParameter::Display(items) => items.first().map(|s| s.as_str()),
            ArchivedICalendarParameter::Email(v) => v.as_str().into(),
            ArchivedICalendarParameter::Feature(items) => items.first().map(|s| s.as_str()),
            ArchivedICalendarParameter::Label(v) => v.as_str().into(),
            ArchivedICalendarParameter::Size(_) => None,
            ArchivedICalendarParameter::Filename(v) => v.as_str().into(),
            ArchivedICalendarParameter::ManagedId(v) => v.as_str().into(),
            ArchivedICalendarParameter::Order(_) => None,
            ArchivedICalendarParameter::Schema(v) => v.as_str(),
            ArchivedICalendarParameter::Derived(v) => (if *v { "true" } else { "false" }).into(),
            ArchivedICalendarParameter::Gap(_) => None,
            ArchivedICalendarParameter::Linkrel(v) => v.as_str(),
            ArchivedICalendarParameter::Other(items) => items.first().map(|s| s.as_str()),
        }
    }

    pub fn size(&self) -> usize {
        match self {
            ArchivedICalendarParameter::Altrep(s) => s.size(),
            ArchivedICalendarParameter::Cn(s) => s.len(),
            ArchivedICalendarParameter::Cutype(s) => s.as_str().len(),
            ArchivedICalendarParameter::DelegatedFrom(v) => v.iter().map(|s| s.size()).sum(),
            ArchivedICalendarParameter::DelegatedTo(v) => v.iter().map(|s| s.size()).sum(),
            ArchivedICalendarParameter::Dir(s) => s.size(),
            ArchivedICalendarParameter::Fmttype(s) => s.len(),
            ArchivedICalendarParameter::Fbtype(s) => s.as_str().len(),
            ArchivedICalendarParameter::Language(s) => s.len(),
            ArchivedICalendarParameter::Member(v) => v.iter().map(|s| s.size()).sum(),
            ArchivedICalendarParameter::Partstat(s) => s.as_str().len(),
            ArchivedICalendarParameter::Related(ref r) => r.as_str().len(),
            ArchivedICalendarParameter::Reltype(ref r) => r.as_str().len(),
            ArchivedICalendarParameter::Role(ref r) => r.as_str().len(),
            ArchivedICalendarParameter::ScheduleAgent(ref a) => a.as_str().len(),
            ArchivedICalendarParameter::ScheduleForceSend(ref a) => a.as_str().len(),
            ArchivedICalendarParameter::ScheduleStatus(ref a) => a.len(),
            ArchivedICalendarParameter::SentBy(ref u) => u.size(),
            ArchivedICalendarParameter::Tzid(ref t) => t.len(),
            ArchivedICalendarParameter::Value(ref t) => t.as_str().len(),
            ArchivedICalendarParameter::Display(ref d) => d.iter().map(|s| s.as_str().len()).sum(),
            ArchivedICalendarParameter::Email(ref e) => e.len(),
            ArchivedICalendarParameter::Feature(ref f) => f.iter().map(|s| s.as_str().len()).sum(),
            ArchivedICalendarParameter::Label(ref l) => l.len(),
            ArchivedICalendarParameter::Filename(s) => s.as_str().len(),
            ArchivedICalendarParameter::ManagedId(s) => s.as_str().len(),
            ArchivedICalendarParameter::Schema(s) => s.size(),
            ArchivedICalendarParameter::Linkrel(ref l) => l.size(),
            ArchivedICalendarParameter::Other(ref o) => o.iter().map(|s| s.len()).sum(),
            _ => std::mem::size_of::<ICalendarParameter>(),
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
