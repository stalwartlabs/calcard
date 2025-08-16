/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

use super::*;
use crate::common::{ArchivedData, ArchivedPartialDateTime};

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

    pub fn version(&self) -> Option<VCardVersion> {
        self.entries
            .iter()
            .find(|e| e.name == VCardProperty::Version)
            .and_then(|e| {
                e.values
                    .first()
                    .and_then(|v| v.as_text())
                    .and_then(VCardVersion::try_parse)
            })
    }
}

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
            VCardParameterName::Jscomps => matches!(self, ArchivedVCardParameter::Jscomps(_)),
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
            | ArchivedVCardParameter::Derived(_)
            | ArchivedVCardParameter::Jscomps(_) => None,
        }
    }
}
