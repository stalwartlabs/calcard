/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

use crate::{
    common::CalendarScale,
    jscontact::{
        Context, Feature, JSContact, JSContactGrammaticalGender, JSContactKind, JSContactLevel,
        JSContactPhoneticSystem, JSContactProperty, JSContactRelation, JSContactType,
        JSContactValue,
    },
};
use chrono::DateTime;
use jmap_tools::{Element, JsonPointer, Key, Value};
use std::{borrow::Cow, str::FromStr};

impl<'x> JSContact<'x> {
    pub fn parse(json: &'x str) -> Result<Self, String> {
        Value::parse_json(json).map(JSContact)
    }
}

impl Element for JSContactValue {
    type Property = JSContactProperty;

    fn try_parse<P>(key: &Key<'_, Self::Property>, value: &str) -> Option<Self> {
        if let Key::Property(prop) = key {
            match prop {
                JSContactProperty::Type => JSContactType::from_str(value)
                    .ok()
                    .map(JSContactValue::Type),
                JSContactProperty::CalendarScale => {
                    CalendarScale::parse(value.as_bytes()).map(JSContactValue::CalendarScale)
                }
                JSContactProperty::Created
                | JSContactProperty::Updated
                | JSContactProperty::Utc => DateTime::parse_from_rfc3339(value)
                    .map(|dt| JSContactValue::Timestamp(dt.timestamp()))
                    .ok(),
                JSContactProperty::Kind => JSContactKind::from_str(value)
                    .ok()
                    .map(JSContactValue::Kind),
                JSContactProperty::GrammaticalGender => JSContactGrammaticalGender::from_str(value)
                    .ok()
                    .map(JSContactValue::GrammaticalGender),
                JSContactProperty::PhoneticSystem => JSContactPhoneticSystem::from_str(value)
                    .ok()
                    .map(JSContactValue::PhoneticSystem),
                JSContactProperty::Relation => JSContactRelation::from_str(value)
                    .ok()
                    .map(JSContactValue::Relation),
                JSContactProperty::Level => JSContactLevel::from_str(value)
                    .ok()
                    .map(JSContactValue::Level),
                _ => None,
            }
        } else {
            None
        }
    }

    fn to_cow(&self) -> Cow<'_, str> {
        match self {
            JSContactValue::Type(v) => v.as_str().into(),
            JSContactValue::GrammaticalGender(v) => v.as_str().into(),
            JSContactValue::Kind(v) => v.as_str().into(),
            JSContactValue::Level(v) => v.as_str().into(),
            JSContactValue::Relation(v) => v.as_str().into(),
            JSContactValue::PhoneticSystem(v) => v.as_str().into(),
            JSContactValue::Timestamp(v) => mail_parser::DateTime::from_timestamp(*v)
                .to_rfc3339()
                .into(),
            JSContactValue::CalendarScale(v) => v.as_str().into(),
        }
    }
}

impl jmap_tools::Property for JSContactProperty {
    fn try_parse(key: Option<&Key<'_, Self>>, value: &str) -> Option<Self> {
        match key {
            Some(Key::Property(JSContactProperty::Contexts)) => Context::from_str(value)
                .ok()
                .map(JSContactProperty::Context),
            Some(Key::Property(JSContactProperty::Features)) => Feature::from_str(value)
                .ok()
                .map(JSContactProperty::Feature),
            Some(Key::Property(JSContactProperty::SortAs)) => JSContactKind::from_str(value)
                .ok()
                .map(JSContactProperty::SortAsKind),
            Some(Key::Property(
                JSContactProperty::ConvertedProperties | JSContactProperty::Localizations,
            )) => JSContactProperty::Pointer(JsonPointer::parse(value)).into(),
            Some(Key::Property(JSContactProperty::Pointer(ptr))) => {
                if let Some(Key::Property(prop)) = ptr.last().and_then(|p| p.as_key()) {
                    match prop {
                        JSContactProperty::Contexts => Context::from_str(value)
                            .ok()
                            .map(JSContactProperty::Context),
                        JSContactProperty::Features => Feature::from_str(value)
                            .ok()
                            .map(JSContactProperty::Feature),
                        JSContactProperty::SortAs => JSContactKind::from_str(value)
                            .ok()
                            .map(JSContactProperty::SortAsKind),
                        _ => JSContactProperty::from_str(value).ok(),
                    }
                } else {
                    JSContactProperty::from_str(value).ok()
                }
            }
            _ => JSContactProperty::from_str(value).ok(),
        }
    }

    fn to_cow(&self) -> Cow<'static, str> {
        self.to_string()
    }
}
