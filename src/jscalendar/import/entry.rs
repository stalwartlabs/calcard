/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

use crate::{
    icalendar::*,
    jscalendar::{
        import::{EntryState, State},
        *,
    },
};
use jmap_tools::{JsonPointer, Key, Map, Value};

impl ICalendarComponent {
    pub(crate) fn entries_to_jscalendar(&mut self) -> State {
        let mut state = State::default();

        let todo = "import sub components";

        let has_locations = state
            .entries
            .contains_key(&Key::Property(JSCalendarProperty::Locations));

        for entry in std::mem::take(&mut self.entries) {
            let mut entry = EntryState::new(entry);
            let mut values = std::mem::take(&mut entry.entry.values).into_iter();

            match (
                &entry.entry.name,
                values.next().map(|v| v.uri_to_string()),
                &self.component_type,
            ) {
                (
                    ICalendarProperty::Acknowledged,
                    Some(ICalendarValue::PartialDateTime(value)),
                    ICalendarComponentType::VAlarm,
                ) if value.has_date_and_time() => {
                    state.entries.insert(
                        Key::Property(JSCalendarProperty::Acknowledged),
                        Value::Element(JSCalendarValue::DateTime(JSCalendarDateTime::new(
                            value.to_timestamp().unwrap(),
                            false,
                        ))),
                    );
                    entry
                        .set_converted_to(&[JSCalendarProperty::Acknowledged.to_string().as_ref()]);
                }
                (
                    ICalendarProperty::Action,
                    Some(ICalendarValue::Action(
                        value @ (ICalendarAction::Display | ICalendarAction::Email),
                    )),
                    ICalendarComponentType::VAlarm,
                ) => {
                    state.entries.insert(
                        Key::Property(JSCalendarProperty::Action),
                        Value::Element(JSCalendarValue::AlertAction(match value {
                            ICalendarAction::Display => JSCalendarAlertAction::Display,
                            ICalendarAction::Email => JSCalendarAlertAction::Email,
                            _ => unreachable!(),
                        })),
                    );
                    entry.set_converted_to(&[JSCalendarProperty::Action.to_string().as_ref()]);
                }
                (
                    ICalendarProperty::Attach,
                    Some(ICalendarValue::Text(uri)),
                    ICalendarComponentType::VEvent
                    | ICalendarComponentType::VTodo
                    | ICalendarComponentType::Participant
                    | ICalendarComponentType::VLocation
                    | ICalendarComponentType::VCalendar,
                ) => {
                    state.map_named_entry(
                        &mut entry,
                        &[
                            ICalendarParameterName::Fmttype,
                            ICalendarParameterName::Label,
                            ICalendarParameterName::Size,
                            ICalendarParameterName::Jsid,
                        ],
                        JSCalendarProperty::Links,
                        [
                            (
                                Key::Property(JSCalendarProperty::Type),
                                Value::Element(JSCalendarValue::Type(JSCalendarType::Link)),
                            ),
                            (
                                Key::Property(JSCalendarProperty::Href),
                                Value::Str(uri.into()),
                            ),
                        ],
                    );
                    entry.set_map_name();
                }
                (
                    ICalendarProperty::Image,
                    Some(ICalendarValue::Text(uri)),
                    ICalendarComponentType::VEvent
                    | ICalendarComponentType::VTodo
                    | ICalendarComponentType::Participant
                    | ICalendarComponentType::VLocation
                    | ICalendarComponentType::VCalendar,
                ) => {
                    state.map_named_entry(
                        &mut entry,
                        &[
                            ICalendarParameterName::Display,
                            ICalendarParameterName::Fmttype,
                            ICalendarParameterName::Label,
                            ICalendarParameterName::Size,
                            ICalendarParameterName::Jsid,
                        ],
                        JSCalendarProperty::Links,
                        [
                            (
                                Key::Property(JSCalendarProperty::Type),
                                Value::Element(JSCalendarValue::Type(JSCalendarType::Link)),
                            ),
                            (
                                Key::Property(JSCalendarProperty::Href),
                                Value::Str(uri.into()),
                            ),
                        ],
                    );
                    entry.set_map_name();
                }
                (
                    ICalendarProperty::Link,
                    Some(ICalendarValue::Text(uri)),
                    ICalendarComponentType::VEvent
                    | ICalendarComponentType::VTodo
                    | ICalendarComponentType::Participant
                    | ICalendarComponentType::VLocation
                    | ICalendarComponentType::VCalendar,
                ) => {
                    state.map_named_entry(
                        &mut entry,
                        &[
                            ICalendarParameterName::Fmttype,
                            ICalendarParameterName::Label,
                            ICalendarParameterName::Linkrel,
                            ICalendarParameterName::Jsid,
                        ],
                        JSCalendarProperty::Links,
                        [
                            (
                                Key::Property(JSCalendarProperty::Type),
                                Value::Element(JSCalendarValue::Type(JSCalendarType::Link)),
                            ),
                            (
                                Key::Property(JSCalendarProperty::Href),
                                Value::Str(uri.into()),
                            ),
                        ],
                    );
                    entry.set_map_name();
                }
                (
                    ICalendarProperty::Attendee,
                    Some(ICalendarValue::Text(uri)),
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo,
                ) => {
                    let is_task = matches!(&self.component_type, ICalendarComponentType::VTodo);
                    let mut has_role = false;
                    let mut progress = None;

                    for param in &mut entry.entry.params {
                        match (&param.name, &mut param.value) {
                            (ICalendarParameterName::Role, _) => {
                                has_role = true;
                            }
                            (
                                ICalendarParameterName::Partstat,
                                ICalendarParameterValue::Partstat(partstat),
                            ) if is_task => {
                                let status = match partstat {
                                    ICalendarParticipationStatus::Completed => {
                                        JSCalendarProgress::Completed
                                    }
                                    ICalendarParticipationStatus::InProcess => {
                                        JSCalendarProgress::InProcess
                                    }
                                    ICalendarParticipationStatus::Failed => {
                                        JSCalendarProgress::Failed
                                    }
                                    _ => {
                                        continue;
                                    }
                                };
                                *partstat = ICalendarParticipationStatus::Accepted;
                                progress = Some((
                                    Key::Property(JSCalendarProperty::Progress),
                                    Value::Element(JSCalendarValue::Progress(status)),
                                ));
                            }
                            _ => {}
                        }
                    }

                    state.map_named_entry(
                        &mut entry,
                        &[
                            ICalendarParameterName::Cn,
                            ICalendarParameterName::Cutype,
                            ICalendarParameterName::DelegatedFrom,
                            ICalendarParameterName::DelegatedTo,
                            ICalendarParameterName::Email,
                            ICalendarParameterName::Member,
                            ICalendarParameterName::Partstat,
                            ICalendarParameterName::Role,
                            ICalendarParameterName::Rsvp,
                            ICalendarParameterName::SentBy,
                            ICalendarParameterName::Jsid,
                        ],
                        JSCalendarProperty::Participants,
                        [
                            Some((
                                Key::Property(JSCalendarProperty::Type),
                                Value::Element(JSCalendarValue::Type(JSCalendarType::Participant)),
                            )),
                            progress,
                            (!has_role).then_some((
                                Key::Property(JSCalendarProperty::Roles),
                                Value::Object(Map::from(vec![(
                                    Key::Property(JSCalendarProperty::ParticipantRole(
                                        JSCalendarParticipantRole::Attendee,
                                    )),
                                    Value::Bool(true),
                                )])),
                            )),
                            Some((
                                Key::Property(JSCalendarProperty::CalendarAddress),
                                Value::Str(uri.to_string().into()),
                            )),
                        ]
                        .into_iter()
                        .flatten(),
                    );
                }
                (
                    ICalendarProperty::CalendarAddress,
                    Some(ICalendarValue::Text(value)),
                    ICalendarComponentType::Participant,
                ) => {
                    state.map_named_entry(
                        &mut entry,
                        &[ICalendarParameterName::Jsid],
                        JSCalendarProperty::Participants,
                        [
                            (
                                Key::Property(JSCalendarProperty::Type),
                                Value::Element(JSCalendarValue::Type(JSCalendarType::Participant)),
                            ),
                            (
                                Key::Property(JSCalendarProperty::CalendarAddress),
                                Value::Str(value.into()),
                            ),
                        ],
                    );
                    entry.set_map_name();
                }
                (
                    ICalendarProperty::Categories,
                    Some(ICalendarValue::Text(value)),
                    ICalendarComponentType::VEvent
                    | ICalendarComponentType::VTodo
                    | ICalendarComponentType::VCalendar,
                ) => {
                    let obj = state
                        .entries
                        .entry(Key::Property(JSCalendarProperty::Keywords))
                        .or_insert_with(Value::new_object)
                        .as_object_mut()
                        .unwrap();
                    obj.insert(Key::Owned(value), Value::Bool(true));
                    for value in values {
                        if let Some(value) = value.into_text() {
                            obj.insert(Key::from(value), Value::Bool(true));
                        }
                    }
                    entry.set_converted_to(&[JSCalendarProperty::Keywords.to_string().as_ref()]);
                }
                (
                    ICalendarProperty::Class,
                    Some(ICalendarValue::Classification(value)),
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo,
                ) => {
                    state.entries.insert(
                        Key::Property(JSCalendarProperty::Privacy),
                        Value::Element(JSCalendarValue::Privacy(match value {
                            ICalendarClassification::Public => JSCalendarPrivacy::Public,
                            ICalendarClassification::Private => JSCalendarPrivacy::Private,
                            ICalendarClassification::Confidential => JSCalendarPrivacy::Secret,
                        })),
                    );
                    entry.set_converted_to(&[JSCalendarProperty::Privacy.to_string().as_ref()]);
                }
                (
                    ICalendarProperty::Color,
                    Some(ICalendarValue::Text(value)),
                    ICalendarComponentType::VEvent
                    | ICalendarComponentType::VTodo
                    | ICalendarComponentType::VCalendar,
                ) => {
                    state.entries.insert(
                        Key::Property(JSCalendarProperty::Color),
                        Value::Str(value.into()),
                    );
                    entry.set_converted_to(&[JSCalendarProperty::Color.to_string().as_ref()]);
                }
                (
                    ICalendarProperty::Concept,
                    Some(ICalendarValue::Text(value)),
                    ICalendarComponentType::VEvent
                    | ICalendarComponentType::VTodo
                    | ICalendarComponentType::VCalendar,
                ) => {
                    let obj = state
                        .entries
                        .entry(Key::Property(JSCalendarProperty::Categories))
                        .or_insert_with(Value::new_object)
                        .as_object_mut()
                        .unwrap();
                    obj.insert(Key::Owned(value), Value::Bool(true));
                    for value in values {
                        if let Some(value) = value.into_text() {
                            obj.insert(Key::from(value), Value::Bool(true));
                        }
                    }
                    entry.set_converted_to(&[JSCalendarProperty::Categories.to_string().as_ref()]);
                }
                (
                    ICalendarProperty::Conference,
                    Some(ICalendarValue::Text(value)),
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo,
                ) => {
                    state.map_named_entry(
                        &mut entry,
                        &[
                            ICalendarParameterName::Jsid,
                            ICalendarParameterName::Feature,
                            ICalendarParameterName::Label,
                        ],
                        JSCalendarProperty::VirtualLocations,
                        [
                            (
                                Key::Property(JSCalendarProperty::Type),
                                Value::Element(JSCalendarValue::Type(
                                    JSCalendarType::VirtualLocation,
                                )),
                            ),
                            (
                                Key::Property(JSCalendarProperty::Uri),
                                Value::Str(value.into()),
                            ),
                        ],
                    );
                }
                (
                    ICalendarProperty::Coordinates,
                    Some(ICalendarValue::Text(value)),
                    ICalendarComponentType::VLocation,
                ) => {
                    state.map_named_entry(
                        &mut entry,
                        &[ICalendarParameterName::Jsid],
                        JSCalendarProperty::Locations,
                        [
                            (
                                Key::Property(JSCalendarProperty::Type),
                                Value::Element(JSCalendarValue::Type(JSCalendarType::Location)),
                            ),
                            (
                                Key::Property(JSCalendarProperty::Coordinates),
                                Value::Str(value.into()),
                            ),
                        ],
                    );
                }
                (
                    ICalendarProperty::Geo,
                    Some(ICalendarValue::Text(value)),
                    ICalendarComponentType::VLocation,
                ) if !entry
                    .entry
                    .parameters(&ICalendarParameterName::Derived)
                    .any(|v| matches!(v, ICalendarParameterValue::Bool(true))) =>
                {
                    state.map_named_entry(
                        &mut entry,
                        &[ICalendarParameterName::Jsid],
                        JSCalendarProperty::Locations,
                        [
                            (
                                Key::Property(JSCalendarProperty::Type),
                                Value::Element(JSCalendarValue::Type(JSCalendarType::Location)),
                            ),
                            (
                                Key::Property(JSCalendarProperty::Coordinates),
                                Value::Str(value.into()),
                            ),
                        ],
                    );
                    entry.set_map_name();
                }
                (
                    ICalendarProperty::Geo,
                    Some(ICalendarValue::Float(coord1)),
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo,
                ) if !entry
                    .entry
                    .parameters(&ICalendarParameterName::Derived)
                    .any(|v| matches!(v, ICalendarParameterValue::Bool(true))) =>
                {
                    let coord2 = values.next().and_then(|v| v.as_float()).unwrap_or_default();

                    state.map_named_entry(
                        &mut entry,
                        &[ICalendarParameterName::Jsid],
                        JSCalendarProperty::Locations,
                        [
                            (
                                Key::Property(JSCalendarProperty::Type),
                                Value::Element(JSCalendarValue::Type(JSCalendarType::Location)),
                            ),
                            (
                                Key::Property(JSCalendarProperty::Coordinates),
                                Value::Str(format!("geo:{coord1},{coord2}").into()),
                            ),
                        ],
                    );
                    entry.set_map_name();
                }
                (
                    ICalendarProperty::Name,
                    Some(ICalendarValue::Text(value)),
                    ICalendarComponentType::VLocation,
                ) => {
                    state.entries.insert(
                        Key::Property(JSCalendarProperty::Name),
                        Value::Str(value.into()),
                    );
                    entry.set_converted_to(&[JSCalendarProperty::Name.to_string().as_ref()]);
                }
                (
                    ICalendarProperty::Name,
                    Some(ICalendarValue::Text(value)),
                    ICalendarComponentType::VCalendar,
                )
                | (
                    ICalendarProperty::Summary,
                    Some(ICalendarValue::Text(value)),
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo,
                ) => {
                    state.extract_params(&mut entry.entry, &[ICalendarParameterName::Language]);
                    state.entries.insert(
                        Key::Property(JSCalendarProperty::Title),
                        Value::Str(value.into()),
                    );
                    entry.set_converted_to(&[JSCalendarProperty::Title.to_string().as_ref()]);
                }
                (
                    ICalendarProperty::Summary,
                    Some(ICalendarValue::Text(value)),
                    ICalendarComponentType::Participant,
                ) => {
                    state.entries.insert(
                        Key::Property(JSCalendarProperty::Name),
                        Value::Str(value.into()),
                    );
                    entry.set_converted_to(&[JSCalendarProperty::Name.to_string().as_ref()]);
                }
                (
                    ICalendarProperty::Location,
                    Some(ICalendarValue::Text(value)),
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo,
                ) => {
                    let is_derived = !entry
                        .entry
                        .parameters(&ICalendarParameterName::Derived)
                        .any(|v| matches!(v, ICalendarParameterValue::Bool(true)));

                    if !is_derived {
                        let location_id = if has_locations {
                            entry.entry.jsid().map(|s| s.to_string())
                        } else {
                            None
                        };

                        state.map_named_entry(
                            &mut entry,
                            &[ICalendarParameterName::Jsid],
                            JSCalendarProperty::Locations,
                            [
                                (
                                    Key::Property(JSCalendarProperty::Type),
                                    Value::Element(JSCalendarValue::Type(JSCalendarType::Location)),
                                ),
                                (
                                    Key::Property(JSCalendarProperty::Name),
                                    Value::Str(value.into()),
                                ),
                            ],
                        );
                        entry.set_map_name();

                        if has_locations {
                            let location_id =
                                location_id.unwrap_or_else(|| state.main_location_id().to_string());
                            state.entries.insert(
                                Key::Property(JSCalendarProperty::MainLocationId),
                                Value::Str(location_id.into()),
                            );
                        }
                    } else {
                        if has_locations {
                            let location_id = uuid5(&value);
                            if state
                                .entries
                                .get(&Key::Property(JSCalendarProperty::Locations))
                                .and_then(|v| v.as_object())
                                .is_some_and(|v| {
                                    v.contains_key(&Key::Borrowed(location_id.as_str()))
                                })
                            {
                                state.entries.insert(
                                    Key::Property(JSCalendarProperty::MainLocationId),
                                    Value::Str(location_id.into()),
                                );
                            }
                        }

                        entry.entry.values = [ICalendarValue::Text(value)]
                            .into_iter()
                            .chain(values)
                            .collect();
                    }
                }
                (
                    ICalendarProperty::LocationType,
                    Some(ICalendarValue::Text(value)),
                    ICalendarComponentType::VLocation,
                ) => {
                    let obj = state
                        .entries
                        .entry(Key::Property(JSCalendarProperty::LocationTypes))
                        .or_insert_with(Value::new_object)
                        .as_object_mut()
                        .unwrap();
                    obj.insert(Key::Owned(value), Value::Bool(true));
                    for value in values {
                        if let Some(value) = value.into_text() {
                            obj.insert(Key::from(value), Value::Bool(true));
                        }
                    }
                    entry.set_converted_to(&[JSCalendarProperty::LocationTypes
                        .to_string()
                        .as_ref()]);
                }

                // Other props
                (ICalendarProperty::Jsid, Some(ICalendarValue::Text(value)), _) => {
                    state.jsid = Some(value);
                }
                (ICalendarProperty::Jsprop, Some(ICalendarValue::Text(value)), _) => {
                    if let Some(ICalendarParameter {
                        name: ICalendarParameterName::Jsptr,
                        value: ICalendarParameterValue::Text(ptr),
                    }) = entry.entry.params.first()
                    {
                        let ptr = JsonPointer::<JSCalendarProperty>::parse(ptr);

                        if let Ok(jscalendar) = JSCalendar::parse(&value) {
                            state.patch_objects.push((ptr, jscalendar.0.into_owned()));
                            continue;
                        }
                    }
                    entry.entry.values = [ICalendarValue::Text(value)]
                        .into_iter()
                        .chain(values)
                        .collect();
                }

                (ICalendarProperty::Begin | ICalendarProperty::End, _, _) => {
                    continue;
                }

                (_, value, _) => {
                    entry.entry.values = [value].into_iter().flatten().chain(values).collect();
                }
            }

            state.add_conversion_props(entry);
        }

        state
    }
}
