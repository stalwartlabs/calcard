/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

use crate::{
    common::PartialDateTime,
    icalendar::{
        ICalendarAction, ICalendarEntry, ICalendarParameter, ICalendarProperty, ICalendarValue, Uri,
    },
    jscalendar::{
        JSCalendarAlertAction, JSCalendarProperty, JSCalendarType, JSCalendarValue, export::State,
    },
};
use jmap_tools::{JsonPointer, JsonPointerItem, Key, Map, Value};

impl State<'_> {
    pub(crate) fn entries_from_jscalendar(
        &mut self,
        typ: JSCalendarType,
        entries: Map<'static, JSCalendarProperty, JSCalendarValue>,
    ) {
        let mut properties = entries.into_vec();

        for (property, value) in &mut properties {
            if let (Key::Property(JSCalendarProperty::ICalComponent), Value::Object(obj)) =
                (property, value)
            {
                for (sub_property, value) in std::mem::take(obj.as_mut_vec()) {
                    match sub_property {
                        Key::Property(JSCalendarProperty::ConvertedProperties) => {
                            for (key, value) in value.into_expanded_object() {
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

                                self.converted_props.push((keys, value));
                            }
                        }
                        Key::Property(JSCalendarProperty::Properties) => {
                            if let Some(value) = value.into_array() {
                                self.import_properties(value);
                            }
                        }
                        Key::Property(JSCalendarProperty::Components) => {
                            let todo = "implement";
                        }
                        _ => {
                            self.insert_jsprop(
                                &[
                                    JSCalendarProperty::ICalComponent.to_string().as_ref(),
                                    sub_property.to_string().as_ref(),
                                ],
                                value,
                            );
                        }
                    }
                }

                if !self.converted_props.is_empty() {
                    self.converted_props.sort_unstable_by(|a, b| a.0.cmp(&b.0));
                }
            }
        }

        for (key, value) in properties {
            let Key::Property(property) = key else {
                self.insert_jsprop(&[key.to_string().as_ref()], value);
                continue;
            };

            match (&property, value, typ) {
                (
                    JSCalendarProperty::Acknowledged,
                    Value::Element(JSCalendarValue::DateTime(dt)),
                    JSCalendarType::Alert,
                ) => {
                    self.insert_ical(
                        &[property],
                        ICalendarEntry::new(ICalendarProperty::Acknowledged).with_value(
                            ICalendarValue::PartialDateTime(Box::new(
                                PartialDateTime::from_utc_timestamp(dt.timestamp),
                            )),
                        ),
                    );
                }
                (
                    JSCalendarProperty::Action,
                    Value::Element(JSCalendarValue::AlertAction(action)),
                    JSCalendarType::Alert,
                ) => {
                    self.insert_ical(
                        &[property],
                        ICalendarEntry::new(ICalendarProperty::Action).with_value(
                            ICalendarValue::Action(match action {
                                JSCalendarAlertAction::Display => ICalendarAction::Display,
                                JSCalendarAlertAction::Email => ICalendarAction::Email,
                            }),
                        ),
                    );
                }
                (
                    JSCalendarProperty::Links,
                    Value::Object(obj),
                    JSCalendarType::Event
                    | JSCalendarType::Task
                    | JSCalendarType::Location
                    | JSCalendarType::VirtualLocation
                    | JSCalendarType::Participant,
                ) => {
                    for (name, value) in obj.into_vec() {
                        let mut entry = ICalendarEntry::new(ICalendarProperty::Link);

                        for (sub_property, value) in value.into_expanded_object() {
                            match (sub_property, value) {
                                (Key::Property(JSCalendarProperty::Type), _) => {}
                                (Key::Property(JSCalendarProperty::Href), Value::Str(text)) => {
                                    entry
                                        .values
                                        .push(ICalendarValue::Uri(Uri::parse(text.into_owned())));
                                }
                                (
                                    Key::Property(JSCalendarProperty::ContentType),
                                    Value::Str(text),
                                ) => {
                                    /*entry
                                    .params
                                    .push(ICalendarParameter::Fmttype(text.into_owned()));*/
                                }
                                (
                                    Key::Property(JSCalendarProperty::Size),
                                    Value::Number(number),
                                ) => {
                                    /*entry
                                    .params
                                    .push(ICalendarParameter::Size(number.cast_to_u64()));*/
                                }
                                (Key::Property(JSCalendarProperty::Rel), Value::Str(text)) => {
                                    /*entry
                                    .params
                                    .push(ICalendarParameter::Linkrel(text.into_owned()));*/
                                }
                                (Key::Property(JSCalendarProperty::Display), Value::Str(text)) => {}
                                (Key::Property(JSCalendarProperty::Title), Value::Str(text)) => {}
                                /*Key::Property(JSCalendarProperty::Pref) => {
                                    if let Some(pref) = value.as_u64() {
                                        entry.params.push(ICalendarParameter::Pref(pref as u32));
                                    }
                                }
                                Key::Property(JSCalendarProperty::ListAs) => {
                                    if let Some(index) = value.as_u64() {
                                        entry.params.push(ICalendarParameter::Index(index as u32));
                                    }
                                }
                                Key::Property(JSCalendarProperty::MediaType) => {
                                    if let Some(text) = value.into_string() {
                                        entry.params.push(ICalendarParameter::Mediatype(text));
                                    }
                                }
                                Key::Property(JSCalendarProperty::Label) => {
                                    if let Some(text) = value.into_string() {
                                        entry.values.push(ICalendarValue::Text(text));
                                    }
                                }
                                Key::Property(JSCalendarProperty::Contexts) => {
                                    if let Some(types) = convert_types(value, true) {
                                        entry.params.push(ICalendarParameter::Type(types));
                                    }
                                }*/
                                (sub_property, value) => {
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

                        /* self.insert_ical(
                            &[JSCalendarProperty::Links],
                            entry.with_param(ICalendarParameter::Jsid(name.into_string())),
                        );*/
                    }
                }
                (
                    JSCalendarProperty::Alerts,
                    Value::Str(text),
                    JSCalendarType::Event | JSCalendarType::Task,
                ) => {}
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
                    JSCalendarProperty::Participants,
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
    }
}
