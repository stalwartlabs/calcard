/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

use crate::{
    common::IanaType,
    jscontact::{
        JSContactId, JSContactKind, JSContactLevel, JSContactPhoneticSystem, JSContactProperty,
        JSContactValue,
        import::{ExtractedParams, VCardParams},
    },
    vcard::{VCardLevel, VCardPhonetic, VCardProperty, VCardType},
};
use jmap_tools::{Key, Map, Value};

impl ExtractedParams {
    pub(super) fn prop_id(&mut self) -> Option<String> {
        self.prop_id.take()
    }

    pub(super) fn alt_id(&mut self) -> Option<String> {
        self.alt_id.take()
    }

    pub(super) fn language(&mut self) -> Option<String> {
        self.language.take()
    }

    pub(super) fn types(&mut self) -> Vec<IanaType<VCardType, String>> {
        std::mem::take(&mut self.types)
    }

    #[allow(clippy::type_complexity)]
    pub(super) fn into_iter<I: JSContactId, B: JSContactId>(
        mut self,
        property: &VCardProperty,
    ) -> impl Iterator<
        Item = (
            Key<'static, JSContactProperty<I>>,
            Value<'static, JSContactProperty<I>, JSContactValue<I, B>>,
        ),
    > {
        let mut contexts: Option<
            Vec<(
                Key<'static, JSContactProperty<I>>,
                Value<'static, JSContactProperty<I>, JSContactValue<I, B>>,
            )>,
        > = None;
        let mut features: Option<
            Vec<(
                Key<'static, JSContactProperty<I>>,
                Value<'static, JSContactProperty<I>, JSContactValue<I, B>>,
            )>,
        > = None;

        if !self.types.is_empty() {
            let is_phone = matches!(property, VCardProperty::Tel);

            for typ in std::mem::take(&mut self.types) {
                if is_phone
                    && matches!(
                        typ,
                        IanaType::Iana(
                            VCardType::Fax
                                | VCardType::Cell
                                | VCardType::Video
                                | VCardType::Pager
                                | VCardType::Textphone
                                | VCardType::MainNumber
                                | VCardType::Text
                                | VCardType::Voice
                        )
                    )
                {
                    features.get_or_insert_default()
                } else {
                    contexts.get_or_insert_default()
                }
                .push((
                    match typ {
                        IanaType::Iana(VCardType::Home) => Key::Borrowed("private"),
                        IanaType::Iana(VCardType::Cell) => Key::Borrowed("mobile"),
                        _ => Key::Owned(typ.into_string().to_ascii_lowercase()),
                    },
                    Value::Bool(true),
                ));
            }
        }

        let author = if self.author.is_some() || self.author_name.is_some() {
            Value::Object(Map::from_iter(
                [
                    self.author_name.map(|name| {
                        (
                            Key::Property(JSContactProperty::Name),
                            Value::Str(name.into()),
                        )
                    }),
                    self.author.map(|author| {
                        (
                            Key::Property(JSContactProperty::Uri),
                            Value::Str(author.into()),
                        )
                    }),
                ]
                .into_iter()
                .flatten(),
            ))
            .into()
        } else {
            None
        };

        [
            (
                Key::Property(JSContactProperty::Contexts),
                contexts.map(|v| Value::Object(Map::from(v))),
            ),
            (
                Key::Property(JSContactProperty::Features),
                features.map(|v| Value::Object(Map::from(v))),
            ),
            (
                Key::Property(JSContactProperty::Language),
                self.language.map(Into::into).map(Value::Str),
            ),
            (
                Key::Property(JSContactProperty::Pref),
                self.pref.map(|v| Value::Number((v as u64).into())),
            ),
            (Key::Property(JSContactProperty::Author), author),
            (
                Key::Property(JSContactProperty::MediaType),
                self.media_type.map(Into::into).map(Value::Str),
            ),
            (
                Key::Property(JSContactProperty::PhoneticSystem),
                self.phonetic_system.map(|v| match v {
                    IanaType::Iana(value) => {
                        Value::Element(JSContactValue::PhoneticSystem(match value {
                            VCardPhonetic::Ipa => JSContactPhoneticSystem::Ipa,
                            VCardPhonetic::Jyut => JSContactPhoneticSystem::Jyut,
                            VCardPhonetic::Piny => JSContactPhoneticSystem::Piny,
                            VCardPhonetic::Script => JSContactPhoneticSystem::Script,
                        }))
                    }
                    IanaType::Other(value) => Value::Str(value.into()),
                }),
            ),
            (
                Key::Property(JSContactProperty::PhoneticScript),
                self.phonetic_script.map(Into::into).map(Value::Str),
            ),
            (
                Key::Property(JSContactProperty::CalendarScale),
                self.calscale.map(|value| match value {
                    IanaType::Iana(value) => Value::Element(JSContactValue::CalendarScale(value)),
                    IanaType::Other(value) => Value::Str(value.into()),
                }),
            ),
            (
                Key::Property(JSContactProperty::SortAs),
                self.sort_as.map(|sort_as| {
                    if matches!(property, VCardProperty::N) {
                        if let Some((surname, given)) =
                            sort_as.split_once(',').and_then(|(surname, given)| {
                                let surname = surname.trim();
                                let given = given.trim();

                                if surname.is_empty() || given.is_empty() {
                                    None
                                } else {
                                    Some((surname.to_string(), given.to_string()))
                                }
                            })
                        {
                            Value::Object(Map::from_iter(vec![
                                (
                                    Key::Property(JSContactProperty::SortAsKind(
                                        JSContactKind::Surname,
                                    )),
                                    Value::Str(surname.into()),
                                ),
                                (
                                    Key::Property(JSContactProperty::SortAsKind(
                                        JSContactKind::Given,
                                    )),
                                    Value::Str(given.into()),
                                ),
                            ]))
                        } else {
                            Value::Object(Map::from_iter(vec![(
                                Key::Property(JSContactProperty::SortAsKind(
                                    JSContactKind::Surname,
                                )),
                                Value::Str(sort_as.into()),
                            )]))
                        }
                    } else {
                        Value::Str(sort_as.into())
                    }
                }),
            ),
            (
                Key::Property(JSContactProperty::Coordinates),
                self.geo.map(Into::into).map(Value::Str),
            ),
            (
                Key::Property(JSContactProperty::TimeZone),
                self.tz.map(Into::into).map(Value::Str),
            ),
            (
                Key::Property(JSContactProperty::ListAs),
                self.index.map(|v| Value::Number((v as u64).into())),
            ),
            (
                Key::Property(JSContactProperty::Level),
                self.level.map(|value| match value {
                    IanaType::Iana(value) => Value::Element(JSContactValue::Level(match value {
                        VCardLevel::Beginner | VCardLevel::Low => JSContactLevel::Low,
                        VCardLevel::Average | VCardLevel::Medium => JSContactLevel::Medium,
                        VCardLevel::Expert | VCardLevel::High => JSContactLevel::High,
                    })),
                    IanaType::Other(value) => Value::Str(value.into()),
                }),
            ),
            (
                Key::Property(JSContactProperty::CountryCode),
                self.country_code.map(Into::into).map(Value::Str),
            ),
            (
                Key::Property(JSContactProperty::Created),
                self.created
                    .map(|v| Value::Element(JSContactValue::Timestamp(v))),
            ),
            (
                Key::Property(if matches!(property, VCardProperty::Adr) {
                    JSContactProperty::Full
                } else {
                    JSContactProperty::Label
                }),
                self.label.map(Into::into).map(Value::Str),
            ),
            (
                Key::Property(JSContactProperty::Service),
                self.service_type.map(Into::into).map(Value::Str),
            ),
            (
                Key::Property(JSContactProperty::User),
                self.username.map(Into::into).map(Value::Str),
            ),
        ]
        .into_iter()
        .filter_map(|(k, v)| v.map(|v| (k, v)))
    }
}

impl<I: JSContactId, B: JSContactId> VCardParams<I, B> {
    pub(super) fn into_jscontact_value(
        self,
    ) -> Option<Map<'static, JSContactProperty<I>, JSContactValue<I, B>>> {
        if !self.0.is_empty() {
            let mut obj = Map::from(Vec::with_capacity(self.0.len()));

            for (param, value) in self.0 {
                let value = if value.len() > 1 {
                    Value::Array(value)
                } else {
                    value.into_iter().next().unwrap()
                };
                obj.insert_unchecked(Key::Owned(param.into_string().to_ascii_lowercase()), value);
            }
            Some(obj)
        } else {
            None
        }
    }
}
