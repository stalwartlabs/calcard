/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

use std::{borrow::Cow, collections::hash_map::Entry};

use jmap_tools::{JsonPointer, JsonPointerHandler, Key, Map, Property, Value};

use crate::{
    common::{IanaType, timezone::TzTimestamp},
    icalendar::{
        ICalendarComponentType, ICalendarEntry, ICalendarParameterName, ICalendarValue,
        ICalendarValueType,
    },
    jscalendar::{
        JSCalendarDateTime, JSCalendarProperty, JSCalendarValue,
        import::{EntryState, ICalendarConvertedProperty, ICalendarParams, State},
        uuid5,
    },
};

impl State {
    pub(super) fn map_named_entry(
        &mut self,
        entry: &mut EntryState,
        extract: &[ICalendarParameterName],
        top_property_name: JSCalendarProperty,
        values: impl IntoIterator<
            Item = (
                Key<'static, JSCalendarProperty>,
                Value<'static, JSCalendarProperty, JSCalendarValue>,
            ),
        >,
    ) {
        // Obtain main property and value
        let mut values = values.into_iter().peekable();
        let (property, value) = match values.peek() {
            Some((property, Value::Str(s))) => (property, s.as_ref()),
            Some((property, _)) => {
                debug_assert!(false, "Cannot generate jsid without a string value");
                (property, "unknown")
            }
            _ => {
                panic!("Cannot generate jsid without a value");
            }
        };

        // Obtain or calculate JSID
        let js_id = self
            .extract_params(&mut entry.entry, extract)
            .unwrap_or_else(|| uuid5(value));

        // Set converted props
        entry.set_converted_to(&[
            top_property_name.to_cow().as_ref(),
            js_id.as_str(),
            property.to_string().as_ref(),
        ]);

        let obj = self
            .get_mut_object_or_insert(top_property_name)
            .insert_or_get_mut(Key::Owned(js_id), Value::new_object())
            .as_object_mut()
            .unwrap();

        for (key, value) in values {
            if let Some(current_value) = obj.get_mut(&key) {
                match (value, current_value) {
                    (Value::Object(new_obj), Value::Object(existing_obj)) => {
                        existing_obj.extend(new_obj.into_vec());
                    }
                    (value, current_value) => {
                        *current_value = value;
                    }
                }
            } else {
                obj.insert_unchecked(key, value);
            }
        }
    }

    pub(super) fn add_conversion_props(&mut self, mut entry: EntryState) {
        if let Some(converted_to) = entry.converted_to.take() {
            if entry.map_name || !entry.entry.params.is_empty() {
                let mut value_type = None;

                match self.ical_converted_properties.entry(converted_to) {
                    Entry::Occupied(mut conv_prop) => {
                        entry.jcal_parameters(&mut conv_prop.get_mut().params, &mut value_type);
                    }
                    Entry::Vacant(conv_prop) => {
                        let mut params = ICalendarParams::default();
                        entry.jcal_parameters(&mut params, &mut value_type);
                        if let Some(value_type) = value_type {
                            params.0.insert(
                                ICalendarParameterName::Value,
                                vec![Value::Str(value_type.into_string())],
                            );
                        }
                        if !params.0.is_empty() || entry.map_name {
                            conv_prop.insert(ICalendarConvertedProperty {
                                name: if entry.map_name {
                                    Some(entry.entry.name)
                                } else {
                                    None
                                },
                                params,
                            });
                        }
                    }
                }
            }
        } else {
            let mut value_type = None;
            let mut params = ICalendarParams::default();

            entry.jcal_parameters(&mut params, &mut value_type);

            let values = if entry.entry.values.len() == 1 {
                entry
                    .entry
                    .values
                    .into_iter()
                    .next()
                    .unwrap()
                    .into_jscalendar_value(value_type.as_ref())
            } else {
                let mut values = Vec::with_capacity(entry.entry.values.len());
                for value in entry.entry.values {
                    values.push(value.into_jscalendar_value(value_type.as_ref()));
                }
                Value::Array(values)
            };
            self.ical_properties.push(Value::Array(vec![
                Value::Str(entry.entry.name.as_str().to_string().into()),
                Value::Object(
                    params
                        .into_jscalendar_value()
                        .unwrap_or(Map::from(Vec::new())),
                ),
                Value::Str(
                    value_type
                        .map(|v| v.into_string())
                        .unwrap_or(Cow::Borrowed("text")),
                ),
                values,
            ]));
        }
    }

    pub(super) fn into_object(
        mut self,
        component: ICalendarComponentType,
    ) -> Value<'static, JSCalendarProperty, JSCalendarValue> {
        let mut ical_obj = Map::from(Vec::new());
        if !self.ical_converted_properties.is_empty() {
            let mut converted_properties =
                Map::from(Vec::with_capacity(self.ical_converted_properties.len()));

            for (converted_to, props) in self.ical_converted_properties {
                let mut obj = Map::from(Vec::with_capacity(2));
                if let Some(params) = props.params.into_jscalendar_value() {
                    obj.insert(
                        Key::Property(JSCalendarProperty::Parameters),
                        Value::Object(params),
                    );
                }
                if let Some(name) = props.name {
                    obj.insert(
                        Key::Property(JSCalendarProperty::Name),
                        Value::Str(name.into_string()),
                    );
                }

                converted_properties.insert_unchecked(Key::Owned(converted_to), Value::Object(obj));
            }

            ical_obj.insert_unchecked(
                Key::Property(JSCalendarProperty::ConvertedProperties),
                Value::Object(converted_properties),
            );
        }

        if !self.ical_properties.is_empty() {
            ical_obj.insert_unchecked(
                Key::Property(JSCalendarProperty::Properties),
                Value::Array(self.ical_properties),
            );
        }

        if !ical_obj.is_empty() {
            ical_obj.insert_unchecked(
                Key::Property(JSCalendarProperty::Name),
                Value::Str(component.as_str().to_ascii_lowercase().into()),
            );
            self.entries.insert(
                Key::Property(JSCalendarProperty::ICalComponent),
                Value::Object(ical_obj),
            );
        }

        if self.has_dates {
            self.entries.insert(
                Key::Property(JSCalendarProperty::TimeZone),
                self.tz_start
                    .and_then(|tz| tz.name())
                    .map(Value::Str)
                    .unwrap_or(Value::Null),
            );

            if self.tz_end.is_some() && self.tz_start.is_some() && self.tz_end != self.tz_start {
                self.entries.insert(
                    Key::Property(JSCalendarProperty::EndTimeZone),
                    self.tz_end
                        .and_then(|tz| tz.name())
                        .map(Value::Str)
                        .unwrap_or(Value::Null),
                );
            }

            if let Some(recurrence_id) = self.recurrence_id {
                self.entries.insert(
                    Key::Property(JSCalendarProperty::RecurrenceId),
                    Value::Element(JSCalendarValue::DateTime(JSCalendarDateTime::new(
                        recurrence_id
                            .with_timezone(&self.tz_start.unwrap_or_default())
                            .to_naive_timestamp(),
                        true,
                    ))),
                );
                let rid_tz = recurrence_id.timezone().to_resolved();
                if rid_tz.is_some() && self.tz_start.is_some() && rid_tz != self.tz_start {
                    self.entries.insert(
                        Key::Property(JSCalendarProperty::RecurrenceIdTimeZone),
                        rid_tz
                            .and_then(|tz| tz.name())
                            .map(Value::Str)
                            .unwrap_or(Value::Null),
                    );
                }
            }
        }

        if let Some(uid) = self.uid {
            self.entries.insert(
                Key::Property(JSCalendarProperty::Uid),
                Value::Str(uid.into()),
            );
        }

        let mut obj = Value::Object(self.entries.into_iter().collect());
        if self.patch_objects.is_empty() {
            for (ptr, patch) in self.patch_objects {
                obj.patch_jptr(ptr.iter(), patch);
            }
        }

        obj
    }

    #[inline]
    pub(super) fn get_mut_object_or_insert(
        &mut self,
        key: JSCalendarProperty,
    ) -> &mut Map<'static, JSCalendarProperty, JSCalendarValue> {
        self.entries
            .entry(Key::Property(key))
            .or_insert_with(|| Value::Object(Map::from(Vec::new())))
            .as_object_mut()
            .unwrap()
    }
}

impl EntryState {
    pub(super) fn new(entry: ICalendarEntry) -> Self {
        Self {
            entry,
            converted_to: None,
            map_name: true,
        }
    }

    pub(super) fn set_converted_to(&mut self, converted_to: &[&str]) {
        self.converted_to = Some(JsonPointer::<JSCalendarProperty>::encode(converted_to));
    }

    pub(super) fn set_map_name(&mut self) {
        self.map_name = true;
    }

    pub(super) fn jcal_parameters(
        &mut self,
        params: &mut ICalendarParams,
        value_type: &mut Option<IanaType<ICalendarValueType, String>>,
    ) {
        if self.entry.params.is_empty() {
            return;
        }
        let (default_type, _) = self.entry.name.default_types();
        let default_type = default_type.unwrap_ical();

        for param in std::mem::take(&mut self.entry.params) {
            if matches!(param.name, ICalendarParameterName::Value) {
                if let Some(v) = param
                    .value
                    .into_value_type()
                    .filter(|v| !v.is_iana_and(|v| v == &default_type))
                {
                    *value_type = Some(v);
                }
            } else if let Some(value) = param.value.into_text() {
                params
                    .0
                    .entry(param.name)
                    .or_default()
                    .push(Value::Str(value));
            }
        }
    }
}

impl ICalendarValue {
    pub(super) fn uri_to_string(self) -> Self {
        match self {
            ICalendarValue::Uri(uri) => ICalendarValue::Text(uri.to_string()),
            other => other,
        }
    }
}
