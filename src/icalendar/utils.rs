use super::{
    ICalendar, ICalendarComponent, ICalendarDuration, ICalendarEntry, ICalendarParameter,
    ICalendarProperty, ICalendarRecurrenceRule, ICalendarValue, Uri,
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
            ICalendarValue::Text(s) => Some(s),
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
        let secs = self.seconds as i64
            + self.minutes as i64 * 60
            + self.hours as i64 * 3600
            + self.days as i64 * 86400
            + self.weeks as i64 * 604800;

        chrono::TimeDelta::new(if self.neg { -secs } else { secs }, 0)
    }
}
