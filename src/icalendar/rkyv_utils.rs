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

    pub fn is_timezone(&self) -> bool {
        self.components
            .iter()
            .filter(|comp| {
                matches!(
                    comp.component_type,
                    ArchivedICalendarComponentType::VTimezone
                )
            })
            .count()
            == 1
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
    pub fn size(&self) -> usize {
        match self {
            ArchivedUri::Data(data) => {
                data.data.len() + data.content_type.as_ref().map(|s| s.len()).unwrap_or(0)
            }
            ArchivedUri::Location(loc) => loc.len(),
        }
    }
}
