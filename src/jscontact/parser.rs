/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

use crate::{
    common::{CalendarScale, IanaParse, IdReference},
    jscontact::{
        Context, Feature, JSContact, JSContactGrammaticalGender, JSContactId, JSContactKind,
        JSContactLevel, JSContactPhoneticSystem, JSContactProperty, JSContactRelation,
        JSContactType, JSContactValue,
    },
};
use chrono::DateTime;
use jmap_tools::{Element, JsonPointer, JsonPointerItem, Key, Value};
use std::{borrow::Cow, str::FromStr};

impl<'x, I, B> JSContact<'x, I, B>
where
    I: JSContactId,
    B: JSContactId,
{
    pub fn parse(json: &'x str) -> Result<Self, String> {
        Value::parse_json(json).map(JSContact)
    }

    pub fn to_string_pretty(&self) -> String {
        serde_json::to_string_pretty(&self.0).unwrap_or_default()
    }
}

impl<I, B> Element for JSContactValue<I, B>
where
    I: JSContactId,
    B: JSContactId,
{
    type Property = JSContactProperty<I>;

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
                JSContactProperty::BlobId => match IdReference::parse(value) {
                    IdReference::Value(value) => JSContactValue::BlobId(value).into(),
                    IdReference::Reference(value) => JSContactValue::IdReference(value).into(),
                    IdReference::Error => None,
                },
                JSContactProperty::Id => match IdReference::parse(value) {
                    IdReference::Value(value) => JSContactValue::Id(value).into(),
                    IdReference::Reference(value) => JSContactValue::IdReference(value).into(),
                    IdReference::Error => None,
                },
                _ => None,
            }
        } else {
            None
        }
    }

    fn to_cow(&self) -> Cow<'static, str> {
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
            JSContactValue::CalendarScale(v) => v.as_js_str().into(),
            JSContactValue::Id(v) => v.to_string().into(),
            JSContactValue::BlobId(v) => v.to_string().into(),
            JSContactValue::IdReference(s) => format!("#{}", s).into(),
        }
    }
}

impl<I: JSContactId> jmap_tools::Property for JSContactProperty<I> {
    fn try_parse(key: Option<&Key<'_, Self>>, value: &str) -> Option<Self> {
        match key {
            Some(Key::Property(key)) => match key.patch_or_prop() {
                JSContactProperty::Contexts => Context::from_str(value)
                    .ok()
                    .map(JSContactProperty::Context),
                JSContactProperty::Features => Feature::from_str(value)
                    .ok()
                    .map(JSContactProperty::Feature),
                JSContactProperty::SortAs => JSContactKind::from_str(value)
                    .ok()
                    .map(JSContactProperty::SortAsKind),
                JSContactProperty::ConvertedProperties | JSContactProperty::Localizations => {
                    JSContactProperty::Pointer(JsonPointer::parse(value)).into()
                }
                JSContactProperty::AddressBookIds => match IdReference::parse(value) {
                    IdReference::Value(value) => JSContactProperty::IdValue(value).into(),
                    IdReference::Reference(value) => JSContactProperty::IdReference(value).into(),
                    IdReference::Error => None,
                },
                _ => JSContactProperty::from_str(value).ok(),
            },
            None if value.contains('/') => {
                JSContactProperty::Pointer(JsonPointer::parse(value)).into()
            }
            _ => JSContactProperty::from_str(value).ok(),
        }
    }

    fn to_cow(&self) -> Cow<'static, str> {
        self.to_string()
    }
}

impl<I: JSContactId> JSContactProperty<I> {
    fn patch_or_prop(&self) -> &JSContactProperty<I> {
        if let JSContactProperty::Pointer(ptr) = self
            && let Some(JsonPointerItem::Key(Key::Property(prop))) = ptr.last()
        {
            prop
        } else {
            self
        }
    }
}
