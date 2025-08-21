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
            ArchivedVCardValue::Text(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_integer(&self) -> Option<i64> {
        match self {
            ArchivedVCardValue::Integer(i) => Some(i.to_native()),
            _ => None,
        }
    }

    pub fn as_float(&self) -> Option<f64> {
        match self {
            ArchivedVCardValue::Float(f) => Some(f.to_native()),
            _ => None,
        }
    }

    pub fn as_boolean(&self) -> Option<bool> {
        match self {
            ArchivedVCardValue::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_partial_date_time(&self) -> Option<&ArchivedPartialDateTime> {
        match self {
            ArchivedVCardValue::PartialDateTime(dt) => Some(dt),
            _ => None,
        }
    }

    pub fn as_binary(&self) -> Option<&ArchivedData> {
        match self {
            ArchivedVCardValue::Binary(d) => Some(d),
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
            VCardParameterName::Other(s) => {
                if let ArchivedVCardParameter::Other(v) = self {
                    v.first().is_some_and(|x| x == s)
                } else {
                    false
                }
            }
        }
    }

    pub fn as_text(&self) -> Option<&str> {
        match self {
            ArchivedVCardParameter::Language(s) => Some(s),
            ArchivedVCardParameter::Altid(s) => Some(s),
            ArchivedVCardParameter::Pid(s) => s.first().map(|x| x.as_str()),
            ArchivedVCardParameter::Mediatype(s) => Some(s),
            ArchivedVCardParameter::Calscale(s) => Some(s.as_str()),
            ArchivedVCardParameter::SortAs(s) => Some(s),
            ArchivedVCardParameter::Geo(s) => Some(s),
            ArchivedVCardParameter::Tz(s) => Some(s),
            ArchivedVCardParameter::Level(s) => Some(s.as_str()),
            ArchivedVCardParameter::Group(s) => Some(s),
            ArchivedVCardParameter::Cc(s) => Some(s),
            ArchivedVCardParameter::Author(s) => Some(s),
            ArchivedVCardParameter::AuthorName(s) => Some(s),
            ArchivedVCardParameter::Label(s) => Some(s),
            ArchivedVCardParameter::Phonetic(s) => Some(s.as_str()),
            ArchivedVCardParameter::PropId(s) => Some(s),
            ArchivedVCardParameter::Script(s) => Some(s),
            ArchivedVCardParameter::ServiceType(s) => Some(s),
            ArchivedVCardParameter::Username(s) => Some(s),
            ArchivedVCardParameter::Jsptr(s) => Some(s),
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
