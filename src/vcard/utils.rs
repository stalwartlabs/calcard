/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

use super::{VCard, VCardEntry, VCardParameterName, VCardProperty, VCardValue, VCardVersion};
use crate::{
    common::{
        CalendarScale, Data, IanaString, IanaType, PartialDateTime,
        parser::{Boolean, Timestamp},
        writer::{write_bytes, write_jscomps},
    },
    vcard::{
        Jscomp, VCardLevel, VCardParameter, VCardParameterValue, VCardPhonetic, VCardType,
        VCardValueType,
    },
};
use std::borrow::Cow;

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
    #[inline]
    pub fn parameters(
        &self,
        prop: &VCardParameterName,
    ) -> impl Iterator<Item = &VCardParameterValue> {
        self.params.iter().filter_map(move |param| {
            if &param.name == prop {
                Some(&param.value)
            } else {
                None
            }
        })
    }

    pub fn language(&self) -> Option<&str> {
        self.parameters(&VCardParameterName::Language)
            .find_map(|v| v.as_text())
    }

    pub fn alt_id(&self) -> Option<&str> {
        self.parameters(&VCardParameterName::Altid)
            .find_map(|v| v.as_text())
    }

    pub fn prop_id(&self) -> Option<&str> {
        self.parameters(&VCardParameterName::PropId)
            .find_map(|v| v.as_text())
    }

    pub fn phonetic_system(&self) -> Option<IanaType<&VCardPhonetic, &str>> {
        self.parameters(&VCardParameterName::Phonetic)
            .find_map(|v| v.as_phonetic())
    }

    pub fn phonetic_script(&self) -> Option<&str> {
        self.parameters(&VCardParameterName::Script)
            .find_map(|v| v.as_text())
    }
}

impl VCardValue {
    pub fn as_text(&self) -> Option<&str> {
        match self {
            VCardValue::Text(v) => v.as_str().into(),
            VCardValue::Sex(v) => v.as_str().into(),
            VCardValue::GramGender(v) => v.as_str().into(),
            VCardValue::Kind(v) => v.as_str().into(),
            VCardValue::Component(v) => v.first().map(|s| s.as_str()),
            VCardValue::Integer(_)
            | VCardValue::Float(_)
            | VCardValue::Boolean(_)
            | VCardValue::PartialDateTime(_)
            | VCardValue::Binary(_) => None,
        }
    }

    pub fn into_text(self) -> Option<Cow<'static, str>> {
        match self {
            VCardValue::Text(s) => Some(Cow::Owned(s)),
            VCardValue::Sex(v) => Some(Cow::Borrowed(v.as_str())),
            VCardValue::GramGender(v) => Some(Cow::Borrowed(v.as_str())),
            VCardValue::Kind(v) => Some(Cow::Borrowed(v.as_str())),
            VCardValue::Component(v) => Some(Cow::Owned(v.join(","))),
            VCardValue::Integer(_)
            | VCardValue::Float(_)
            | VCardValue::Boolean(_)
            | VCardValue::PartialDateTime(_)
            | VCardValue::Binary(_) => None,
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
    pub fn new(name: VCardParameterName, value: impl Into<VCardParameterValue>) -> Self {
        VCardParameter {
            name,
            value: value.into(),
        }
    }

    pub fn language(value: impl Into<VCardParameterValue>) -> Self {
        VCardParameter {
            name: VCardParameterName::Language,
            value: value.into(),
        }
    }

    pub fn value(value: impl Into<VCardParameterValue>) -> Self {
        VCardParameter {
            name: VCardParameterName::Value,
            value: value.into(),
        }
    }

    pub fn pref(value: impl Into<VCardParameterValue>) -> Self {
        VCardParameter {
            name: VCardParameterName::Pref,
            value: value.into(),
        }
    }

    pub fn altid(value: impl Into<VCardParameterValue>) -> Self {
        VCardParameter {
            name: VCardParameterName::Altid,
            value: value.into(),
        }
    }

    pub fn pid(value: impl Into<VCardParameterValue>) -> Self {
        VCardParameter {
            name: VCardParameterName::Pid,
            value: value.into(),
        }
    }

    pub fn typ(value: impl Into<VCardParameterValue>) -> Self {
        VCardParameter {
            name: VCardParameterName::Type,
            value: value.into(),
        }
    }

    pub fn mediatype(value: impl Into<VCardParameterValue>) -> Self {
        VCardParameter {
            name: VCardParameterName::Mediatype,
            value: value.into(),
        }
    }

    pub fn calscale(value: impl Into<VCardParameterValue>) -> Self {
        VCardParameter {
            name: VCardParameterName::Calscale,
            value: value.into(),
        }
    }

    pub fn sort_as(value: impl Into<VCardParameterValue>) -> Self {
        VCardParameter {
            name: VCardParameterName::SortAs,
            value: value.into(),
        }
    }

    pub fn geo(value: impl Into<VCardParameterValue>) -> Self {
        VCardParameter {
            name: VCardParameterName::Geo,
            value: value.into(),
        }
    }

    pub fn tz(value: impl Into<VCardParameterValue>) -> Self {
        VCardParameter {
            name: VCardParameterName::Tz,
            value: value.into(),
        }
    }

    pub fn index(value: impl Into<VCardParameterValue>) -> Self {
        VCardParameter {
            name: VCardParameterName::Index,
            value: value.into(),
        }
    }

    pub fn level(value: impl Into<VCardParameterValue>) -> Self {
        VCardParameter {
            name: VCardParameterName::Level,
            value: value.into(),
        }
    }

    pub fn group(value: impl Into<VCardParameterValue>) -> Self {
        VCardParameter {
            name: VCardParameterName::Group,
            value: value.into(),
        }
    }

    pub fn cc(value: impl Into<VCardParameterValue>) -> Self {
        VCardParameter {
            name: VCardParameterName::Cc,
            value: value.into(),
        }
    }

    pub fn author(value: impl Into<VCardParameterValue>) -> Self {
        VCardParameter {
            name: VCardParameterName::Author,
            value: value.into(),
        }
    }

    pub fn author_name(value: impl Into<VCardParameterValue>) -> Self {
        VCardParameter {
            name: VCardParameterName::AuthorName,
            value: value.into(),
        }
    }

    pub fn created(value: impl Into<VCardParameterValue>) -> Self {
        VCardParameter {
            name: VCardParameterName::Created,
            value: value.into(),
        }
    }

    pub fn derived(value: impl Into<VCardParameterValue>) -> Self {
        VCardParameter {
            name: VCardParameterName::Derived,
            value: value.into(),
        }
    }

    pub fn label(value: impl Into<VCardParameterValue>) -> Self {
        VCardParameter {
            name: VCardParameterName::Label,
            value: value.into(),
        }
    }

    pub fn phonetic(value: impl Into<VCardParameterValue>) -> Self {
        VCardParameter {
            name: VCardParameterName::Phonetic,
            value: value.into(),
        }
    }

    pub fn prop_id(value: impl Into<VCardParameterValue>) -> Self {
        VCardParameter {
            name: VCardParameterName::PropId,
            value: value.into(),
        }
    }

    pub fn script(value: impl Into<VCardParameterValue>) -> Self {
        VCardParameter {
            name: VCardParameterName::Script,
            value: value.into(),
        }
    }

    pub fn service_type(value: impl Into<VCardParameterValue>) -> Self {
        VCardParameter {
            name: VCardParameterName::ServiceType,
            value: value.into(),
        }
    }

    pub fn username(value: impl Into<VCardParameterValue>) -> Self {
        VCardParameter {
            name: VCardParameterName::Username,
            value: value.into(),
        }
    }

    pub fn jsptr(value: impl Into<VCardParameterValue>) -> Self {
        VCardParameter {
            name: VCardParameterName::Jsptr,
            value: value.into(),
        }
    }

    pub fn jscomps(value: impl Into<VCardParameterValue>) -> Self {
        VCardParameter {
            name: VCardParameterName::Jscomps,
            value: value.into(),
        }
    }
}

impl VCardParameterValue {
    pub fn into_text(self) -> Cow<'static, str> {
        match self {
            VCardParameterValue::Text(s) => Cow::Owned(s),
            VCardParameterValue::ValueType(v) => Cow::Borrowed(v.as_str()),
            VCardParameterValue::Type(v) => Cow::Borrowed(v.as_str()),
            VCardParameterValue::Calscale(v) => Cow::Borrowed(v.as_str()),
            VCardParameterValue::Level(v) => Cow::Borrowed(v.as_str()),
            VCardParameterValue::Phonetic(v) => Cow::Borrowed(v.as_str()),
            VCardParameterValue::Jscomps(v) => {
                let mut jscomps = String::new();
                let _ = write_jscomps(&mut jscomps, &mut 0, &v);
                Cow::Owned(jscomps)
            }
            VCardParameterValue::Integer(i) => Cow::Owned(i.to_string()),
            VCardParameterValue::Timestamp(t) => Cow::Owned(t.to_string()),
            VCardParameterValue::Bool(b) => Cow::Borrowed(if b { "true" } else { "false" }),
            VCardParameterValue::Null => Cow::Borrowed(""),
        }
    }

    pub fn as_text(&self) -> Option<&str> {
        match self {
            VCardParameterValue::Text(v) => Some(v.as_str()),
            VCardParameterValue::ValueType(v) => Some(v.as_str()),
            VCardParameterValue::Type(v) => Some(v.as_str()),
            VCardParameterValue::Calscale(v) => Some(v.as_str()),
            VCardParameterValue::Level(v) => Some(v.as_str()),
            VCardParameterValue::Phonetic(v) => Some(v.as_str()),
            VCardParameterValue::Jscomps(_)
            | VCardParameterValue::Integer(_)
            | VCardParameterValue::Timestamp(_)
            | VCardParameterValue::Bool(_)
            | VCardParameterValue::Null => None,
        }
    }

    pub fn into_phonetic(self) -> Option<IanaType<VCardPhonetic, String>> {
        match self {
            VCardParameterValue::Phonetic(v) => Some(IanaType::Iana(v)),
            VCardParameterValue::Text(v) => Some(IanaType::Other(v)),
            _ => None,
        }
    }

    pub fn as_phonetic(&self) -> Option<IanaType<&VCardPhonetic, &str>> {
        match self {
            VCardParameterValue::Phonetic(v) => Some(IanaType::Iana(v)),
            VCardParameterValue::Text(v) => Some(IanaType::Other(v.as_str())),
            _ => None,
        }
    }

    pub fn into_calscale(self) -> Option<IanaType<CalendarScale, String>> {
        match self {
            VCardParameterValue::Calscale(v) => Some(IanaType::Iana(v)),
            VCardParameterValue::Text(v) => Some(IanaType::Other(v)),
            _ => None,
        }
    }

    pub fn as_calscale(&self) -> Option<IanaType<&CalendarScale, &str>> {
        match self {
            VCardParameterValue::Calscale(v) => Some(IanaType::Iana(v)),
            VCardParameterValue::Text(v) => Some(IanaType::Other(v.as_str())),
            _ => None,
        }
    }

    pub fn into_level(self) -> Option<IanaType<VCardLevel, String>> {
        match self {
            VCardParameterValue::Level(v) => Some(IanaType::Iana(v)),
            VCardParameterValue::Text(v) => Some(IanaType::Other(v)),
            _ => None,
        }
    }

    pub fn as_level(&self) -> Option<IanaType<&VCardLevel, &str>> {
        match self {
            VCardParameterValue::Level(v) => Some(IanaType::Iana(v)),
            VCardParameterValue::Text(v) => Some(IanaType::Other(v.as_str())),
            _ => None,
        }
    }

    pub fn into_type(self) -> Option<IanaType<VCardType, String>> {
        match self {
            VCardParameterValue::Type(v) => Some(IanaType::Iana(v)),
            VCardParameterValue::Text(v) => Some(IanaType::Other(v)),
            _ => None,
        }
    }

    pub fn as_type(&self) -> Option<IanaType<&VCardType, &str>> {
        match self {
            VCardParameterValue::Type(v) => Some(IanaType::Iana(v)),
            VCardParameterValue::Text(v) => Some(IanaType::Other(v.as_str())),
            _ => None,
        }
    }

    pub fn into_value_type(self) -> Option<IanaType<VCardValueType, String>> {
        match self {
            VCardParameterValue::ValueType(v) => Some(IanaType::Iana(v)),
            VCardParameterValue::Text(v) => Some(IanaType::Other(v)),
            _ => None,
        }
    }

    pub fn as_value_type(&self) -> Option<IanaType<&VCardValueType, &str>> {
        match self {
            VCardParameterValue::ValueType(v) => Some(IanaType::Iana(v)),
            VCardParameterValue::Text(v) => Some(IanaType::Other(v.as_str())),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<IanaType<bool, &str>> {
        match self {
            VCardParameterValue::Bool(b) => Some(IanaType::Iana(*b)),
            VCardParameterValue::Text(v) => Some(IanaType::Other(v.as_str())),
            _ => None,
        }
    }

    pub fn as_integer(&self) -> Option<IanaType<u32, &str>> {
        match self {
            VCardParameterValue::Integer(i) => Some(IanaType::Iana(*i)),
            VCardParameterValue::Text(v) => Some(IanaType::Other(v.as_str())),
            _ => None,
        }
    }

    pub fn into_timestamp(self) -> Option<IanaType<i64, String>> {
        match self {
            VCardParameterValue::Timestamp(t) => Some(IanaType::Iana(t)),
            VCardParameterValue::Text(v) => Some(IanaType::Other(v)),
            _ => None,
        }
    }

    pub fn as_timestamp(&self) -> Option<IanaType<i64, &str>> {
        match self {
            VCardParameterValue::Timestamp(t) => Some(IanaType::Iana(*t)),
            VCardParameterValue::Text(v) => Some(IanaType::Other(v.as_str())),
            _ => None,
        }
    }

    pub fn into_jscomps(self) -> Option<Vec<Jscomp>> {
        match self {
            VCardParameterValue::Jscomps(v) => Some(v),
            _ => None,
        }
    }
}

impl From<String> for VCardParameterValue {
    fn from(value: String) -> Self {
        VCardParameterValue::Text(value)
    }
}

impl From<u32> for VCardParameterValue {
    fn from(value: u32) -> Self {
        VCardParameterValue::Integer(value)
    }
}

impl From<VCardType> for VCardParameterValue {
    fn from(value: VCardType) -> Self {
        VCardParameterValue::Type(value)
    }
}

impl From<VCardValueType> for VCardParameterValue {
    fn from(value: VCardValueType) -> Self {
        VCardParameterValue::ValueType(value)
    }
}

impl From<CalendarScale> for VCardParameterValue {
    fn from(value: CalendarScale) -> Self {
        VCardParameterValue::Calscale(value)
    }
}

impl From<VCardPhonetic> for VCardParameterValue {
    fn from(value: VCardPhonetic) -> Self {
        VCardParameterValue::Phonetic(value)
    }
}

impl From<VCardLevel> for VCardParameterValue {
    fn from(value: VCardLevel) -> Self {
        VCardParameterValue::Level(value)
    }
}

impl From<bool> for VCardParameterValue {
    fn from(value: bool) -> Self {
        VCardParameterValue::Bool(value)
    }
}

impl From<Vec<Jscomp>> for VCardParameterValue {
    fn from(value: Vec<Jscomp>) -> Self {
        VCardParameterValue::Jscomps(value)
    }
}

impl From<Timestamp> for VCardParameterValue {
    fn from(value: Timestamp) -> Self {
        VCardParameterValue::Timestamp(value.0)
    }
}

impl From<Boolean> for VCardParameterValue {
    fn from(value: Boolean) -> Self {
        VCardParameterValue::Bool(value.0)
    }
}

impl<T: Into<VCardParameterValue>> From<IanaType<T, String>> for VCardParameterValue {
    fn from(value: IanaType<T, String>) -> Self {
        match value {
            IanaType::Iana(v) => v.into(),
            IanaType::Other(s) => s.into(),
        }
    }
}
