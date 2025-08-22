/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

use crate::{
    jscontact::{
        JSContactProperty, JSContactValue,
        export::{State, props::convert_value},
    },
    vcard::{VCard, VCardEntry, VCardParameter, VCardProperty, VCardValueType},
};
use jmap_tools::{Element, JsonPointer, Key, Property, Value};
use std::borrow::Cow;

impl<'x> State<'x> {
    pub(super) fn insert_vcard(&mut self, path: &[JSContactProperty], mut entry: VCardEntry) {
        if self.converted_props_count < self.converted_props.len() {
            // Obtain propId
            let mut prop_id =
                if matches!(entry.name, VCardProperty::Member | VCardProperty::Related) {
                    entry.values.first().and_then(|v| v.as_text())
                } else {
                    entry.prop_id()
                };

            // Try mapping X-ABLabel
            if let Some(prop_id_) = prop_id {
                let mut remove_pos = None;
                for (param_pos, param) in entry.params.iter().enumerate() {
                    if let VCardParameter::Label(label) = param {
                        if self.converted_props.iter().any(|(prop, _)| {
                            prop.len() == 3
                                && prop[0].to_string() == path[0].to_string()
                                && prop[1] == prop_id_
                                && prop[2] == Key::Property(JSContactProperty::Label)
                        }) {
                            self.insert_vcard(
                                &[path[0].clone(), JSContactProperty::Label],
                                VCardEntry::new(VCardProperty::Other("X-ABLabel".into()))
                                    .with_value(label.to_string()),
                            );
                            remove_pos = Some(param_pos);
                        }
                        break;
                    }
                }

                if let Some(pos) = remove_pos {
                    entry.params.swap_remove(pos);
                    prop_id =
                        if matches!(entry.name, VCardProperty::Member | VCardProperty::Related) {
                            entry.values.first().and_then(|v| v.as_text())
                        } else {
                            entry.prop_id()
                        };
                }
            }

            let skip_tz_geo = matches!(entry.name, VCardProperty::Adr);
            let mut matched_once = false;

            'outer: for (keys, value) in self.converted_props.iter_mut() {
                let is_localized_key = keys
                    .first()
                    .is_some_and(|k| matches!(k, Key::Property(JSContactProperty::Localizations)));

                if let Some(lang) = &self.language {
                    if !is_localized_key || keys.get(1).is_none_or(|k| &k.to_string() != lang) {
                        continue;
                    }
                } else if is_localized_key {
                    continue;
                }
                if matches!(value, Value::Null) {
                    continue;
                }

                for (pos, item) in path.iter().enumerate() {
                    if !keys
                        .iter()
                        .any(|k| matches!(k, Key::Property(p) if p == item))
                    {
                        if pos == 0 && matched_once {
                            // Array is sorted, so if we didn't match the first item,
                            // we won't match any further.
                            break 'outer;
                        } else {
                            continue 'outer;
                        }
                    } else {
                        matched_once = true;
                    }
                }

                if prop_id
                    .map(Key::Borrowed)
                    .is_none_or(|prop_id| keys.iter().any(|k| k == &prop_id))
                    && (!skip_tz_geo
                        || !keys.iter().any(|k| {
                            matches!(
                                k,
                                Key::Property(
                                    JSContactProperty::TimeZone | JSContactProperty::Coordinates
                                )
                            )
                        }))
                {
                    entry.import_converted_properties(std::mem::take(value));
                    self.converted_props_count += 1;
                    break;
                }
            }
        }

        if let Some(lang) = &self.language {
            entry.params.push(VCardParameter::Language(lang.clone()));
        }

        self.vcard.entries.push(entry);
    }

    pub(super) fn insert_jsprop(
        &mut self,
        path: &[&str],
        value: Value<'x, JSContactProperty, JSContactValue>,
    ) {
        let path = if let Some(lang) = &self.language {
            JsonPointer::<JSContactProperty>::encode([
                JSContactProperty::Localizations.to_string().as_ref(),
                lang.as_str(),
                JsonPointer::<JSContactProperty>::encode(path).as_str(),
            ])
        } else {
            JsonPointer::<JSContactProperty>::encode(path)
        };

        self.js_props.push((path, value));
    }

    pub(super) fn import_properties(
        &mut self,
        props: Vec<Value<'x, JSContactProperty, JSContactValue>>,
    ) {
        for prop in props.into_iter().flat_map(|prop| prop.into_array()) {
            let mut prop = prop.into_iter();
            let Some(name) = prop.next().and_then(|v| v.into_string()).map(|name| {
                VCardProperty::try_from(name.as_bytes()).unwrap_or(VCardProperty::Other(name))
            }) else {
                continue;
            };
            let Some(params) = prop.next() else {
                continue;
            };
            let Some(value_type) = prop.next().and_then(|v| v.into_string()).map(|v| {
                VCardValueType::try_from(v.as_bytes()).unwrap_or(VCardValueType::Other(v))
            }) else {
                continue;
            };

            let (default_type, _) = name.default_types();
            let Some(values) = prop.next().and_then(|v| match v {
                Value::Array(arr) => Some(
                    arr.into_iter()
                        .filter_map(|v| convert_value(v, &default_type).ok())
                        .collect::<Vec<_>>(),
                ),
                v => convert_value(v, &default_type).ok().map(|v| vec![v]),
            }) else {
                continue;
            };

            let mut entry = VCardEntry::new(name);
            entry.import_jcard_params(params);
            entry.values = values;
            if default_type.unwrap_vcard() != value_type {
                entry.params.push(VCardParameter::Value(vec![value_type]));
            }
            self.vcard.entries.push(entry);
        }
    }

    pub(super) fn into_vcard(mut self) -> VCard {
        for (ptr, value) in self.js_props {
            self.vcard.entries.push(
                VCardEntry::new(VCardProperty::Jsprop)
                    .with_param(VCardParameter::Jsptr(ptr))
                    .with_value(serde_json::to_string(&value).unwrap_or_default()),
            );
        }
        self.vcard
    }
}

pub(super) enum ParamValue<'x> {
    Text(Cow<'x, str>),
    Number(i64),
    Bool(bool),
}

impl<'x> ParamValue<'x> {
    pub(super) fn try_from_value<P: Property, E: Element>(value: Value<'x, P, E>) -> Option<Self> {
        match value {
            Value::Str(s) => Some(Self::Text(s)),
            Value::Number(n) => Some(Self::Number(n.cast_to_i64())),
            Value::Bool(b) => Some(Self::Bool(b)),
            Value::Element(e) => Some(Self::Text(e.to_cow().to_string().into())),
            _ => None,
        }
    }

    pub(super) fn into_string(self) -> Cow<'x, str> {
        match self {
            Self::Text(s) => s,
            Self::Number(n) => n.to_string().into(),
            Self::Bool(b) => b.to_string().into(),
        }
    }

    pub(super) fn into_number(self) -> Result<i64, Self> {
        match self {
            Self::Number(n) => Ok(n),
            Self::Text(s) => s.parse().map_err(|_| Self::Text(s)),
            _ => Err(self),
        }
    }

    pub(super) fn into_bool(self) -> Result<bool, Self> {
        match self {
            Self::Bool(b) => Ok(b),
            _ => Err(self),
        }
    }
}
