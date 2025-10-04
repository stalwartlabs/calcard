/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

use crate::{
    common::{Data, IanaString, IanaType},
    icalendar::{
        ICalendarDisplayType, ICalendarEntry, ICalendarFeatureType, ICalendarParameter,
        ICalendarParameterName, ICalendarParameterValue, ICalendarParticipationRole,
        ICalendarParticipationStatus, ICalendarProperty, ICalendarRelated, ICalendarUserTypes,
        ICalendarValue, ICalendarValueType, Uri,
    },
    jscalendar::{
        JSCalendarId, JSCalendarLinkDisplay, JSCalendarParticipantKind, JSCalendarParticipantRole,
        JSCalendarParticipationStatus, JSCalendarProperty, JSCalendarRelativeTo, JSCalendarValue,
        JSCalendarVirtualLocationFeature, import::ICalendarParams,
    },
};
use ahash::AHashMap;
use jmap_tools::{Key, Map, Value};

pub(super) trait ExtractParams {
    fn extract_params(
        &mut self,
        entry: &mut ICalendarEntry,
        extract: &[ICalendarParameterName],
    ) -> Option<String>;
}

impl<I: JSCalendarId> ExtractParams
    for AHashMap<
        Key<'static, JSCalendarProperty<I>>,
        Value<'static, JSCalendarProperty<I>, JSCalendarValue<I>>,
    >
{
    fn extract_params(
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
                        self.insert(Key::Property(JSCalendarProperty::Name), Value::Str(text));
                    }
                }
                ICalendarParameterName::Cutype => {
                    self.insert(
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
                        self.insert(
                            Key::Property(JSCalendarProperty::DelegatedFrom),
                            Value::Str(text),
                        );
                    }
                }
                ICalendarParameterName::DelegatedTo => {
                    if let Some(text) = param.value.into_text() {
                        self.insert(
                            Key::Property(JSCalendarProperty::DelegatedTo),
                            Value::Str(text),
                        );
                    }
                }
                ICalendarParameterName::Email => {
                    if let Some(text) = param.value.into_text() {
                        self.insert(Key::Property(JSCalendarProperty::Email), Value::Str(text));
                    }
                }
                ICalendarParameterName::Rsvp => {
                    if let Some(boolean) = param.value.as_bool() {
                        self.insert(
                            Key::Property(JSCalendarProperty::ExpectReply),
                            Value::Bool(boolean),
                        );
                    }
                }
                ICalendarParameterName::Member => {
                    if let Some(text) = param.value.into_text() {
                        self.entry(Key::Property(JSCalendarProperty::MemberOf))
                            .or_insert_with(Value::new_object)
                            .as_object_mut()
                            .unwrap()
                            .insert(Key::from(text), Value::Bool(true));
                    }
                }
                ICalendarParameterName::Partstat => {
                    self.insert(
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
                    self.entry(Key::Property(JSCalendarProperty::Roles))
                        .or_insert_with(Value::new_object)
                        .as_object_mut()
                        .unwrap()
                        .insert(role, Value::Bool(true));
                }
                ICalendarParameterName::SentBy => {
                    if let Some(text) = param.value.into_text() {
                        self.insert(Key::Property(JSCalendarProperty::SentBy), Value::Str(text));
                    }
                }
                ICalendarParameterName::Fmttype => {
                    if let Some(text) = param.value.into_text() {
                        self.insert(
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
                        self.insert(
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
                        self.insert(
                            Key::Property(JSCalendarProperty::Size),
                            Value::Number(number.into()),
                        );
                    } else {
                        entry.params.push(ICalendarParameter::size(param.value));
                    }
                }
                ICalendarParameterName::Linkrel => match param.value {
                    ICalendarParameterValue::Linkrel(linkrel) => {
                        self.insert(
                            Key::Property(JSCalendarProperty::Rel),
                            Value::Element(JSCalendarValue::LinkRelation(linkrel)),
                        );
                    }
                    ICalendarParameterValue::Text(value) => {
                        self.insert(
                            Key::Property(JSCalendarProperty::Rel),
                            Value::Str(value.into()),
                        );
                    }
                    value => {
                        entry.params.push(ICalendarParameter::linkrel(value));
                    }
                },
                ICalendarParameterName::Related => {
                    if let ICalendarParameterValue::Related(related) = param.value {
                        self.insert(
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
                    self.entry(Key::Property(JSCalendarProperty::Features))
                        .or_insert_with(Value::new_object)
                        .as_object_mut()
                        .unwrap()
                        .insert(feature, Value::Bool(true));
                }
                ICalendarParameterName::Language => {
                    if let Some(text) = param.value.into_text() {
                        self.insert(Key::Property(JSCalendarProperty::Locale), Value::Str(text));
                    }
                }
                ICalendarParameterName::Display => {
                    let display = match param.value {
                        ICalendarParameterValue::Display(value) => {
                            Key::Property(JSCalendarProperty::LinkDisplay(match value {
                                ICalendarDisplayType::Badge => JSCalendarLinkDisplay::Badge,
                                ICalendarDisplayType::Graphic => JSCalendarLinkDisplay::Graphic,
                                ICalendarDisplayType::Fullsize => JSCalendarLinkDisplay::Fullsize,
                                ICalendarDisplayType::Thumbnail => JSCalendarLinkDisplay::Thumbnail,
                            }))
                        }
                        ICalendarParameterValue::Text(value) => Key::Owned(value),
                        value => {
                            entry.params.push(ICalendarParameter::display(value));
                            continue;
                        }
                    };
                    self.entry(Key::Property(JSCalendarProperty::Display))
                        .or_insert_with(Value::new_object)
                        .as_object_mut()
                        .unwrap()
                        .insert(display, Value::Bool(true));
                }
                ICalendarParameterName::Jsid => {
                    jsid = param.value.into_text().map(|v| v.into_owned());
                }
                ICalendarParameterName::Range
                | ICalendarParameterName::Reltype
                | ICalendarParameterName::Dir
                | ICalendarParameterName::Fbtype
                | ICalendarParameterName::ScheduleAgent
                | ICalendarParameterName::ScheduleForceSend
                | ICalendarParameterName::ScheduleStatus
                | ICalendarParameterName::Tzid
                | ICalendarParameterName::Value
                | ICalendarParameterName::Filename
                | ICalendarParameterName::ManagedId
                | ICalendarParameterName::Order
                | ICalendarParameterName::Schema
                | ICalendarParameterName::Derived
                | ICalendarParameterName::Gap
                | ICalendarParameterName::Jsptr
                | ICalendarParameterName::Other(_)
                | ICalendarParameterName::Altrep => {}
            }
        }

        jsid
    }
}

impl<I: JSCalendarId> ICalendarParams<I> {
    pub(super) fn into_jscalendar_value(
        self,
    ) -> Option<Map<'static, JSCalendarProperty<I>, JSCalendarValue<I>>> {
        if !self.0.is_empty() {
            let mut obj = Map::from(Vec::with_capacity(self.0.len()));

            for (param, value) in self.0 {
                let value = if value.len() > 1 {
                    Value::Array(value)
                } else {
                    value.into_iter().next().unwrap()
                };
                obj.insert_unchecked(Key::from(param.into_string()), value);
            }
            Some(obj)
        } else {
            None
        }
    }
}

impl ICalendarValue {
    pub(super) fn into_jscalendar_value<I: JSCalendarId>(
        self,
        value_type: Option<&IanaType<ICalendarValueType, String>>,
    ) -> Value<'static, JSCalendarProperty<I>, JSCalendarValue<I>> {
        match self {
            ICalendarValue::Text(v) => Value::Str(v.into()),
            ICalendarValue::Integer(v) => Value::Number(v.into()),
            ICalendarValue::Float(v) => Value::Number(v.into()),
            ICalendarValue::Boolean(v) => Value::Bool(v),
            ICalendarValue::PartialDateTime(v) => {
                let mut out = String::new();
                let _ = v.format_as_ical(
                    &mut out,
                    value_type.and_then(|v| v.iana()).unwrap_or(
                        match (v.has_date(), v.has_time(), v.has_zone()) {
                            (true, true, _) => &ICalendarValueType::DateTime,
                            (true, _, _) => &ICalendarValueType::Date,
                            (_, true, _) => &ICalendarValueType::Time,
                            (_, _, true) => &ICalendarValueType::UtcOffset,
                            _ => &ICalendarValueType::Text,
                        },
                    ),
                );
                Value::Str(out.into())
            }
            ICalendarValue::Binary(v) => Value::Str(
                Uri::Data(Data {
                    content_type: None,
                    data: v,
                })
                .into_unwrapped_string()
                .into(),
            ),
            ICalendarValue::Uri(v) => Value::Str(v.into_unwrapped_string().into()),
            ICalendarValue::Duration(v) => Value::Str(v.to_string().into()),
            ICalendarValue::RecurrenceRule(v) => Value::Str(v.to_string().into()),
            ICalendarValue::Period(v) => Value::Str(v.to_string().into()),
            ICalendarValue::CalendarScale(v) => Value::Str(v.as_str().into()),
            ICalendarValue::Method(v) => Value::Str(v.as_str().into()),
            ICalendarValue::Classification(v) => Value::Str(v.as_str().into()),
            ICalendarValue::Status(v) => Value::Str(v.as_str().into()),
            ICalendarValue::Transparency(v) => Value::Str(v.as_str().into()),
            ICalendarValue::Action(v) => Value::Str(v.as_str().into()),
            ICalendarValue::BusyType(v) => Value::Str(v.as_str().into()),
            ICalendarValue::ParticipantType(v) => Value::Str(v.as_str().into()),
            ICalendarValue::ResourceType(v) => Value::Str(v.as_str().into()),
            ICalendarValue::Proximity(v) => Value::Str(v.as_str().into()),
        }
    }
}
