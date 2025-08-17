/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

use crate::{
    common::{parser::Timestamp, CalendarScale},
    jscontact::{export::params::ParamValue, JSContactProperty, JSContactValue},
    vcard::{
        VCardEntry, VCardLevel, VCardParameter, VCardParameterName, VCardPhonetic, VCardProperty,
        VCardType, VCardValueType,
    },
};
use jmap_tools::{Key, Value};

impl VCardEntry {
    pub(super) fn import_converted_properties(
        &mut self,
        props: Value<'_, JSContactProperty, JSContactValue>,
    ) {
        for (key, value) in props.into_expanded_object() {
            match key {
                Key::Property(JSContactProperty::Name) => {
                    if let Some(name) = value.into_string() {
                        self.name = VCardProperty::try_from(name.as_ref())
                            .unwrap_or(VCardProperty::Other(name));
                    }
                }
                Key::Property(JSContactProperty::Parameters) => {
                    self.import_jcard_params(value);
                }
                _ => {}
            }
        }
    }

    pub(super) fn import_jcard_params(
        &mut self,
        params: Value<'_, JSContactProperty, JSContactValue>,
    ) {
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
            let Some(param) = VCardParameterName::try_parse(key.as_ref()) else {
                self.params.push(VCardParameter::Other(
                    [key.into_owned()]
                        .into_iter()
                        .chain(values.map(|v| v.into_string().into_owned()))
                        .collect(),
                ));
                continue;
            };

            let param = match param {
                VCardParameterName::Language => VCardParameter::Language(
                    values.map(|v| v.into_string().into_owned()).next().unwrap(),
                ),
                VCardParameterName::Value => VCardParameter::Value(
                    values
                        .map(|v| {
                            let v = v.into_string();
                            VCardValueType::try_from(v.as_bytes())
                                .unwrap_or_else(|_| VCardValueType::Other(v.into_owned()))
                        })
                        .collect(),
                ),
                VCardParameterName::Pref => values
                    .next()
                    .map(|v| match v.into_number() {
                        Ok(n) => VCardParameter::Pref(n as u32),
                        Err(v) => VCardParameter::Other(vec![
                            VCardParameterName::Pref.as_str().to_string(),
                            v.into_string().into_owned(),
                        ]),
                    })
                    .unwrap(),
                VCardParameterName::Altid => VCardParameter::Altid(
                    values.map(|v| v.into_string().into_owned()).next().unwrap(),
                ),
                VCardParameterName::Pid => {
                    VCardParameter::Pid(values.map(|v| v.into_string().into_owned()).collect())
                }
                VCardParameterName::Type => VCardParameter::Type(
                    values
                        .map(|v| {
                            let v = v.into_string();
                            VCardType::try_from(v.as_bytes())
                                .unwrap_or_else(|_| VCardType::Other(v.into_owned()))
                        })
                        .collect(),
                ),
                VCardParameterName::Mediatype => VCardParameter::Mediatype(
                    values.map(|v| v.into_string().into_owned()).next().unwrap(),
                ),
                VCardParameterName::Calscale => {
                    let value = values.next().map(|v| v.into_string()).unwrap();
                    CalendarScale::try_from(value.as_bytes())
                        .map(VCardParameter::Calscale)
                        .unwrap_or_else(|_| {
                            VCardParameter::Other(vec![
                                VCardParameterName::Calscale.as_str().to_string(),
                                value.into_owned(),
                            ])
                        })
                }
                VCardParameterName::SortAs => VCardParameter::SortAs(
                    values.map(|v| v.into_string().into_owned()).next().unwrap(),
                ),
                VCardParameterName::Geo => VCardParameter::Geo(
                    values.map(|v| v.into_string().into_owned()).next().unwrap(),
                ),
                VCardParameterName::Tz => {
                    VCardParameter::Tz(values.map(|v| v.into_string().into_owned()).next().unwrap())
                }
                VCardParameterName::Index => values
                    .next()
                    .map(|v| match v.into_number() {
                        Ok(n) => VCardParameter::Index(n as u32),
                        Err(v) => VCardParameter::Other(vec![
                            VCardParameterName::Index.as_str().to_string(),
                            v.into_string().into_owned(),
                        ]),
                    })
                    .unwrap(),
                VCardParameterName::Level => {
                    let value = values.next().map(|v| v.into_string()).unwrap();
                    VCardLevel::try_from(value.as_bytes())
                        .map(VCardParameter::Level)
                        .unwrap_or_else(|_| {
                            VCardParameter::Other(vec![
                                VCardParameterName::Level.as_str().to_string(),
                                value.into_owned(),
                            ])
                        })
                }
                VCardParameterName::Group => {
                    self.group = values.map(|v| v.into_string().into_owned()).next();
                    continue;
                }
                VCardParameterName::Cc => {
                    VCardParameter::Cc(values.map(|v| v.into_string().into_owned()).next().unwrap())
                }
                VCardParameterName::Author => VCardParameter::Author(
                    values.map(|v| v.into_string().into_owned()).next().unwrap(),
                ),
                VCardParameterName::AuthorName => VCardParameter::AuthorName(
                    values.map(|v| v.into_string().into_owned()).next().unwrap(),
                ),
                VCardParameterName::Created => {
                    let value = values.next().map(|v| v.into_string()).unwrap();
                    Timestamp::try_from(value.as_bytes())
                        .map(|v| VCardParameter::Created(v.0))
                        .unwrap_or_else(|_| {
                            VCardParameter::Other(vec![
                                VCardParameterName::Created.as_str().to_string(),
                                value.into_owned(),
                            ])
                        })
                }
                VCardParameterName::Derived => values
                    .next()
                    .map(|v| match v.into_bool() {
                        Ok(n) => VCardParameter::Derived(n),
                        Err(v) => VCardParameter::Other(vec![
                            VCardParameterName::Derived.as_str().to_string(),
                            v.into_string().into_owned(),
                        ]),
                    })
                    .unwrap(),
                VCardParameterName::Label => VCardParameter::Label(
                    values.map(|v| v.into_string().into_owned()).next().unwrap(),
                ),
                VCardParameterName::Phonetic => {
                    let value = values.next().map(|v| v.into_string()).unwrap();
                    VCardPhonetic::try_from(value.as_bytes())
                        .map(VCardParameter::Phonetic)
                        .unwrap_or_else(|_| {
                            VCardParameter::Other(vec![
                                VCardParameterName::Phonetic.as_str().to_string(),
                                value.into_owned(),
                            ])
                        })
                }
                VCardParameterName::PropId => VCardParameter::PropId(
                    values.map(|v| v.into_string().into_owned()).next().unwrap(),
                ),
                VCardParameterName::Script => VCardParameter::Script(
                    values.map(|v| v.into_string().into_owned()).next().unwrap(),
                ),
                VCardParameterName::ServiceType => VCardParameter::ServiceType(
                    values.map(|v| v.into_string().into_owned()).next().unwrap(),
                ),
                VCardParameterName::Username => VCardParameter::Username(
                    values.map(|v| v.into_string().into_owned()).next().unwrap(),
                ),
                VCardParameterName::Jsptr => VCardParameter::Jsptr(
                    values.map(|v| v.into_string().into_owned()).next().unwrap(),
                ),
                _ => unreachable!(),
            };

            self.params.push(param);
        }
    }
}
