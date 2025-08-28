/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

use jmap_tools::{JsonPointer, JsonPointerHandler, Key, Map, Value};

use crate::{
    icalendar::{ICalendarEntry, ICalendarParameterName},
    jscalendar::{
        JSCalendarProperty, JSCalendarValue,
        import::{EntryState, State},
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
        todo!()
    }

    pub(super) fn add_conversion_props(&mut self, mut entry: EntryState) {
        let todo = "implement";
    }

    pub(super) fn into_object(mut self) -> Value<'static, JSCalendarProperty, JSCalendarValue> {
        let mut ical_obj = Map::from(Vec::new());
        if !self.ical_converted_properties.is_empty() {
            let todo = "implement";
            /*let mut converted_properties =
                Map::from(Vec::with_capacity(self.ical_converted_properties.len()));

            for (converted_to, props) in self.ical_converted_properties {
                let mut obj = Map::from(Vec::with_capacity(2));
                if let Some(params) = props.params.into_jscontact_value() {
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
            );*/
        }

        if !self.ical_properties.is_empty() {
            ical_obj.insert_unchecked(
                Key::Property(JSCalendarProperty::Properties),
                Value::Array(self.ical_properties),
            );
        }

        if !ical_obj.is_empty() {
            self.entries.insert(
                Key::Property(JSCalendarProperty::ICalComponent),
                Value::Object(ical_obj),
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
}
