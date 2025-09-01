/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

use crate::{
    common::timezone::TzTimestamp,
    icalendar::{timezone::TzResolver, *},
    jscalendar::{
        import::{EntryState, State},
        *,
    },
};
use jmap_tools::{JsonPointer, Key, Map, Value};

impl ICalendarComponent {
    pub(super) fn entries_to_jscalendar(&mut self, tz_resolver: &TzResolver<String>) -> State {
        let mut state = State::default();

        let todo = "import sub components + converted props components";

        let has_locations = state
            .entries
            .contains_key(&Key::Property(JSCalendarProperty::Locations));
        let mut main_location_id = None;

        self.entries.sort_by_key(|entry| match &entry.name {
            ICalendarProperty::Dtstart => 0,
            ICalendarProperty::RecurrenceId => 1,
            ICalendarProperty::Dtend | ICalendarProperty::Due | ICalendarProperty::Location => 2,
            _ => 3,
        });
        let mut start_date = None;

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
                                Key::Property(JSCalendarProperty::Href),
                                Value::Str(uri.into()),
                            ),
                            (
                                Key::Property(JSCalendarProperty::Type),
                                Value::Element(JSCalendarValue::Type(JSCalendarType::Link)),
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
                                Key::Property(JSCalendarProperty::Href),
                                Value::Str(uri.into()),
                            ),
                            (
                                Key::Property(JSCalendarProperty::Type),
                                Value::Element(JSCalendarValue::Type(JSCalendarType::Link)),
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
                                Key::Property(JSCalendarProperty::Href),
                                Value::Str(uri.into()),
                            ),
                            (
                                Key::Property(JSCalendarProperty::Type),
                                Value::Element(JSCalendarValue::Type(JSCalendarType::Link)),
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
                    let mut progress = None;

                    for param in &mut entry.entry.params {
                        match (&param.name, &mut param.value) {
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
                                Key::Property(JSCalendarProperty::CalendarAddress),
                                Value::Str(uri.to_string().into()),
                            )),
                            Some((
                                Key::Property(JSCalendarProperty::Type),
                                Value::Element(JSCalendarValue::Type(JSCalendarType::Participant)),
                            )),
                            progress,
                            Some((
                                Key::Property(JSCalendarProperty::Roles),
                                Value::Object(Map::from(vec![(
                                    Key::Property(JSCalendarProperty::ParticipantRole(
                                        JSCalendarParticipantRole::Attendee,
                                    )),
                                    Value::Bool(true),
                                )])),
                            )),
                        ]
                        .into_iter()
                        .flatten(),
                    );
                }
                (
                    ICalendarProperty::Organizer,
                    Some(ICalendarValue::Text(uri)),
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo,
                ) => {
                    state.entries.insert(
                        Key::Property(JSCalendarProperty::OrganizerCalendarAddress),
                        Value::Str(uri.clone().into()),
                    );
                    state.map_named_entry(
                        &mut entry,
                        &[ICalendarParameterName::Cn, ICalendarParameterName::Jsid],
                        JSCalendarProperty::Participants,
                        [
                            (
                                Key::Property(JSCalendarProperty::CalendarAddress),
                                Value::Str(uri.to_string().into()),
                            ),
                            (
                                Key::Property(JSCalendarProperty::Type),
                                Value::Element(JSCalendarValue::Type(JSCalendarType::Participant)),
                            ),
                            (
                                Key::Property(JSCalendarProperty::Roles),
                                Value::Object(Map::from(vec![(
                                    Key::Property(JSCalendarProperty::ParticipantRole(
                                        JSCalendarParticipantRole::Owner,
                                    )),
                                    Value::Bool(true),
                                )])),
                            ),
                        ],
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
                                Key::Property(JSCalendarProperty::CalendarAddress),
                                Value::Str(value.into()),
                            ),
                            (
                                Key::Property(JSCalendarProperty::Type),
                                Value::Element(JSCalendarValue::Type(JSCalendarType::Participant)),
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
                                Key::Property(JSCalendarProperty::Uri),
                                Value::Str(value.into()),
                            ),
                            (
                                Key::Property(JSCalendarProperty::Type),
                                Value::Element(JSCalendarValue::Type(
                                    JSCalendarType::VirtualLocation,
                                )),
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
                                Key::Property(JSCalendarProperty::Coordinates),
                                Value::Str(value.into()),
                            ),
                            (
                                Key::Property(JSCalendarProperty::Type),
                                Value::Element(JSCalendarValue::Type(JSCalendarType::Location)),
                            ),
                        ],
                    );
                }
                (
                    ICalendarProperty::Geo,
                    Some(ICalendarValue::Text(value)),
                    ICalendarComponentType::VLocation,
                ) if !entry.entry.is_derived() => {
                    state.map_named_entry(
                        &mut entry,
                        &[ICalendarParameterName::Jsid],
                        JSCalendarProperty::Locations,
                        [
                            (
                                Key::Property(JSCalendarProperty::Coordinates),
                                Value::Str(value.into()),
                            ),
                            (
                                Key::Property(JSCalendarProperty::Type),
                                Value::Element(JSCalendarValue::Type(JSCalendarType::Location)),
                            ),
                        ],
                    );
                    entry.set_map_name();
                }
                (
                    ICalendarProperty::Geo,
                    Some(ICalendarValue::Float(coord1)),
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo,
                ) if !entry.entry.is_derived() => {
                    let coord2 = values.next().and_then(|v| v.as_float()).unwrap_or_default();
                    if entry.entry.jsid().is_none()
                        && let Some(main_location_id) = main_location_id.take()
                    {
                        entry
                            .entry
                            .add_param(ICalendarParameter::jsid(main_location_id));
                    }

                    state.map_named_entry(
                        &mut entry,
                        &[ICalendarParameterName::Jsid],
                        JSCalendarProperty::Locations,
                        [
                            (
                                Key::Property(JSCalendarProperty::Coordinates),
                                Value::Str(format!("geo:{coord1},{coord2}").into()),
                            ),
                            (
                                Key::Property(JSCalendarProperty::Type),
                                Value::Element(JSCalendarValue::Type(JSCalendarType::Location)),
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
                    if !entry.entry.is_derived() {
                        let location_id = if let Some(location_id) = entry.entry.jsid() {
                            main_location_id = Some(location_id.to_string());
                            location_id
                        } else {
                            main_location_id = Some(uuid5(&value));
                            main_location_id.as_deref().unwrap()
                        };

                        if has_locations {
                            state.entries.insert(
                                Key::Property(JSCalendarProperty::MainLocationId),
                                Value::Str(location_id.to_string().into()),
                            );
                        }

                        state.map_named_entry(
                            &mut entry,
                            &[ICalendarParameterName::Jsid],
                            JSCalendarProperty::Locations,
                            [
                                (
                                    Key::Property(JSCalendarProperty::Name),
                                    Value::Str(value.into()),
                                ),
                                (
                                    Key::Property(JSCalendarProperty::Type),
                                    Value::Element(JSCalendarValue::Type(JSCalendarType::Location)),
                                ),
                            ],
                        );
                        entry.set_map_name();
                    } else {
                        let location_id = uuid5(&value);
                        if has_locations
                            && state
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
                        entry.entry.params.clear();
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

                (
                    ICalendarProperty::LastModified,
                    Some(ICalendarValue::PartialDateTime(value)),
                    ICalendarComponentType::VCalendar,
                ) if value.has_date_and_time() => {
                    state.entries.insert(
                        Key::Property(JSCalendarProperty::Updated),
                        Value::Element(JSCalendarValue::DateTime(JSCalendarDateTime::new(
                            value.to_timestamp().unwrap(),
                            false,
                        ))),
                    );
                    entry.set_converted_to(&[JSCalendarProperty::Updated.to_string().as_ref()]);
                }
                (
                    ICalendarProperty::Created,
                    Some(ICalendarValue::PartialDateTime(value)),
                    ICalendarComponentType::VEvent
                    | ICalendarComponentType::VTodo
                    | ICalendarComponentType::VCalendar,
                ) if value.has_date_and_time() => {
                    state.entries.insert(
                        Key::Property(JSCalendarProperty::Created),
                        Value::Element(JSCalendarValue::DateTime(JSCalendarDateTime::new(
                            value.to_timestamp().unwrap(),
                            false,
                        ))),
                    );
                    entry.set_converted_to(&[JSCalendarProperty::Created.to_string().as_ref()]);
                }
                (
                    ICalendarProperty::Description,
                    Some(ICalendarValue::Text(value)),
                    ICalendarComponentType::VEvent
                    | ICalendarComponentType::VTodo
                    | ICalendarComponentType::Participant
                    | ICalendarComponentType::VCalendar,
                ) if !entry.entry.is_derived() => {
                    state.entries.insert(
                        Key::Property(JSCalendarProperty::Description),
                        Value::Str(value.into()),
                    );
                    entry.set_converted_to(&[JSCalendarProperty::Description.to_string().as_ref()]);
                }
                (
                    ICalendarProperty::Dtstart,
                    Some(ICalendarValue::PartialDateTime(value)),
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo,
                ) if value.has_date() => {
                    state.tz_start = entry.entry.tz_id().and_then(|v| tz_resolver.resolve(v));
                    if let Some(dt) = value
                        .to_date_time()
                        .and_then(|dt| dt.to_date_time_with_tz(state.tz_start.unwrap_or_default()))
                    {
                        state.has_dates = true;
                        state.entries.insert(
                            Key::Property(JSCalendarProperty::Start),
                            Value::Element(JSCalendarValue::DateTime(JSCalendarDateTime::new(
                                dt.to_naive_timestamp(),
                                true,
                            ))),
                        );

                        if !value.has_time() {
                            state.entries.insert(
                                Key::Property(JSCalendarProperty::ShowWithoutTime),
                                Value::Bool(true),
                            );
                        }
                        entry.set_converted_to(&[JSCalendarProperty::Start.to_string().as_ref()]);
                        start_date = Some(dt);

                        if state.tz_start.is_none() {
                            state.tz_start = dt.timezone().to_resolved();
                        }
                    } else {
                        state.tz_start = None;
                        entry.entry.values = [ICalendarValue::PartialDateTime(value)]
                            .into_iter()
                            .chain(values)
                            .collect();
                    }
                }
                (
                    ICalendarProperty::Dtend,
                    Some(ICalendarValue::PartialDateTime(value)),
                    ICalendarComponentType::VEvent,
                ) if value.has_date() && start_date.is_some() => {
                    state.tz_end = entry
                        .entry
                        .tz_id()
                        .and_then(|v| tz_resolver.resolve(v))
                        .or(state.tz_start);
                    if let Some((delta, tz)) = value
                        .to_date_time()
                        .and_then(|dt| dt.to_date_time_with_tz(state.tz_end.unwrap_or_default()))
                        .and_then(|dt| {
                            let delta = dt.signed_duration_since(start_date.unwrap()).num_seconds();
                            (delta > 0).then_some((delta, dt.timezone()))
                        })
                    {
                        state.has_dates = true;
                        state.entries.insert(
                            Key::Property(JSCalendarProperty::Duration),
                            Value::Element(JSCalendarValue::Duration(
                                ICalendarDuration::from_seconds(delta),
                            )),
                        );

                        if !value.has_time() {
                            state.entries.insert(
                                Key::Property(JSCalendarProperty::ShowWithoutTime),
                                Value::Bool(true),
                            );
                        }
                        entry
                            .set_converted_to(&[JSCalendarProperty::Duration.to_string().as_ref()]);
                        entry.set_map_name();

                        if state.tz_end.is_none() {
                            state.tz_end = tz.to_resolved();
                        }
                        if state.tz_start.is_none() {
                            state.tz_start = state.tz_end;
                        }
                    } else {
                        state.tz_end = None;
                        entry.entry.values = [ICalendarValue::PartialDateTime(value)]
                            .into_iter()
                            .chain(values)
                            .collect();
                    }
                }
                (
                    ICalendarProperty::Due,
                    Some(ICalendarValue::PartialDateTime(value)),
                    ICalendarComponentType::VTodo,
                ) if value.has_date() => {
                    let due_tz = entry
                        .entry
                        .tz_id()
                        .and_then(|v| tz_resolver.resolve(v))
                        .or(state.tz_start);
                    if let Some(dt) = value
                        .to_date_time()
                        .and_then(|dt| dt.to_date_time_with_tz(due_tz.unwrap_or_default()))
                    {
                        state.has_dates = true;
                        if state.tz_start.is_none() {
                            state.tz_start = dt.timezone().to_resolved();
                        }

                        state.entries.insert(
                            Key::Property(JSCalendarProperty::Due),
                            Value::Element(JSCalendarValue::DateTime(JSCalendarDateTime::new(
                                dt.with_timezone(&state.tz_start.unwrap_or_default())
                                    .to_naive_timestamp(),
                                true,
                            ))),
                        );

                        if !value.has_time() {
                            state.entries.insert(
                                Key::Property(JSCalendarProperty::ShowWithoutTime),
                                Value::Bool(true),
                            );
                        }

                        entry.set_converted_to(&[JSCalendarProperty::Due.to_string().as_ref()]);
                    } else {
                        entry.entry.values = [ICalendarValue::PartialDateTime(value)]
                            .into_iter()
                            .chain(values)
                            .collect();
                    }
                }
                (
                    ICalendarProperty::Dtstamp,
                    Some(ICalendarValue::PartialDateTime(value)),
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo,
                ) if value.has_date_and_time() => {
                    state.entries.insert(
                        Key::Property(JSCalendarProperty::Updated),
                        Value::Element(JSCalendarValue::DateTime(JSCalendarDateTime::new(
                            value.to_timestamp().unwrap_or_default(),
                            false,
                        ))),
                    );

                    entry.set_converted_to(&[JSCalendarProperty::Updated.to_string().as_ref()]);
                }
                (
                    ICalendarProperty::Duration,
                    Some(ICalendarValue::Duration(value)),
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo,
                ) => {
                    state.entries.insert(
                        Key::Property(JSCalendarProperty::Duration),
                        Value::Element(JSCalendarValue::Duration(value)),
                    );

                    entry.set_converted_to(&[JSCalendarProperty::Duration.to_string().as_ref()]);
                }
                (
                    ICalendarProperty::EstimatedDuration,
                    Some(ICalendarValue::Duration(value)),
                    ICalendarComponentType::VTodo,
                ) => {
                    state.entries.insert(
                        Key::Property(JSCalendarProperty::EstimatedDuration),
                        Value::Element(JSCalendarValue::Duration(value)),
                    );

                    entry.set_converted_to(&[JSCalendarProperty::EstimatedDuration
                        .to_string()
                        .as_ref()]);
                }
                (
                    ICalendarProperty::RecurrenceId,
                    Some(ICalendarValue::PartialDateTime(value)),
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo,
                ) if value.has_date_and_time() => {
                    let rid_tz = entry
                        .entry
                        .tz_id()
                        .and_then(|v| tz_resolver.resolve(v))
                        .or(state.tz_start);

                    if let Some(dt) = value
                        .to_date_time()
                        .and_then(|dt| dt.to_date_time_with_tz(rid_tz.unwrap_or_default()))
                    {
                        state.has_dates = true;
                        state.recurrence_id = Some(dt);

                        state.extract_params(&mut entry.entry, &[ICalendarParameterName::Range]);

                        entry.set_converted_to(&[JSCalendarProperty::RecurrenceId
                            .to_string()
                            .as_ref()]);

                        if state.tz_start.is_none() {
                            state.tz_start = dt.timezone().to_resolved();
                        }
                    } else {
                        entry.entry.values = [ICalendarValue::PartialDateTime(value)]
                            .into_iter()
                            .chain(values)
                            .collect();
                    }
                }
                (
                    ICalendarProperty::Rdate | ICalendarProperty::Exdate,
                    Some(ICalendarValue::PartialDateTime(value)),
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo,
                ) if value.has_date() => {
                    let tz_id = entry
                        .entry
                        .tz_id()
                        .and_then(|v| tz_resolver.resolve(v))
                        .or(state.tz_start);

                    if let Some(dt) = value
                        .to_date_time()
                        .and_then(|dt| dt.to_date_time_with_tz(tz_id.unwrap_or_default()))
                    {
                        state.has_dates = true;

                        if state.tz_start.is_none() {
                            state.tz_start = dt.timezone().to_resolved();
                        }

                        let overrides = state
                            .entries
                            .entry(Key::Property(JSCalendarProperty::RecurrenceOverrides))
                            .or_insert_with(Value::new_object)
                            .as_object_mut()
                            .unwrap();
                        let value = Value::Object(Map::from(
                            if matches!(entry.entry.name, ICalendarProperty::Exdate) {
                                vec![(
                                    Key::Property(JSCalendarProperty::Excluded),
                                    Value::Bool(true),
                                )]
                            } else {
                                vec![]
                            },
                        ));

                        for (pos, dt) in [dt]
                            .into_iter()
                            .chain(values.filter_map(|v| {
                                v.into_partial_date_time()
                                    .and_then(|dt| dt.to_date_time())
                                    .and_then(|dt| {
                                        dt.to_date_time_with_tz(tz_id.unwrap_or_default())
                                    })
                            }))
                            .enumerate()
                        {
                            let key = Key::Property(JSCalendarProperty::DateTime(
                                JSCalendarDateTime::new(
                                    dt.with_timezone(&state.tz_start.unwrap_or_default())
                                        .to_naive_timestamp(),
                                    true,
                                ),
                            ));

                            if pos == 0 {
                                entry.set_converted_to(&[
                                    JSCalendarProperty::RecurrenceOverrides.to_string().as_ref(),
                                    key.to_string().as_ref().to_string().as_ref(),
                                ]);
                            }

                            overrides.insert(key, value.clone());
                        }
                    } else {
                        entry.entry.values = [ICalendarValue::PartialDateTime(value)]
                            .into_iter()
                            .chain(values)
                            .collect();
                    }
                }
                (
                    ICalendarProperty::Rrule,
                    Some(ICalendarValue::RecurrenceRule(value)),
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo,
                ) => {
                    let mut rrule = Map::from(Vec::with_capacity(4));

                    rrule.insert_unchecked(
                        JSCalendarProperty::Frequency,
                        Value::Element(JSCalendarValue::Frequency(value.freq)),
                    );

                    for (key, value) in [
                        (
                            JSCalendarProperty::Until,
                            value
                                .until
                                .as_ref()
                                .and_then(|v| {
                                    v.to_date_time_with_tz(state.tz_start.unwrap_or_default())
                                })
                                .map(|dt| {
                                    Value::Element(JSCalendarValue::DateTime(
                                        JSCalendarDateTime::new(
                                            dt.with_timezone(&state.tz_start.unwrap_or_default())
                                                .to_naive_timestamp(),
                                            true,
                                        ),
                                    ))
                                }),
                        ),
                        (
                            JSCalendarProperty::Count,
                            value.count.map(|v| Value::Number((v as u64).into())),
                        ),
                        (
                            JSCalendarProperty::Interval,
                            value.interval.map(|v| Value::Number((v as u64).into())),
                        ),
                        (
                            JSCalendarProperty::FirstDayOfWeek,
                            value
                                .wkst
                                .map(|v| Value::Element(JSCalendarValue::Weekday(v))),
                        ),
                        (
                            JSCalendarProperty::Rscale,
                            value
                                .rscale
                                .map(|v| Value::Element(JSCalendarValue::CalendarScale(v))),
                        ),
                        (
                            JSCalendarProperty::Skip,
                            value.skip.map(|v| Value::Element(JSCalendarValue::Skip(v))),
                        ),
                    ] {
                        if let Some(value) = value {
                            rrule.insert_unchecked(key, value);
                        }
                    }

                    for (key, values) in [
                        (
                            JSCalendarProperty::BySecond,
                            (!value.bysecond.is_empty()).then(|| {
                                value
                                    .bysecond
                                    .iter()
                                    .map(|&v| Value::Number((v as u64).into()))
                                    .collect::<Vec<_>>()
                            }),
                        ),
                        (
                            JSCalendarProperty::ByMinute,
                            (!value.byminute.is_empty()).then(|| {
                                value
                                    .bysecond
                                    .iter()
                                    .map(|&v| Value::Number((v as u64).into()))
                                    .collect::<Vec<_>>()
                            }),
                        ),
                        (
                            JSCalendarProperty::ByHour,
                            (!value.byhour.is_empty()).then(|| {
                                value
                                    .byhour
                                    .iter()
                                    .map(|&v| Value::Number((v as u64).into()))
                                    .collect::<Vec<_>>()
                            }),
                        ),
                        (
                            JSCalendarProperty::ByDay,
                            (!value.byday.is_empty()).then(|| {
                                value
                                    .byday
                                    .iter()
                                    .map(|v| {
                                        Value::Object(Map::from_iter(
                                            [
                                                Some((
                                                    Key::Property(JSCalendarProperty::Day),
                                                    Value::Element(JSCalendarValue::Weekday(
                                                        v.weekday,
                                                    )),
                                                )),
                                                v.ordwk.map(|v| {
                                                    (
                                                        Key::Property(
                                                            JSCalendarProperty::NthOfPeriod,
                                                        ),
                                                        Value::Number((v as i64).into()),
                                                    )
                                                }),
                                            ]
                                            .into_iter()
                                            .flatten(),
                                        ))
                                    })
                                    .collect::<Vec<_>>()
                            }),
                        ),
                        (
                            JSCalendarProperty::ByMonthDay,
                            (!value.bymonthday.is_empty()).then(|| {
                                value
                                    .bymonthday
                                    .iter()
                                    .map(|&v| Value::Number((v as i64).into()))
                                    .collect::<Vec<_>>()
                            }),
                        ),
                        (
                            JSCalendarProperty::ByYearDay,
                            (!value.byyearday.is_empty()).then(|| {
                                value
                                    .byyearday
                                    .iter()
                                    .map(|&v| Value::Number((v as i64).into()))
                                    .collect::<Vec<_>>()
                            }),
                        ),
                        (
                            JSCalendarProperty::ByWeekNo,
                            (!value.byweekno.is_empty()).then(|| {
                                value
                                    .byweekno
                                    .iter()
                                    .map(|&v| Value::Number((v as i64).into()))
                                    .collect::<Vec<_>>()
                            }),
                        ),
                        (
                            JSCalendarProperty::ByMonth,
                            (!value.bymonth.is_empty()).then(|| {
                                value
                                    .bymonth
                                    .iter()
                                    .map(|&v| Value::Element(JSCalendarValue::Month(v)))
                                    .collect::<Vec<_>>()
                            }),
                        ),
                        (
                            JSCalendarProperty::BySetPosition,
                            (!value.bysetpos.is_empty()).then(|| {
                                value
                                    .bysetpos
                                    .iter()
                                    .map(|&v| Value::Number((v as i64).into()))
                                    .collect::<Vec<_>>()
                            }),
                        ),
                    ] {
                        if let Some(values) = values {
                            rrule.insert_unchecked(key, Value::Array(values));
                        }
                    }

                    state.entries.insert(
                        Key::Property(JSCalendarProperty::RecurrenceRule),
                        Value::Object(rrule),
                    );

                    entry.set_converted_to(&[JSCalendarProperty::RecurrenceRule
                        .to_string()
                        .as_ref()]);
                }
                (
                    ICalendarProperty::Method,
                    Some(ICalendarValue::Method(value)),
                    ICalendarComponentType::VCalendar,
                ) => {
                    state.method = Some(value);
                }
                (
                    ICalendarProperty::PercentComplete,
                    Some(ICalendarValue::Integer(value)),
                    ICalendarComponentType::VTodo,
                ) => {
                    state.entries.insert(
                        Key::Property(JSCalendarProperty::PercentComplete),
                        Value::Number(value.into()),
                    );
                    entry.set_converted_to(&[JSCalendarProperty::PercentComplete
                        .to_string()
                        .as_ref()]);
                }
                (
                    ICalendarProperty::Priority,
                    Some(ICalendarValue::Integer(value)),
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo,
                ) => {
                    state.entries.insert(
                        Key::Property(JSCalendarProperty::Priority),
                        Value::Number(value.into()),
                    );
                    entry.set_converted_to(&[JSCalendarProperty::Priority.to_string().as_ref()]);
                }
                (
                    ICalendarProperty::Sequence,
                    Some(ICalendarValue::Integer(value)),
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo,
                ) => {
                    state.entries.insert(
                        Key::Property(JSCalendarProperty::Sequence),
                        Value::Number(value.into()),
                    );
                    entry.set_converted_to(&[JSCalendarProperty::Sequence.to_string().as_ref()]);
                }
                (
                    ICalendarProperty::Prodid,
                    Some(ICalendarValue::Text(value)),
                    ICalendarComponentType::VCalendar,
                ) => {
                    state.entries.insert(
                        Key::Property(JSCalendarProperty::ProdId),
                        Value::Str(value.into()),
                    );
                    entry.set_converted_to(&[JSCalendarProperty::ProdId.to_string().as_ref()]);
                }
                (
                    ICalendarProperty::RelatedTo,
                    Some(ICalendarValue::Text(value)),
                    ICalendarComponentType::VEvent
                    | ICalendarComponentType::VTodo
                    | ICalendarComponentType::VAlarm,
                ) => {
                    let mut rels: Map<'_, JSCalendarProperty, JSCalendarValue> = Map::from(vec![]);
                    for param in std::mem::take(&mut entry.entry.params) {
                        match (param.name, param.value) {
                            (
                                ICalendarParameterName::Reltype,
                                ICalendarParameterValue::Reltype(value),
                            ) => {
                                rels.insert(
                                    match value {
                                        ICalendarRelationshipType::Child => {
                                            Key::Property(JSCalendarProperty::RelationValue(
                                                JSCalendarRelation::Child,
                                            ))
                                        }
                                        ICalendarRelationshipType::Parent => {
                                            Key::Property(JSCalendarProperty::RelationValue(
                                                JSCalendarRelation::Parent,
                                            ))
                                        }
                                        ICalendarRelationshipType::Snooze => {
                                            Key::Property(JSCalendarProperty::RelationValue(
                                                JSCalendarRelation::Snooze,
                                            ))
                                        }
                                        ICalendarRelationshipType::First => {
                                            Key::Property(JSCalendarProperty::RelationValue(
                                                JSCalendarRelation::First,
                                            ))
                                        }
                                        ICalendarRelationshipType::Next => {
                                            Key::Property(JSCalendarProperty::RelationValue(
                                                JSCalendarRelation::Next,
                                            ))
                                        }
                                        other => Key::Borrowed(other.as_str()),
                                    },
                                    Value::Bool(true),
                                );
                            }
                            (
                                ICalendarParameterName::Reltype,
                                ICalendarParameterValue::Text(value),
                            ) => {
                                rels.insert(Key::Owned(value), Value::Bool(true));
                            }
                            (name, value) => {
                                entry.entry.params.push(ICalendarParameter { name, value });
                            }
                        }
                    }

                    state
                        .entries
                        .entry(Key::Property(JSCalendarProperty::RelatedTo))
                        .or_insert_with(Value::new_object)
                        .as_object_mut()
                        .unwrap()
                        .insert(
                            Key::Owned(value),
                            Value::Object(Map::from(vec![(
                                Key::Property(JSCalendarProperty::Relation),
                                Value::Object(rels),
                            )])),
                        );
                    entry.set_converted_to(&[JSCalendarProperty::RelatedTo.to_string().as_ref()]);
                }
                (
                    ICalendarProperty::ShowWithoutTime,
                    Some(ICalendarValue::Boolean(value)),
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo,
                ) => {
                    state.entries.insert(
                        Key::Property(JSCalendarProperty::ShowWithoutTime),
                        Value::Bool(value),
                    );
                    entry.set_converted_to(&[JSCalendarProperty::ShowWithoutTime
                        .to_string()
                        .as_ref()]);
                }
                (
                    ICalendarProperty::Status,
                    Some(ICalendarValue::Status(value)),
                    ICalendarComponentType::VEvent,
                ) => {
                    state.entries.insert(
                        Key::Property(JSCalendarProperty::Status),
                        match value {
                            ICalendarStatus::Tentative => Value::Element(
                                JSCalendarValue::EventStatus(JSCalendarEventStatus::Tentative),
                            ),
                            ICalendarStatus::Confirmed => Value::Element(
                                JSCalendarValue::EventStatus(JSCalendarEventStatus::Confirmed),
                            ),
                            ICalendarStatus::Cancelled => Value::Element(
                                JSCalendarValue::EventStatus(JSCalendarEventStatus::Cancelled),
                            ),
                            other => Value::Str(other.as_str().into()),
                        },
                    );
                    entry.set_converted_to(&[JSCalendarProperty::Status.to_string().as_ref()]);
                }
                (
                    ICalendarProperty::Status,
                    Some(ICalendarValue::Status(value)),
                    ICalendarComponentType::VTodo,
                ) => {
                    state.entries.insert(
                        Key::Property(JSCalendarProperty::Progress),
                        match value {
                            ICalendarStatus::Cancelled => Value::Element(
                                JSCalendarValue::Progress(JSCalendarProgress::Cancelled),
                            ),
                            ICalendarStatus::NeedsAction => Value::Element(
                                JSCalendarValue::Progress(JSCalendarProgress::NeedsAction),
                            ),
                            ICalendarStatus::Completed => Value::Element(
                                JSCalendarValue::Progress(JSCalendarProgress::Completed),
                            ),
                            ICalendarStatus::InProcess => Value::Element(
                                JSCalendarValue::Progress(JSCalendarProgress::InProcess),
                            ),
                            ICalendarStatus::Failed => Value::Element(JSCalendarValue::Progress(
                                JSCalendarProgress::Failed,
                            )),
                            other => Value::Str(other.as_str().into()),
                        },
                    );
                    entry.set_converted_to(&[JSCalendarProperty::Progress.to_string().as_ref()]);
                }
                (
                    ICalendarProperty::Status,
                    Some(ICalendarValue::Text(value)),
                    ICalendarComponentType::VCalendar | ICalendarComponentType::VTodo,
                ) => {
                    let prop = if matches!(self.component_type, ICalendarComponentType::VTodo) {
                        JSCalendarProperty::Progress
                    } else {
                        JSCalendarProperty::Status
                    };

                    entry.set_converted_to(&[prop.to_string().as_ref()]);
                    state
                        .entries
                        .insert(Key::Property(prop), Value::Str(value.into()));
                }
                (
                    ICalendarProperty::Source,
                    Some(ICalendarValue::Text(value)),
                    ICalendarComponentType::VCalendar,
                ) => {
                    state.entries.insert(
                        Key::Property(JSCalendarProperty::Source),
                        Value::Str(value.into()),
                    );
                    entry.set_converted_to(&[JSCalendarProperty::Source.to_string().as_ref()]);
                }
                (
                    ICalendarProperty::Transp,
                    Some(ICalendarValue::Transparency(value)),
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo,
                ) => {
                    state.entries.insert(
                        Key::Property(JSCalendarProperty::FreeBusyStatus),
                        Value::Element(JSCalendarValue::FreeBusyStatus(match value {
                            ICalendarTransparency::Opaque => JSCalendarFreeBusyStatus::Busy,
                            ICalendarTransparency::Transparent => JSCalendarFreeBusyStatus::Free,
                        })),
                    );
                    entry.set_converted_to(&[JSCalendarProperty::FreeBusyStatus
                        .to_string()
                        .as_ref()]);
                }
                (
                    ICalendarProperty::Trigger,
                    Some(ICalendarValue::Duration(value)),
                    ICalendarComponentType::VAlarm,
                ) => {
                    let mut obj = Map::from(vec![
                        (
                            Key::Property(JSCalendarProperty::Type),
                            Value::Element(JSCalendarValue::Type(JSCalendarType::OffsetTrigger)),
                        ),
                        (
                            Key::Property(JSCalendarProperty::Offset),
                            Value::Element(JSCalendarValue::Duration(value)),
                        ),
                    ]);

                    for param in std::mem::take(&mut entry.entry.params) {
                        match (param.name, param.value) {
                            (
                                ICalendarParameterName::Related,
                                ICalendarParameterValue::Related(value),
                            ) => {
                                obj.insert(
                                    Key::Property(JSCalendarProperty::RelativeTo),
                                    Value::Element(JSCalendarValue::RelativeTo(match value {
                                        ICalendarRelated::Start => JSCalendarRelativeTo::Start,
                                        ICalendarRelated::End => JSCalendarRelativeTo::End,
                                    })),
                                );
                            }
                            (name, value) => {
                                entry.entry.params.push(ICalendarParameter { name, value });
                            }
                        }
                    }

                    state.entries.insert(
                        Key::Property(JSCalendarProperty::Trigger),
                        Value::Object(obj),
                    );
                    entry.set_converted_to(&[JSCalendarProperty::Trigger.to_string().as_ref()]);
                }
                (
                    ICalendarProperty::Trigger,
                    Some(ICalendarValue::PartialDateTime(value)),
                    ICalendarComponentType::VAlarm,
                ) if value.has_date_and_time() => {
                    state.entries.insert(
                        Key::Property(JSCalendarProperty::Trigger),
                        Value::Object(Map::from(vec![
                            (
                                Key::Property(JSCalendarProperty::Type),
                                Value::Element(JSCalendarValue::Type(
                                    JSCalendarType::AbsoluteTrigger,
                                )),
                            ),
                            (
                                Key::Property(JSCalendarProperty::When),
                                Value::Element(JSCalendarValue::DateTime(JSCalendarDateTime::new(
                                    value.to_timestamp().unwrap_or_default(),
                                    false,
                                ))),
                            ),
                        ])),
                    );
                    entry.set_converted_to(&[JSCalendarProperty::Trigger.to_string().as_ref()]);
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
