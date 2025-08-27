/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

use super::*;
use crate::common::{ArchivedCalendarScale, ArchivedData, ArchivedPartialDateTime};

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

impl ArchivedVCardEntry {
    #[inline]
    pub fn parameters(
        &self,
        prop: &VCardParameterName,
    ) -> impl Iterator<Item = &ArchivedVCardParameterValue> {
        self.params.iter().filter_map(move |param| {
            if &param.name == prop {
                Some(&param.value)
            } else {
                None
            }
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

impl ArchivedVCardParameterName {
    pub fn as_str(&self) -> &str {
        match self {
            ArchivedVCardParameterName::Language => "LANGUAGE",
            ArchivedVCardParameterName::Value => "VALUE",
            ArchivedVCardParameterName::Pref => "PREF",
            ArchivedVCardParameterName::Altid => "ALTID",
            ArchivedVCardParameterName::Pid => "PID",
            ArchivedVCardParameterName::Type => "TYPE",
            ArchivedVCardParameterName::Mediatype => "MEDIATYPE",
            ArchivedVCardParameterName::Calscale => "CALSCALE",
            ArchivedVCardParameterName::SortAs => "SORT-AS",
            ArchivedVCardParameterName::Geo => "GEO",
            ArchivedVCardParameterName::Tz => "TZ",
            ArchivedVCardParameterName::Index => "INDEX",
            ArchivedVCardParameterName::Level => "LEVEL",
            ArchivedVCardParameterName::Group => "GROUP",
            ArchivedVCardParameterName::Cc => "CC",
            ArchivedVCardParameterName::Author => "AUTHOR",
            ArchivedVCardParameterName::AuthorName => "AUTHOR-NAME",
            ArchivedVCardParameterName::Created => "CREATED",
            ArchivedVCardParameterName::Derived => "DERIVED",
            ArchivedVCardParameterName::Label => "LABEL",
            ArchivedVCardParameterName::Phonetic => "PHONETIC",
            ArchivedVCardParameterName::PropId => "PROP-ID",
            ArchivedVCardParameterName::Script => "SCRIPT",
            ArchivedVCardParameterName::ServiceType => "SERVICE-TYPE",
            ArchivedVCardParameterName::Username => "USERNAME",
            ArchivedVCardParameterName::Jsptr => "JSPTR",
            ArchivedVCardParameterName::Jscomps => "JSCOMPS",
            ArchivedVCardParameterName::Other(name) => name.as_str(),
        }
    }
}

impl ArchivedVCardParameterValue {
    pub fn as_text(&self) -> Option<&str> {
        match self {
            ArchivedVCardParameterValue::Text(v) => Some(v.as_str()),
            ArchivedVCardParameterValue::ValueType(v) => Some(v.as_str()),
            ArchivedVCardParameterValue::Type(v) => Some(v.as_str()),
            ArchivedVCardParameterValue::Calscale(v) => Some(v.as_str()),
            ArchivedVCardParameterValue::Level(v) => Some(v.as_str()),
            ArchivedVCardParameterValue::Phonetic(v) => Some(v.as_str()),
            ArchivedVCardParameterValue::Jscomps(_)
            | ArchivedVCardParameterValue::Integer(_)
            | ArchivedVCardParameterValue::Timestamp(_)
            | ArchivedVCardParameterValue::Bool(_)
            | ArchivedVCardParameterValue::Null => None,
        }
    }

    pub fn as_phonetic(&self) -> Option<IanaType<&ArchivedVCardPhonetic, &str>> {
        match self {
            ArchivedVCardParameterValue::Phonetic(v) => Some(IanaType::Iana(v)),
            ArchivedVCardParameterValue::Text(v) => Some(IanaType::Other(v.as_str())),
            _ => None,
        }
    }

    pub fn as_calscale(&self) -> Option<IanaType<&ArchivedCalendarScale, &str>> {
        match self {
            ArchivedVCardParameterValue::Calscale(v) => Some(IanaType::Iana(v)),
            ArchivedVCardParameterValue::Text(v) => Some(IanaType::Other(v.as_str())),
            _ => None,
        }
    }

    pub fn as_level(&self) -> Option<IanaType<&ArchivedVCardLevel, &str>> {
        match self {
            ArchivedVCardParameterValue::Level(v) => Some(IanaType::Iana(v)),
            ArchivedVCardParameterValue::Text(v) => Some(IanaType::Other(v.as_str())),
            _ => None,
        }
    }

    pub fn as_type(&self) -> Option<IanaType<&ArchivedVCardType, &str>> {
        match self {
            ArchivedVCardParameterValue::Type(v) => Some(IanaType::Iana(v)),
            ArchivedVCardParameterValue::Text(v) => Some(IanaType::Other(v.as_str())),
            _ => None,
        }
    }

    pub fn as_value_type(&self) -> Option<IanaType<&ArchivedVCardValueType, &str>> {
        match self {
            ArchivedVCardParameterValue::ValueType(v) => Some(IanaType::Iana(v)),
            ArchivedVCardParameterValue::Text(v) => Some(IanaType::Other(v.as_str())),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<IanaType<bool, &str>> {
        match self {
            ArchivedVCardParameterValue::Bool(b) => Some(IanaType::Iana(*b)),
            ArchivedVCardParameterValue::Text(v) => Some(IanaType::Other(v.as_str())),
            _ => None,
        }
    }

    pub fn as_integer(&self) -> Option<IanaType<u32, &str>> {
        match self {
            ArchivedVCardParameterValue::Integer(i) => Some(IanaType::Iana(i.to_native())),
            ArchivedVCardParameterValue::Text(v) => Some(IanaType::Other(v.as_str())),
            _ => None,
        }
    }

    pub fn as_timestamp(&self) -> Option<IanaType<i64, &str>> {
        match self {
            ArchivedVCardParameterValue::Timestamp(t) => Some(IanaType::Iana(t.to_native())),
            ArchivedVCardParameterValue::Text(v) => Some(IanaType::Other(v.as_str())),
            _ => None,
        }
    }
}
