/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

use crate::{
    common::{IanaParse, PartialDateTime},
    icalendar::{
        ICalendarAction, ICalendarComponent, ICalendarComponentType, ICalendarDisplayType,
        ICalendarEntry, ICalendarParameter, ICalendarParameterValue, ICalendarParticipationRole,
        ICalendarParticipationStatus, ICalendarProperty, ICalendarRelationshipType,
        ICalendarUserTypes, ICalendarValue, ICalendarValueType, Related, Uri,
    },
    jscalendar::{
        JSCalendarAlertAction, JSCalendarLinkDisplay, JSCalendarParticipantKind,
        JSCalendarParticipantRole, JSCalendarParticipationStatus, JSCalendarProgress,
        JSCalendarProperty, JSCalendarRelation, JSCalendarRelativeTo, JSCalendarType,
        JSCalendarValue, export::ConvertedComponent,
    },
};
use jmap_tools::{JsonPointer, JsonPointerItem, Key, Map, Value};

enum Target {
    Component(ICalendarComponent),
    Entry(ICalendarEntry),
}

impl ICalendarComponent {
    pub(crate) fn entries_from_jscalendar(
        &mut self,
        typ: JSCalendarType,
        mut entries: Map<'static, JSCalendarProperty, JSCalendarValue>,
        components: &mut Vec<ICalendarComponent>,
    ) {
        let mut root_conversions = ConvertedComponent::build(&mut entries);

        for (key, value) in entries.into_vec() {
            let Key::Property(property) = key else {
                self.insert_jsprop(&[key.to_string().as_ref()], value);
                continue;
            };

            match (&property, value, typ) {
                (
                    JSCalendarProperty::Links,
                    Value::Object(obj),
                    JSCalendarType::Event
                    | JSCalendarType::Task
                    | JSCalendarType::Location
                    | JSCalendarType::VirtualLocation
                    | JSCalendarType::Participant,
                ) => {
                    self.import_links(obj, &mut root_conversions);
                }
                (
                    JSCalendarProperty::Participants,
                    Value::Object(obj),
                    JSCalendarType::Event | JSCalendarType::Task,
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
                                                    entry.name = ICalendarProperty::Organizer;
                                                    continue;
                                                }
                                                JSCalendarParticipantRole::Attendee => continue,
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
                                    component.import_links(obj, &mut root_conversions);
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

                        let has_component = !component.entries.is_empty();

                        if let Some(calendar_address) = calendar_address {
                            let calendar_address =
                                ICalendarValue::Uri(Uri::parse(calendar_address.into_owned()));

                            if has_component {
                                component.entries.push(
                                    ICalendarEntry::new(ICalendarProperty::CalendarAddress)
                                        .with_value(calendar_address.clone()),
                                );
                            }
                            entry.values.push(calendar_address);
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
                            entry
                                .params
                                .push(ICalendarParameter::cn(participant_name.into_owned()));
                        }

                        if has_component {
                            component.entries.push(
                                ICalendarEntry::new(ICalendarProperty::Jsid)
                                    .with_value(name.to_string().into_owned()),
                            );
                            component.apply_conversions(item_conversions);
                            let comp_num = components.len();
                            components.push(component);
                            self.component_ids.push(comp_num as u16);
                        }

                        if !entry.values.is_empty() {
                            self.entries.push(
                                entry
                                    .with_param(ICalendarParameter::jsid(name.into_string()))
                                    .import_converted(
                                        &[JSCalendarProperty::Participants],
                                        &[&mut root_conversions],
                                    ),
                            );
                        }
                    }
                }
                (
                    JSCalendarProperty::Alerts,
                    Value::Object(obj),
                    JSCalendarType::Event | JSCalendarType::Task,
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
                                                            Related::Start
                                                        }
                                                        JSCalendarRelativeTo::End => Related::End,
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
                            let comp_num = components.len();
                            components.push(component);
                            self.component_ids.push(comp_num as u16);
                        }
                    }
                }
                (
                    JSCalendarProperty::BaseEventId,
                    Value::Str(text),
                    JSCalendarType::Event | JSCalendarType::Task,
                ) => {}
                (
                    JSCalendarProperty::ByDay,
                    Value::Str(text),
                    JSCalendarType::Event | JSCalendarType::Task,
                ) => {}
                (
                    JSCalendarProperty::ByHour,
                    Value::Str(text),
                    JSCalendarType::Event | JSCalendarType::Task,
                ) => {}
                (
                    JSCalendarProperty::ByMinute,
                    Value::Str(text),
                    JSCalendarType::Event | JSCalendarType::Task,
                ) => {}
                (
                    JSCalendarProperty::ByMonth,
                    Value::Str(text),
                    JSCalendarType::Event | JSCalendarType::Task,
                ) => {}
                (
                    JSCalendarProperty::ByMonthDay,
                    Value::Str(text),
                    JSCalendarType::Event | JSCalendarType::Task,
                ) => {}
                (
                    JSCalendarProperty::BySecond,
                    Value::Str(text),
                    JSCalendarType::Event | JSCalendarType::Task,
                ) => {}
                (
                    JSCalendarProperty::BySetPosition,
                    Value::Str(text),
                    JSCalendarType::Event | JSCalendarType::Task,
                ) => {}
                (
                    JSCalendarProperty::ByWeekNo,
                    Value::Str(text),
                    JSCalendarType::Event | JSCalendarType::Task,
                ) => {}
                (
                    JSCalendarProperty::ByYearDay,
                    Value::Str(text),
                    JSCalendarType::Event | JSCalendarType::Task,
                ) => {}
                (
                    JSCalendarProperty::CalendarAddress,
                    Value::Str(text),
                    JSCalendarType::Event | JSCalendarType::Task,
                ) => {}
                (
                    JSCalendarProperty::CalendarIds,
                    Value::Str(text),
                    JSCalendarType::Event | JSCalendarType::Task,
                ) => {}
                (
                    JSCalendarProperty::Categories,
                    Value::Str(text),
                    JSCalendarType::Event | JSCalendarType::Task,
                ) => {}
                (
                    JSCalendarProperty::Color,
                    Value::Str(text),
                    JSCalendarType::Event | JSCalendarType::Task,
                ) => {}
                (
                    JSCalendarProperty::ContentType,
                    Value::Str(text),
                    JSCalendarType::Event | JSCalendarType::Task,
                ) => {}
                (
                    JSCalendarProperty::Coordinates,
                    Value::Str(text),
                    JSCalendarType::Event | JSCalendarType::Task,
                ) => {}
                (
                    JSCalendarProperty::Count,
                    Value::Str(text),
                    JSCalendarType::Event | JSCalendarType::Task,
                ) => {}
                (
                    JSCalendarProperty::Created,
                    Value::Str(text),
                    JSCalendarType::Event | JSCalendarType::Task,
                ) => {}
                (
                    JSCalendarProperty::Day,
                    Value::Str(text),
                    JSCalendarType::Event | JSCalendarType::Task,
                ) => {}
                (
                    JSCalendarProperty::DelegatedFrom,
                    Value::Str(text),
                    JSCalendarType::Event | JSCalendarType::Task,
                ) => {}
                (
                    JSCalendarProperty::DelegatedTo,
                    Value::Str(text),
                    JSCalendarType::Event | JSCalendarType::Task,
                ) => {}
                (
                    JSCalendarProperty::Description,
                    Value::Str(text),
                    JSCalendarType::Event | JSCalendarType::Task,
                ) => {}
                (
                    JSCalendarProperty::DescriptionContentType,
                    Value::Str(text),
                    JSCalendarType::Event | JSCalendarType::Task,
                ) => {}
                (
                    JSCalendarProperty::Display,
                    Value::Str(text),
                    JSCalendarType::Event | JSCalendarType::Task,
                ) => {}
                (
                    JSCalendarProperty::Due,
                    Value::Str(text),
                    JSCalendarType::Event | JSCalendarType::Task,
                ) => {}
                (
                    JSCalendarProperty::Duration,
                    Value::Str(text),
                    JSCalendarType::Event | JSCalendarType::Task,
                ) => {}
                (
                    JSCalendarProperty::Email,
                    Value::Str(text),
                    JSCalendarType::Event | JSCalendarType::Task,
                ) => {}
                (
                    JSCalendarProperty::Entries,
                    Value::Str(text),
                    JSCalendarType::Event | JSCalendarType::Task,
                ) => {}
                (
                    JSCalendarProperty::EstimatedDuration,
                    Value::Str(text),
                    JSCalendarType::Event | JSCalendarType::Task,
                ) => {}
                (
                    JSCalendarProperty::Excluded,
                    Value::Str(text),
                    JSCalendarType::Event | JSCalendarType::Task,
                ) => {}
                (
                    JSCalendarProperty::ExpectReply,
                    Value::Str(text),
                    JSCalendarType::Event | JSCalendarType::Task,
                ) => {}
                (
                    JSCalendarProperty::Features,
                    Value::Str(text),
                    JSCalendarType::Event | JSCalendarType::Task,
                ) => {}
                (
                    JSCalendarProperty::FirstDayOfWeek,
                    Value::Str(text),
                    JSCalendarType::Event | JSCalendarType::Task,
                ) => {}
                (
                    JSCalendarProperty::FreeBusyStatus,
                    Value::Str(text),
                    JSCalendarType::Event | JSCalendarType::Task,
                ) => {}
                (
                    JSCalendarProperty::Frequency,
                    Value::Str(text),
                    JSCalendarType::Event | JSCalendarType::Task,
                ) => {}
                (
                    JSCalendarProperty::HideAttendees,
                    Value::Str(text),
                    JSCalendarType::Event | JSCalendarType::Task,
                ) => {}
                (
                    JSCalendarProperty::Href,
                    Value::Str(text),
                    JSCalendarType::Event | JSCalendarType::Task,
                ) => {}
                (
                    JSCalendarProperty::Id,
                    Value::Str(text),
                    JSCalendarType::Event | JSCalendarType::Task,
                ) => {}
                (
                    JSCalendarProperty::Interval,
                    Value::Str(text),
                    JSCalendarType::Event | JSCalendarType::Task,
                ) => {}
                (
                    JSCalendarProperty::InvitedBy,
                    Value::Str(text),
                    JSCalendarType::Event | JSCalendarType::Task,
                ) => {}
                (
                    JSCalendarProperty::IsDraft,
                    Value::Str(text),
                    JSCalendarType::Event | JSCalendarType::Task,
                ) => {}
                (
                    JSCalendarProperty::IsOrigin,
                    Value::Str(text),
                    JSCalendarType::Event | JSCalendarType::Task,
                ) => {}
                (
                    JSCalendarProperty::Keywords,
                    Value::Str(text),
                    JSCalendarType::Event | JSCalendarType::Task,
                ) => {}
                (
                    JSCalendarProperty::Kind,
                    Value::Str(text),
                    JSCalendarType::Event | JSCalendarType::Task,
                ) => {}

                (
                    JSCalendarProperty::Locale,
                    Value::Str(text),
                    JSCalendarType::Event | JSCalendarType::Task,
                ) => {}
                (
                    JSCalendarProperty::Localizations,
                    Value::Str(text),
                    JSCalendarType::Event | JSCalendarType::Task,
                ) => {}
                (
                    JSCalendarProperty::Locations,
                    Value::Str(text),
                    JSCalendarType::Event | JSCalendarType::Task,
                ) => {}
                (
                    JSCalendarProperty::LocationTypes,
                    Value::Str(text),
                    JSCalendarType::Event | JSCalendarType::Task,
                ) => {}
                (
                    JSCalendarProperty::MayInviteOthers,
                    Value::Str(text),
                    JSCalendarType::Event | JSCalendarType::Task,
                ) => {}
                (
                    JSCalendarProperty::MayInviteSelf,
                    Value::Str(text),
                    JSCalendarType::Event | JSCalendarType::Task,
                ) => {}
                (
                    JSCalendarProperty::MemberOf,
                    Value::Str(text),
                    JSCalendarType::Event | JSCalendarType::Task,
                ) => {}
                (
                    JSCalendarProperty::Method,
                    Value::Str(text),
                    JSCalendarType::Event | JSCalendarType::Task,
                ) => {}
                (
                    JSCalendarProperty::Name,
                    Value::Str(text),
                    JSCalendarType::Event | JSCalendarType::Task,
                ) => {}
                (
                    JSCalendarProperty::NthOfPeriod,
                    Value::Str(text),
                    JSCalendarType::Event | JSCalendarType::Task,
                ) => {}
                (
                    JSCalendarProperty::Offset,
                    Value::Str(text),
                    JSCalendarType::Event | JSCalendarType::Task,
                ) => {}

                (
                    JSCalendarProperty::ParticipationComment,
                    Value::Str(text),
                    JSCalendarType::Event | JSCalendarType::Task,
                ) => {}
                (
                    JSCalendarProperty::ParticipationStatus,
                    Value::Str(text),
                    JSCalendarType::Event | JSCalendarType::Task,
                ) => {}
                (
                    JSCalendarProperty::PercentComplete,
                    Value::Str(text),
                    JSCalendarType::Event | JSCalendarType::Task,
                ) => {}
                (
                    JSCalendarProperty::Priority,
                    Value::Str(text),
                    JSCalendarType::Event | JSCalendarType::Task,
                ) => {}
                (
                    JSCalendarProperty::Privacy,
                    Value::Str(text),
                    JSCalendarType::Event | JSCalendarType::Task,
                ) => {}
                (
                    JSCalendarProperty::ProdId,
                    Value::Str(text),
                    JSCalendarType::Event | JSCalendarType::Task,
                ) => {}
                (
                    JSCalendarProperty::Progress,
                    Value::Str(text),
                    JSCalendarType::Event | JSCalendarType::Task,
                ) => {}
                (
                    JSCalendarProperty::RecurrenceId,
                    Value::Str(text),
                    JSCalendarType::Event | JSCalendarType::Task,
                ) => {}
                (
                    JSCalendarProperty::RecurrenceIdTimeZone,
                    Value::Str(text),
                    JSCalendarType::Event | JSCalendarType::Task,
                ) => {}
                (
                    JSCalendarProperty::RecurrenceOverrides,
                    Value::Str(text),
                    JSCalendarType::Event | JSCalendarType::Task,
                ) => {}
                (
                    JSCalendarProperty::Rel,
                    Value::Str(text),
                    JSCalendarType::Event | JSCalendarType::Task,
                ) => {}
                (
                    JSCalendarProperty::RelatedTo,
                    Value::Str(text),
                    JSCalendarType::Event | JSCalendarType::Task,
                ) => {}
                (
                    JSCalendarProperty::Relation,
                    Value::Str(text),
                    JSCalendarType::Event | JSCalendarType::Task,
                ) => {}
                (
                    JSCalendarProperty::RelativeTo,
                    Value::Str(text),
                    JSCalendarType::Event | JSCalendarType::Task,
                ) => {}
                (
                    JSCalendarProperty::ReplyTo,
                    Value::Str(text),
                    JSCalendarType::Event | JSCalendarType::Task,
                ) => {}
                (
                    JSCalendarProperty::RequestStatus,
                    Value::Str(text),
                    JSCalendarType::Event | JSCalendarType::Task,
                ) => {}
                (
                    JSCalendarProperty::Roles,
                    Value::Str(text),
                    JSCalendarType::Event | JSCalendarType::Task,
                ) => {}
                (
                    JSCalendarProperty::Rscale,
                    Value::Str(text),
                    JSCalendarType::Event | JSCalendarType::Task,
                ) => {}
                (
                    JSCalendarProperty::SentBy,
                    Value::Str(text),
                    JSCalendarType::Event | JSCalendarType::Task,
                ) => {}
                (
                    JSCalendarProperty::ScheduleAgent,
                    Value::Str(text),
                    JSCalendarType::Event | JSCalendarType::Task,
                ) => {}
                (
                    JSCalendarProperty::ScheduleForceSend,
                    Value::Str(text),
                    JSCalendarType::Event | JSCalendarType::Task,
                ) => {}
                (
                    JSCalendarProperty::ScheduleSequence,
                    Value::Str(text),
                    JSCalendarType::Event | JSCalendarType::Task,
                ) => {}
                (
                    JSCalendarProperty::ScheduleStatus,
                    Value::Str(text),
                    JSCalendarType::Event | JSCalendarType::Task,
                ) => {}
                (
                    JSCalendarProperty::ScheduleUpdated,
                    Value::Str(text),
                    JSCalendarType::Event | JSCalendarType::Task,
                ) => {}
                (
                    JSCalendarProperty::SendTo,
                    Value::Str(text),
                    JSCalendarType::Event | JSCalendarType::Task,
                ) => {}
                (
                    JSCalendarProperty::Sequence,
                    Value::Str(text),
                    JSCalendarType::Event | JSCalendarType::Task,
                ) => {}
                (
                    JSCalendarProperty::ShowWithoutTime,
                    Value::Str(text),
                    JSCalendarType::Event | JSCalendarType::Task,
                ) => {}
                (
                    JSCalendarProperty::Size,
                    Value::Str(text),
                    JSCalendarType::Event | JSCalendarType::Task,
                ) => {}
                (
                    JSCalendarProperty::Skip,
                    Value::Str(text),
                    JSCalendarType::Event | JSCalendarType::Task,
                ) => {}
                (
                    JSCalendarProperty::Source,
                    Value::Str(text),
                    JSCalendarType::Event | JSCalendarType::Task,
                ) => {}
                (
                    JSCalendarProperty::Start,
                    Value::Str(text),
                    JSCalendarType::Event | JSCalendarType::Task,
                ) => {}
                (
                    JSCalendarProperty::Status,
                    Value::Str(text),
                    JSCalendarType::Event | JSCalendarType::Task,
                ) => {}
                (
                    JSCalendarProperty::TimeZone,
                    Value::Str(text),
                    JSCalendarType::Event | JSCalendarType::Task,
                ) => {}
                (
                    JSCalendarProperty::Title,
                    Value::Str(text),
                    JSCalendarType::Event | JSCalendarType::Task,
                ) => {}
                (
                    JSCalendarProperty::Trigger,
                    Value::Str(text),
                    JSCalendarType::Event | JSCalendarType::Task,
                ) => {}
                (
                    JSCalendarProperty::Uid,
                    Value::Str(text),
                    JSCalendarType::Event | JSCalendarType::Task,
                ) => {}
                (
                    JSCalendarProperty::Until,
                    Value::Str(text),
                    JSCalendarType::Event | JSCalendarType::Task,
                ) => {}
                (
                    JSCalendarProperty::Updated,
                    Value::Str(text),
                    JSCalendarType::Event | JSCalendarType::Task,
                ) => {}
                (
                    JSCalendarProperty::Uri,
                    Value::Str(text),
                    JSCalendarType::Event | JSCalendarType::Task,
                ) => {}
                (
                    JSCalendarProperty::UseDefaultAlerts,
                    Value::Str(text),
                    JSCalendarType::Event | JSCalendarType::Task,
                ) => {}
                (
                    JSCalendarProperty::UtcEnd,
                    Value::Str(text),
                    JSCalendarType::Event | JSCalendarType::Task,
                ) => {}
                (
                    JSCalendarProperty::UtcStart,
                    Value::Str(text),
                    JSCalendarType::Event | JSCalendarType::Task,
                ) => {}
                (
                    JSCalendarProperty::VirtualLocations,
                    Value::Str(text),
                    JSCalendarType::Event | JSCalendarType::Task,
                ) => {}
                (
                    JSCalendarProperty::When,
                    Value::Str(text),
                    JSCalendarType::Event | JSCalendarType::Task,
                ) => {}
                (
                    JSCalendarProperty::EndTimeZone,
                    Value::Str(text),
                    JSCalendarType::Event | JSCalendarType::Task,
                ) => {}
                (
                    JSCalendarProperty::MainLocationId,
                    Value::Str(text),
                    JSCalendarType::Event | JSCalendarType::Task,
                ) => {}
                (
                    JSCalendarProperty::OrganizerCalendarAddress,
                    Value::Str(text),
                    JSCalendarType::Event | JSCalendarType::Task,
                ) => {}
                (
                    JSCalendarProperty::RecurrenceRule,
                    Value::Str(text),
                    JSCalendarType::Event | JSCalendarType::Task,
                ) => {}
                (
                    JSCalendarProperty::DateTime(jscalendar_date_time),
                    Value::Str(text),
                    JSCalendarType::Event | JSCalendarType::Task,
                ) => {}
                (
                    JSCalendarProperty::LinkDisplay(jscalendar_link_display),
                    Value::Str(text),
                    JSCalendarType::Event | JSCalendarType::Task,
                ) => {}
                (
                    JSCalendarProperty::VirtualLocationFeature(jscalendar_virtual_location_feature),
                    Value::Str(text),
                    JSCalendarType::Event | JSCalendarType::Task,
                ) => {
                    todo!()
                }
                (
                    JSCalendarProperty::ParticipantRole(jscalendar_participant_role),
                    Value::Str(text),
                    JSCalendarType::Event | JSCalendarType::Task,
                ) => {}
                (
                    JSCalendarProperty::RelationValue(jscalendar_relation),
                    Value::Str(text),
                    JSCalendarType::Event | JSCalendarType::Task,
                ) => {}
                // Skip type and ICalComponent
                (JSCalendarProperty::Type | JSCalendarProperty::ICalComponent, _, _) => {}
                (property, value, _) => {
                    self.insert_jsprop(&[property.to_string().as_ref()], value);
                }
            }
        }

        self.apply_conversions(root_conversions);
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
    fn build(entries: &mut Map<'static, JSCalendarProperty, JSCalendarValue>) -> Option<Self> {
        for (property, value) in entries.as_mut_vec() {
            if let (Key::Property(JSCalendarProperty::ICalComponent), Value::Object(obj)) =
                (property, value)
            {
                let mut converted = ConvertedComponent {
                    name: ICalendarComponentType::Other(String::new()),
                    converted_props: Vec::new(),
                    converted_props_count: 0,
                    properties: Vec::new(),
                    components: Vec::new(),
                };
                let mut has_name = false;
                for (sub_property, value) in std::mem::take(obj.as_mut_vec()) {
                    match (sub_property, value) {
                        (
                            Key::Property(JSCalendarProperty::ConvertedProperties),
                            Value::Object(obj),
                        ) => {
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
                                                    JsonPointerItem::Number(n) => {
                                                        Key::Owned(n.to_string())
                                                    }
                                                    JsonPointerItem::Root
                                                    | JsonPointerItem::Wildcard => continue,
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
                                .unwrap_or_else(|| {
                                    ICalendarComponentType::Other(text.into_owned())
                                });
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

                if !converted.properties.is_empty()
                    || !converted.components.is_empty()
                    || has_name
                    || !converted.converted_props.is_empty()
                {
                    return Some(converted);
                }
            }
        }

        None
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
