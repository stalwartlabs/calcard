/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

use jmap_tools::{JsonPointer, JsonPointerItem, Key, Map, Value};

use crate::{
    common::{CalendarScale, IanaParse, IanaType, PartialDateTime},
    icalendar::{
        ICalendarAction, ICalendarClassification, ICalendarComponent, ICalendarComponentType,
        ICalendarEntry, ICalendarFreeBusyType, ICalendarMethod, ICalendarParameter,
        ICalendarParticipantType, ICalendarProperty, ICalendarProximityValue,
        ICalendarResourceType, ICalendarStatus, ICalendarTransparency, ICalendarValue,
        ICalendarValueType, ValueType,
    },
    jscalendar::{
        JSCalendarDateTime, JSCalendarProperty, JSCalendarValue, export::ConvertedComponent,
    },
};

impl ICalendarComponent {
    pub(super) fn apply_conversions(&mut self, conversions: Option<ConvertedComponent<'_>>) {
        let Some(conversions) = conversions else {
            return;
        };
        if !conversions.properties.is_empty() {
            self.import_properties(conversions.properties);
        }
        if !conversions.components.is_empty() {
            let todo = "implement";
        }
    }

    pub(super) fn import_properties(
        &mut self,
        props: Vec<Value<'_, JSCalendarProperty, JSCalendarValue>>,
    ) {
        for prop in props.into_iter().flat_map(|prop| prop.into_array()) {
            let mut prop = prop.into_iter();
            let Some(name) = prop.next().and_then(|v| v.into_string()).map(|name| {
                ICalendarProperty::parse(name.as_bytes())
                    .unwrap_or(ICalendarProperty::Other(name.into_owned()))
            }) else {
                continue;
            };
            let Some(params) = prop.next() else {
                continue;
            };
            let Some(value_type) =
                prop.next()
                    .and_then(|v| v.into_string())
                    .map(|v| match ICalendarValueType::parse(v.as_bytes()) {
                        Some(v) => IanaType::Iana(v),
                        None => IanaType::Other(v.into_owned()),
                    })
            else {
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

            let mut entry = ICalendarEntry::new(name);
            entry.import_jcal_params(params);
            entry.values = values;
            if !value_type.is_iana_and(|v| v == &default_type.unwrap_ical()) {
                entry.params.push(ICalendarParameter::value(value_type));
            }
            self.entries.push(entry);
        }
    }
}

pub(super) fn convert_value<'x>(
    value: Value<'x, JSCalendarProperty, JSCalendarValue>,
    value_type: &'_ ValueType,
) -> Result<ICalendarValue, Value<'x, JSCalendarProperty, JSCalendarValue>> {
    match value {
        Value::Element(e) => match e {
            JSCalendarValue::CalendarScale(v) => Ok(ICalendarValue::CalendarScale(v)),
            JSCalendarValue::DateTime(v) => Ok(ICalendarValue::PartialDateTime(Box::new(v.into()))),
            JSCalendarValue::Duration(v) => Ok(ICalendarValue::Duration(v)),
            JSCalendarValue::Method(v) => Ok(ICalendarValue::Method(v)),
            JSCalendarValue::AlertAction(_)
            | JSCalendarValue::FreeBusyStatus(_)
            | JSCalendarValue::ParticipantKind(_)
            | JSCalendarValue::ParticipationStatus(_)
            | JSCalendarValue::Privacy(_)
            | JSCalendarValue::Progress(_)
            | JSCalendarValue::RelativeTo(_)
            | JSCalendarValue::ScheduleAgent(_)
            | JSCalendarValue::EventStatus(_)
            | JSCalendarValue::Frequency(_)
            | JSCalendarValue::Skip(_)
            | JSCalendarValue::Weekday(_)
            | JSCalendarValue::Month(_)
            | JSCalendarValue::LinkRelation(_)
            | JSCalendarValue::Type(_) => Err(Value::Element(e)),
        },
        Value::Str(s) => {
            match value_type {
                ValueType::CalendarScale => {
                    if let Some(value) = CalendarScale::parse(s.as_ref().as_bytes()) {
                        return Ok(ICalendarValue::CalendarScale(value));
                    }
                }
                ValueType::Method => {
                    if let Some(value) = ICalendarMethod::parse(s.as_ref().as_bytes()) {
                        return Ok(ICalendarValue::Method(value));
                    }
                }
                ValueType::Classification => {
                    if let Some(value) = ICalendarClassification::parse(s.as_ref().as_bytes()) {
                        return Ok(ICalendarValue::Classification(value));
                    }
                }
                ValueType::Status => {
                    if let Some(value) = ICalendarStatus::parse(s.as_ref().as_bytes()) {
                        return Ok(ICalendarValue::Status(value));
                    }
                }
                ValueType::Transparency => {
                    if let Some(value) = ICalendarTransparency::parse(s.as_ref().as_bytes()) {
                        return Ok(ICalendarValue::Transparency(value));
                    }
                }
                ValueType::Action => {
                    if let Some(value) = ICalendarAction::parse(s.as_ref().as_bytes()) {
                        return Ok(ICalendarValue::Action(value));
                    }
                }
                ValueType::BusyType => {
                    if let Some(value) = ICalendarFreeBusyType::parse(s.as_ref().as_bytes()) {
                        return Ok(ICalendarValue::BusyType(value));
                    }
                }
                ValueType::ParticipantType => {
                    if let Some(value) = ICalendarParticipantType::parse(s.as_ref().as_bytes()) {
                        return Ok(ICalendarValue::ParticipantType(value));
                    }
                }
                ValueType::ResourceType => {
                    if let Some(value) = ICalendarResourceType::parse(s.as_ref().as_bytes()) {
                        return Ok(ICalendarValue::ResourceType(value));
                    }
                }
                ValueType::Proximity => {
                    if let Some(value) = ICalendarProximityValue::parse(s.as_ref().as_bytes()) {
                        return Ok(ICalendarValue::Proximity(value));
                    }
                }
                ValueType::Ical(_) => (),
            }

            Ok(ICalendarValue::Text(s.into_owned()))
        }
        Value::Bool(b) => Ok(ICalendarValue::Boolean(b)),
        Value::Number(n) => match n.try_cast_to_i64() {
            Ok(i) => Ok(ICalendarValue::Integer(i)),
            Err(f) => Ok(ICalendarValue::Float(f)),
        },
        value => Err(value),
    }
}

impl<'x> ConvertedComponent<'x> {
    pub(super) fn try_from_object(
        obj: Vec<(
            Key<'x, JSCalendarProperty>,
            Value<'x, JSCalendarProperty, JSCalendarValue>,
        )>,
    ) -> Option<Self> {
        let mut converted = ConvertedComponent {
            name: ICalendarComponentType::Other(String::new()),
            converted_props: Vec::new(),
            converted_props_count: 0,
            properties: Vec::new(),
            components: Vec::new(),
        };
        let mut has_name = false;
        for (sub_property, value) in obj {
            match (sub_property, value) {
                (Key::Property(JSCalendarProperty::ConvertedProperties), Value::Object(obj)) => {
                    for (key, value) in obj.into_vec() {
                        let ptr = match key {
                            Key::Property(JSCalendarProperty::Pointer(ptr)) => ptr,
                            _ => JsonPointer::parse(key.to_string().as_ref()),
                        };

                        let mut keys = Vec::with_capacity(2);
                        for item in ptr.into_iter() {
                            match item {
                                JsonPointerItem::Key(key) => {
                                    let key = match &key {
                                        Key::Borrowed(v) if v.contains('/') => v,
                                        Key::Owned(v) if v.contains('/') => v.as_str(),
                                        _ => {
                                            keys.push(key);
                                            continue;
                                        }
                                    };
                                    for item in JsonPointer::parse(key).into_iter() {
                                        keys.push(match item {
                                            JsonPointerItem::Key(k) => k,
                                            JsonPointerItem::Number(n) => Key::Owned(n.to_string()),
                                            JsonPointerItem::Root | JsonPointerItem::Wildcard => {
                                                continue;
                                            }
                                        });
                                    }
                                }
                                JsonPointerItem::Number(v) => {
                                    keys.push(Key::Owned(v.to_string()));
                                }
                                JsonPointerItem::Root | JsonPointerItem::Wildcard => (),
                            }
                        }

                        converted.converted_props.push((keys, value));
                    }
                }
                (Key::Property(JSCalendarProperty::Properties), Value::Array(array)) => {
                    converted.properties = array;
                }
                (Key::Property(JSCalendarProperty::Components), Value::Array(array)) => {
                    converted.components = array;
                }
                (Key::Property(JSCalendarProperty::Name), Value::Str(text)) => {
                    converted.name = ICalendarComponentType::parse(text.as_bytes())
                        .unwrap_or_else(|| ICalendarComponentType::Other(text.into_owned()));
                    has_name = true;
                }
                _ => {}
            }
        }

        if !converted.converted_props.is_empty() {
            converted
                .converted_props
                .sort_unstable_by(|a, b| a.0.cmp(&b.0));
        }

        (!converted.properties.is_empty()
            || !converted.components.is_empty()
            || has_name
            || !converted.converted_props.is_empty())
        .then_some(converted)
    }

    pub(super) fn build(
        entries: &mut Map<'x, JSCalendarProperty, JSCalendarValue>,
    ) -> Option<Self> {
        for (property, value) in entries.as_mut_vec() {
            if let (Key::Property(JSCalendarProperty::ICalComponent), Value::Object(obj)) =
                (property, value)
            {
                return Self::try_from_object(std::mem::take(obj.as_mut_vec()));
            }
        }

        None
    }
}

impl From<JSCalendarDateTime> for PartialDateTime {
    fn from(dt: JSCalendarDateTime) -> Self {
        if !dt.is_local {
            Self::from_utc_timestamp(dt.timestamp)
        } else {
            Self::from_naive_timestamp(dt.timestamp)
        }
    }
}
