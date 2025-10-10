/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

use crate::{
    common::timezone::TzTimestamp,
    icalendar::{timezone::TzResolver, *},
    jscalendar::{
        import::{ConversionOptions, EntryState, State, params::ExtractParams},
        *,
    },
};
use jmap_tools::{JsonPointer, Key, Map, Value};

impl ICalendar {
    pub fn into_jscalendar<I: JSCalendarId, B: JSCalendarId>(
        mut self,
    ) -> JSCalendar<'static, I, B> {
        JSCalendar(
            self.to_jscalendar(
                &self.build_owned_tz_resolver(),
                0,
                &ConversionOptions::default(),
            )
            .into_object(),
        )
    }

    pub fn into_jscalendar_with_opt<I: JSCalendarId, B: JSCalendarId>(
        mut self,
        options: ConversionOptions,
    ) -> JSCalendar<'static, I, B> {
        JSCalendar(
            self.to_jscalendar(&self.build_owned_tz_resolver(), 0, &options)
                .into_object(),
        )
    }
}

impl ICalendar {
    #[allow(clippy::wrong_self_convention)]
    pub(super) fn to_jscalendar<I: JSCalendarId, B: JSCalendarId>(
        &mut self,
        tz_resolver: &TzResolver<String>,
        component_id: u32,
        options: &ConversionOptions,
    ) -> State<I, B> {
        let mut state = State {
            include_ical_components: options.include_ical_components,
            ..Default::default()
        };

        // Take component
        let Some(component) = self.components.get_mut(component_id as usize) else {
            debug_assert!(false, "Invalid component ID {component_id}");
            return state;
        };
        let mut entries = std::mem::take(&mut component.entries);
        state.component_type = component.component_type.clone();

        // Process subcomponents
        let mut group_components = Vec::new();
        let mut alarm_component_ids = Vec::new();
        let mut alarm_with_related = false;
        let mut unsupported_component_ids = Vec::new();
        let mut uid_jsid_mappings = Vec::new();
        let mut has_locations = false;

        for component_id in std::mem::take(&mut component.component_ids) {
            let component = &mut self.components[component_id as usize];
            match (&state.component_type, &component.component_type) {
                (
                    ICalendarComponentType::VCalendar,
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo,
                ) => {
                    group_components.push(self.to_jscalendar(tz_resolver, component_id, options));
                }
                (
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo,
                    component_type @ (ICalendarComponentType::Participant
                    | ICalendarComponentType::VLocation),
                ) => {
                    let entries = state.get_mut_object_or_insert(match component_type {
                        ICalendarComponentType::Participant => JSCalendarProperty::Participants,
                        ICalendarComponentType::VLocation => {
                            has_locations = true;
                            JSCalendarProperty::Locations
                        }
                        _ => unreachable!(),
                    });
                    let mut subcomponent_state =
                        self.to_jscalendar(tz_resolver, component_id, options);
                    let jsid = subcomponent_state.jsid.take();
                    let subcomponent = subcomponent_state.into_object();
                    entries.insert_named(jsid, subcomponent);
                }
                (
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo,
                    ICalendarComponentType::VAlarm,
                ) => {
                    let mut jsid = None;

                    for entry in &mut component.entries {
                        match &entry.name {
                            ICalendarProperty::RelatedTo => {
                                alarm_with_related = true;
                            }
                            ICalendarProperty::Jsid => {
                                jsid = std::mem::take(&mut entry.values)
                                    .pop()
                                    .and_then(|v| v.into_text());
                            }
                            _ => {}
                        }
                    }
                    let jsid = jsid.unwrap_or_else(|| {
                        Cow::Owned(format!("k{}", alarm_component_ids.len() + 1))
                    });
                    if let Some(uid) = component.uid() {
                        uid_jsid_mappings.push((uid.to_string(), jsid.clone()));
                    }
                    alarm_component_ids.push((component_id, jsid));
                }
                _ => {
                    unsupported_component_ids.push(component_id);
                }
            }
        }

        // Process alarms
        if !alarm_component_ids.is_empty() {
            for (component_id, jsid) in alarm_component_ids {
                if alarm_with_related {
                    for entry in &mut self.components[component_id as usize].entries {
                        if matches!(entry.name, ICalendarProperty::RelatedTo) {
                            for value in &mut entry.values {
                                if let Some((_, jsid)) = value.as_text().and_then(|uid| {
                                    uid_jsid_mappings.iter().find(|(u, _)| u == uid)
                                }) {
                                    *value = ICalendarValue::Text(jsid.to_string());
                                }
                            }
                        }
                    }
                }

                state
                    .get_mut_object_or_insert(JSCalendarProperty::Alerts)
                    .insert_unchecked(
                        Key::from(jsid),
                        self.to_jscalendar(tz_resolver, component_id, options)
                            .into_object(),
                    );
            }
        }

        // Build group component list
        let mut group_objects = Vec::with_capacity(group_components.len());
        if !group_components.is_empty() {
            // Order by UID placing the main components first
            group_components.sort_unstable_by(|a, b| match a.uid.cmp(&b.uid) {
                std::cmp::Ordering::Equal => {
                    a.recurrence_id.is_some().cmp(&b.recurrence_id.is_some())
                }
                other => other,
            });
            let mut group_components = group_components.into_iter().peekable();

            // Bundle recurrence overrides together by UID
            while let Some(mut component) = group_components.next() {
                if component.uid.is_some() && component.recurrence_id.is_none() {
                    while group_components
                        .peek()
                        .is_some_and(|r| r.uid == component.uid && r.recurrence_id.is_some())
                    {
                        let mut recurrence = group_components.next().unwrap();
                        let recurrence_id = recurrence
                            .recurrence_id
                            .take()
                            .unwrap()
                            .with_timezone(&component.tz_start.unwrap_or_default())
                            .to_naive_timestamp();
                        let _ = recurrence.uid.take();

                        recurrence.set_is_recurrence_instance();

                        component
                            .get_mut_object_or_insert(JSCalendarProperty::RecurrenceOverrides)
                            .insert(
                                Key::Property(JSCalendarProperty::DateTime(
                                    JSCalendarDateTime::new(recurrence_id, true),
                                )),
                                recurrence.into_object(),
                            );
                    }
                }

                if !options.return_first {
                    group_objects.push(component.into_object());
                } else {
                    return component;
                }
            }
        }

        // Build unsupported component jcal
        if options.include_ical_components && !unsupported_component_ids.is_empty() {
            let mut component_iter = unsupported_component_ids.into_iter();
            let mut component_stack = Vec::with_capacity(4);
            let mut components = vec![];

            loop {
                if let Some(component_id) = component_iter.next() {
                    let component = self.components.get_mut(component_id as usize).unwrap();

                    let mut entries = Vec::with_capacity(component.entries.len());
                    for entry in std::mem::take(&mut component.entries) {
                        entries.push(EntryState::new(entry).into_jcal());
                    }

                    components.push(Value::Array(vec![
                        Value::Str(std::mem::take(&mut component.component_type).into_string()),
                        Value::Array(entries),
                        Value::Array(vec![]),
                    ]));

                    if !component.component_ids.is_empty() {
                        component_stack.push((components, component_iter));
                        component_iter = std::mem::take(&mut component.component_ids).into_iter();
                        components = vec![];
                    }
                } else if let Some((mut parent_components, iter)) = component_stack.pop() {
                    if let Some(parent_component) = parent_components
                        .last_mut()
                        .and_then(|v| v.as_array_mut())
                        .and_then(|v| v.last_mut())
                        .and_then(|v| v.as_array_mut())
                    {
                        if !parent_component.is_empty() {
                            parent_component.extend(components);
                        } else {
                            *parent_component = components;
                        }
                    } else {
                        debug_assert!(false, "Invalid component stack state");
                    }
                    components = parent_components;
                    component_iter = iter;
                } else {
                    break;
                }
            }

            state.ical_components = Some(Value::Array(components));
        }

        let mut main_location_id = None;

        entries.sort_by_key(|entry| match &entry.name {
            ICalendarProperty::Dtstart | ICalendarProperty::Jsid => 0,
            ICalendarProperty::RecurrenceId => 1,
            ICalendarProperty::Dtend
            | ICalendarProperty::Due
            | ICalendarProperty::Location
            | ICalendarProperty::Name => 2,
            _ => 3,
        });
        let mut start_date = None;

        for entry in entries {
            let mut entry = EntryState::new(entry);
            let mut values = std::mem::take(&mut entry.entry.values).into_iter();

            match (
                &entry.entry.name,
                values.next().map(|v| v.uri_to_string()),
                &state.component_type,
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
                    entry.set_converted_to::<I>(&[JSCalendarProperty::Acknowledged::<I>
                        .to_string()
                        .as_ref()]);
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
                    entry.set_converted_to::<I>(&[JSCalendarProperty::Action::<I>
                        .to_string()
                        .as_ref()]);
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
                    let is_task = matches!(&state.component_type, ICalendarComponentType::VTodo);
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
                        ],
                    );
                }
                (
                    ICalendarProperty::CalendarAddress,
                    Some(ICalendarValue::Text(value)),
                    ICalendarComponentType::Participant,
                ) => {
                    if state.jsid.is_none() {
                        state.jsid = Some(uuid5(&value));
                    }
                    state.entries.insert(
                        Key::Property(JSCalendarProperty::CalendarAddress),
                        Value::Str(value.into()),
                    );
                    entry.set_converted_to::<I>(&[JSCalendarProperty::CalendarAddress::<I>
                        .to_string()
                        .as_ref()]);
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
                    entry.set_converted_to::<I>(&[JSCalendarProperty::Keywords::<I>
                        .to_string()
                        .as_ref()]);
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
                    entry.set_converted_to::<I>(&[JSCalendarProperty::Privacy::<I>
                        .to_string()
                        .as_ref()]);
                }
                (
                    ICalendarProperty::Class,
                    Some(ICalendarValue::Text(value)),
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo,
                ) => {
                    state.entries.insert(
                        Key::Property(JSCalendarProperty::Privacy),
                        Value::Str(value.into()),
                    );
                    entry.set_converted_to::<I>(&[JSCalendarProperty::Privacy::<I>
                        .to_string()
                        .as_ref()]);
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
                    entry.set_converted_to::<I>(&[JSCalendarProperty::Color::<I>
                        .to_string()
                        .as_ref()]);
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
                    entry.set_converted_to::<I>(&[JSCalendarProperty::Categories::<I>
                        .to_string()
                        .as_ref()]);
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
                    if state.jsid.is_none() {
                        state.jsid = Some(uuid5(&value));
                    }
                    state.entries.insert(
                        Key::Property(JSCalendarProperty::Coordinates),
                        Value::Str(value.into()),
                    );
                    entry.set_converted_to::<I>(&[JSCalendarProperty::Coordinates::<I>
                        .to_string()
                        .as_ref()]);
                }
                (
                    ICalendarProperty::Geo,
                    Some(ICalendarValue::Float(coord1)),
                    ICalendarComponentType::VLocation,
                ) if !entry.entry.is_derived() => {
                    let coord2 = values.next().and_then(|v| v.as_float()).unwrap_or_default();

                    state.entries.insert(
                        Key::Property(JSCalendarProperty::Coordinates),
                        Value::Str(format!("geo:{coord1},{coord2}").into()),
                    );
                    entry.set_converted_to::<I>(&[JSCalendarProperty::Coordinates::<I>
                        .to_string()
                        .as_ref()]);
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
                    if state.jsid.is_none() {
                        state.jsid = Some(uuid5(&value));
                    }
                    state.entries.insert(
                        Key::Property(JSCalendarProperty::Name),
                        Value::Str(value.into()),
                    );
                    entry.set_converted_to::<I>(&[JSCalendarProperty::Name::<I>
                        .to_string()
                        .as_ref()]);
                    state.set_map_component();
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
                    state
                        .entries
                        .extract_params(&mut entry.entry, &[ICalendarParameterName::Language]);
                    state.entries.insert(
                        Key::Property(JSCalendarProperty::Title),
                        Value::Str(value.into()),
                    );
                    entry.set_converted_to::<I>(&[JSCalendarProperty::Title::<I>
                        .to_string()
                        .as_ref()]);
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
                    entry.set_converted_to::<I>(&[JSCalendarProperty::Name::<I>
                        .to_string()
                        .as_ref()]);
                    state.set_map_component();
                }
                (
                    ICalendarProperty::Location,
                    Some(ICalendarValue::Text(value)),
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo,
                ) => {
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
                        &[
                            ICalendarParameterName::Jsid,
                            ICalendarParameterName::Derived,
                        ],
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
                    entry.set_converted_to::<I>(&[JSCalendarProperty::LocationTypes::<I>
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
                    entry.set_converted_to::<I>(&[JSCalendarProperty::Updated::<I>
                        .to_string()
                        .as_ref()]);
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
                    entry.set_converted_to::<I>(&[JSCalendarProperty::Created::<I>
                        .to_string()
                        .as_ref()]);
                }
                (
                    ICalendarProperty::Description,
                    Some(ICalendarValue::Text(value)),
                    ICalendarComponentType::VEvent
                    | ICalendarComponentType::VTodo
                    | ICalendarComponentType::Participant
                    | ICalendarComponentType::VCalendar
                    | ICalendarComponentType::VLocation,
                ) if !entry.entry.is_derived() => {
                    state.entries.insert(
                        Key::Property(JSCalendarProperty::Description),
                        Value::Str(value.into()),
                    );
                    entry.set_converted_to::<I>(&[JSCalendarProperty::Description::<I>
                        .to_string()
                        .as_ref()]);
                    if matches!(state.component_type, ICalendarComponentType::Participant) {
                        state.set_map_component();
                    }
                }
                (
                    ICalendarProperty::StyledDescription,
                    Some(ICalendarValue::Text(value)),
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo,
                ) if !entry.entry.is_derived()
                    && entry
                        .entry
                        .parameter(&ICalendarParameterName::Fmttype)
                        .is_none_or(|v| v.as_text().is_some_and(|v| v.starts_with("text/"))) =>
                {
                    state
                        .entries
                        .extract_params(&mut entry.entry, &[ICalendarParameterName::Fmttype]);
                    state.entries.insert(
                        Key::Property(JSCalendarProperty::Description),
                        Value::Str(value.into()),
                    );
                    entry.set_converted_to::<I>(&[JSCalendarProperty::Description::<I>
                        .to_string()
                        .as_ref()]);
                    if !state
                        .entries
                        .contains_key(&Key::Property(JSCalendarProperty::DescriptionContentType))
                    {
                        entry.set_map_name();
                    }
                }
                (
                    ICalendarProperty::Dtstart,
                    Some(ICalendarValue::PartialDateTime(value)),
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo,
                ) if value.has_date() => {
                    let tzid = entry.entry.tz_id();
                    state.tz_start = tzid.and_then(|v| tz_resolver.resolve(v));
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

                        // Remove IANA TZ references
                        if tzid.is_some()
                            && state.tz_start.and_then(|tz| tz.name()).as_deref() == tzid
                        {
                            entry
                                .entry
                                .params
                                .retain(|p| p.name != ICalendarParameterName::Tzid);
                        }

                        if !value.has_time() {
                            state.entries.insert(
                                Key::Property(JSCalendarProperty::ShowWithoutTime),
                                Value::Bool(true),
                            );
                        }
                        entry.set_converted_to::<I>(&[JSCalendarProperty::Start::<I>
                            .to_string()
                            .as_ref()]);
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
                    let tzid = entry.entry.tz_id();
                    state.tz_end = tzid.and_then(|v| tz_resolver.resolve(v)).or(state.tz_start);
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

                        // Remove IANA TZ references
                        if tzid.is_some()
                            && state.tz_end.and_then(|tz| tz.name()).as_deref() == tzid
                        {
                            entry
                                .entry
                                .params
                                .retain(|p| p.name != ICalendarParameterName::Tzid);
                        }

                        if !value.has_time() {
                            state.entries.insert(
                                Key::Property(JSCalendarProperty::ShowWithoutTime),
                                Value::Bool(true),
                            );
                        }
                        entry.set_converted_to::<I>(&[JSCalendarProperty::Duration::<I>
                            .to_string()
                            .as_ref()]);
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
                    let tzid = entry.entry.tz_id();
                    let due_tz = tzid.and_then(|v| tz_resolver.resolve(v)).or(state.tz_start);
                    if let Some(dt) = value
                        .to_date_time()
                        .and_then(|dt| dt.to_date_time_with_tz(due_tz.unwrap_or_default()))
                    {
                        // Remove IANA TZ references
                        if tzid.is_some()
                            && due_tz != state.tz_start
                            && due_tz.and_then(|tz| tz.name()).as_deref() == tzid
                        {
                            entry
                                .entry
                                .params
                                .retain(|p| p.name != ICalendarParameterName::Tzid);
                        }

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

                        entry.set_converted_to::<I>(&[JSCalendarProperty::Due::<I>
                            .to_string()
                            .as_ref()]);
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

                    entry.set_converted_to::<I>(&[JSCalendarProperty::Updated::<I>
                        .to_string()
                        .as_ref()]);
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

                    entry.set_converted_to::<I>(&[JSCalendarProperty::Duration::<I>
                        .to_string()
                        .as_ref()]);
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

                    entry.set_converted_to::<I>(&[JSCalendarProperty::EstimatedDuration::<I>
                        .to_string()
                        .as_ref()]);
                }
                (
                    ICalendarProperty::RecurrenceId,
                    Some(ICalendarValue::PartialDateTime(value)),
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo,
                ) if value.has_date_and_time() => {
                    let tzid = entry.entry.tz_id();
                    let rid_tz = tzid.and_then(|v| tz_resolver.resolve(v)).or(state.tz_start);

                    if let Some(dt) = value
                        .to_date_time()
                        .and_then(|dt| dt.to_date_time_with_tz(rid_tz.unwrap_or_default()))
                    {
                        state.has_dates = true;
                        state.recurrence_id = Some(dt);

                        // Remove IANA TZ references
                        if tzid.is_some() && rid_tz.and_then(|tz| tz.name()).as_deref() == tzid {
                            entry
                                .entry
                                .params
                                .retain(|p| p.name != ICalendarParameterName::Tzid);
                        }

                        entry.set_converted_to::<I>(&[JSCalendarProperty::RecurrenceId::<I>
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
                    let tzid = entry.entry.tz_id();
                    let tz = tzid.and_then(|v| tz_resolver.resolve(v)).or(state.tz_start);

                    if let Some(dt) = value
                        .to_date_time()
                        .and_then(|dt| dt.to_date_time_with_tz(tz.unwrap_or_default()))
                    {
                        state.has_dates = true;

                        // Remove IANA TZ references
                        if tzid.is_some()
                            && tz == state.tz_start
                            && tz.and_then(|tz| tz.name()).as_deref() == tzid
                        {
                            entry
                                .entry
                                .params
                                .retain(|p| p.name != ICalendarParameterName::Tzid);
                        }

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
                                    .and_then(|dt| dt.to_date_time_with_tz(tz.unwrap_or_default()))
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
                                entry.set_converted_to::<I>(&[
                                    JSCalendarProperty::RecurrenceOverrides::<I>
                                        .to_string()
                                        .as_ref(),
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
                                    .byminute
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

                    entry.set_converted_to::<I>(&[JSCalendarProperty::RecurrenceRule::<I>
                        .to_string()
                        .as_ref()]);
                }
                (
                    ICalendarProperty::Method,
                    Some(ICalendarValue::Method(value)),
                    ICalendarComponentType::VCalendar,
                ) => {
                    for object in &mut group_objects {
                        object.as_object_mut().unwrap().insert_unchecked(
                            Key::Property(JSCalendarProperty::Method),
                            Value::Element(JSCalendarValue::Method(value.clone())),
                        );
                    }
                    continue;
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
                    entry.set_converted_to::<I>(&[JSCalendarProperty::PercentComplete::<I>
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
                    entry.set_converted_to::<I>(&[JSCalendarProperty::Priority::<I>
                        .to_string()
                        .as_ref()]);
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
                    entry.set_converted_to::<I>(&[JSCalendarProperty::Sequence::<I>
                        .to_string()
                        .as_ref()]);
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
                    entry.set_converted_to::<I>(&[JSCalendarProperty::ProdId::<I>
                        .to_string()
                        .as_ref()]);
                }
                (
                    ICalendarProperty::RelatedTo,
                    Some(ICalendarValue::Text(value)),
                    ICalendarComponentType::VEvent
                    | ICalendarComponentType::VTodo
                    | ICalendarComponentType::VAlarm,
                ) => {
                    let mut rels: Map<'_, JSCalendarProperty<I>, JSCalendarValue<I, B>> =
                        Map::from(vec![]);
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

                    entry.set_converted_to::<I>(&[
                        JSCalendarProperty::RelatedTo::<I>.to_string().as_ref(),
                        value.as_str(),
                    ]);

                    state
                        .entries
                        .entry(Key::Property(JSCalendarProperty::RelatedTo))
                        .or_insert_with(Value::new_object)
                        .as_object_mut()
                        .unwrap()
                        .insert(
                            Key::Owned(value),
                            Value::Object(Map::from(if !rels.is_empty() {
                                vec![(
                                    Key::Property(JSCalendarProperty::Relation),
                                    Value::Object(rels),
                                )]
                            } else {
                                vec![]
                            })),
                        );
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
                    entry.set_converted_to::<I>(&[JSCalendarProperty::ShowWithoutTime::<I>
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
                    entry.set_converted_to::<I>(&[JSCalendarProperty::Status::<I>
                        .to_string()
                        .as_ref()]);
                }
                (
                    ICalendarProperty::Status,
                    Some(ICalendarValue::Status(value)),
                    ICalendarComponentType::VTodo,
                ) => {
                    state.entries.insert(
                        Key::Property(JSCalendarProperty::Progress),
                        match value {
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
                    entry.set_converted_to::<I>(&[JSCalendarProperty::Progress::<I>
                        .to_string()
                        .as_ref()]);
                }
                (
                    ICalendarProperty::Status,
                    Some(ICalendarValue::Text(value)),
                    ICalendarComponentType::VCalendar | ICalendarComponentType::VTodo,
                ) => {
                    let prop = if matches!(state.component_type, ICalendarComponentType::VTodo) {
                        JSCalendarProperty::Progress
                    } else {
                        JSCalendarProperty::Status
                    };

                    entry.set_converted_to::<I>(&[prop.to_string().as_ref()]);
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
                    entry.set_converted_to::<I>(&[JSCalendarProperty::Source::<I>
                        .to_string()
                        .as_ref()]);
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
                    entry.set_converted_to::<I>(&[JSCalendarProperty::FreeBusyStatus::<I>
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
                    entry.set_converted_to::<I>(&[JSCalendarProperty::Trigger::<I>
                        .to_string()
                        .as_ref()]);
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
                    entry.set_converted_to::<I>(&[JSCalendarProperty::Trigger::<I>
                        .to_string()
                        .as_ref()]);
                }
                (
                    ICalendarProperty::Uid,
                    Some(ICalendarValue::Text(value)),
                    ICalendarComponentType::VCalendar
                    | ICalendarComponentType::VEvent
                    | ICalendarComponentType::VTodo,
                ) => {
                    state.uid = Some(value);
                    continue;
                }
                (ICalendarProperty::Jsid, Some(ICalendarValue::Text(value)), _) => {
                    state.jsid = Some(value);
                    continue;
                }
                (ICalendarProperty::Jsid, _, _) => {
                    continue;
                }
                (ICalendarProperty::Jsprop, Some(ICalendarValue::Text(value)), _) => {
                    if let Some(ICalendarParameter {
                        name: ICalendarParameterName::Jsptr,
                        value: ICalendarParameterValue::Text(ptr),
                    }) = entry.entry.params.first()
                    {
                        let ptr = JsonPointer::<JSCalendarProperty<I>>::parse(ptr);

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

        if !group_objects.is_empty() {
            state.entries.insert(
                Key::Property(JSCalendarProperty::Entries),
                Value::Array(group_objects),
            );
        }

        state
    }
}
