/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

use crate::{
    common::{CalendarScale, IanaParse, IanaType, PartialDateTime, parser::Integer},
    icalendar::Uri,
    jscontact::{
        JSContactGrammaticalGender, JSContactKind, JSContactLevel, JSContactPhoneticSystem,
        JSContactProperty, JSContactValue,
    },
    vcard::{
        VCardGramGender, VCardKind, VCardLevel, VCardPhonetic, VCardSex, VCardType, VCardValue,
        VCardValueType, ValueType,
    },
};
use jmap_tools::{JsonPointerItem, Key, Map, Value};
use std::{borrow::Cow, iter::Peekable, vec::IntoIter};

pub(super) fn build_path<'x>(
    obj: &mut Value<'x, JSContactProperty, JSContactValue>,
    mut ptr: Peekable<IntoIter<JsonPointerItem<JSContactProperty>>>,
    value: Value<'x, JSContactProperty, JSContactValue>,
) -> Option<Value<'x, JSContactProperty, JSContactValue>> {
    if let Some(item) = ptr.next() {
        match item {
            JsonPointerItem::Root | JsonPointerItem::Wildcard => {}
            JsonPointerItem::Key(key) => match obj {
                Value::Object(obj) => {
                    return build_path(
                        obj.insert_or_get_mut(
                            key,
                            if matches!(ptr.peek(), Some(JsonPointerItem::Key(_))) {
                                Value::Object(Map::from(Vec::new()))
                            } else {
                                Value::Array(Vec::new())
                            },
                        ),
                        ptr,
                        value,
                    );
                }
                Value::Null => {
                    *obj = Value::Object(Map::from(vec![(key, Value::Null)]));
                    return build_path(
                        &mut obj
                            .as_object_mut()
                            .unwrap()
                            .as_mut_vec()
                            .last_mut()
                            .unwrap()
                            .1,
                        ptr,
                        value,
                    );
                }
                _ => {}
            },
            JsonPointerItem::Number(idx) => {
                if let Some(arr) = obj.as_array_mut() {
                    if (idx as usize) < arr.len() {
                        return build_path(&mut arr[idx as usize], ptr, value);
                    }

                    if idx < 20 {
                        arr.resize_with(idx as usize + 1, || Value::Null);
                        return build_path(&mut arr[idx as usize], ptr, value);
                    }
                }
            }
        }
        Some(value)
    } else {
        *obj = value;
        None
    }
}

pub(super) fn convert_anniversary(
    value: Value<'_, JSContactProperty, JSContactValue>,
) -> Result<(PartialDateTime, Option<CalendarScale>), Value<'_, JSContactProperty, JSContactValue>>
{
    let mut date = PartialDateTime::default();
    let mut calendar_scale = None;
    let Some(object) = value.as_object() else {
        return Err(value);
    };

    for (key, value) in object.as_vec() {
        match key {
            Key::Property(JSContactProperty::Day) => {
                if let Value::Number(day) = value {
                    date.day = Some(day.cast_to_i64() as u8);
                }
            }
            Key::Property(JSContactProperty::Month) => {
                if let Value::Number(month) = value {
                    date.month = Some(month.cast_to_i64() as u8);
                }
            }
            Key::Property(JSContactProperty::Year) => {
                if let Value::Number(year) = value {
                    date.year = Some(year.cast_to_i64() as u16);
                }
            }
            Key::Property(JSContactProperty::CalendarScale) => {
                if let Value::Element(JSContactValue::CalendarScale(scale)) = value {
                    calendar_scale = Some(scale.clone());
                }
            }
            Key::Property(JSContactProperty::Utc) => {
                if let Value::Element(JSContactValue::Timestamp(timestamp)) = value {
                    return Ok((PartialDateTime::from_utc_timestamp(*timestamp), None));
                }
            }
            _ => {}
        }
    }

    if date.year.is_some() || date.month.is_some() || date.day.is_some() {
        Ok((date, calendar_scale))
    } else {
        Err(value)
    }
}

pub(super) fn convert_value<'x>(
    value: Value<'x, JSContactProperty, JSContactValue>,
    value_type: &'_ ValueType,
) -> Result<VCardValue, Value<'x, JSContactProperty, JSContactValue>> {
    match value {
        Value::Element(e) => match e {
            JSContactValue::Timestamp(t) => Ok(VCardValue::PartialDateTime(
                PartialDateTime::from_utc_timestamp(t),
            )),
            JSContactValue::GrammaticalGender(g) => Ok(VCardValue::GramGender(match g {
                JSContactGrammaticalGender::Animate => VCardGramGender::Animate,
                JSContactGrammaticalGender::Common => VCardGramGender::Common,
                JSContactGrammaticalGender::Feminine => VCardGramGender::Feminine,
                JSContactGrammaticalGender::Inanimate => VCardGramGender::Inanimate,
                JSContactGrammaticalGender::Masculine => VCardGramGender::Masculine,
                JSContactGrammaticalGender::Neuter => VCardGramGender::Neuter,
            })),
            JSContactValue::Kind(k) => match k {
                JSContactKind::Individual => Ok(VCardValue::Kind(VCardKind::Individual)),
                JSContactKind::Group => Ok(VCardValue::Kind(VCardKind::Group)),
                JSContactKind::Location => Ok(VCardValue::Kind(VCardKind::Location)),
                JSContactKind::Org => Ok(VCardValue::Kind(VCardKind::Org)),
                JSContactKind::Application => Ok(VCardValue::Kind(VCardKind::Application)),
                JSContactKind::Device => Ok(VCardValue::Kind(VCardKind::Device)),
                _ => Err(Value::Element(JSContactValue::Kind(k))),
            },
            JSContactValue::Level(_)
            | JSContactValue::Type(_)
            | JSContactValue::Relation(_)
            | JSContactValue::PhoneticSystem(_)
            | JSContactValue::CalendarScale(_) => Err(Value::Element(e)),
        },
        Value::Str(s) => {
            match value_type {
                ValueType::Kind => {
                    if let Some(kind) = VCardKind::parse(s.as_ref().as_bytes()) {
                        return Ok(VCardValue::Kind(kind));
                    }
                }
                ValueType::Sex => {
                    if let Some(sex) = VCardSex::parse(s.as_ref().as_bytes()) {
                        return Ok(VCardValue::Sex(sex));
                    }
                }
                ValueType::GramGender => {
                    if let Some(gender) = VCardGramGender::parse(s.as_ref().as_bytes()) {
                        return Ok(VCardValue::GramGender(gender));
                    }
                }
                ValueType::Vcard(typ) => match typ {
                    VCardValueType::Boolean => {
                        if s.eq_ignore_ascii_case("true") {
                            return Ok(VCardValue::Boolean(true));
                        } else if s.eq_ignore_ascii_case("false") {
                            return Ok(VCardValue::Boolean(false));
                        }
                    }
                    VCardValueType::Date => {
                        let mut dt = PartialDateTime::default();
                        dt.parse_vcard_date(&mut s.as_ref().as_bytes().iter().peekable());
                        if !dt.is_null() {
                            return Ok(VCardValue::PartialDateTime(dt));
                        }
                    }
                    VCardValueType::DateAndOrTime => {
                        let mut dt = PartialDateTime::default();
                        dt.parse_vcard_date_and_or_time(
                            &mut s.as_ref().as_bytes().iter().peekable(),
                        );
                        if !dt.is_null() {
                            return Ok(VCardValue::PartialDateTime(dt));
                        }
                    }
                    VCardValueType::DateTime => {
                        let mut dt = PartialDateTime::default();
                        dt.parse_vcard_date_time(&mut s.as_ref().as_bytes().iter().peekable());
                        if !dt.is_null() {
                            return Ok(VCardValue::PartialDateTime(dt));
                        }
                    }
                    VCardValueType::Time => {
                        let mut dt = PartialDateTime::default();
                        dt.parse_vcard_time(&mut s.as_ref().as_bytes().iter().peekable(), false);
                        if !dt.is_null() {
                            return Ok(VCardValue::PartialDateTime(dt));
                        }
                    }
                    VCardValueType::Timestamp => {
                        let mut dt = PartialDateTime::default();
                        if dt.parse_timestamp(&mut s.as_ref().as_bytes().iter().peekable(), true) {
                            return Ok(VCardValue::PartialDateTime(dt));
                        }
                    }
                    VCardValueType::UtcOffset => {
                        let mut dt = PartialDateTime::default();
                        dt.parse_zone(&mut s.as_ref().as_bytes().iter().peekable());
                        if !dt.is_null() {
                            return Ok(VCardValue::PartialDateTime(dt));
                        }
                    }
                    VCardValueType::Float => {
                        if let Ok(float) = s.as_ref().parse::<f64>() {
                            return Ok(VCardValue::Float(float));
                        }
                    }
                    VCardValueType::Integer => {
                        if let Some(integer) = Integer::parse(s.as_ref().as_bytes()) {
                            return Ok(VCardValue::Integer(integer.0));
                        }
                    }
                    VCardValueType::Uri => {
                        return Ok(match Uri::parse(s) {
                            Uri::Data(data) => VCardValue::Binary(data),
                            Uri::Location(text) => VCardValue::Text(text),
                        });
                    }
                    VCardValueType::LanguageTag | VCardValueType::Text => (),
                },
            }

            Ok(VCardValue::Text(s.into_owned()))
        }
        Value::Bool(b) => Ok(VCardValue::Boolean(b)),
        Value::Number(n) => match n.try_cast_to_i64() {
            Ok(i) => Ok(VCardValue::Integer(i)),
            Err(f) => Ok(VCardValue::Float(f)),
        },
        value => Err(value),
    }
}

pub(super) fn convert_types(
    value: Value<'_, JSContactProperty, JSContactValue>,
    is_context: bool,
) -> Option<Vec<IanaType<VCardType, String>>> {
    let mut types = Vec::new();
    for typ in value.into_expanded_boolean_set() {
        let typ = typ.to_string();
        match VCardType::parse(typ.as_ref().as_bytes()) {
            Some(typ) => types.push(IanaType::Iana(typ)),
            None => {
                if is_context && typ.eq_ignore_ascii_case("private") {
                    types.push(IanaType::Iana(VCardType::Home));
                } else {
                    types.push(IanaType::Other(typ.into_owned()));
                }
            }
        }
    }
    if !types.is_empty() { Some(types) } else { None }
}

pub(super) fn map_kind<T>(
    value: &Value<'_, JSContactProperty, JSContactValue>,
    types: impl IntoIterator<Item = (JSContactKind, T)>,
) -> Option<T> {
    value
        .as_object()
        .and_then(|obj| obj.get(&Key::Property(JSContactProperty::Kind)))
        .and_then(|v| match v {
            Value::Element(JSContactValue::Kind(kind)) => {
                types.into_iter().find_map(|(js_kind, vcard_property)| {
                    if js_kind == *kind {
                        Some(vcard_property)
                    } else {
                        None
                    }
                })
            }

            _ => None,
        })
}

impl TryFrom<Value<'_, JSContactProperty, JSContactValue>> for IanaType<VCardPhonetic, String> {
    type Error = ();
    fn try_from(value: Value<'_, JSContactProperty, JSContactValue>) -> Result<Self, Self::Error> {
        match value {
            Value::Element(JSContactValue::PhoneticSystem(system)) => {
                Ok(IanaType::Iana(match system {
                    JSContactPhoneticSystem::Ipa => VCardPhonetic::Ipa,
                    JSContactPhoneticSystem::Jyut => VCardPhonetic::Jyut,
                    JSContactPhoneticSystem::Piny => VCardPhonetic::Piny,
                    JSContactPhoneticSystem::Script => VCardPhonetic::Script,
                }))
            }
            Value::Str(text) => match VCardPhonetic::parse(text.as_ref().as_bytes()) {
                Some(phonetic) => Ok(IanaType::Iana(phonetic)),
                None => Ok(IanaType::Other(text.into_owned())),
            },
            _ => Err(()),
        }
    }
}

impl TryFrom<Value<'_, JSContactProperty, JSContactValue>> for IanaType<VCardLevel, String> {
    type Error = ();

    fn try_from(value: Value<'_, JSContactProperty, JSContactValue>) -> Result<Self, Self::Error> {
        match value {
            Value::Element(JSContactValue::Level(level)) => Ok(IanaType::Iana(match level {
                JSContactLevel::High => VCardLevel::High,
                JSContactLevel::Low => VCardLevel::Low,
                JSContactLevel::Medium => VCardLevel::Medium,
            })),
            Value::Str(text) => match VCardLevel::parse(text.as_ref().as_bytes()) {
                Some(level) => Ok(IanaType::Iana(level)),
                None => Ok(IanaType::Other(text.into_owned())),
            },
            _ => Err(()),
        }
    }
}

pub(super) fn find_text_param<'x>(
    value: &'x Value<'x, JSContactProperty, JSContactValue>,
    name: &str,
) -> Option<Cow<'x, str>> {
    value
        .as_object()
        .and_then(|obj| obj.get(&Key::Property(JSContactProperty::Parameters)))
        .and_then(|obj| obj.as_object())
        .and_then(|obj| obj.get_ignore_case(name))
        .and_then(|obj| obj.as_str())
}
