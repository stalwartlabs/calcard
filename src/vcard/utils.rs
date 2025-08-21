/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

use super::{
    VCard, VCardEntry, VCardParameter, VCardParameterName, VCardProperty, VCardValue, VCardVersion,
};
use crate::{
    common::{Data, PartialDateTime, writer::write_bytes},
    vcard::VCardPhonetic,
};

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

impl VCardEntry {
    pub fn language(&self) -> Option<&str> {
        self.params.iter().find_map(|p| match p {
            VCardParameter::Language(lang) => Some(lang.as_str()),
            _ => None,
        })
    }

    pub fn alt_id(&self) -> Option<&str> {
        self.params.iter().find_map(|p| match p {
            VCardParameter::Altid(altid) => Some(altid.as_str()),
            _ => None,
        })
    }

    pub fn prop_id(&self) -> Option<&str> {
        self.params.iter().find_map(|p| match p {
            VCardParameter::PropId(prop_id) => Some(prop_id.as_str()),
            _ => None,
        })
    }

    pub fn phonetic_system(&self) -> Option<&VCardPhonetic> {
        self.params.iter().find_map(|p| match p {
            VCardParameter::Phonetic(script) => Some(script),
            _ => None,
        })
    }

    pub fn phonetic_script(&self) -> Option<&str> {
        self.params.iter().find_map(|p| match p {
            VCardParameter::Script(script) => Some(script.as_str()),
            _ => None,
        })
    }
}

impl VCardValue {
    pub fn as_text(&self) -> Option<&str> {
        match self {
            VCardValue::Text(v) => Some(v.as_str()),
            VCardValue::Sex(v) => v.as_str().into(),
            VCardValue::GramGender(v) => v.as_str().into(),
            VCardValue::Kind(v) => v.as_str().into(),
            VCardValue::Component(v) => v.first().map(|s| s.as_str()),
            _ => None,
        }
    }

    pub fn into_text(self) -> Option<String> {
        match self {
            VCardValue::Text(s) => Some(s),
            VCardValue::Sex(v) => v.as_str().to_string().into(),
            VCardValue::GramGender(v) => v.as_str().to_string().into(),
            VCardValue::Kind(v) => v.as_str().to_string().into(),
            VCardValue::Component(v) => Some(v.join(",")),
            _ => None,
        }
    }

    pub fn into_uri(self) -> Option<String> {
        match self {
            VCardValue::Binary(v) => Some(v.to_string()),
            VCardValue::Text(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_integer(&self) -> Option<i64> {
        match self {
            VCardValue::Integer(i) => Some(*i),
            _ => None,
        }
    }

    pub fn as_float(&self) -> Option<f64> {
        match self {
            VCardValue::Float(f) => Some(*f),
            _ => None,
        }
    }

    pub fn as_boolean(&self) -> Option<bool> {
        match self {
            VCardValue::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_partial_date_time(&self) -> Option<&PartialDateTime> {
        match self {
            VCardValue::PartialDateTime(dt) => Some(dt),
            _ => None,
        }
    }

    pub fn as_binary(&self) -> Option<&Data> {
        match self {
            VCardValue::Binary(d) => Some(d),
            _ => None,
        }
    }
}

impl Data {
    #[allow(clippy::inherent_to_string)]
    pub fn to_string(&self) -> String {
        use std::fmt::Write;
        let mut out = String::with_capacity(
            self.data.len().div_ceil(4) + self.content_type.as_ref().map_or(0, |ct| ct.len() + 5),
        );
        let _ = write!(&mut out, "data:");
        if let Some(ct) = &self.content_type {
            let _ = write!(&mut out, "{ct};");
        }
        let _ = write!(&mut out, "base64\\,");
        let _ = write_bytes(&mut out, &mut 0, &self.data);
        out
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
            VCardParameterName::Jscomps => matches!(self, VCardParameter::Jscomps(_)),
            VCardParameterName::Other(s) => {
                if let VCardParameter::Other(v) = self {
                    v.first().is_some_and(|x| x == s)
                } else {
                    false
                }
            }
        }
    }

    pub fn as_text(&self) -> Option<&str> {
        match self {
            VCardParameter::Language(s) => Some(s),
            VCardParameter::Altid(s) => Some(s),
            VCardParameter::Pid(s) => s.first().map(|x| x.as_str()),
            VCardParameter::Mediatype(s) => Some(s),
            VCardParameter::Calscale(s) => Some(s.as_str()),
            VCardParameter::SortAs(s) => Some(s),
            VCardParameter::Geo(s) => Some(s),
            VCardParameter::Tz(s) => Some(s),
            VCardParameter::Level(s) => Some(s.as_str()),
            VCardParameter::Group(s) => Some(s),
            VCardParameter::Cc(s) => Some(s),
            VCardParameter::Author(s) => Some(s),
            VCardParameter::AuthorName(s) => Some(s),
            VCardParameter::Label(s) => Some(s),
            VCardParameter::Phonetic(s) => Some(s.as_str()),
            VCardParameter::PropId(s) => Some(s),
            VCardParameter::Script(s) => Some(s),
            VCardParameter::ServiceType(s) => Some(s),
            VCardParameter::Username(s) => Some(s),
            VCardParameter::Jsptr(s) => Some(s),
            VCardParameter::Other(items) => items.get(1).map(|x| x.as_str()),
            VCardParameter::Value(_)
            | VCardParameter::Pref(_)
            | VCardParameter::Type(_)
            | VCardParameter::Index(_)
            | VCardParameter::Created(_)
            | VCardParameter::Derived(_)
            | VCardParameter::Jscomps(_) => None,
        }
    }
}
