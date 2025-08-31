/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

use std::str::FromStr;

use crate::{
    common::{
        IanaParse, PartialDateTime,
        timezone::{Tz, TzTimestamp},
    },
    datecalc::rrule,
    icalendar::*,
    jscalendar::{export::ConvertedComponent, *},
};
use ahash::AHashSet;
use chrono::{DateTime, TimeDelta, TimeZone};
use jmap_tools::{JsonPointer, JsonPointerItem, Key, Map, Value};

enum Target {
    Component(ICalendarComponent),
    Entry(ICalendarEntry),
}

pub(crate) struct State<'x> {
    tz: Option<Tz>,
    tz_end: Option<Tz>,
    tz_rid: Option<Tz>,
    start: Option<DateTime<Tz>>,
    recurrence_id: Option<DateTime<Tz>>,
    components: &'x mut Vec<ICalendarComponent>,
}

#[derive(Default)]
pub(crate) struct ParentEntries {
    components: Vec<u32>,
    method: Option<ICalendarMethod>,
}

impl ICalendarComponent {
    pub(crate) fn entries_from_jscalendar(
        &mut self,
        mut state: State<'_>,
        mut entries: Map<'_, JSCalendarProperty, JSCalendarValue>,
    ) -> ParentEntries {
        let mut root_conversions = None;
        let mut locale = None;
        let mut uid = None;
        let mut main_location_id = None;
        let mut start = None;
        let mut result = ParentEntries::default();

        let todo = "don't add JSID if it is equal to UUID5";

        for (key, value) in entries.as_mut_vec() {
            match (key, value) {
                (Key::Property(JSCalendarProperty::ICalComponent), Value::Object(obj)) => {
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
                (
                    Key::Property(JSCalendarProperty::Start),
                    Value::Element(JSCalendarValue::DateTime(dt)),
                ) if matches!(
                    self.component_type,
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo
                ) =>
                {
                    start = dt.to_naive_date_time();
                }
                _ => (),
            }
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
            self.entries.push(
                ICalendarEntry::new(ICalendarProperty::Dtstart)
                    .import_converted(&[JSCalendarProperty::Start], &[&mut root_conversions])
                    .with_date_time(dt),
            );
        }

        // Add UID
        if let Some(uid) = &uid {
            self.entries.push(
                ICalendarEntry::new(ICalendarProperty::Uid)
                    .import_converted(&[JSCalendarProperty::Uid], &[&mut root_conversions])
                    .with_value(uid.clone().into_owned()),
            );
        }

        let mut add_recurrence_id = state.recurrence_id.is_some();
        for (key, value) in entries.into_vec() {
            let Key::Property(property) = key else {
                self.insert_jsprop(&[key.to_string().as_ref()], value);
                continue;
            };

            match (&property, value, &self.component_type) {
                (
                    JSCalendarProperty::Links,
                    Value::Object(obj),
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo,
                ) => {
                    self.import_links(obj, &mut root_conversions);
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
                        let mut component =
                            ICalendarComponent::new(ICalendarComponentType::Participant);
                        let mut calendar_address = None;
                        let mut status = None;
                        let mut progress = None;
                        let mut description = None;
                        let mut description_content_type = None;
                        let mut participant_name = None;
                        let mut is_attendee = false;
                        let mut is_organizer = false;

                        for (sub_property, value) in value.into_vec() {
                            match (sub_property, value) {
                                (
                                    Key::Property(JSCalendarProperty::CalendarAddress),
                                    Value::Str(text),
                                ) => {
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
                                        if let Key::Property(JSCalendarProperty::ParticipantRole(
                                            role,
                                        )) = key
                                        {
                                            let role = match role {
                                                JSCalendarParticipantRole::Owner => {
                                                    is_organizer = true;
                                                    continue;
                                                }
                                                JSCalendarParticipantRole::Attendee => {
                                                    is_attendee = true;
                                                    continue;
                                                }
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
                                            };
                                            is_attendee = true;
                                            entry.params.push(ICalendarParameter::role(role));
                                        } else {
                                            self.insert_jsprop(
                                                &[
                                                    property.to_string().as_ref(),
                                                    name.to_string().as_ref(),
                                                    JSCalendarProperty::Roles.to_string().as_ref(),
                                                    key.to_string().as_ref(),
                                                ],
                                                Value::Bool(true),
                                            );
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
                                    component.import_links(obj, &mut item_conversions);
                                }
                                (
                                    Key::Property(JSCalendarProperty::PercentComplete),
                                    Value::Number(number),
                                ) => {
                                    component.entries.push(
                                        ICalendarEntry::new(ICalendarProperty::PercentComplete)
                                            .with_value(number.cast_to_i64())
                                            .import_converted(
                                                &[JSCalendarProperty::PercentComplete],
                                                &[&mut item_conversions],
                                            ),
                                    );
                                }
                                (
                                    Key::Property(
                                        JSCalendarProperty::Type
                                        | JSCalendarProperty::ICalComponent,
                                    ),
                                    _,
                                ) => {}
                                (sub_property, value) => {
                                    let todo = "add to component";
                                    self.insert_jsprop(
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

                        match (description, description_content_type) {
                            (Some(description), Some(content_type)) => {
                                component.entries.push(
                                    ICalendarEntry::new(ICalendarProperty::StyledDescription)
                                        .with_param(ICalendarParameter::fmttype(
                                            content_type.into_owned(),
                                        ))
                                        .with_value(description.into_owned())
                                        .import_converted(
                                            &[JSCalendarProperty::Description],
                                            &[&mut item_conversions],
                                        ),
                                );
                            }
                            (Some(description), None) => {
                                component.entries.push(
                                    ICalendarEntry::new(ICalendarProperty::Description)
                                        .with_value(description.into_owned())
                                        .import_converted(
                                            &[JSCalendarProperty::Description],
                                            &[&mut item_conversions],
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
                                        JSCalendarProgress::Cancelled => {
                                            ICalendarParticipationStatus::Declined
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
                            !component.entries.is_empty() || item_conversions.is_some();
                        let has_entry = !entry.params.is_empty() || item_conversions.is_none();

                        if let Some(calendar_address) = calendar_address {
                            let calendar_address =
                                ICalendarValue::Uri(Uri::parse(calendar_address.into_owned()));

                            if has_component {
                                component.entries.push(
                                    ICalendarEntry::new(ICalendarProperty::CalendarAddress)
                                        .with_value(calendar_address.clone()),
                                );
                            }
                            if has_entry {
                                entry.values.push(calendar_address);
                            }
                        }

                        if let Some(participant_name) = participant_name {
                            if has_component {
                                component.entries.push(
                                    ICalendarEntry::new(ICalendarProperty::Summary)
                                        .with_value(participant_name.as_ref().to_string())
                                        .import_converted(
                                            &[JSCalendarProperty::Name],
                                            &[&mut item_conversions],
                                        ),
                                );
                            }
                            if has_entry {
                                entry
                                    .params
                                    .push(ICalendarParameter::cn(participant_name.into_owned()));
                            }
                        }

                        if has_component {
                            component.entries.push(
                                ICalendarEntry::new(ICalendarProperty::Jsid)
                                    .with_value(name.to_string().into_owned()),
                            );
                            component.apply_conversions(item_conversions);
                            self.component_ids.push(state.push_component(component));
                        }

                        if !entry.values.is_empty() {
                            let attendee = entry
                                .with_param(ICalendarParameter::jsid(name.into_string()))
                                .import_converted(
                                    &[JSCalendarProperty::Participants],
                                    &[&mut root_conversions],
                                );

                            if is_organizer {
                                let mut organizer =
                                    ICalendarEntry::new(ICalendarProperty::Organizer);
                                organizer.values = attendee.values.clone();
                                for param in &attendee.params {
                                    if matches!(
                                        param.name,
                                        ICalendarParameterName::Cn
                                            | ICalendarParameterName::Email
                                            | ICalendarParameterName::SentBy
                                            | ICalendarParameterName::Dir
                                            | ICalendarParameterName::Language
                                    ) {
                                        organizer.params.push(param.clone());
                                    }
                                }
                                self.entries.push(organizer);
                            }

                            if is_attendee || !is_organizer {
                                self.entries.push(attendee);
                            }
                        }
                    }
                }
                (
                    JSCalendarProperty::Alerts,
                    Value::Object(obj),
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo,
                ) => {
                    for (name, value) in obj.into_vec() {
                        let Value::Object(mut value) = value else {
                            continue;
                        };

                        let mut item_conversions = ConvertedComponent::build(&mut value);
                        let mut component = ICalendarComponent::new(ICalendarComponentType::VAlarm);

                        for (sub_property, value) in value.into_vec() {
                            match (sub_property, value) {
                                (
                                    Key::Property(
                                        JSCalendarProperty::Type
                                        | JSCalendarProperty::ICalComponent,
                                    ),
                                    _,
                                ) => {}
                                (
                                    Key::Property(JSCalendarProperty::Acknowledged),
                                    Value::Element(JSCalendarValue::DateTime(dt)),
                                ) => {
                                    component.entries.push(
                                        ICalendarEntry::new(ICalendarProperty::Acknowledged)
                                            .with_value(PartialDateTime::from_utc_timestamp(
                                                dt.timestamp,
                                            ))
                                            .import_converted(
                                                &[JSCalendarProperty::Acknowledged],
                                                &[&mut item_conversions],
                                            ),
                                    );
                                }
                                (
                                    Key::Property(JSCalendarProperty::Action),
                                    Value::Element(JSCalendarValue::AlertAction(action)),
                                ) => {
                                    component.entries.push(
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
                                                &[&mut item_conversions],
                                            ),
                                    );
                                }
                                (
                                    Key::Property(JSCalendarProperty::RelatedTo),
                                    Value::Object(obj),
                                ) => {
                                    self.import_relations(obj, &mut item_conversions);
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
                                            (key, value) => {
                                                self.insert_jsprop(
                                                    &[
                                                        JSCalendarProperty::Trigger
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
                                        component.entries.push(
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
                                                    &[&mut item_conversions],
                                                ),
                                        );
                                    } else if let Some(offset) = offset {
                                        component.entries.push(
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
                                                    &[&mut item_conversions],
                                                ),
                                        );
                                    }
                                }
                                (sub_property, value) => {
                                    component
                                        .insert_jsprop(&[sub_property.to_string().as_ref()], value);
                                }
                            }
                        }

                        component.apply_conversions(item_conversions);

                        if !component.entries.is_empty() {
                            component.entries.push(
                                ICalendarEntry::new(ICalendarProperty::Jsid)
                                    .with_value(name.to_string().into_owned()),
                            );
                            self.component_ids.push(state.push_component(component));
                        }
                    }
                }
                (
                    JSCalendarProperty::Keywords,
                    Value::Object(obj),
                    ICalendarComponentType::VEvent
                    | ICalendarComponentType::VTodo
                    | ICalendarComponentType::VCalendar,
                ) => {
                    self.entries.push(
                        ICalendarEntry::new(ICalendarProperty::Categories)
                            .with_values(
                                obj.into_expanded_boolean_set()
                                    .map(|v| ICalendarValue::Text(v.into_string()))
                                    .collect(),
                            )
                            .import_converted(
                                &[JSCalendarProperty::Keywords],
                                &[&mut root_conversions],
                            ),
                    );
                }
                (
                    JSCalendarProperty::Privacy,
                    Value::Element(JSCalendarValue::Privacy(value)),
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo,
                ) => {
                    self.entries.push(
                        ICalendarEntry::new(ICalendarProperty::Class)
                            .with_value(match value {
                                JSCalendarPrivacy::Public => ICalendarClassification::Public,
                                JSCalendarPrivacy::Private => ICalendarClassification::Private,
                                JSCalendarPrivacy::Secret => ICalendarClassification::Confidential,
                            })
                            .import_converted(
                                &[JSCalendarProperty::Privacy],
                                &[&mut root_conversions],
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
                    self.entries.push(
                        ICalendarEntry::new(ICalendarProperty::Color)
                            .with_value(text.into_owned())
                            .import_converted(
                                &[JSCalendarProperty::Color],
                                &[&mut root_conversions],
                            ),
                    );
                }
                (
                    JSCalendarProperty::Categories,
                    Value::Object(obj),
                    ICalendarComponentType::VEvent
                    | ICalendarComponentType::VTodo
                    | ICalendarComponentType::VCalendar,
                ) => {
                    self.entries.push(
                        ICalendarEntry::new(ICalendarProperty::Concept)
                            .with_values(
                                obj.into_expanded_boolean_set()
                                    .map(|v| ICalendarValue::Text(v.into_string()))
                                    .collect(),
                            )
                            .import_converted(
                                &[JSCalendarProperty::Categories],
                                &[&mut root_conversions],
                            ),
                    );
                }
                (
                    JSCalendarProperty::VirtualLocations,
                    Value::Object(obj),
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo,
                ) => {
                    for (name, value) in obj.into_vec() {
                        let mut entry = ICalendarEntry::new(ICalendarProperty::Conference);

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
                                    entry
                                        .values
                                        .push(ICalendarValue::Uri(Uri::parse(text.into_owned())));
                                }
                                (sub_property, value) => {
                                    self.insert_jsprop(
                                        &[
                                            JSCalendarProperty::VirtualLocations
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

                        self.entries.push(
                            entry
                                .with_param(ICalendarParameter::jsid(name.into_string()))
                                .import_converted(
                                    &[JSCalendarProperty::VirtualLocations],
                                    &[&mut root_conversions],
                                ),
                        );
                    }
                }
                (
                    JSCalendarProperty::Title,
                    Value::Str(text),
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo,
                ) => {
                    self.entries.push(
                        ICalendarEntry::new(ICalendarProperty::Summary)
                            .with_param_opt(
                                locale
                                    .as_ref()
                                    .map(|locale| ICalendarParameter::language(locale.to_string())),
                            )
                            .with_value(text.into_owned())
                            .import_converted(
                                &[JSCalendarProperty::Title],
                                &[&mut root_conversions],
                            ),
                    );
                }
                (
                    JSCalendarProperty::Title,
                    Value::Str(text),
                    ICalendarComponentType::VCalendar,
                ) => {
                    self.entries.push(
                        ICalendarEntry::new(ICalendarProperty::Name)
                            .with_param_opt(
                                locale
                                    .as_ref()
                                    .map(|locale| ICalendarParameter::language(locale.to_string())),
                            )
                            .with_value(text.into_owned())
                            .import_converted(
                                &[JSCalendarProperty::Title],
                                &[&mut root_conversions],
                            ),
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
                        let mut component = (item_conversions
                            .as_ref()
                            .is_some_and(|v| v.name.is_location())
                            || value.iter().any(|(k, v)| match (k, v) {
                                (
                                    Key::Property(
                                        JSCalendarProperty::Links
                                        | JSCalendarProperty::LocationTypes,
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
                            || (has_multi_location
                                && main_location_id
                                    .as_ref()
                                    .is_none_or(|l| l != &name.to_string())))
                        .then_some(ICalendarComponent::new(ICalendarComponentType::VLocation));

                        for (sub_property, value) in value.into_vec() {
                            match (sub_property, value) {
                                (
                                    Key::Property(JSCalendarProperty::LocationTypes),
                                    Value::Object(obj),
                                ) => {
                                    if let Some(component) = &mut component {
                                        component.entries.push(
                                            ICalendarEntry::new(ICalendarProperty::LocationType)
                                                .import_converted(
                                                    &[JSCalendarProperty::LocationTypes],
                                                    &[&mut item_conversions],
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
                                    if let Some(component) = &mut component {
                                        component.entries.push(
                                            ICalendarEntry::new(ICalendarProperty::Location)
                                                .import_converted(
                                                    &[JSCalendarProperty::Name],
                                                    &[&mut item_conversions],
                                                )
                                                .with_value(text.into_owned()),
                                        );
                                    } else {
                                        self.entries.push(
                                            ICalendarEntry::new(ICalendarProperty::Location)
                                                .with_param(ICalendarParameter::jsid(
                                                    name.clone().into_string(),
                                                ))
                                                .with_value(text.into_owned())
                                                .import_converted(
                                                    &[JSCalendarProperty::Locations],
                                                    &[&mut root_conversions],
                                                ),
                                        );
                                    }
                                }
                                (
                                    Key::Property(JSCalendarProperty::Coordinates),
                                    Value::Str(text),
                                ) => {
                                    if let Some(component) = &mut component {
                                        component.entries.push(
                                            ICalendarEntry::new(ICalendarProperty::Coordinates)
                                                .import_converted(
                                                    &[JSCalendarProperty::Coordinates],
                                                    &[&mut item_conversions],
                                                )
                                                .with_value(Uri::parse(text.into_owned())),
                                        );
                                    } else {
                                        let value = if let Some((a, b)) = text
                                            .strip_prefix("geo:")
                                            .and_then(|v| v.trim().split_once(','))
                                            .and_then(|(a, b)| {
                                                let a = a.parse::<f64>().ok()?;
                                                let b = b.parse::<f64>().ok()?;
                                                Some((a, b))
                                            }) {
                                            vec![ICalendarValue::Float(a), ICalendarValue::Float(b)]
                                        } else {
                                            vec![ICalendarValue::Text(text.into_owned())]
                                        };
                                        self.entries.push(
                                            ICalendarEntry::new(ICalendarProperty::Geo)
                                                .with_param(ICalendarParameter::jsid(
                                                    name.clone().into_string(),
                                                ))
                                                .with_values(value)
                                                .import_converted(
                                                    &[JSCalendarProperty::Locations],
                                                    &[&mut root_conversions],
                                                ),
                                        );
                                    }
                                }
                                (Key::Property(JSCalendarProperty::Links), Value::Object(obj)) => {
                                    if let Some(component) = &mut component {
                                        component.import_links(obj, &mut item_conversions);
                                    }
                                }
                                (
                                    Key::Property(
                                        JSCalendarProperty::Type
                                        | JSCalendarProperty::ICalComponent,
                                    ),
                                    _,
                                ) => {}
                                (sub_property, value) => {
                                    self.insert_jsprop(
                                        &[
                                            JSCalendarProperty::Locations.to_string().as_ref(),
                                            name.to_string().as_ref(),
                                            sub_property.to_string().as_ref(),
                                        ],
                                        value,
                                    );
                                }
                            }
                        }

                        if let Some(mut component) = component {
                            component.entries.push(
                                ICalendarEntry::new(ICalendarProperty::Jsid)
                                    .with_value(name.to_string().into_owned()),
                            );
                            component.apply_conversions(item_conversions);
                            self.component_ids.push(state.push_component(component));
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
                    self.entries.push(
                        ICalendarEntry::new(ICalendarProperty::Created)
                            .with_value(PartialDateTime::from_utc_timestamp(dt.timestamp))
                            .import_converted(
                                &[JSCalendarProperty::Created],
                                &[&mut root_conversions],
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
                    self.entries.push(
                        ICalendarEntry::new(ICalendarProperty::Description)
                            .with_value(text.into_owned())
                            .import_converted(
                                &[JSCalendarProperty::Description],
                                &[&mut root_conversions],
                            ),
                    );
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
                            (key, value) => {
                                self.insert_jsprop(
                                    &[
                                        JSCalendarProperty::RecurrenceRule.to_string().as_ref(),
                                        key.to_string().as_ref(),
                                    ],
                                    value,
                                );
                            }
                        }
                    }

                    self.entries.push(
                        ICalendarEntry::new(ICalendarProperty::Rrule)
                            .with_value(rrule)
                            .import_converted(
                                &[JSCalendarProperty::RecurrenceRule],
                                &[&mut root_conversions],
                            ),
                    );
                }
                (
                    JSCalendarProperty::Updated,
                    Value::Element(JSCalendarValue::DateTime(dt)),
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo,
                ) => {
                    self.entries.push(
                        ICalendarEntry::new(ICalendarProperty::Dtstamp)
                            .with_value(PartialDateTime::from_utc_timestamp(dt.timestamp))
                            .import_converted(
                                &[JSCalendarProperty::Updated],
                                &[&mut root_conversions],
                            ),
                    );
                }
                (
                    JSCalendarProperty::Updated,
                    Value::Element(JSCalendarValue::DateTime(dt)),
                    ICalendarComponentType::VCalendar,
                ) => {
                    self.entries.push(
                        ICalendarEntry::new(ICalendarProperty::LastModified)
                            .with_value(PartialDateTime::from_utc_timestamp(dt.timestamp))
                            .import_converted(
                                &[JSCalendarProperty::Updated],
                                &[&mut root_conversions],
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
                        self.entries.push(
                            ICalendarEntry::new(ICalendarProperty::Due)
                                .import_converted(
                                    &[JSCalendarProperty::Due],
                                    &[&mut root_conversions],
                                )
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
                        self.entries.push(
                            ICalendarEntry::new(ICalendarProperty::RecurrenceId)
                                .import_converted(
                                    &[JSCalendarProperty::RecurrenceId],
                                    &[&mut root_conversions],
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
                    let entry = ICalendarEntry::new(ICalendarProperty::Duration).import_converted(
                        &[JSCalendarProperty::Duration],
                        &[&mut root_conversions],
                    );
                    if entry.name == ICalendarProperty::Dtend {
                        if let Some(end) = state.start.and_then(|start| {
                            start.checked_add_signed(TimeDelta::seconds(duration.as_seconds()))
                        }) {
                            self.entries.push(
                                entry.with_date_time(
                                    state
                                        .tz_end
                                        .map(|tz_end| end.with_timezone(&tz_end))
                                        .unwrap_or(end),
                                ),
                            );
                        } else {
                            self.insert_jsprop(
                                &[property.to_string().as_ref()],
                                Value::Element(JSCalendarValue::Duration(duration)),
                            );
                        }
                    } else {
                        self.entries.push(entry.with_value(duration));
                    }
                }
                (
                    JSCalendarProperty::EstimatedDuration,
                    Value::Element(JSCalendarValue::Duration(duration)),
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo,
                ) => {
                    self.entries.push(
                        ICalendarEntry::new(ICalendarProperty::EstimatedDuration)
                            .with_value(duration)
                            .import_converted(
                                &[JSCalendarProperty::EstimatedDuration],
                                &[&mut root_conversions],
                            ),
                    );
                }
                (
                    JSCalendarProperty::RecurrenceOverrides,
                    Value::Object(obj),
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo,
                ) => {
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

                    for (key, value) in obj.into_vec() {
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
                                let mut component =
                                    ICalendarComponent::new(self.component_type.clone());
                                component.entries_from_jscalendar(
                                    State {
                                        components: state.components,
                                        tz: state.tz,
                                        tz_end: state.tz_end,
                                        tz_rid: state.tz_rid,
                                        start: state.start,
                                        recurrence_id: Some(dt),
                                    },
                                    obj,
                                );
                                if !component.has_property(&ICalendarProperty::Uid)
                                    && let Some(uid) = &uid
                                {
                                    component.entries.push(
                                        ICalendarEntry::new(ICalendarProperty::Uid)
                                            .with_value(uid.to_string()),
                                    );
                                }
                                result.components.push(state.push_component(component));
                            } else {
                                // EXDATE
                                if has_converted_prop {
                                    self.entries.push(
                                        ICalendarEntry::new(ICalendarProperty::Exdate)
                                            .import_converted(
                                                &[
                                                    JSCalendarProperty::RecurrenceOverrides,
                                                    JSCalendarProperty::DateTime(jsdt),
                                                ],
                                                &[&mut root_conversions],
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
                                self.entries.push(
                                    ICalendarEntry::new(ICalendarProperty::Rdate)
                                        .import_converted(
                                            &[
                                                JSCalendarProperty::RecurrenceOverrides,
                                                JSCalendarProperty::DateTime(jsdt),
                                            ],
                                            &[&mut root_conversions],
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
                            self.entries
                                .push(ICalendarEntry::new(prop).with_date_times(values));
                        }
                    }
                }
                (
                    JSCalendarProperty::Method,
                    Value::Element(JSCalendarValue::Method(method)),
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo,
                ) => {
                    result.method = Some(method);
                }
                (
                    JSCalendarProperty::PercentComplete,
                    Value::Number(number),
                    ICalendarComponentType::VTodo,
                ) => {
                    self.entries.push(
                        ICalendarEntry::new(ICalendarProperty::PercentComplete)
                            .with_value(number.cast_to_i64())
                            .import_converted(
                                &[JSCalendarProperty::PercentComplete],
                                &[&mut root_conversions],
                            ),
                    );
                }
                (
                    JSCalendarProperty::Priority,
                    Value::Number(number),
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo,
                ) => {
                    self.entries.push(
                        ICalendarEntry::new(ICalendarProperty::Priority)
                            .with_value(number.cast_to_i64())
                            .import_converted(
                                &[JSCalendarProperty::Priority],
                                &[&mut root_conversions],
                            ),
                    );
                }
                (
                    JSCalendarProperty::Sequence,
                    Value::Number(number),
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo,
                ) => {
                    self.entries.push(
                        ICalendarEntry::new(ICalendarProperty::Sequence)
                            .with_value(number.cast_to_i64())
                            .import_converted(
                                &[JSCalendarProperty::Sequence],
                                &[&mut root_conversions],
                            ),
                    );
                }
                (
                    JSCalendarProperty::ProdId,
                    Value::Str(text),
                    ICalendarComponentType::VCalendar,
                ) => {
                    self.entries.push(
                        ICalendarEntry::new(ICalendarProperty::Prodid)
                            .with_value(text.into_owned())
                            .import_converted(
                                &[JSCalendarProperty::ProdId],
                                &[&mut root_conversions],
                            ),
                    );
                }
                (
                    JSCalendarProperty::RelatedTo,
                    Value::Object(obj),
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo,
                ) => {
                    self.import_relations(obj, &mut root_conversions);
                }
                (
                    JSCalendarProperty::ShowWithoutTime,
                    Value::Bool(value),
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo,
                ) => {
                    self.entries.push(
                        ICalendarEntry::new(ICalendarProperty::ShowWithoutTime)
                            .with_value(value)
                            .import_converted(
                                &[JSCalendarProperty::ShowWithoutTime],
                                &[&mut root_conversions],
                            ),
                    );
                }
                (
                    JSCalendarProperty::Progress,
                    Value::Element(JSCalendarValue::Progress(value)),
                    ICalendarComponentType::VTodo,
                ) => {
                    self.entries.push(
                        ICalendarEntry::new(ICalendarProperty::Status)
                            .with_value(match value {
                                JSCalendarProgress::NeedsAction => ICalendarStatus::NeedsAction,
                                JSCalendarProgress::InProcess => ICalendarStatus::InProcess,
                                JSCalendarProgress::Completed => ICalendarStatus::Completed,
                                JSCalendarProgress::Failed => ICalendarStatus::Failed,
                                JSCalendarProgress::Cancelled => ICalendarStatus::Cancelled,
                            })
                            .import_converted(
                                &[JSCalendarProperty::Progress],
                                &[&mut root_conversions],
                            ),
                    );
                }
                (
                    JSCalendarProperty::Status,
                    Value::Element(JSCalendarValue::EventStatus(value)),
                    ICalendarComponentType::VEvent,
                ) => {
                    self.entries.push(
                        ICalendarEntry::new(ICalendarProperty::Status)
                            .with_value(match value {
                                JSCalendarEventStatus::Confirmed => ICalendarStatus::Confirmed,
                                JSCalendarEventStatus::Cancelled => ICalendarStatus::Cancelled,
                                JSCalendarEventStatus::Tentative => ICalendarStatus::Tentative,
                            })
                            .import_converted(
                                &[JSCalendarProperty::Status],
                                &[&mut root_conversions],
                            ),
                    );
                }
                (
                    JSCalendarProperty::Status | JSCalendarProperty::Progress,
                    Value::Str(text),
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo,
                ) => {
                    self.entries.push(
                        ICalendarEntry::new(ICalendarProperty::Status)
                            .with_value(
                                ICalendarStatus::parse(text.as_bytes())
                                    .map(ICalendarValue::Status)
                                    .unwrap_or_else(|| ICalendarValue::Text(text.into_owned())),
                            )
                            .import_converted(
                                &[JSCalendarProperty::Status],
                                &[&mut root_conversions],
                            ),
                    );
                }
                (
                    JSCalendarProperty::Source,
                    Value::Str(text),
                    ICalendarComponentType::VCalendar,
                ) => {
                    self.entries.push(
                        ICalendarEntry::new(ICalendarProperty::Source)
                            .with_value(Uri::parse(text.into_owned()))
                            .import_converted(
                                &[JSCalendarProperty::Source],
                                &[&mut root_conversions],
                            ),
                    );
                }
                (
                    JSCalendarProperty::FreeBusyStatus,
                    Value::Element(JSCalendarValue::FreeBusyStatus(value)),
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo,
                ) => {
                    self.entries.push(
                        ICalendarEntry::new(ICalendarProperty::Transp)
                            .with_value(match value {
                                JSCalendarFreeBusyStatus::Free => {
                                    ICalendarTransparency::Transparent
                                }
                                JSCalendarFreeBusyStatus::Busy => ICalendarTransparency::Opaque,
                            })
                            .import_converted(
                                &[JSCalendarProperty::FreeBusyStatus],
                                &[&mut root_conversions],
                            ),
                    );
                }

                // Skip previously processed properties
                (
                    JSCalendarProperty::Type
                    | JSCalendarProperty::ICalComponent
                    | JSCalendarProperty::Uid
                    | JSCalendarProperty::MainLocationId
                    | JSCalendarProperty::Start
                    | JSCalendarProperty::TimeZone
                    | JSCalendarProperty::EndTimeZone
                    | JSCalendarProperty::RecurrenceIdTimeZone
                    | JSCalendarProperty::Locale,
                    _,
                    _,
                )
                | (_, Value::Null, _) => {}
                (property, value, _) => {
                    self.insert_jsprop(&[property.to_string().as_ref()], value);
                }
            }
        }

        // Add parent recurrence ID
        if add_recurrence_id && let Some(dt) = state.recurrence_id {
            self.entries.push(
                ICalendarEntry::new(ICalendarProperty::RecurrenceId)
                    .import_converted(
                        &[JSCalendarProperty::RecurrenceId],
                        &[&mut root_conversions],
                    )
                    .with_date_time(dt),
            );
        }

        self.apply_conversions(root_conversions);
        result
    }

    fn import_links(
        &mut self,
        obj: Map<'_, JSCalendarProperty, JSCalendarValue>,
        conversion: &mut Option<ConvertedComponent<'_>>,
    ) {
        for (name, value) in obj.into_vec() {
            let mut entry = ICalendarEntry::new(ICalendarProperty::Link);
            let mut has_link_rel = false;
            let mut has_display = false;

            for (sub_property, value) in value.into_expanded_object() {
                match (sub_property, value) {
                    (
                        Key::Property(JSCalendarProperty::Type | JSCalendarProperty::ICalComponent),
                        _,
                    ) => {}
                    (Key::Property(JSCalendarProperty::Href), Value::Str(text)) => {
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
                                JSCalendarProperty::Links.to_string().as_ref(),
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

            self.entries.push(
                entry
                    .with_param(ICalendarParameter::jsid(name.into_string()))
                    .import_converted(&[JSCalendarProperty::Links], &[conversion]),
            );
        }
    }

    fn import_relations(
        &mut self,
        obj: Map<'_, JSCalendarProperty, JSCalendarValue>,
        conversion: &mut Option<ConvertedComponent<'_>>,
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
                            entry.params.push(ICalendarParameter::display(value));
                        }
                    }
                    (sub_property, value) => {
                        self.insert_jsprop(
                            &[
                                JSCalendarProperty::RelatedTo.to_string().as_ref(),
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
                    .import_converted(&[JSCalendarProperty::RelatedTo], &[conversion]),
            );
        }
    }

    fn insert_jsprop(
        &mut self,
        path: &[&str],
        value: Value<'_, JSCalendarProperty, JSCalendarValue>,
    ) {
        self.entries.push(
            ICalendarEntry::new(ICalendarProperty::Jsprop)
                .with_param(ICalendarParameter::jsptr(
                    JsonPointer::<JSCalendarProperty>::encode(path),
                ))
                .with_value(serde_json::to_string(&value).unwrap_or_default()),
        );
    }

    fn apply_conversions(&mut self, conversions: Option<ConvertedComponent<'_>>) {
        let todo = "relation uses value, not jsid";
        todo!()
    }
}

impl<'x> ConvertedComponent<'x> {
    fn try_from_object(
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

    fn build(entries: &mut Map<'x, JSCalendarProperty, JSCalendarValue>) -> Option<Self> {
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

impl ICalendarEntry {
    fn with_date_time(mut self, mut dt: DateTime<Tz>) -> Self {
        debug_assert!(self.values.is_empty());

        // Best effort to restore the original timezone
        let tz_id = self.tz_id();
        let has_tz_id = tz_id.is_some();
        if let Some(tz) = tz_id.and_then(|id| Tz::from_str(id).ok())
            && tz != dt.timezone()
        {
            dt = dt.with_timezone(&tz);
        }

        let tz = dt.timezone();
        if has_tz_id {
            self.values
                .push(PartialDateTime::from_naive_timestamp(dt.to_naive_timestamp()).into());
        } else if tz.is_utc() {
            self.values
                .push(PartialDateTime::from_utc_timestamp(dt.timestamp()).into());
        } else {
            if let Some(tz_name) = tz.name() {
                self.params
                    .push(ICalendarParameter::tzid(tz_name.into_owned()));
            }
            self.values
                .push(PartialDateTime::from_naive_timestamp(dt.to_naive_timestamp()).into());
        }

        self
    }

    fn with_date_times(mut self, dts: Vec<DateTime<Tz>>) -> Self {
        debug_assert!(self.values.is_empty());
        for dt in dts {
            let tz = dt.timezone();
            if tz.is_utc() {
                self.values
                    .push(PartialDateTime::from_utc_timestamp(dt.timestamp()).into());
            } else {
                if let Some(tz_name) = tz.name() {
                    self.params
                        .push(ICalendarParameter::tzid(tz_name.into_owned()));
                }
                self.values
                    .push(PartialDateTime::from_naive_timestamp(dt.to_naive_timestamp()).into());
            }
        }
        self
    }
}

impl Target {
    fn new(entry_type: ICalendarProperty, converted: &Option<ConvertedComponent<'_>>) -> Self {
        if let Some(conversion) = converted {
            Target::Component(ICalendarComponent::new(conversion.name.clone()))
        } else {
            Target::Entry(ICalendarEntry::new(entry_type))
        }
    }

    fn is_component(&self) -> bool {
        match self {
            Target::Component(_) => true,
            Target::Entry(_) => false,
        }
    }
}

impl<'x> State<'x> {
    pub fn push_component(&mut self, component: ICalendarComponent) -> u32 {
        let comp_num = self.components.len();
        self.components.push(component);
        comp_num as u32
    }
}
