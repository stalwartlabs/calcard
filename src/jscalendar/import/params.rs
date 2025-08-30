/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

use crate::{
    common::IanaString,
    icalendar::{
        ICalendarEntry, ICalendarFeatureType, ICalendarParameter, ICalendarParameterName,
        ICalendarParameterValue, ICalendarParticipationRole, ICalendarParticipationStatus,
        ICalendarProperty, ICalendarRelated, ICalendarUserTypes,
    },
    jscalendar::{
        JSCalendarParticipantKind, JSCalendarParticipantRole, JSCalendarParticipationStatus,
        JSCalendarProperty, JSCalendarRelativeTo, JSCalendarValue,
        JSCalendarVirtualLocationFeature, import::State,
    },
};
use jmap_tools::{Key, Map, Value};

impl State {
    pub(super) fn extract_params(
        &mut self,
        entry: &mut ICalendarEntry,
        extract: &[ICalendarParameterName],
    ) -> Option<String> {
        let mut jsid = None;

        for param in std::mem::take(&mut entry.params) {
            if !extract.contains(&param.name) {
                entry.params.push(param);
                continue;
            }

            match param.name {
                ICalendarParameterName::Cn => {
                    if let Some(text) = param.value.into_text() {
                        self.entries
                            .insert(Key::Property(JSCalendarProperty::Name), Value::Str(text));
                    }
                }
                ICalendarParameterName::Cutype => {
                    self.entries.insert(
                        Key::Property(JSCalendarProperty::Kind),
                        match param.value {
                            ICalendarParameterValue::Text(value) => Value::Str(value.into()),
                            ICalendarParameterValue::Cutype(value) => match value {
                                ICalendarUserTypes::Individual => {
                                    Value::Element(JSCalendarValue::ParticipantKind(
                                        JSCalendarParticipantKind::Individual,
                                    ))
                                }
                                ICalendarUserTypes::Group => {
                                    Value::Element(JSCalendarValue::ParticipantKind(
                                        JSCalendarParticipantKind::Group,
                                    ))
                                }
                                ICalendarUserTypes::Resource => {
                                    Value::Element(JSCalendarValue::ParticipantKind(
                                        JSCalendarParticipantKind::Resource,
                                    ))
                                }
                                ICalendarUserTypes::Room => {
                                    Value::Element(JSCalendarValue::ParticipantKind(
                                        JSCalendarParticipantKind::Location,
                                    ))
                                }
                                ICalendarUserTypes::Unknown => {
                                    Value::Str(value.as_str().to_lowercase().into())
                                }
                            },
                            _ => continue,
                        },
                    );
                }
                ICalendarParameterName::DelegatedFrom => {
                    if let Some(text) = param.value.into_text() {
                        self.entries.insert(
                            Key::Property(JSCalendarProperty::DelegatedFrom),
                            Value::Str(text),
                        );
                    }
                }
                ICalendarParameterName::DelegatedTo => {
                    if let Some(text) = param.value.into_text() {
                        self.entries.insert(
                            Key::Property(JSCalendarProperty::DelegatedTo),
                            Value::Str(text),
                        );
                    }
                }
                ICalendarParameterName::Email => {
                    if let Some(text) = param.value.into_text() {
                        self.entries
                            .insert(Key::Property(JSCalendarProperty::Email), Value::Str(text));
                    }
                }
                ICalendarParameterName::Rsvp => {
                    if let Some(boolean) = param.value.as_bool() {
                        self.entries.insert(
                            Key::Property(JSCalendarProperty::ExpectReply),
                            Value::Bool(boolean),
                        );
                    }
                }
                ICalendarParameterName::Member => {
                    if let Some(text) = param.value.into_text() {
                        self.entries
                            .entry(Key::Property(JSCalendarProperty::MemberOf))
                            .or_insert_with(Value::new_object)
                            .as_object_mut()
                            .unwrap()
                            .insert(Key::from(text), Value::Bool(true));
                    }
                }
                ICalendarParameterName::Partstat => {
                    self.entries.insert(
                        Key::Property(JSCalendarProperty::ParticipationStatus),
                        match param.value {
                            ICalendarParameterValue::Partstat(value) => {
                                Value::Element(JSCalendarValue::ParticipationStatus(match value {
                                    ICalendarParticipationStatus::NeedsAction => {
                                        JSCalendarParticipationStatus::NeedsAction
                                    }
                                    ICalendarParticipationStatus::Declined => {
                                        JSCalendarParticipationStatus::Declined
                                    }
                                    ICalendarParticipationStatus::Tentative => {
                                        JSCalendarParticipationStatus::Tentative
                                    }
                                    ICalendarParticipationStatus::Delegated => {
                                        JSCalendarParticipationStatus::Delegated
                                    }
                                    ICalendarParticipationStatus::Completed
                                    | ICalendarParticipationStatus::InProcess
                                    | ICalendarParticipationStatus::Failed
                                    | ICalendarParticipationStatus::Accepted => {
                                        JSCalendarParticipationStatus::Accepted
                                    }
                                }))
                            }
                            ICalendarParameterValue::Text(value) => Value::Str(value.into()),
                            _ => continue,
                        },
                    );
                }
                ICalendarParameterName::Role => {
                    let role = match param.value {
                        ICalendarParameterValue::Role(value) => {
                            Key::Property(JSCalendarProperty::ParticipantRole(match value {
                                ICalendarParticipationRole::Chair => {
                                    JSCalendarParticipantRole::Chair
                                }
                                ICalendarParticipationRole::ReqParticipant => {
                                    JSCalendarParticipantRole::Required
                                }
                                ICalendarParticipationRole::OptParticipant => {
                                    JSCalendarParticipantRole::Optional
                                }
                                ICalendarParticipationRole::NonParticipant => {
                                    JSCalendarParticipantRole::Informational
                                }
                            }))
                        }
                        ICalendarParameterValue::Text(value) => Key::Owned(value),
                        _ => continue,
                    };
                    self.entries
                        .entry(Key::Property(JSCalendarProperty::Roles))
                        .or_insert_with(Value::new_object)
                        .as_object_mut()
                        .unwrap()
                        .insert(role, Value::Bool(true));
                }
                ICalendarParameterName::SentBy => {
                    if let Some(text) = param.value.into_text() {
                        self.entries
                            .insert(Key::Property(JSCalendarProperty::SentBy), Value::Str(text));
                    }
                }
                ICalendarParameterName::Fmttype => {
                    if let Some(text) = param.value.into_text() {
                        self.entries.insert(
                            Key::Property(
                                if matches!(entry.name, ICalendarProperty::StyledDescription) {
                                    JSCalendarProperty::DescriptionContentType
                                } else {
                                    JSCalendarProperty::ContentType
                                },
                            ),
                            Value::Str(text),
                        );
                    }
                }
                ICalendarParameterName::Label => {
                    if let Some(text) = param.value.into_text() {
                        self.entries.insert(
                            Key::Property(if matches!(entry.name, ICalendarProperty::Conference) {
                                JSCalendarProperty::Name
                            } else {
                                JSCalendarProperty::Title
                            }),
                            Value::Str(text),
                        );
                    }
                }
                ICalendarParameterName::Size => {
                    if let Some(number) = param.value.as_integer() {
                        self.entries.insert(
                            Key::Property(JSCalendarProperty::Size),
                            Value::Number(number.into()),
                        );
                    } else {
                        entry.params.push(ICalendarParameter::size(param.value));
                    }
                }
                ICalendarParameterName::Linkrel => match param.value {
                    ICalendarParameterValue::Linkrel(linkrel) => {
                        self.entries.insert(
                            Key::Property(JSCalendarProperty::Rel),
                            Value::Element(JSCalendarValue::LinkRelation(linkrel)),
                        );
                    }
                    value => {
                        entry.params.push(ICalendarParameter::linkrel(value));
                    }
                },
                ICalendarParameterName::Related => {
                    if let ICalendarParameterValue::Related(related) = param.value {
                        self.entries.insert(
                            Key::Property(JSCalendarProperty::RelativeTo),
                            Value::Element(JSCalendarValue::RelativeTo(match related {
                                ICalendarRelated::Start => JSCalendarRelativeTo::Start,
                                ICalendarRelated::End => JSCalendarRelativeTo::End,
                            })),
                        );
                    } else {
                        entry.params.push(ICalendarParameter::related(param.value));
                    }
                }
                ICalendarParameterName::Feature => {
                    let feature = match param.value {
                        ICalendarParameterValue::Feature(value) => {
                            Key::Property(JSCalendarProperty::VirtualLocationFeature(match value {
                                ICalendarFeatureType::Audio => {
                                    JSCalendarVirtualLocationFeature::Audio
                                }
                                ICalendarFeatureType::Chat => {
                                    JSCalendarVirtualLocationFeature::Chat
                                }
                                ICalendarFeatureType::Feed => {
                                    JSCalendarVirtualLocationFeature::Feed
                                }
                                ICalendarFeatureType::Moderator => {
                                    JSCalendarVirtualLocationFeature::Moderator
                                }
                                ICalendarFeatureType::Phone => {
                                    JSCalendarVirtualLocationFeature::Phone
                                }
                                ICalendarFeatureType::Screen => {
                                    JSCalendarVirtualLocationFeature::Screen
                                }
                                ICalendarFeatureType::Video => {
                                    JSCalendarVirtualLocationFeature::Video
                                }
                            }))
                        }
                        ICalendarParameterValue::Text(value) => Key::Owned(value),
                        _ => continue,
                    };
                    self.entries
                        .entry(Key::Property(JSCalendarProperty::Features))
                        .or_insert_with(Value::new_object)
                        .as_object_mut()
                        .unwrap()
                        .insert(feature, Value::Bool(true));
                }
                ICalendarParameterName::Language => {
                    if let Some(text) = param.value.into_text() {
                        self.entries
                            .insert(Key::Property(JSCalendarProperty::Locale), Value::Str(text));
                    }
                }
                ICalendarParameterName::Range => {
                    self.entries.insert(
                        Key::Property(JSCalendarProperty::ThisAndFuture),
                        Value::Bool(true),
                    );
                }

                ICalendarParameterName::Dir => {}
                ICalendarParameterName::Fbtype => {}
                ICalendarParameterName::Reltype => {}
                ICalendarParameterName::ScheduleAgent => {}
                ICalendarParameterName::ScheduleForceSend => {}
                ICalendarParameterName::ScheduleStatus => {}
                ICalendarParameterName::Tzid => {}
                ICalendarParameterName::Value => {}
                ICalendarParameterName::Display => {}

                ICalendarParameterName::Filename => {}
                ICalendarParameterName::ManagedId => {}
                ICalendarParameterName::Order => {}
                ICalendarParameterName::Schema => {}
                ICalendarParameterName::Derived => {}
                ICalendarParameterName::Gap => {}
                ICalendarParameterName::Jsptr => {}
                ICalendarParameterName::Other(_) => {}
                ICalendarParameterName::Altrep => {}
                ICalendarParameterName::Jsid => {
                    let todo = "remove todos";
                    jsid = param.value.into_text().map(|v| v.into_owned());
                }
            }
        }

        jsid
    }
}
