/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

use jmap_tools::{JsonPointer, JsonPointerHandler, Key, Map, Value};

use crate::{
    common::timezone::TzTimestamp,
    icalendar::{ICalendarEntry, ICalendarParameterName, ICalendarValue},
    jscalendar::{
        JSCalendarDateTime, JSCalendarProperty, JSCalendarValue,
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
        let todo =
            "for LOCATION and GEO use the same key and set main-location-id, otherwise use uuid5";
        let todo = "merge values by uuid5, for example when receiving ORGANIZER and ATTENDEE";

        todo!()
    }

    pub(super) fn add_conversion_props(&mut self, mut entry: EntryState) {
        let todo = "implement";
    }

    pub(super) fn main_location_id(&self) -> &str {
        todo!()
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

impl ICalendarValue {
    pub(super) fn uri_to_string(self) -> Self {
        match self {
            ICalendarValue::Uri(uri) => ICalendarValue::Text(uri.to_string()),
            other => other,
        }
    }
}
