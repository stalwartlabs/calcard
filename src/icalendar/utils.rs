#[cfg(feature = "rkyv")]
use super::{
    ArchivedICalendar, ArchivedICalendarComponent, ArchivedICalendarEntry, ArchivedICalendarValue,
};
use super::{ICalendar, ICalendarComponent, ICalendarEntry, ICalendarProperty, ICalendarValue};
#[cfg(feature = "rkyv")]
use crate::common::ArchivedPartialDateTime;
use crate::common::PartialDateTime;

impl ICalendar {
    pub fn uids(&self) -> impl Iterator<Item = &str> {
        self.components
            .iter()
            .filter_map(|component| component.uid())
    }
}

#[cfg(feature = "rkyv")]
impl ArchivedICalendar {
    pub fn uids(&self) -> impl Iterator<Item = &str> {
        self.components
            .iter()
            .filter_map(|component| component.uid())
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
}

#[cfg(feature = "rkyv")]
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
}

impl ICalendarValue {
    pub fn as_text(&self) -> Option<&str> {
        match self {
            ICalendarValue::Text(ref s) => Some(s),
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

#[cfg(feature = "rkyv")]
impl ArchivedICalendarValue {
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
