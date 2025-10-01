/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

use crate::{
    common::{
        CalendarScale, IanaParse,
        parser::{Boolean, Timestamp},
    },
    jscontact::{JSContactId, JSContactProperty, JSContactValue, export::params::ParamValue},
    vcard::{
        VCardEntry, VCardLevel, VCardParameter, VCardParameterName, VCardParameterValue,
        VCardPhonetic, VCardProperty, VCardType, VCardValueType,
    },
};
use jmap_tools::{Key, Value};

impl VCardEntry {
    pub(super) fn import_converted_properties<I, B>(
        &mut self,
        props: Value<'_, JSContactProperty<I>, JSContactValue<I, B>>,
    ) where
        I: JSContactId,
        B: JSContactId,
    {
        for (key, value) in props.into_expanded_object() {
            match key {
                Key::Property(JSContactProperty::Name) => {
                    if let Some(name) = value.into_string() {
                        self.name = VCardProperty::parse(name.as_bytes())
                            .unwrap_or(VCardProperty::Other(name.into_owned()));
                    }
                }
                Key::Property(JSContactProperty::Parameters) => {
                    self.import_jcard_params(value);
                }
                _ => {}
            }
        }
    }

    pub(super) fn import_jcard_params<I, B>(
        &mut self,
        params: Value<'_, JSContactProperty<I>, JSContactValue<I, B>>,
    ) where
        I: JSContactId,
        B: JSContactId,
    {
        for (key, value) in params.into_expanded_object() {
            let mut values = match value {
                Value::Array(values) => values.into_iter().filter_map(ParamValue::try_from_value),
                value => vec![value]
                    .into_iter()
                    .filter_map(ParamValue::try_from_value),
            }
            .peekable();

            if values.peek().is_none() {
                continue;
            }

            let key = key.to_string();
            let Some(param) = VCardParameterName::try_parse(key.as_bytes()) else {
                let key = key.into_owned();

                for value in values {
                    self.params.push(VCardParameter {
                        name: VCardParameterName::Other(key.clone()),
                        value: value.into_string().into_owned().into(),
                    });
                }

                continue;
            };

            for value in values {
                let value = match &param {
                    VCardParameterName::Value => {
                        let value = value.into_string();
                        VCardValueType::parse(value.as_bytes())
                            .map(VCardParameterValue::ValueType)
                            .unwrap_or_else(|| VCardParameterValue::Text(value.into_owned()))
                    }
                    VCardParameterName::Pref | VCardParameterName::Index => {
                        match value.into_number() {
                            Ok(n) => VCardParameterValue::Integer(n as u32),
                            Err(value) => {
                                VCardParameterValue::Text(value.into_string().into_owned())
                            }
                        }
                    }
                    VCardParameterName::Calscale => {
                        let value = value.into_string();
                        CalendarScale::parse(value.as_bytes())
                            .map(VCardParameterValue::Calscale)
                            .unwrap_or_else(|| VCardParameterValue::Text(value.into_owned()))
                    }
                    VCardParameterName::Phonetic => {
                        let value = value.into_string();
                        VCardPhonetic::parse(value.as_bytes())
                            .map(VCardParameterValue::Phonetic)
                            .unwrap_or_else(|| VCardParameterValue::Text(value.into_owned()))
                    }
                    VCardParameterName::Level => {
                        let value = value.into_string();
                        VCardLevel::parse(value.as_bytes())
                            .map(VCardParameterValue::Level)
                            .unwrap_or_else(|| VCardParameterValue::Text(value.into_owned()))
                    }
                    VCardParameterName::Created => {
                        let value = value.into_string();
                        Timestamp::parse(value.as_bytes())
                            .map(|v| VCardParameterValue::Timestamp(v.0))
                            .unwrap_or_else(|| VCardParameterValue::Text(value.into_owned()))
                    }
                    VCardParameterName::Derived => {
                        let value = value.into_string();
                        Boolean::parse(value.as_bytes())
                            .map(|v| VCardParameterValue::Bool(v.0))
                            .unwrap_or_else(|| VCardParameterValue::Text(value.into_owned()))
                    }
                    VCardParameterName::Type => {
                        let value = value.into_string();
                        VCardType::parse(value.as_bytes())
                            .map(VCardParameterValue::Type)
                            .unwrap_or_else(|| VCardParameterValue::Text(value.into_owned()))
                    }
                    VCardParameterName::Group => {
                        self.group = Some(value.into_string().into_owned());
                        continue;
                    }
                    _ => VCardParameterValue::Text(value.into_string().into_owned()),
                };

                self.params.push(VCardParameter {
                    name: param.clone(),
                    value,
                });
            }
        }
    }
}
