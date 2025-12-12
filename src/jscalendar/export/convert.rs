/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

use crate::{
    common::{IanaParse, PartialDateTime, timezone::Tz},
    icalendar::*,
    jscalendar::{
        export::{ConvertedComponent, State},
        *,
    },
};
use ahash::{AHashMap, AHashSet};
use chrono::{TimeDelta, TimeZone};
use jmap_tools::{JsonPointer, JsonPointerHandler, JsonPointerItem, Key, Map, Value};
use std::str::FromStr;

impl<'x, I: JSCalendarId, B: JSCalendarId> JSCalendar<'x, I, B> {
    pub fn into_icalendar(self) -> Option<ICalendar> {
        self.0.into_object().map(|entries| {
            let mut ical = ICalendar::default();

            ical.from_jscalendar(
                State {
                    tz: Default::default(),
                    tz_end: Default::default(),
                    tz_rid: Default::default(),
                    start: Default::default(),
                    recurrence_id: Default::default(),
                    uid: Default::default(),
                    entries,
                    default_component_type: ICalendarComponentType::VCalendar,
                },
                None,
            );

            ical
        })
    }

    pub fn into_inner(self) -> Value<'x, JSCalendarProperty<I>, JSCalendarValue<I, B>> {
        self.0
    }
}

impl ICalendar {
    #[allow(clippy::wrong_self_convention)]
    pub(super) fn from_jscalendar<I: JSCalendarId, B: JSCalendarId>(
        &mut self,
        mut state: State<'_, I, B>,
        mut parent_component: Option<&mut ICalendarComponent>,
    ) {
        let mut root_conversions = None;
        let mut locale = None;
        let mut uid = state.uid.map(Cow::Borrowed);
        let mut organizer_address = None;
        let mut organizer_name = None;
        let mut main_location_id = None;
        let mut start = None;
        let mut component_type = None;
        let mut description_content_type = None;
        let mut overrides = None;

        for (key, value) in state.entries.as_mut_vec() {
            match (key, value) {
                (Key::Property(JSCalendarProperty::ICalendar), Value::Object(obj)) => {
                    root_conversions =
                        ConvertedComponent::try_from_object(std::mem::take(obj.as_mut_vec()));
                }
                (Key::Property(JSCalendarProperty::MainLocationId), Value::Str(text)) => {
                    main_location_id = Some(std::mem::take(text));
                }
                (Key::Property(JSCalendarProperty::TimeZone), Value::Str(text)) => {
                    state.tz = Tz::from_str(text.as_ref()).ok();
                }
                (Key::Property(JSCalendarProperty::EndTimeZone), Value::Str(text)) => {
                    state.tz_end = Tz::from_str(text.as_ref()).ok();
                }
                (Key::Property(JSCalendarProperty::RecurrenceIdTimeZone), Value::Str(text)) => {
                    state.tz_rid = Tz::from_str(text.as_ref()).ok();
                }
                (Key::Property(JSCalendarProperty::Uid), Value::Str(text)) => {
                    uid = Some(std::mem::take(text));
                }
                (Key::Property(JSCalendarProperty::Locale), Value::Str(text)) => {
                    locale = Some(std::mem::take(text));
                }
                (Key::Property(JSCalendarProperty::OrganizerCalendarAddress), Value::Str(text)) => {
                    organizer_address = Some(std::mem::take(text));
                }
                (Key::Property(JSCalendarProperty::DescriptionContentType), Value::Str(text)) => {
                    description_content_type = Some(std::mem::take(text));
                }
                (
                    Key::Property(JSCalendarProperty::Start),
                    Value::Element(JSCalendarValue::DateTime(dt)),
                ) if matches!(
                    state.default_component_type,
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo
                ) =>
                {
                    start = dt.to_naive_date_time();
                }
                (
                    Key::Property(JSCalendarProperty::Type),
                    Value::Element(JSCalendarValue::Type(typ)),
                ) => {
                    component_type = typ.to_icalendar_component_type();
                }
                (Key::Property(JSCalendarProperty::RecurrenceOverrides), Value::Object(obj))
                    if matches!(
                        state.default_component_type,
                        ICalendarComponentType::VEvent | ICalendarComponentType::VTodo
                    ) =>
                {
                    overrides = Some(std::mem::take(obj));
                }
                _ => (),
            }
        }

        // Apply override patch objects
        if let Some(overrides) = &mut overrides {
            for (_, value) in overrides.as_mut_vec().iter_mut() {
                let Value::Object(obj) = value else {
                    continue;
                };

                let mut patched_obj = None;

                for (key, value) in obj.as_mut_vec() {
                    if let Key::Property(JSCalendarProperty::Pointer(ptr)) = key {
                        patched_obj
                            .get_or_insert_with(|| Value::Object(state.entries.clone()))
                            .patch_jptr(ptr.iter(), std::mem::take(value));
                    }
                }

                if let Some(patched_obj) = patched_obj {
                    let mut patched_props: AHashMap<
                        Key<'static, JSCalendarProperty<I>>,
                        Vec<Key<'static, JSCalendarProperty<I>>>,
                    > = AHashMap::new();

                    for (key, value) in std::mem::take(obj.as_mut_vec()) {
                        if let Key::Property(JSCalendarProperty::Pointer(ptr)) = key {
                            let mut ptr = ptr.into_iter();
                            if let (
                                Some(JsonPointerItem::Key(key)),
                                Some(JsonPointerItem::Key(subkey)),
                            ) = (ptr.next(), ptr.next())
                            {
                                patched_props.entry(key).or_default().push(subkey);
                            }
                        } else {
                            obj.insert_unchecked(key, value);
                        }
                    }

                    for (key, patched_value) in patched_obj.into_expanded_object() {
                        if let Some(keys) = patched_props.get(&key) {
                            let Value::Object(obj) =
                                obj.insert_or_get_mut(key, Value::Object(Map::default()))
                            else {
                                continue;
                            };
                            for (key, value) in patched_value.into_expanded_object() {
                                if keys.contains(&key) {
                                    obj.insert_unchecked(key, value);
                                }
                            }
                        }
                    }
                }
            }
        }

        // Build component
        let mut component =
            ICalendarComponent::new(component_type.unwrap_or(state.default_component_type));
        if parent_component.is_none() {
            debug_assert!(self.components.is_empty());
            self.components
                .push(ICalendarComponent::new(ICalendarComponentType::VCalendar));
        }

        // Add start date
        if let Some(start) = start.and_then(|dt| {
            state
                .tz
                .unwrap_or_default()
                .from_local_datetime(&dt)
                .single()
        }) {
            state.start = Some(start);
        }
        if let Some(dt) = state.start {
            component.entries.push(
                ICalendarEntry::new(ICalendarProperty::Dtstart)
                    .import_converted(&[JSCalendarProperty::Start], &mut root_conversions)
                    .with_date_time(dt),
            );
        }

        // Add UID
        if let Some(uid) = &uid {
            component.entries.push(
                ICalendarEntry::new(ICalendarProperty::Uid)
                    .import_converted(&[JSCalendarProperty::Uid], &mut root_conversions)
                    .with_value(uid.clone().into_owned()),
            );
        }

        let mut show_without_time = None;
        let mut add_recurrence_id = state.recurrence_id.is_some();
        for (key, value) in state.entries.into_vec() {
            let Key::Property(property) = key else {
                component.insert_jsprop(&[key.to_string().as_ref()], value);
                continue;
            };

            match (&property, value, &component.component_type) {
                (
                    JSCalendarProperty::Links,
                    Value::Object(obj),
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo,
                ) => {
                    component.import_links(obj, &mut root_conversions);
                }
                (
                    JSCalendarProperty::Participants,
                    Value::Object(obj),
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo,
                ) => {
                    for (name, value) in obj.into_vec() {
                        let Value::Object(mut value) = value else {
                            continue;
                        };

                        let mut item_conversions = ConvertedComponent::build(&mut value);
                        let mut entry = ICalendarEntry::new(ICalendarProperty::Attendee);
                        let mut participant =
                            ICalendarComponent::new(ICalendarComponentType::Participant);
                        let mut participant_name = None;
                        let mut calendar_address = None;
                        let mut status = None;
                        let mut progress = None;
                        let mut description = None;
                        let mut description_content_type = None;
                        let mut is_uuid5_key = false;

                        for (sub_property, value) in value.into_vec() {
                            match (sub_property, value) {
                                (
                                    Key::Property(JSCalendarProperty::CalendarAddress),
                                    Value::Str(text),
                                ) => {
                                    is_uuid5_key = uuid5(text.as_ref()) == name.to_string();
                                    calendar_address = Some(text);
                                }
                                (
                                    Key::Property(JSCalendarProperty::DelegatedFrom),
                                    Value::Str(text),
                                ) => {
                                    entry.params.push(ICalendarParameter::delegated_from(
                                        Uri::parse(text.into_owned()),
                                    ));
                                }
                                (
                                    Key::Property(JSCalendarProperty::DelegatedTo),
                                    Value::Str(text),
                                ) => {
                                    entry.params.push(ICalendarParameter::delegated_to(
                                        Uri::parse(text.into_owned()),
                                    ));
                                }
                                (Key::Property(JSCalendarProperty::Email), Value::Str(text)) => {
                                    entry.params.push(ICalendarParameter::email(Uri::parse(
                                        text.into_owned(),
                                    )));
                                }
                                (
                                    Key::Property(JSCalendarProperty::ExpectReply),
                                    Value::Bool(value),
                                ) => {
                                    entry.params.push(ICalendarParameter::rsvp(value));
                                }
                                (
                                    Key::Property(JSCalendarProperty::Kind),
                                    Value::Element(JSCalendarValue::ParticipantKind(kind)),
                                ) => {
                                    entry.params.push(ICalendarParameter::cutype(match kind {
                                        JSCalendarParticipantKind::Individual => {
                                            ICalendarUserTypes::Individual
                                        }
                                        JSCalendarParticipantKind::Group => {
                                            ICalendarUserTypes::Group
                                        }
                                        JSCalendarParticipantKind::Resource => {
                                            ICalendarUserTypes::Resource
                                        }
                                        JSCalendarParticipantKind::Location => {
                                            ICalendarUserTypes::Room
                                        }
                                    }));
                                }
                                (Key::Property(JSCalendarProperty::Kind), Value::Str(text)) => {
                                    entry
                                        .params
                                        .push(ICalendarParameter::cutype(text.to_lowercase()));
                                }
                                (
                                    Key::Property(JSCalendarProperty::MemberOf),
                                    Value::Object(obj),
                                ) => {
                                    for key in obj.into_expanded_boolean_set() {
                                        entry.params.push(ICalendarParameter::member(Uri::parse(
                                            key.into_string(),
                                        )));
                                    }
                                }
                                (Key::Property(JSCalendarProperty::Name), Value::Str(text)) => {
                                    participant_name = Some(text);
                                }
                                (
                                    Key::Property(JSCalendarProperty::ParticipationStatus),
                                    Value::Element(JSCalendarValue::ParticipationStatus(status_)),
                                ) => {
                                    status = Some(status_);
                                }
                                (
                                    Key::Property(JSCalendarProperty::Progress),
                                    Value::Element(JSCalendarValue::Progress(progress_)),
                                ) => {
                                    progress = Some(progress_);
                                }
                                (Key::Property(JSCalendarProperty::Roles), Value::Object(obj)) => {
                                    for key in obj.into_expanded_boolean_set() {
                                        match key {
                                            Key::Property(JSCalendarProperty::ParticipantRole(
                                                role,
                                            )) => {
                                                let role = match role {
                                                    JSCalendarParticipantRole::Optional => {
                                                        ICalendarParticipationRole::OptParticipant
                                                    }
                                                    JSCalendarParticipantRole::Informational => {
                                                        ICalendarParticipationRole::NonParticipant
                                                    }
                                                    JSCalendarParticipantRole::Chair => {
                                                        ICalendarParticipationRole::Chair
                                                    }
                                                    JSCalendarParticipantRole::Required => {
                                                        ICalendarParticipationRole::ReqParticipant
                                                    }
                                                    JSCalendarParticipantRole::Owner => {
                                                        ICalendarParticipationRole::Owner
                                                    }
                                                };
                                                entry.params.push(ICalendarParameter::role(role));
                                            }
                                            key => {
                                                component.insert_jsprop::<I, B>(
                                                    &[
                                                        property.to_string().as_ref(),
                                                        name.to_string().as_ref(),
                                                        JSCalendarProperty::Roles::<I>
                                                            .to_string()
                                                            .as_ref(),
                                                        key.to_string().as_ref(),
                                                    ],
                                                    Value::Bool(true),
                                                );
                                            }
                                        }
                                    }
                                }
                                (Key::Property(JSCalendarProperty::SentBy), Value::Str(text)) => {
                                    entry.params.push(ICalendarParameter::sent_by(Uri::parse(
                                        text.into_owned(),
                                    )));
                                }
                                (
                                    Key::Property(JSCalendarProperty::Description),
                                    Value::Str(text),
                                ) => {
                                    description = Some(text);
                                }
                                (
                                    Key::Property(JSCalendarProperty::DescriptionContentType),
                                    Value::Str(text),
                                ) => {
                                    description_content_type = Some(text);
                                }
                                (Key::Property(JSCalendarProperty::Links), Value::Object(obj)) => {
                                    participant.import_links(obj, &mut item_conversions);
                                }
                                (
                                    Key::Property(JSCalendarProperty::PercentComplete),
                                    Value::Number(number),
                                ) => {
                                    participant.entries.push(
                                        ICalendarEntry::new(ICalendarProperty::PercentComplete)
                                            .with_value(number.cast_to_i64())
                                            .import_converted(
                                                &[JSCalendarProperty::PercentComplete],
                                                &mut item_conversions,
                                            ),
                                    );
                                }
                                (
                                    Key::Property(
                                        JSCalendarProperty::Type | JSCalendarProperty::ICalendar,
                                    ),
                                    _,
                                ) => {}
                                (sub_property, value) => {
                                    if item_conversions.is_none() {
                                        component.insert_jsprop(
                                            &[
                                                property.to_string().as_ref(),
                                                name.to_string().as_ref(),
                                                sub_property.to_string().as_ref(),
                                            ],
                                            value,
                                        );
                                    } else {
                                        participant.insert_jsprop(
                                            &[
                                                property.to_string().as_ref(),
                                                name.to_string().as_ref(),
                                                sub_property.to_string().as_ref(),
                                            ],
                                            value,
                                        );
                                    }
                                }
                            }
                        }

                        match (description, description_content_type) {
                            (Some(description), Some(content_type)) => {
                                participant.entries.push(
                                    ICalendarEntry::new(ICalendarProperty::StyledDescription)
                                        .with_param(ICalendarParameter::fmttype(
                                            content_type.into_owned(),
                                        ))
                                        .with_value(description.into_owned())
                                        .import_converted(
                                            &[JSCalendarProperty::Description],
                                            &mut item_conversions,
                                        ),
                                );
                            }
                            (Some(description), None) => {
                                participant.entries.push(
                                    ICalendarEntry::new(ICalendarProperty::Description)
                                        .with_value(description.into_owned())
                                        .import_converted(
                                            &[JSCalendarProperty::Description],
                                            &mut item_conversions,
                                        ),
                                );
                            }
                            _ => {}
                        }

                        match (status, progress) {
                            (
                                Some(JSCalendarParticipationStatus::Accepted) | None,
                                Some(progress),
                            ) => {
                                entry
                                    .params
                                    .push(ICalendarParameter::partstat(match progress {
                                        JSCalendarProgress::NeedsAction => {
                                            ICalendarParticipationStatus::NeedsAction
                                        }
                                        JSCalendarProgress::InProcess => {
                                            ICalendarParticipationStatus::InProcess
                                        }
                                        JSCalendarProgress::Completed => {
                                            ICalendarParticipationStatus::Completed
                                        }
                                        JSCalendarProgress::Failed => {
                                            ICalendarParticipationStatus::Failed
                                        }
                                    }));
                            }
                            (Some(status), _) => {
                                entry
                                    .params
                                    .push(ICalendarParameter::partstat(match status {
                                        JSCalendarParticipationStatus::Accepted => {
                                            ICalendarParticipationStatus::Accepted
                                        }
                                        JSCalendarParticipationStatus::Declined => {
                                            ICalendarParticipationStatus::Declined
                                        }
                                        JSCalendarParticipationStatus::NeedsAction => {
                                            ICalendarParticipationStatus::NeedsAction
                                        }
                                        JSCalendarParticipationStatus::Tentative => {
                                            ICalendarParticipationStatus::Tentative
                                        }
                                        JSCalendarParticipationStatus::Delegated => {
                                            ICalendarParticipationStatus::Delegated
                                        }
                                    }));
                            }
                            _ => {}
                        }

                        let has_component =
                            !participant.entries.is_empty() || item_conversions.is_some();
                        let has_entry = !entry.params.is_empty() || item_conversions.is_none();
                        let is_organizer =
                            organizer_address.is_some() && organizer_address == calendar_address;

                        if let Some(calendar_address) = calendar_address {
                            let calendar_address =
                                ICalendarValue::Uri(Uri::parse(calendar_address.into_owned()));

                            if has_component {
                                participant.entries.push(
                                    ICalendarEntry::new(ICalendarProperty::CalendarAddress)
                                        .with_value(calendar_address.clone()),
                                );
                            }
                            if has_entry {
                                entry.values.push(calendar_address);
                            }
                        }

                        if let Some(participant_name) = participant_name {
                            if is_organizer {
                                organizer_name = Some(participant_name.clone());
                            }
                            if has_component {
                                participant.entries.push(
                                    ICalendarEntry::new(ICalendarProperty::Summary)
                                        .with_value(participant_name.as_ref().to_string())
                                        .import_converted(
                                            &[JSCalendarProperty::Name],
                                            &mut item_conversions,
                                        ),
                                );
                            }
                            if has_entry {
                                entry
                                    .params
                                    .push(ICalendarParameter::cn(participant_name.into_owned()));
                            }
                        }

                        debug_assert!(has_component || !entry.values.is_empty());

                        if has_component {
                            if !is_uuid5_key {
                                participant.entries.push(
                                    ICalendarEntry::new(ICalendarProperty::Jsid)
                                        .with_value(name.to_string().into_owned()),
                                );
                            }
                            if let Some(item_conversions) = item_conversions {
                                participant = item_conversions.apply_conversions(participant, self);
                            }
                            component
                                .component_ids
                                .push(self.push_component(participant));
                        }

                        if !entry.values.is_empty() && (!is_organizer || !entry.params.is_empty()) {
                            let attendee = entry
                                .with_param_opt(
                                    (!is_uuid5_key)
                                        .then(|| ICalendarParameter::jsid(name.into_string())),
                                )
                                .import_converted(
                                    &[JSCalendarProperty::Participants],
                                    &mut root_conversions,
                                );

                            component.entries.push(attendee);
                        }
                    }
                }
                (
                    JSCalendarProperty::Alerts,
                    Value::Object(obj),
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo,
                ) => {
                    let mut alerts = Vec::new();
                    let mut has_related_to = false;

                    for (name, value) in obj.into_vec() {
                        let Value::Object(mut value) = value else {
                            continue;
                        };

                        let mut item_conversions = ConvertedComponent::build(&mut value);
                        let mut alert = ICalendarComponent::new(ICalendarComponentType::VAlarm);

                        for (sub_property, value) in value.into_vec() {
                            match (sub_property, value) {
                                (
                                    Key::Property(
                                        JSCalendarProperty::Type | JSCalendarProperty::ICalendar,
                                    ),
                                    _,
                                ) => {}
                                (
                                    Key::Property(JSCalendarProperty::Acknowledged),
                                    Value::Element(JSCalendarValue::DateTime(dt)),
                                ) => {
                                    alert.entries.push(
                                        ICalendarEntry::new(ICalendarProperty::Acknowledged)
                                            .with_value(PartialDateTime::from_utc_timestamp(
                                                dt.timestamp,
                                            ))
                                            .import_converted(
                                                &[JSCalendarProperty::Acknowledged],
                                                &mut item_conversions,
                                            ),
                                    );
                                }
                                (
                                    Key::Property(JSCalendarProperty::Action),
                                    Value::Element(JSCalendarValue::AlertAction(action)),
                                ) => {
                                    alert.entries.push(
                                        ICalendarEntry::new(ICalendarProperty::Action)
                                            .with_value(ICalendarValue::Action(match action {
                                                JSCalendarAlertAction::Display => {
                                                    ICalendarAction::Display
                                                }
                                                JSCalendarAlertAction::Email => {
                                                    ICalendarAction::Email
                                                }
                                            }))
                                            .import_converted(
                                                &[JSCalendarProperty::Action],
                                                &mut item_conversions,
                                            ),
                                    );
                                }
                                (
                                    Key::Property(JSCalendarProperty::RelatedTo),
                                    Value::Object(obj),
                                ) => {
                                    has_related_to = true;
                                    alert.import_relations(obj, &mut item_conversions);
                                }
                                (
                                    Key::Property(JSCalendarProperty::Trigger),
                                    Value::Object(obj),
                                ) => {
                                    let mut offset = None;
                                    let mut rel_to = None;
                                    let mut when = None;

                                    for (key, value) in obj.into_vec() {
                                        match (key, value) {
                                            (
                                                Key::Property(JSCalendarProperty::Offset),
                                                Value::Element(JSCalendarValue::Duration(value)),
                                            ) => {
                                                offset = Some(value);
                                            }
                                            (
                                                Key::Property(JSCalendarProperty::RelativeTo),
                                                Value::Element(JSCalendarValue::RelativeTo(value)),
                                            ) => {
                                                rel_to = Some(value);
                                            }
                                            (
                                                Key::Property(JSCalendarProperty::When),
                                                Value::Element(JSCalendarValue::DateTime(value)),
                                            ) => {
                                                when = Some(value);
                                            }
                                            (Key::Property(JSCalendarProperty::Type), _) => {}
                                            (key, value) => {
                                                alert.insert_jsprop(
                                                    &[
                                                        JSCalendarProperty::Trigger::<I>
                                                            .to_string()
                                                            .as_ref(),
                                                        key.to_string().as_ref(),
                                                    ],
                                                    value,
                                                );
                                            }
                                        }
                                    }

                                    if let Some(when) = when {
                                        alert.entries.push(
                                            ICalendarEntry::new(ICalendarProperty::Trigger)
                                                .with_param(ICalendarParameter::value(
                                                    ICalendarValueType::DateTime,
                                                ))
                                                .with_value(PartialDateTime::from_utc_timestamp(
                                                    when.timestamp,
                                                ))
                                                .import_converted(
                                                    &[
                                                        JSCalendarProperty::Trigger,
                                                        JSCalendarProperty::When,
                                                    ],
                                                    &mut item_conversions,
                                                ),
                                        );
                                    } else if let Some(offset) = offset {
                                        alert.entries.push(
                                            ICalendarEntry::new(ICalendarProperty::Trigger)
                                                .with_param_opt(rel_to.map(|rel_to| {
                                                    ICalendarParameter::related(match rel_to {
                                                        JSCalendarRelativeTo::Start => {
                                                            ICalendarRelated::Start
                                                        }
                                                        JSCalendarRelativeTo::End => {
                                                            ICalendarRelated::End
                                                        }
                                                    })
                                                }))
                                                .with_value(offset)
                                                .import_converted(
                                                    &[
                                                        JSCalendarProperty::Trigger,
                                                        JSCalendarProperty::Offset,
                                                    ],
                                                    &mut item_conversions,
                                                ),
                                        );
                                    }
                                }
                                (sub_property, value) => {
                                    alert
                                        .insert_jsprop(&[sub_property.to_string().as_ref()], value);
                                }
                            }
                        }

                        if let Some(item_conversions) = item_conversions {
                            alert = item_conversions.apply_conversions(alert, self);
                        }

                        if !alert.entries.is_empty() {
                            alert.entries.push(
                                ICalendarEntry::new(ICalendarProperty::Jsid)
                                    .with_value(name.to_string().into_owned()),
                            );
                            alerts.push(alert);
                        }
                    }

                    // Map RELATED-TO properties
                    let mut alert_mappings = Vec::new();
                    if has_related_to {
                        alert_mappings = alerts
                            .iter()
                            .filter_map(|alert| {
                                Some((alert.jsid()?.to_string(), alert.uid()?.to_string()))
                            })
                            .collect();
                    }
                    for mut alert in alerts {
                        if has_related_to {
                            for related_to in alert.properties_mut(&ICalendarProperty::RelatedTo) {
                                if let Some(ICalendarValue::Text(related_to)) =
                                    related_to.values.first_mut()
                                    && let Some((_, uid)) =
                                        alert_mappings.iter().find(|(jsid, _)| jsid == related_to)
                                {
                                    *related_to = uid.clone();
                                }
                            }
                        }
                        component.component_ids.push(self.push_component(alert));
                    }
                }
                (
                    JSCalendarProperty::Keywords,
                    Value::Object(obj),
                    ICalendarComponentType::VEvent
                    | ICalendarComponentType::VTodo
                    | ICalendarComponentType::VCalendar,
                ) => {
                    component.entries.push(
                        ICalendarEntry::new(ICalendarProperty::Categories)
                            .with_values(
                                obj.into_expanded_boolean_set()
                                    .map(|v| ICalendarValue::Text(v.into_string()))
                                    .collect(),
                            )
                            .import_converted(
                                &[JSCalendarProperty::Keywords],
                                &mut root_conversions,
                            ),
                    );
                }
                (
                    JSCalendarProperty::Privacy,
                    Value::Element(JSCalendarValue::Privacy(value)),
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo,
                ) => {
                    component.entries.push(
                        ICalendarEntry::new(ICalendarProperty::Class)
                            .with_value(match value {
                                JSCalendarPrivacy::Public => ICalendarClassification::Public,
                                JSCalendarPrivacy::Private => ICalendarClassification::Private,
                                JSCalendarPrivacy::Secret => ICalendarClassification::Confidential,
                            })
                            .import_converted(
                                &[JSCalendarProperty::Privacy],
                                &mut root_conversions,
                            ),
                    );
                }
                (
                    JSCalendarProperty::Color,
                    Value::Str(text),
                    ICalendarComponentType::VEvent
                    | ICalendarComponentType::VTodo
                    | ICalendarComponentType::VCalendar,
                ) => {
                    component.entries.push(
                        ICalendarEntry::new(ICalendarProperty::Color)
                            .with_value(text.into_owned())
                            .import_converted(&[JSCalendarProperty::Color], &mut root_conversions),
                    );
                }
                (
                    JSCalendarProperty::Categories,
                    Value::Object(obj),
                    ICalendarComponentType::VEvent
                    | ICalendarComponentType::VTodo
                    | ICalendarComponentType::VCalendar,
                ) => {
                    for value in obj.into_expanded_boolean_set() {
                        component.entries.push(
                            ICalendarEntry::new(ICalendarProperty::Concept)
                                .with_value(ICalendarValue::Text(value.into_string()))
                                .import_converted(
                                    &[JSCalendarProperty::Categories],
                                    &mut root_conversions,
                                ),
                        );
                    }
                }
                (
                    JSCalendarProperty::VirtualLocations,
                    Value::Object(obj),
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo,
                ) => {
                    for (name, value) in obj.into_vec() {
                        let mut entry = ICalendarEntry::new(ICalendarProperty::Conference);
                        let mut is_uuid5_key = false;

                        for (sub_property, value) in value.into_expanded_object() {
                            match (sub_property, value) {
                                (
                                    Key::Property(JSCalendarProperty::Features),
                                    Value::Object(obj),
                                ) => {
                                    for key in obj.into_expanded_boolean_set() {
                                        let value = match key {
                                            Key::Property(
                                                JSCalendarProperty::VirtualLocationFeature(feature),
                                            ) => ICalendarParameterValue::Feature(match feature {
                                                JSCalendarVirtualLocationFeature::Audio => {
                                                    ICalendarFeatureType::Audio
                                                }
                                                JSCalendarVirtualLocationFeature::Chat => {
                                                    ICalendarFeatureType::Chat
                                                }
                                                JSCalendarVirtualLocationFeature::Feed => {
                                                    ICalendarFeatureType::Feed
                                                }
                                                JSCalendarVirtualLocationFeature::Moderator => {
                                                    ICalendarFeatureType::Moderator
                                                }
                                                JSCalendarVirtualLocationFeature::Phone => {
                                                    ICalendarFeatureType::Phone
                                                }
                                                JSCalendarVirtualLocationFeature::Screen => {
                                                    ICalendarFeatureType::Screen
                                                }
                                                JSCalendarVirtualLocationFeature::Video => {
                                                    ICalendarFeatureType::Video
                                                }
                                            }),
                                            other => ICalendarParameterValue::Text(
                                                other.to_string().into_owned(),
                                            ),
                                        };
                                        entry.params.push(ICalendarParameter::feature(value));
                                    }
                                }
                                (Key::Property(JSCalendarProperty::Name), Value::Str(text)) => {
                                    entry
                                        .params
                                        .push(ICalendarParameter::label(text.into_owned()));
                                }
                                (Key::Property(JSCalendarProperty::Uri), Value::Str(text)) => {
                                    is_uuid5_key = uuid5(text.as_ref()) == name.to_string();
                                    entry
                                        .values
                                        .push(ICalendarValue::Uri(Uri::parse(text.into_owned())));
                                }
                                (
                                    Key::Property(
                                        JSCalendarProperty::Type | JSCalendarProperty::ICalendar,
                                    ),
                                    _,
                                ) => {}
                                (sub_property, value) => {
                                    component.insert_jsprop(
                                        &[
                                            JSCalendarProperty::VirtualLocations::<I>
                                                .to_string()
                                                .as_ref(),
                                            name.to_string().as_ref(),
                                            sub_property.to_string().as_ref(),
                                        ],
                                        value,
                                    );
                                }
                            }
                        }

                        if !is_uuid5_key {
                            entry.add_param(ICalendarParameter::jsid(name.into_string()));
                        }

                        component.entries.push(entry.import_converted(
                            &[JSCalendarProperty::VirtualLocations],
                            &mut root_conversions,
                        ));
                    }
                }
                (
                    JSCalendarProperty::Title,
                    Value::Str(text),
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo,
                ) => {
                    component.entries.push(
                        ICalendarEntry::new(ICalendarProperty::Summary)
                            .with_param_opt(
                                locale
                                    .as_ref()
                                    .map(|locale| ICalendarParameter::language(locale.to_string())),
                            )
                            .with_value(text.into_owned())
                            .import_converted(&[JSCalendarProperty::Title], &mut root_conversions),
                    );
                }
                (
                    JSCalendarProperty::Title,
                    Value::Str(text),
                    ICalendarComponentType::VCalendar,
                ) => {
                    component.entries.push(
                        ICalendarEntry::new(ICalendarProperty::Name)
                            .with_param_opt(
                                locale
                                    .as_ref()
                                    .map(|locale| ICalendarParameter::language(locale.to_string())),
                            )
                            .with_value(text.into_owned())
                            .import_converted(&[JSCalendarProperty::Title], &mut root_conversions),
                    );
                }
                (
                    JSCalendarProperty::Locations,
                    Value::Object(obj),
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo,
                ) => {
                    let has_multi_location = obj.len() > 1;
                    for (name, value) in obj.into_vec() {
                        let Value::Object(mut value) = value else {
                            continue;
                        };
                        let mut item_conversions = ConvertedComponent::build(&mut value);
                        let is_main_location = main_location_id
                            .as_ref()
                            .is_some_and(|l| l == &name.to_string());
                        let mut is_uuid5_key = false;
                        let mut location = (item_conversions
                            .as_ref()
                            .is_some_and(|v| v.name.is_location())
                            || value.iter().any(|(k, v)| match (k, v) {
                                (
                                    Key::Property(
                                        JSCalendarProperty::Links
                                        | JSCalendarProperty::LocationTypes
                                        | JSCalendarProperty::Description,
                                    ),
                                    _,
                                ) => true,
                                (
                                    Key::Property(JSCalendarProperty::Coordinates),
                                    Value::Str(uri),
                                ) => uri.strip_prefix("geo:").is_none_or(|v| {
                                    !v.as_bytes()
                                        .iter()
                                        .all(|b| matches!(b, b'0'..=b'9' | b'.' | b',' | b'-'))
                                }),
                                _ => false,
                            })
                            || (has_multi_location && !is_main_location))
                            .then_some(ICalendarComponent::new(ICalendarComponentType::VLocation));

                        for (sub_property, value) in value.into_vec() {
                            match (sub_property, value) {
                                (
                                    Key::Property(JSCalendarProperty::LocationTypes),
                                    Value::Object(obj),
                                ) => {
                                    if let Some(location) = &mut location {
                                        location.entries.push(
                                            ICalendarEntry::new(ICalendarProperty::LocationType)
                                                .import_converted(
                                                    &[JSCalendarProperty::LocationTypes],
                                                    &mut item_conversions,
                                                )
                                                .with_values(
                                                    obj.into_expanded_boolean_set()
                                                        .map(|v| {
                                                            ICalendarValue::Text(v.into_string())
                                                        })
                                                        .collect(),
                                                ),
                                        );
                                    }
                                }
                                (Key::Property(JSCalendarProperty::Name), Value::Str(text)) => {
                                    is_uuid5_key |= uuid5(text.as_ref()) == name.to_string();

                                    if let Some(location) = &mut location {
                                        location.entries.push(
                                            ICalendarEntry::new(ICalendarProperty::Name)
                                                .import_converted(
                                                    &[JSCalendarProperty::Name],
                                                    &mut item_conversions,
                                                )
                                                .with_value(text.clone().into_owned()),
                                        );
                                    }

                                    if location.is_none() || is_main_location {
                                        component.entries.push(
                                            ICalendarEntry::new(ICalendarProperty::Location)
                                                .with_param_opt((!is_uuid5_key).then(|| {
                                                    ICalendarParameter::jsid(
                                                        name.clone().into_string(),
                                                    )
                                                }))
                                                .with_param_opt(
                                                    (has_multi_location)
                                                        .then(|| ICalendarParameter::derived(true)),
                                                )
                                                .with_value(text.into_owned())
                                                .import_converted(
                                                    &[JSCalendarProperty::Locations],
                                                    &mut root_conversions,
                                                ),
                                        );
                                    }
                                }
                                (
                                    Key::Property(JSCalendarProperty::Description),
                                    Value::Str(text),
                                ) => {
                                    if let Some(location) = &mut location {
                                        location.entries.push(
                                            ICalendarEntry::new(ICalendarProperty::Description)
                                                .import_converted(
                                                    &[JSCalendarProperty::Description],
                                                    &mut item_conversions,
                                                )
                                                .with_value(text.clone().into_owned()),
                                        );
                                    }
                                }
                                (
                                    Key::Property(JSCalendarProperty::Coordinates),
                                    Value::Str(text),
                                ) => {
                                    is_uuid5_key |= uuid5(text.as_ref()) == name.to_string();
                                    if let Some(location) = &mut location {
                                        let entry =
                                            ICalendarEntry::new(ICalendarProperty::Coordinates)
                                                .import_converted(
                                                    &[JSCalendarProperty::Coordinates],
                                                    &mut item_conversions,
                                                );
                                        location.entries.push(
                                            if entry.name == ICalendarProperty::Geo {
                                                entry.with_values(parse_geo(text))
                                            } else {
                                                entry.with_value(Uri::parse(text.into_owned()))
                                            },
                                        );
                                    } else {
                                        component.entries.push(
                                            ICalendarEntry::new(ICalendarProperty::Geo)
                                                .with_param_opt((!is_uuid5_key).then(|| {
                                                    ICalendarParameter::jsid(
                                                        name.clone().into_string(),
                                                    )
                                                }))
                                                .with_values(parse_geo(text))
                                                .import_converted(
                                                    &[JSCalendarProperty::Locations],
                                                    &mut root_conversions,
                                                ),
                                        );
                                    }
                                }
                                (Key::Property(JSCalendarProperty::Links), Value::Object(obj)) => {
                                    if let Some(location) = &mut location {
                                        location.import_links(obj, &mut item_conversions);
                                    }
                                }
                                (
                                    Key::Property(
                                        JSCalendarProperty::Type | JSCalendarProperty::ICalendar,
                                    ),
                                    _,
                                ) => {}
                                (sub_property, value) => {
                                    if let Some(location) = &mut location {
                                        location.insert_jsprop(
                                            &[sub_property.to_string().as_ref()],
                                            value,
                                        );
                                    } else {
                                        component.insert_jsprop(
                                            &[
                                                JSCalendarProperty::Locations::<I>
                                                    .to_string()
                                                    .as_ref(),
                                                name.to_string().as_ref(),
                                                sub_property.to_string().as_ref(),
                                            ],
                                            value,
                                        );
                                    }
                                }
                            }
                        }

                        if let Some(mut location) = location {
                            if !is_uuid5_key {
                                location.entries.push(
                                    ICalendarEntry::new(ICalendarProperty::Jsid)
                                        .with_value(name.to_string().into_owned()),
                                );
                            }
                            if let Some(item_conversions) = item_conversions {
                                location = item_conversions.apply_conversions(location, self);
                            }
                            component.component_ids.push(self.push_component(location));
                        }
                    }
                }
                (
                    JSCalendarProperty::Created,
                    Value::Element(JSCalendarValue::DateTime(dt)),
                    ICalendarComponentType::VEvent
                    | ICalendarComponentType::VTodo
                    | ICalendarComponentType::VCalendar,
                ) => {
                    component.entries.push(
                        ICalendarEntry::new(ICalendarProperty::Created)
                            .with_value(PartialDateTime::from_utc_timestamp(dt.timestamp))
                            .import_converted(
                                &[JSCalendarProperty::Created],
                                &mut root_conversions,
                            ),
                    );
                }
                (
                    JSCalendarProperty::Description,
                    Value::Str(text),
                    ICalendarComponentType::VEvent
                    | ICalendarComponentType::VTodo
                    | ICalendarComponentType::VCalendar,
                ) => {
                    if let Some(description_content_type) = description_content_type.take() {
                        component.entries.push(
                            ICalendarEntry::new(ICalendarProperty::StyledDescription)
                                .with_value(text.into_owned())
                                .with_param(ICalendarParameter::fmttype(
                                    description_content_type.into_owned(),
                                ))
                                .import_converted(
                                    &[JSCalendarProperty::Description],
                                    &mut root_conversions,
                                ),
                        );
                    } else {
                        component.entries.push(
                            ICalendarEntry::new(ICalendarProperty::Description)
                                .with_value(text.into_owned())
                                .import_converted(
                                    &[JSCalendarProperty::Description],
                                    &mut root_conversions,
                                ),
                        );
                    }
                }
                (
                    JSCalendarProperty::RecurrenceRule,
                    Value::Object(obj),
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo,
                ) => {
                    let mut rrule = ICalendarRecurrenceRule::default();

                    for (key, value) in obj.into_vec() {
                        let Key::Property(key) = key else {
                            continue;
                        };
                        match (key, value) {
                            (
                                JSCalendarProperty::Frequency,
                                Value::Element(JSCalendarValue::Frequency(value)),
                            ) => {
                                rrule.freq = value;
                            }
                            (
                                JSCalendarProperty::Until,
                                Value::Element(JSCalendarValue::DateTime(value)),
                            ) => {
                                rrule.until = value
                                    .to_naive_date_time()
                                    .and_then(|dt| {
                                        state
                                            .tz
                                            .unwrap_or_default()
                                            .from_local_datetime(&dt)
                                            .single()
                                    })
                                    .map(|dt| PartialDateTime::from_utc_timestamp(dt.timestamp()));
                            }
                            (JSCalendarProperty::Count, Value::Number(value)) => {
                                rrule.count = Some(value.cast_to_u64() as u32);
                            }
                            (JSCalendarProperty::Interval, Value::Number(value)) => {
                                rrule.interval = Some(value.cast_to_u64() as u16);
                            }
                            (JSCalendarProperty::BySecond, Value::Array(value)) => {
                                rrule.bysecond = value
                                    .into_iter()
                                    .filter_map(|v| v.as_i64().and_then(|n| u8::try_from(n).ok()))
                                    .collect();
                            }
                            (JSCalendarProperty::ByMinute, Value::Array(value)) => {
                                rrule.byminute = value
                                    .into_iter()
                                    .filter_map(|v| v.as_i64().and_then(|n| u8::try_from(n).ok()))
                                    .collect();
                            }
                            (JSCalendarProperty::ByHour, Value::Array(value)) => {
                                rrule.byhour = value
                                    .into_iter()
                                    .filter_map(|v| v.as_i64().and_then(|n| u8::try_from(n).ok()))
                                    .collect();
                            }
                            (JSCalendarProperty::ByDay, Value::Array(value)) => {
                                for item in value {
                                    let mut weekday = None;
                                    let mut ordwk = None;

                                    for (key, value) in item.into_expanded_object() {
                                        match (key, value) {
                                            (
                                                Key::Property(JSCalendarProperty::Day),
                                                Value::Element(JSCalendarValue::Weekday(value)),
                                            ) => {
                                                weekday = Some(value);
                                            }
                                            (
                                                Key::Property(JSCalendarProperty::NthOfPeriod),
                                                Value::Number(value),
                                            ) => {
                                                ordwk = i16::try_from(value.cast_to_i64()).ok();
                                            }
                                            _ => {}
                                        }
                                    }

                                    if let Some(weekday) = weekday {
                                        rrule.byday.push(ICalendarDay { weekday, ordwk });
                                    }
                                }
                            }
                            (JSCalendarProperty::ByMonthDay, Value::Array(value)) => {
                                rrule.bymonthday = value
                                    .into_iter()
                                    .filter_map(|v| v.as_i64().and_then(|n| i8::try_from(n).ok()))
                                    .collect();
                            }
                            (JSCalendarProperty::ByYearDay, Value::Array(value)) => {
                                rrule.byyearday = value
                                    .into_iter()
                                    .filter_map(|v| v.as_i64().and_then(|n| i16::try_from(n).ok()))
                                    .collect();
                            }
                            (JSCalendarProperty::ByWeekNo, Value::Array(value)) => {
                                rrule.byweekno = value
                                    .into_iter()
                                    .filter_map(|v| v.as_i64().and_then(|n| i8::try_from(n).ok()))
                                    .collect();
                            }
                            (JSCalendarProperty::ByMonth, Value::Array(value)) => {
                                rrule.bymonth = value
                                    .into_iter()
                                    .filter_map(|v| {
                                        v.as_str().and_then(|v| ICalendarMonth::parse(v.as_bytes()))
                                    })
                                    .collect();
                            }
                            (JSCalendarProperty::BySetPosition, Value::Array(value)) => {
                                rrule.bysetpos = value
                                    .into_iter()
                                    .filter_map(|v| v.as_i64().and_then(|n| i32::try_from(n).ok()))
                                    .collect();
                            }
                            (
                                JSCalendarProperty::FirstDayOfWeek,
                                Value::Element(JSCalendarValue::Weekday(value)),
                            ) => {
                                rrule.wkst = Some(value);
                            }
                            (
                                JSCalendarProperty::Rscale,
                                Value::Element(JSCalendarValue::CalendarScale(value)),
                            ) => {
                                rrule.rscale = Some(value);
                            }
                            (
                                JSCalendarProperty::Skip,
                                Value::Element(JSCalendarValue::Skip(value)),
                            ) => {
                                rrule.skip = Some(value);
                            }
                            (JSCalendarProperty::Type | JSCalendarProperty::ICalendar, _) => {}
                            (key, value) => {
                                component.insert_jsprop(
                                    &[
                                        JSCalendarProperty::RecurrenceRule::<I>
                                            .to_string()
                                            .as_ref(),
                                        key.to_string().as_ref(),
                                    ],
                                    value,
                                );
                            }
                        }
                    }

                    component.entries.push(
                        ICalendarEntry::new(ICalendarProperty::Rrule)
                            .with_value(rrule)
                            .import_converted(
                                &[JSCalendarProperty::RecurrenceRule],
                                &mut root_conversions,
                            ),
                    );
                }
                (
                    JSCalendarProperty::Updated,
                    Value::Element(JSCalendarValue::DateTime(dt)),
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo,
                ) => {
                    component.entries.push(
                        ICalendarEntry::new(ICalendarProperty::Dtstamp)
                            .with_value(PartialDateTime::from_utc_timestamp(dt.timestamp))
                            .import_converted(
                                &[JSCalendarProperty::Updated],
                                &mut root_conversions,
                            ),
                    );
                }
                (
                    JSCalendarProperty::Updated,
                    Value::Element(JSCalendarValue::DateTime(dt)),
                    ICalendarComponentType::VCalendar,
                ) => {
                    component.entries.push(
                        ICalendarEntry::new(ICalendarProperty::LastModified)
                            .with_value(PartialDateTime::from_utc_timestamp(dt.timestamp))
                            .import_converted(
                                &[JSCalendarProperty::Updated],
                                &mut root_conversions,
                            ),
                    );
                }
                (
                    JSCalendarProperty::Due,
                    Value::Element(JSCalendarValue::DateTime(dt)),
                    ICalendarComponentType::VTodo,
                ) => {
                    if let Some(dt) = dt.to_naive_date_time().and_then(|dt| {
                        state
                            .tz
                            .unwrap_or_default()
                            .from_local_datetime(&dt)
                            .single()
                    }) {
                        component.entries.push(
                            ICalendarEntry::new(ICalendarProperty::Due)
                                .import_converted(&[JSCalendarProperty::Due], &mut root_conversions)
                                .with_date_time(dt),
                        );
                    }
                }
                (
                    JSCalendarProperty::RecurrenceId,
                    Value::Element(JSCalendarValue::DateTime(dt)),
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo,
                ) => {
                    if let Some(dt) = dt.to_naive_date_time().and_then(|dt| {
                        state
                            .tz_rid
                            .or(state.tz)
                            .unwrap_or_default()
                            .from_local_datetime(&dt)
                            .single()
                    }) {
                        add_recurrence_id = false;
                        component.entries.push(
                            ICalendarEntry::new(ICalendarProperty::RecurrenceId)
                                .import_converted(
                                    &[JSCalendarProperty::RecurrenceId],
                                    &mut root_conversions,
                                )
                                .with_date_time(dt),
                        );
                    }
                }
                (
                    JSCalendarProperty::Duration,
                    Value::Element(JSCalendarValue::Duration(duration)),
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo,
                ) => {
                    let entry = ICalendarEntry::new(if state.tz_end.is_none() {
                        ICalendarProperty::Duration
                    } else {
                        ICalendarProperty::Dtend
                    })
                    .import_converted(&[JSCalendarProperty::Duration], &mut root_conversions);
                    if entry.name == ICalendarProperty::Dtend {
                        if let Some(end) = state.start.and_then(|start| {
                            start.checked_add_signed(TimeDelta::seconds(duration.as_seconds()))
                        }) {
                            component.entries.push(
                                entry.with_date_time(
                                    state
                                        .tz_end
                                        .map(|tz_end| end.with_timezone(&tz_end))
                                        .unwrap_or(end),
                                ),
                            );
                        } else {
                            component.insert_jsprop::<I, B>(
                                &[property.to_string().as_ref()],
                                Value::Element(JSCalendarValue::Duration(duration)),
                            );
                        }
                    } else {
                        component.entries.push(entry.with_value(duration));
                    }
                }
                (
                    JSCalendarProperty::EstimatedDuration,
                    Value::Element(JSCalendarValue::Duration(duration)),
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo,
                ) => {
                    component.entries.push(
                        ICalendarEntry::new(ICalendarProperty::EstimatedDuration)
                            .with_value(duration)
                            .import_converted(
                                &[JSCalendarProperty::EstimatedDuration],
                                &mut root_conversions,
                            ),
                    );
                }
                (
                    JSCalendarProperty::Method,
                    Value::Element(JSCalendarValue::Method(method)),
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo,
                ) => {
                    if let Some(parent_component) = &mut parent_component
                        && !parent_component.has_property(&ICalendarProperty::Method)
                    {
                        parent_component.entries.push(
                            ICalendarEntry::new(ICalendarProperty::Method).with_value(method),
                        );
                    }
                }
                (
                    JSCalendarProperty::PercentComplete,
                    Value::Number(number),
                    ICalendarComponentType::VTodo,
                ) => {
                    component.entries.push(
                        ICalendarEntry::new(ICalendarProperty::PercentComplete)
                            .with_value(number.cast_to_i64())
                            .import_converted(
                                &[JSCalendarProperty::PercentComplete],
                                &mut root_conversions,
                            ),
                    );
                }
                (
                    JSCalendarProperty::Priority,
                    Value::Number(number),
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo,
                ) => {
                    component.entries.push(
                        ICalendarEntry::new(ICalendarProperty::Priority)
                            .with_value(number.cast_to_i64())
                            .import_converted(
                                &[JSCalendarProperty::Priority],
                                &mut root_conversions,
                            ),
                    );
                }
                (
                    JSCalendarProperty::Sequence,
                    Value::Number(number),
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo,
                ) => {
                    component.entries.push(
                        ICalendarEntry::new(ICalendarProperty::Sequence)
                            .with_value(number.cast_to_i64())
                            .import_converted(
                                &[JSCalendarProperty::Sequence],
                                &mut root_conversions,
                            ),
                    );
                }
                (
                    JSCalendarProperty::ProdId,
                    Value::Str(text),
                    ICalendarComponentType::VCalendar,
                ) => {
                    component.entries.push(
                        ICalendarEntry::new(ICalendarProperty::Prodid)
                            .with_value(text.into_owned())
                            .import_converted(&[JSCalendarProperty::ProdId], &mut root_conversions),
                    );
                }
                (
                    JSCalendarProperty::RelatedTo,
                    Value::Object(obj),
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo,
                ) => {
                    component.import_relations(obj, &mut root_conversions);
                }
                (
                    JSCalendarProperty::ShowWithoutTime,
                    Value::Bool(value),
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo,
                ) => {
                    show_without_time = Some(
                        ICalendarEntry::new(ICalendarProperty::ShowWithoutTime)
                            .with_value(value)
                            .import_converted(
                                &[JSCalendarProperty::ShowWithoutTime],
                                &mut root_conversions,
                            ),
                    );
                }
                (
                    JSCalendarProperty::Progress,
                    Value::Element(JSCalendarValue::Progress(value)),
                    ICalendarComponentType::VTodo,
                ) => {
                    component.entries.push(
                        ICalendarEntry::new(ICalendarProperty::Status)
                            .with_value(match value {
                                JSCalendarProgress::NeedsAction => ICalendarStatus::NeedsAction,
                                JSCalendarProgress::InProcess => ICalendarStatus::InProcess,
                                JSCalendarProgress::Completed => ICalendarStatus::Completed,
                                JSCalendarProgress::Failed => ICalendarStatus::Failed,
                            })
                            .import_converted(
                                &[JSCalendarProperty::Progress],
                                &mut root_conversions,
                            ),
                    );
                }
                (
                    JSCalendarProperty::Status,
                    Value::Element(JSCalendarValue::EventStatus(value)),
                    ICalendarComponentType::VEvent,
                ) => {
                    component.entries.push(
                        ICalendarEntry::new(ICalendarProperty::Status)
                            .with_value(match value {
                                JSCalendarEventStatus::Confirmed => ICalendarStatus::Confirmed,
                                JSCalendarEventStatus::Cancelled => ICalendarStatus::Cancelled,
                                JSCalendarEventStatus::Tentative => ICalendarStatus::Tentative,
                            })
                            .import_converted(&[JSCalendarProperty::Status], &mut root_conversions),
                    );
                }
                (
                    JSCalendarProperty::Status | JSCalendarProperty::Progress,
                    Value::Str(text),
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo,
                ) => {
                    component.entries.push(
                        ICalendarEntry::new(ICalendarProperty::Status)
                            .with_value(
                                ICalendarStatus::parse(text.as_bytes())
                                    .map(ICalendarValue::Status)
                                    .unwrap_or_else(|| ICalendarValue::Text(text.into_owned())),
                            )
                            .import_converted(&[JSCalendarProperty::Status], &mut root_conversions),
                    );
                }
                (
                    JSCalendarProperty::Source,
                    Value::Str(text),
                    ICalendarComponentType::VCalendar,
                ) => {
                    component.entries.push(
                        ICalendarEntry::new(ICalendarProperty::Source)
                            .with_value(Uri::parse(text.into_owned()))
                            .import_converted(&[JSCalendarProperty::Source], &mut root_conversions),
                    );
                }
                (
                    JSCalendarProperty::FreeBusyStatus,
                    Value::Element(JSCalendarValue::FreeBusyStatus(value)),
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo,
                ) => {
                    component.entries.push(
                        ICalendarEntry::new(ICalendarProperty::Transp)
                            .with_value(match value {
                                JSCalendarFreeBusyStatus::Free => {
                                    ICalendarTransparency::Transparent
                                }
                                JSCalendarFreeBusyStatus::Busy => ICalendarTransparency::Opaque,
                            })
                            .import_converted(
                                &[JSCalendarProperty::FreeBusyStatus],
                                &mut root_conversions,
                            ),
                    );
                }
                (
                    JSCalendarProperty::Entries,
                    Value::Array(items),
                    ICalendarComponentType::VCalendar,
                ) => {
                    for item in items {
                        if let Some(entries) = item.into_object() {
                            self.from_jscalendar(
                                State {
                                    tz: None,
                                    tz_end: None,
                                    tz_rid: None,
                                    start: None,
                                    recurrence_id: None,
                                    uid: None,
                                    entries,
                                    default_component_type: ICalendarComponentType::VEvent,
                                },
                                Some(&mut component),
                            );
                        }
                    }
                }

                // Skip previously processed properties
                (
                    JSCalendarProperty::Type
                    | JSCalendarProperty::ICalendar
                    | JSCalendarProperty::Uid
                    | JSCalendarProperty::MainLocationId
                    | JSCalendarProperty::Start
                    | JSCalendarProperty::TimeZone
                    | JSCalendarProperty::EndTimeZone
                    | JSCalendarProperty::RecurrenceIdTimeZone
                    | JSCalendarProperty::Locale
                    | JSCalendarProperty::OrganizerCalendarAddress
                    | JSCalendarProperty::DescriptionContentType
                    | JSCalendarProperty::RecurrenceOverrides,
                    _,
                    _,
                )
                | (_, Value::Null, _) => {}
                (property, value, _) => {
                    component.insert_jsprop(&[property.to_string().as_ref()], value);
                }
            }
        }

        // Process recurrence overrides
        if let Some(overrides) = overrides {
            let mut exdates = Vec::new();
            let mut rdates = Vec::new();
            let converted_props = root_conversions.as_ref().map(|conv| {
                conv.converted_props
                    .iter()
                    .filter_map(|(keys, _)| match (keys.first()?, keys.get(1)?) {
                        (
                            Key::Property(JSCalendarProperty::RecurrenceOverrides),
                            Key::Property(JSCalendarProperty::DateTime(dt)),
                        ) => Some(*dt),
                        _ => None,
                    })
                    .collect::<AHashSet<_>>()
            });

            for (key, value) in overrides.into_vec() {
                let (Key::Property(JSCalendarProperty::DateTime(jsdt)), Value::Object(obj)) =
                    (key, value)
                else {
                    continue;
                };
                let Some(dt) = jsdt.to_naive_date_time().and_then(|dt| {
                    state
                        .tz
                        .unwrap_or_default()
                        .from_local_datetime(&dt)
                        .single()
                }) else {
                    continue;
                };
                let has_converted_prop = converted_props
                    .as_ref()
                    .is_some_and(|props| props.contains(&jsdt));

                if !obj.is_empty() {
                    if !obj.contains_key_value(
                        &Key::Property(JSCalendarProperty::Excluded),
                        &Value::Bool(true),
                    ) {
                        self.from_jscalendar(
                            State {
                                tz: state.tz,
                                tz_end: state.tz_end,
                                tz_rid: state.tz_rid,
                                start: None,
                                recurrence_id: Some(dt),
                                entries: obj,
                                uid: uid.as_deref(),
                                default_component_type: component.component_type.clone(),
                            },
                            parent_component.as_deref_mut(),
                        );
                    } else {
                        // EXDATE
                        if has_converted_prop {
                            component.entries.push(
                                ICalendarEntry::new(ICalendarProperty::Exdate)
                                    .import_converted(
                                        &[
                                            JSCalendarProperty::RecurrenceOverrides,
                                            JSCalendarProperty::DateTime(jsdt),
                                        ],
                                        &mut root_conversions,
                                    )
                                    .with_date_time(dt),
                            );
                        } else {
                            exdates.push(dt);
                        }
                    }
                } else {
                    // RDATE
                    if has_converted_prop {
                        component.entries.push(
                            ICalendarEntry::new(ICalendarProperty::Rdate)
                                .import_converted(
                                    &[
                                        JSCalendarProperty::RecurrenceOverrides,
                                        JSCalendarProperty::DateTime(jsdt),
                                    ],
                                    &mut root_conversions,
                                )
                                .with_date_time(dt),
                        );
                    } else {
                        rdates.push(dt)
                    }
                }
            }

            for (prop, values) in [
                (ICalendarProperty::Rdate, rdates),
                (ICalendarProperty::Exdate, exdates),
            ] {
                if !values.is_empty() {
                    component
                        .entries
                        .push(ICalendarEntry::new(prop).with_date_times(values));
                }
            }
        }

        // Add organizer, if not present in participants
        if let Some(organizer_address) = organizer_address {
            let mut organizer = ICalendarEntry::new(ICalendarProperty::Organizer).with_param_opt(
                organizer_name.map(|name| ICalendarParameter::cn(name.into_owned())),
            );
            organizer.values = vec![ICalendarValue::Text(organizer_address.into_owned())];
            component.entries.push(organizer);
        }

        // Add parent recurrence ID
        if add_recurrence_id && let Some(dt) = state.recurrence_id {
            component.entries.push(
                ICalendarEntry::new(ICalendarProperty::RecurrenceId)
                    .import_converted(&[JSCalendarProperty::RecurrenceId], &mut root_conversions)
                    .with_date_time(dt),
            );
        }

        // Add showWithoutTime: true unless there are times with DATE types
        if let Some(show_without_time) = show_without_time
            && (!show_without_time.params.is_empty()
                || matches!(
                    show_without_time.values.first(),
                    Some(ICalendarValue::Boolean(false))
                )
                || !component.entries.iter().any(|e| {
                    matches!(
                        e.name,
                        ICalendarProperty::Dtstart
                            | ICalendarProperty::Dtend
                            | ICalendarProperty::Due
                    ) && matches!(
                        e.parameter(&ICalendarParameterName::Value),
                        Some(ICalendarParameterValue::Value(ICalendarValueType::Date))
                    )
                }))
        {
            component.entries.push(show_without_time);
        }

        if let Some(root_conversions) = root_conversions {
            component = root_conversions.apply_conversions(component, self);
        }

        if let Some(parent_component) = parent_component {
            parent_component
                .component_ids
                .push(self.push_component(component));
        } else {
            self.components[0] = component;
        }
    }

    pub fn push_component(&mut self, component: ICalendarComponent) -> u32 {
        let comp_num = self.components.len();
        self.components.push(component);
        comp_num as u32
    }
}

impl ICalendarComponent {
    fn import_links<I: JSCalendarId, B: JSCalendarId>(
        &mut self,
        obj: Map<'_, JSCalendarProperty<I>, JSCalendarValue<I, B>>,
        conversion: &mut Option<ConvertedComponent<'_, I, B>>,
    ) {
        for (name, value) in obj.into_vec() {
            let mut entry = ICalendarEntry::new(ICalendarProperty::Link);
            let mut has_link_rel = false;
            let mut has_display = false;
            let mut is_uuid5_key = false;

            for (sub_property, value) in value.into_expanded_object() {
                match (sub_property, value) {
                    (
                        Key::Property(JSCalendarProperty::Type | JSCalendarProperty::ICalendar),
                        _,
                    ) => {}
                    (Key::Property(JSCalendarProperty::Href), Value::Str(text)) => {
                        is_uuid5_key = uuid5(text.as_ref()) == name.to_string();
                        entry
                            .values
                            .push(ICalendarValue::Uri(Uri::parse(text.into_owned())));
                    }
                    (Key::Property(JSCalendarProperty::ContentType), Value::Str(text)) => {
                        entry
                            .params
                            .push(ICalendarParameter::fmttype(text.into_owned()));
                    }
                    (Key::Property(JSCalendarProperty::Size), Value::Number(number)) => {
                        entry
                            .params
                            .push(ICalendarParameter::size(number.cast_to_u64()));
                    }
                    (
                        Key::Property(JSCalendarProperty::Rel),
                        Value::Element(JSCalendarValue::LinkRelation(relation)),
                    ) => {
                        entry.params.push(ICalendarParameter::linkrel(relation));
                        has_link_rel = true;
                    }
                    (Key::Property(JSCalendarProperty::Display), Value::Object(obj)) => {
                        has_display = true;
                        for key in obj.into_expanded_boolean_set() {
                            let value = match key {
                                Key::Property(JSCalendarProperty::LinkDisplay(display)) => {
                                    ICalendarParameterValue::Display(match display {
                                        JSCalendarLinkDisplay::Badge => ICalendarDisplayType::Badge,
                                        JSCalendarLinkDisplay::Graphic => {
                                            ICalendarDisplayType::Graphic
                                        }
                                        JSCalendarLinkDisplay::Fullsize => {
                                            ICalendarDisplayType::Fullsize
                                        }
                                        JSCalendarLinkDisplay::Thumbnail => {
                                            ICalendarDisplayType::Thumbnail
                                        }
                                    })
                                }
                                other => {
                                    ICalendarParameterValue::Text(other.to_string().into_owned())
                                }
                            };
                            entry.params.push(ICalendarParameter::display(value));
                        }
                    }
                    (Key::Property(JSCalendarProperty::Title), Value::Str(text)) => {
                        entry
                            .params
                            .push(ICalendarParameter::label(text.into_owned()));
                    }
                    (sub_property, value) => {
                        self.insert_jsprop(
                            &[
                                JSCalendarProperty::Links::<I>.to_string().as_ref(),
                                name.to_string().as_ref(),
                                sub_property.to_string().as_ref(),
                            ],
                            value,
                        );
                    }
                }
            }

            if has_display {
                entry.name = ICalendarProperty::Image;
            } else if !has_link_rel {
                entry.name = ICalendarProperty::Attach;
            }
            if !is_uuid5_key {
                entry.add_param(ICalendarParameter::jsid(name.into_string()));
            };

            self.entries
                .push(entry.import_converted(&[JSCalendarProperty::Links], conversion));
        }
    }

    fn import_relations<I: JSCalendarId, B: JSCalendarId>(
        &mut self,
        obj: Map<'_, JSCalendarProperty<I>, JSCalendarValue<I, B>>,
        conversion: &mut Option<ConvertedComponent<'_, I, B>>,
    ) {
        for (name, value) in obj.into_vec() {
            let mut entry = ICalendarEntry::new(ICalendarProperty::RelatedTo);

            for (sub_property, value) in value.into_expanded_object() {
                match (sub_property, value) {
                    (Key::Property(JSCalendarProperty::Relation), Value::Object(obj)) => {
                        for key in obj.into_expanded_boolean_set() {
                            let value = match key {
                                Key::Property(JSCalendarProperty::RelationValue(relation)) => {
                                    ICalendarParameterValue::Reltype(match relation {
                                        JSCalendarRelation::First => {
                                            ICalendarRelationshipType::First
                                        }
                                        JSCalendarRelation::Next => ICalendarRelationshipType::Next,
                                        JSCalendarRelation::Child => {
                                            ICalendarRelationshipType::Child
                                        }
                                        JSCalendarRelation::Parent => {
                                            ICalendarRelationshipType::Parent
                                        }
                                        JSCalendarRelation::Snooze => {
                                            ICalendarRelationshipType::Snooze
                                        }
                                    })
                                }
                                other => {
                                    ICalendarParameterValue::Text(other.to_string().into_owned())
                                }
                            };
                            entry.params.push(ICalendarParameter::reltype(value));
                        }
                    }
                    (
                        Key::Property(JSCalendarProperty::Type | JSCalendarProperty::ICalendar),
                        _,
                    ) => {}
                    (sub_property, value) => {
                        self.insert_jsprop(
                            &[
                                JSCalendarProperty::RelatedTo::<I>.to_string().as_ref(),
                                name.to_string().as_ref(),
                                sub_property.to_string().as_ref(),
                            ],
                            value,
                        );
                    }
                }
            }

            self.entries.push(
                entry
                    .with_value(name.into_string())
                    .import_converted(&[JSCalendarProperty::RelatedTo], conversion),
            );
        }
    }

    fn insert_jsprop<I: JSCalendarId, B: JSCalendarId>(
        &mut self,
        path: &[&str],
        value: Value<'_, JSCalendarProperty<I>, JSCalendarValue<I, B>>,
    ) {
        self.entries.push(
            ICalendarEntry::new(ICalendarProperty::Jsprop)
                .with_param(ICalendarParameter::jsptr(JsonPointer::<
                    JSCalendarProperty<I>,
                >::encode(path)))
                .with_value(serde_json::to_string(&value).unwrap_or_default()),
        );
    }
}

fn parse_geo(text: Cow<'_, str>) -> Vec<ICalendarValue> {
    if let Some((a, b)) = text
        .strip_prefix("geo:")
        .and_then(|v| v.trim().split_once(','))
        .and_then(|(a, b)| {
            let a = a.parse::<f64>().ok()?;
            let b = b.parse::<f64>().ok()?;
            Some((a, b))
        })
    {
        vec![ICalendarValue::Float(a), ICalendarValue::Float(b)]
    } else {
        vec![ICalendarValue::Text(text.into_owned())]
    }
}
