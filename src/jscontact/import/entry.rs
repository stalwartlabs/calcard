/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

use crate::{
    jscontact::{
        import::{EntryState, VCardParams},
        JSContactGrammaticalGender, JSContactKind, JSContactProperty, JSContactType,
        JSContactValue,
    },
    vcard::{
        VCardEntry, VCardGramGender, VCardKind, VCardParameter, VCardParameterName, VCardValue,
        VCardValueType, ValueType,
    },
};
use jmap_tools::{JsonPointer, Key, Map, Value};
use std::borrow::Cow;

#[allow(clippy::wrong_self_convention)]
impl EntryState {
    pub(super) fn new(entry: VCardEntry) -> Self {
        Self {
            entry,
            converted_to: None,
            map_name: false,
        }
    }

    pub(super) fn jcal_parameters(
        &mut self,
        params: &mut VCardParams,
        value_type: &mut Option<VCardValueType>,
    ) {
        if self.entry.params.is_empty() && self.entry.group.is_none() {
            return;
        }
        let (default_type, _) = self.entry.name.default_types();
        let default_type = match default_type {
            ValueType::Vcard(t) => t,
            ValueType::Kind | ValueType::Sex | ValueType::GramGender => VCardValueType::Text,
        };

        for param in std::mem::take(&mut self.entry.params) {
            match param {
                VCardParameter::Language(v) => params
                    .0
                    .entry(VCardParameterName::Language)
                    .or_default()
                    .push(v.into()),
                VCardParameter::Value(v) => {
                    if let Some(v) = v.into_iter().next() {
                        if v != default_type {
                            *value_type = Some(v);
                        }
                    }
                }
                VCardParameter::Pref(v) => params
                    .0
                    .entry(VCardParameterName::Pref)
                    .or_default()
                    .push(v.to_string().into()),
                VCardParameter::Altid(v) => params
                    .0
                    .entry(VCardParameterName::Altid)
                    .or_default()
                    .push(v.into()),
                VCardParameter::Pid(v) => {
                    params
                        .0
                        .entry(VCardParameterName::Pid)
                        .or_default()
                        .extend(v.into_iter().map(Into::into));
                }
                VCardParameter::Type(v) => {
                    params
                        .0
                        .entry(VCardParameterName::Type)
                        .or_default()
                        .extend(v.into_iter().map(|t| Value::from(t.into_string())));
                }
                VCardParameter::Mediatype(v) => params
                    .0
                    .entry(VCardParameterName::Mediatype)
                    .or_default()
                    .push(v.into()),
                VCardParameter::Calscale(v) => params
                    .0
                    .entry(VCardParameterName::Calscale)
                    .or_default()
                    .push(v.into_string().into()),
                VCardParameter::SortAs(v) => params
                    .0
                    .entry(VCardParameterName::SortAs)
                    .or_default()
                    .push(v.into()),
                VCardParameter::Geo(v) => params
                    .0
                    .entry(VCardParameterName::Geo)
                    .or_default()
                    .push(v.into()),
                VCardParameter::Tz(v) => params
                    .0
                    .entry(VCardParameterName::Tz)
                    .or_default()
                    .push(v.into()),
                VCardParameter::Index(v) => params
                    .0
                    .entry(VCardParameterName::Index)
                    .or_default()
                    .push(v.to_string().into()),
                VCardParameter::Level(v) => params
                    .0
                    .entry(VCardParameterName::Level)
                    .or_default()
                    .push(v.as_str().into()),
                VCardParameter::Group(_) => {
                    // The GROUP parameter (Section 7.1 of [RFC7095]) does not convert to JSContact.
                    // It exclusively is for use in jCard and MUST NOT be set in a vCard.
                }
                VCardParameter::Cc(v) => params
                    .0
                    .entry(VCardParameterName::Cc)
                    .or_default()
                    .push(v.into()),
                VCardParameter::Author(v) => params
                    .0
                    .entry(VCardParameterName::Author)
                    .or_default()
                    .push(v.into()),
                VCardParameter::AuthorName(v) => params
                    .0
                    .entry(VCardParameterName::AuthorName)
                    .or_default()
                    .push(v.into()),
                VCardParameter::Created(v) => params
                    .0
                    .entry(VCardParameterName::Created)
                    .or_default()
                    .push(v.into()),
                VCardParameter::Derived(v) => params
                    .0
                    .entry(VCardParameterName::Derived)
                    .or_default()
                    .push(v.to_string().into()),
                VCardParameter::Label(v) => params
                    .0
                    .entry(VCardParameterName::Label)
                    .or_default()
                    .push(v.into()),
                VCardParameter::Phonetic(v) => params
                    .0
                    .entry(VCardParameterName::Phonetic)
                    .or_default()
                    .push(v.into_string().into()),
                VCardParameter::PropId(v) => params
                    .0
                    .entry(VCardParameterName::PropId)
                    .or_default()
                    .push(v.into()),
                VCardParameter::Script(v) => params
                    .0
                    .entry(VCardParameterName::Script)
                    .or_default()
                    .push(v.into()),
                VCardParameter::ServiceType(v) => params
                    .0
                    .entry(VCardParameterName::ServiceType)
                    .or_default()
                    .push(v.into()),
                VCardParameter::Username(v) => params
                    .0
                    .entry(VCardParameterName::Username)
                    .or_default()
                    .push(v.into()),
                VCardParameter::Jsptr(v) => params
                    .0
                    .entry(VCardParameterName::Jsptr)
                    .or_default()
                    .push(v.into()),
                VCardParameter::Other(v) => {
                    if v.len() > 1 {
                        let mut v = v.into_iter();
                        let name = v.next().unwrap();

                        if !name.eq_ignore_ascii_case("group") {
                            params
                                .0
                                .entry(VCardParameterName::Other(name))
                                .or_default()
                                .extend(v.map(Into::into));
                        }
                    }
                }
            }
        }

        if let Some(group) = self.entry.group.take() {
            params
                .0
                .insert(VCardParameterName::Group, vec![group.into()]);
        }
    }

    pub(super) fn set_converted_to(&mut self, converted_to: &[&str]) {
        self.converted_to = Some(JsonPointer::<JSContactProperty>::encode(converted_to));
    }

    pub(super) fn set_map_name(&mut self) {
        self.map_name = true;
    }

    pub(super) fn to_text(&mut self) -> Option<Value<'static, JSContactProperty, JSContactValue>> {
        self.to_string().map(Into::into)
    }

    pub(super) fn to_string(&mut self) -> Option<Cow<'static, str>> {
        let mut values = std::mem::take(&mut self.entry.values).into_iter();
        match values.next()? {
            VCardValue::Text(v) => Some(Cow::Owned(v)),
            VCardValue::Binary(data) => Some(Cow::Owned(data.to_string())),
            VCardValue::Sex(v) => Some(v.as_str().into()),
            VCardValue::GramGender(v) => Some(v.as_str().into()),
            VCardValue::Kind(v) => Some(v.as_str().into()),
            other => {
                self.entry.values.push(other);
                self.entry.values.extend(values);
                None
            }
        }
    }

    pub(super) fn text_parts(&mut self) -> impl Iterator<Item = String> + '_ {
        self.entry
            .values
            .drain(..)
            .filter_map(|value| value.into_text())
    }

    pub(super) fn text_parts_borrowed(&self) -> impl Iterator<Item = &str> + '_ {
        self.entry.values.iter().filter_map(|value| value.as_text())
    }

    pub(super) fn to_kind(&mut self) -> Option<Value<'static, JSContactProperty, JSContactValue>> {
        let mut values = std::mem::take(&mut self.entry.values).into_iter();
        match values.next()? {
            VCardValue::Kind(v) => Value::Element(JSContactValue::Kind(match v {
                VCardKind::Application => JSContactKind::Application,
                VCardKind::Device => JSContactKind::Device,
                VCardKind::Group => JSContactKind::Group,
                VCardKind::Individual => JSContactKind::Individual,
                VCardKind::Location => JSContactKind::Location,
                VCardKind::Org => JSContactKind::Org,
            }))
            .into(),
            VCardValue::Text(v) => Value::Str(v.into()).into(),
            other => {
                self.entry.values.push(other);
                self.entry.values.extend(values);
                None
            }
        }
    }

    pub(super) fn to_gram_gender(
        &mut self,
    ) -> Option<Value<'static, JSContactProperty, JSContactValue>> {
        let mut values = std::mem::take(&mut self.entry.values).into_iter();
        match values.next()? {
            VCardValue::GramGender(v) => {
                Value::Element(JSContactValue::GrammaticalGender(match v {
                    VCardGramGender::Animate => JSContactGrammaticalGender::Animate,
                    VCardGramGender::Common => JSContactGrammaticalGender::Common,
                    VCardGramGender::Feminine => JSContactGrammaticalGender::Feminine,
                    VCardGramGender::Inanimate => JSContactGrammaticalGender::Inanimate,
                    VCardGramGender::Masculine => JSContactGrammaticalGender::Masculine,
                    VCardGramGender::Neuter => JSContactGrammaticalGender::Neuter,
                }))
                .into()
            }
            VCardValue::Text(v) => Value::Str(v.into()).into(),
            other => {
                self.entry.values.push(other);
                self.entry.values.extend(values);
                None
            }
        }
    }

    pub(super) fn to_timestamp(
        &mut self,
    ) -> Option<Value<'static, JSContactProperty, JSContactValue>> {
        if let Some(VCardValue::PartialDateTime(dt)) = self.entry.values.first() {
            if let Some(timestamp) = dt.to_timestamp() {
                return Value::Element(JSContactValue::Timestamp(timestamp)).into();
            }
        }

        None
    }

    pub(super) fn to_tz(&mut self) -> Option<Value<'static, JSContactProperty, JSContactValue>> {
        let mut values = std::mem::take(&mut self.entry.values).into_iter();
        match values.next()? {
            VCardValue::PartialDateTime(v) if v.has_zone() => {
                let hour = v.hour.unwrap_or_default();

                Value::Str(
                    if hour != 0 {
                        format!("Etc/GMT{}{}", if v.tz_minus { "+" } else { "-" }, hour)
                    } else {
                        "Etc/UTC".to_string()
                    }
                    .into(),
                )
                .into()
            }
            VCardValue::Text(v) => Value::Str(v.into()).into(),
            other => {
                self.entry.values.push(other);
                self.entry.values.extend(values);
                None
            }
        }
    }

    pub(super) fn to_anniversary(
        &mut self,
    ) -> Option<Value<'static, JSContactProperty, JSContactValue>> {
        if let Some(VCardValue::PartialDateTime(dt)) = self.entry.values.first() {
            if let Some(timestamp) = dt.to_timestamp() {
                Some(Value::Object(Map::from(vec![
                    (
                        Key::Property(JSContactProperty::Type),
                        Value::Element(JSContactValue::Type(JSContactType::Timestamp)),
                    ),
                    (
                        Key::Property(JSContactProperty::Utc),
                        Value::Element(JSContactValue::Timestamp(timestamp)),
                    ),
                ])))
            } else if dt.day.is_some() || dt.month.is_some() || dt.year.is_some() {
                let mut props = Map::from(vec![(
                    Key::Property(JSContactProperty::Type),
                    Value::Element(JSContactValue::Type(JSContactType::PartialDate)),
                )]);
                if let Some(year) = dt.year {
                    props.insert_unchecked(
                        Key::Property(JSContactProperty::Year),
                        Value::Number((year as u64).into()),
                    );
                }
                if let Some(month) = dt.month {
                    props.insert_unchecked(
                        Key::Property(JSContactProperty::Month),
                        Value::Number((month as u64).into()),
                    );
                }
                if let Some(day) = dt.day {
                    props.insert_unchecked(
                        Key::Property(JSContactProperty::Day),
                        Value::Number((day as u64).into()),
                    );
                }

                Some(Value::Object(props))
            } else {
                None
            }
        } else {
            None
        }
    }

    pub(super) fn non_default_language(&self, default_language: Option<&str>) -> Option<&str> {
        self.entry
            .language()
            .filter(|&lang| Some(lang) != default_language)
    }
}
