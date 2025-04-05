use crate::common::{Data, PartialDateTime};

use super::{VCard, VCardEntry, VCardParameter, VCardParameterName, VCardProperty, VCardValue};

#[cfg(feature = "rkyv")]
use super::{ArchivedVCard, ArchivedVCardEntry, ArchivedVCardParameter, ArchivedVCardValue};

#[cfg(feature = "rkyv")]
use crate::common::{ArchivedData, ArchivedPartialDateTime};

impl VCard {
    pub fn uid(&self) -> Option<&str> {
        self.property(&VCardProperty::Uid)
            .and_then(|e| e.values.first())
            .and_then(|v| v.as_text())
    }

    pub fn property(&self, prop: &VCardProperty) -> Option<&VCardEntry> {
        self.entries.iter().find(|entry| &entry.name == prop)
    }

    pub fn properties<'x, 'y: 'x>(
        &'x self,
        prop: &'y VCardProperty,
    ) -> impl Iterator<Item = &'x VCardEntry> + 'x {
        self.entries.iter().filter(move |entry| &entry.name == prop)
    }
}

#[cfg(feature = "rkyv")]
impl ArchivedVCard {
    pub fn uid(&self) -> Option<&str> {
        self.property(&VCardProperty::Uid)
            .and_then(|e| e.values.first())
            .and_then(|v| v.as_text())
    }

    pub fn property(&self, prop: &VCardProperty) -> Option<&ArchivedVCardEntry> {
        self.entries.iter().find(|entry| &entry.name == prop)
    }

    pub fn properties<'x, 'y: 'x>(
        &'x self,
        prop: &'y VCardProperty,
    ) -> impl Iterator<Item = &'x ArchivedVCardEntry> + 'x {
        self.entries.iter().filter(move |entry| &entry.name == prop)
    }
}

impl VCardValue {
    pub fn as_text(&self) -> Option<&str> {
        match self {
            VCardValue::Text(ref s) => Some(s),
            _ => None,
        }
    }

    pub fn as_integer(&self) -> Option<i64> {
        match self {
            VCardValue::Integer(ref i) => Some(*i),
            _ => None,
        }
    }

    pub fn as_float(&self) -> Option<f64> {
        match self {
            VCardValue::Float(ref f) => Some(*f),
            _ => None,
        }
    }

    pub fn as_boolean(&self) -> Option<bool> {
        match self {
            VCardValue::Boolean(ref b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_partial_date_time(&self) -> Option<&PartialDateTime> {
        match self {
            VCardValue::PartialDateTime(ref dt) => Some(dt),
            _ => None,
        }
    }

    pub fn as_binary(&self) -> Option<&Data> {
        match self {
            VCardValue::Binary(ref d) => Some(d),
            _ => None,
        }
    }
}

#[cfg(feature = "rkyv")]
impl ArchivedVCardValue {
    pub fn as_text(&self) -> Option<&str> {
        match self {
            ArchivedVCardValue::Text(ref s) => Some(s),
            _ => None,
        }
    }

    pub fn as_integer(&self) -> Option<i64> {
        match self {
            ArchivedVCardValue::Integer(ref i) => Some(i.to_native()),
            _ => None,
        }
    }

    pub fn as_float(&self) -> Option<f64> {
        match self {
            ArchivedVCardValue::Float(ref f) => Some(f.to_native()),
            _ => None,
        }
    }

    pub fn as_boolean(&self) -> Option<bool> {
        match self {
            ArchivedVCardValue::Boolean(ref b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_partial_date_time(&self) -> Option<&ArchivedPartialDateTime> {
        match self {
            ArchivedVCardValue::PartialDateTime(ref dt) => Some(dt),
            _ => None,
        }
    }

    pub fn as_binary(&self) -> Option<&ArchivedData> {
        match self {
            ArchivedVCardValue::Binary(ref d) => Some(d),
            _ => None,
        }
    }
}

impl VCardParameter {
    pub fn matches_name(&self, name: &VCardParameterName) -> bool {
        match name {
            VCardParameterName::Language => matches!(self, VCardParameter::Language(_)),
            VCardParameterName::Value => matches!(self, VCardParameter::Value(_)),
            VCardParameterName::Pref => matches!(self, VCardParameter::Pref(_)),
            VCardParameterName::Altid => matches!(self, VCardParameter::Altid(_)),
            VCardParameterName::Pid => matches!(self, VCardParameter::Pid(_)),
            VCardParameterName::Type => matches!(self, VCardParameter::Type(_)),
            VCardParameterName::Mediatype => matches!(self, VCardParameter::Mediatype(_)),
            VCardParameterName::Calscale => matches!(self, VCardParameter::Calscale(_)),
            VCardParameterName::SortAs => matches!(self, VCardParameter::SortAs(_)),
            VCardParameterName::Geo => matches!(self, VCardParameter::Geo(_)),
            VCardParameterName::Tz => matches!(self, VCardParameter::Tz(_)),
            VCardParameterName::Index => matches!(self, VCardParameter::Index(_)),
            VCardParameterName::Level => matches!(self, VCardParameter::Level(_)),
            VCardParameterName::Group => matches!(self, VCardParameter::Group(_)),
            VCardParameterName::Cc => matches!(self, VCardParameter::Cc(_)),
            VCardParameterName::Author => matches!(self, VCardParameter::Author(_)),
            VCardParameterName::AuthorName => matches!(self, VCardParameter::AuthorName(_)),
            VCardParameterName::Created => matches!(self, VCardParameter::Created(_)),
            VCardParameterName::Derived => matches!(self, VCardParameter::Derived(_)),
            VCardParameterName::Label => matches!(self, VCardParameter::Label(_)),
            VCardParameterName::Phonetic => matches!(self, VCardParameter::Phonetic(_)),
            VCardParameterName::PropId => matches!(self, VCardParameter::PropId(_)),
            VCardParameterName::Script => matches!(self, VCardParameter::Script(_)),
            VCardParameterName::ServiceType => matches!(self, VCardParameter::ServiceType(_)),
            VCardParameterName::Username => matches!(self, VCardParameter::Username(_)),
            VCardParameterName::Jsptr => matches!(self, VCardParameter::Jsptr(_)),
            VCardParameterName::Other(ref s) => {
                if let VCardParameter::Other(ref v) = self {
                    v.first().is_some_and(|x| x == s)
                } else {
                    false
                }
            }
        }
    }

    pub fn as_text(&self) -> Option<&str> {
        match self {
            VCardParameter::Language(ref s) => Some(s),
            VCardParameter::Altid(ref s) => Some(s),
            VCardParameter::Pid(ref s) => s.first().map(|x| x.as_str()),
            VCardParameter::Mediatype(ref s) => Some(s),
            VCardParameter::Calscale(ref s) => Some(s.as_str()),
            VCardParameter::SortAs(ref s) => Some(s),
            VCardParameter::Geo(ref s) => Some(s),
            VCardParameter::Tz(ref s) => Some(s),
            VCardParameter::Level(ref s) => Some(s.as_str()),
            VCardParameter::Group(ref s) => Some(s),
            VCardParameter::Cc(ref s) => Some(s),
            VCardParameter::Author(ref s) => Some(s),
            VCardParameter::AuthorName(ref s) => Some(s),
            VCardParameter::Label(ref s) => Some(s),
            VCardParameter::Phonetic(ref s) => Some(s.as_str()),
            VCardParameter::PropId(ref s) => Some(s),
            VCardParameter::Script(ref s) => Some(s),
            VCardParameter::ServiceType(ref s) => Some(s),
            VCardParameter::Username(ref s) => Some(s),
            VCardParameter::Jsptr(ref s) => Some(s),
            VCardParameter::Other(items) => items.get(1).map(|x| x.as_str()),
            VCardParameter::Value(_)
            | VCardParameter::Pref(_)
            | VCardParameter::Type(_)
            | VCardParameter::Index(_)
            | VCardParameter::Created(_)
            | VCardParameter::Derived(_) => None,
        }
    }
}

#[cfg(feature = "rkyv")]
impl ArchivedVCardParameter {
    pub fn matches_name(&self, name: &VCardParameterName) -> bool {
        match name {
            VCardParameterName::Language => matches!(self, ArchivedVCardParameter::Language(_)),
            VCardParameterName::Value => matches!(self, ArchivedVCardParameter::Value(_)),
            VCardParameterName::Pref => matches!(self, ArchivedVCardParameter::Pref(_)),
            VCardParameterName::Altid => matches!(self, ArchivedVCardParameter::Altid(_)),
            VCardParameterName::Pid => matches!(self, ArchivedVCardParameter::Pid(_)),
            VCardParameterName::Type => matches!(self, ArchivedVCardParameter::Type(_)),
            VCardParameterName::Mediatype => matches!(self, ArchivedVCardParameter::Mediatype(_)),
            VCardParameterName::Calscale => matches!(self, ArchivedVCardParameter::Calscale(_)),
            VCardParameterName::SortAs => matches!(self, ArchivedVCardParameter::SortAs(_)),
            VCardParameterName::Geo => matches!(self, ArchivedVCardParameter::Geo(_)),
            VCardParameterName::Tz => matches!(self, ArchivedVCardParameter::Tz(_)),
            VCardParameterName::Index => matches!(self, ArchivedVCardParameter::Index(_)),
            VCardParameterName::Level => matches!(self, ArchivedVCardParameter::Level(_)),
            VCardParameterName::Group => matches!(self, ArchivedVCardParameter::Group(_)),
            VCardParameterName::Cc => matches!(self, ArchivedVCardParameter::Cc(_)),
            VCardParameterName::Author => matches!(self, ArchivedVCardParameter::Author(_)),
            VCardParameterName::AuthorName => matches!(self, ArchivedVCardParameter::AuthorName(_)),
            VCardParameterName::Created => matches!(self, ArchivedVCardParameter::Created(_)),
            VCardParameterName::Derived => matches!(self, ArchivedVCardParameter::Derived(_)),
            VCardParameterName::Label => matches!(self, ArchivedVCardParameter::Label(_)),
            VCardParameterName::Phonetic => matches!(self, ArchivedVCardParameter::Phonetic(_)),
            VCardParameterName::PropId => matches!(self, ArchivedVCardParameter::PropId(_)),
            VCardParameterName::Script => matches!(self, ArchivedVCardParameter::Script(_)),
            VCardParameterName::ServiceType => {
                matches!(self, ArchivedVCardParameter::ServiceType(_))
            }
            VCardParameterName::Username => matches!(self, ArchivedVCardParameter::Username(_)),
            VCardParameterName::Jsptr => matches!(self, ArchivedVCardParameter::Jsptr(_)),
            VCardParameterName::Other(ref s) => {
                if let ArchivedVCardParameter::Other(ref v) = self {
                    v.first().is_some_and(|x| x == s)
                } else {
                    false
                }
            }
        }
    }

    pub fn as_text(&self) -> Option<&str> {
        match self {
            ArchivedVCardParameter::Language(ref s) => Some(s),
            ArchivedVCardParameter::Altid(ref s) => Some(s),
            ArchivedVCardParameter::Pid(ref s) => s.first().map(|x| x.as_str()),
            ArchivedVCardParameter::Mediatype(ref s) => Some(s),
            ArchivedVCardParameter::Calscale(ref s) => Some(s.as_str()),
            ArchivedVCardParameter::SortAs(ref s) => Some(s),
            ArchivedVCardParameter::Geo(ref s) => Some(s),
            ArchivedVCardParameter::Tz(ref s) => Some(s),
            ArchivedVCardParameter::Level(ref s) => Some(s.as_str()),
            ArchivedVCardParameter::Group(ref s) => Some(s),
            ArchivedVCardParameter::Cc(ref s) => Some(s),
            ArchivedVCardParameter::Author(ref s) => Some(s),
            ArchivedVCardParameter::AuthorName(ref s) => Some(s),
            ArchivedVCardParameter::Label(ref s) => Some(s),
            ArchivedVCardParameter::Phonetic(ref s) => Some(s.as_str()),
            ArchivedVCardParameter::PropId(ref s) => Some(s),
            ArchivedVCardParameter::Script(ref s) => Some(s),
            ArchivedVCardParameter::ServiceType(ref s) => Some(s),
            ArchivedVCardParameter::Username(ref s) => Some(s),
            ArchivedVCardParameter::Jsptr(ref s) => Some(s),
            ArchivedVCardParameter::Other(items) => items.get(1).map(|x| x.as_str()),
            ArchivedVCardParameter::Value(_)
            | ArchivedVCardParameter::Pref(_)
            | ArchivedVCardParameter::Type(_)
            | ArchivedVCardParameter::Index(_)
            | ArchivedVCardParameter::Created(_)
            | ArchivedVCardParameter::Derived(_) => None,
        }
    }
}
