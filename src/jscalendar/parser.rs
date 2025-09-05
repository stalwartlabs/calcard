/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

use crate::{
    common::{IanaParse, IanaString, LinkRelation},
    icalendar::{
        ICalendarDuration, ICalendarFrequency, ICalendarMethod, ICalendarMonth, ICalendarSkip,
        ICalendarWeekday,
    },
    jscalendar::{
        JSCalendar, JSCalendarAlertAction, JSCalendarDateTime, JSCalendarEventStatus,
        JSCalendarFreeBusyStatus, JSCalendarLinkDisplay, JSCalendarParticipantKind,
        JSCalendarParticipantRole, JSCalendarParticipationStatus, JSCalendarPrivacy,
        JSCalendarProgress, JSCalendarProperty, JSCalendarRelation, JSCalendarRelativeTo,
        JSCalendarScheduleAgent, JSCalendarType, JSCalendarValue, JSCalendarVirtualLocationFeature,
    },
};
use jmap_tools::{Element, JsonPointer, JsonPointerItem, Key, Value};
use mail_parser::DateTime;
use std::{borrow::Cow, str::FromStr};

impl<'x> JSCalendar<'x> {
    pub fn parse(json: &'x str) -> Result<Self, String> {
        Value::parse_json(json).map(JSCalendar)
    }
}

impl Element for JSCalendarValue {
    type Property = JSCalendarProperty;

    fn try_parse<P>(key: &Key<'_, Self::Property>, value: &str) -> Option<Self> {
        if let Key::Property(prop) = key {
            match prop.patch_or_prop() {
                JSCalendarProperty::Type => JSCalendarType::from_str(value)
                    .ok()
                    .map(JSCalendarValue::Type),
                JSCalendarProperty::Created
                | JSCalendarProperty::Updated
                | JSCalendarProperty::Acknowledged
                | JSCalendarProperty::ScheduleUpdated
                | JSCalendarProperty::When => DateTime::parse_rfc3339(value).map(|dt| {
                    JSCalendarValue::DateTime(JSCalendarDateTime {
                        timestamp: dt.to_timestamp_local(),
                        is_local: false,
                    })
                }),
                JSCalendarProperty::Due
                | JSCalendarProperty::RecurrenceId
                | JSCalendarProperty::Start
                | JSCalendarProperty::Until => DateTime::parse_rfc3339(value).map(|dt| {
                    JSCalendarValue::DateTime(JSCalendarDateTime {
                        timestamp: dt.to_timestamp_local(),
                        is_local: true,
                    })
                }),
                JSCalendarProperty::Duration
                | JSCalendarProperty::EstimatedDuration
                | JSCalendarProperty::Offset => {
                    ICalendarDuration::parse(value.as_bytes()).map(JSCalendarValue::Duration)
                }
                JSCalendarProperty::Action => JSCalendarAlertAction::from_str(value)
                    .ok()
                    .map(JSCalendarValue::AlertAction),
                JSCalendarProperty::FreeBusyStatus => JSCalendarFreeBusyStatus::from_str(value)
                    .ok()
                    .map(JSCalendarValue::FreeBusyStatus),
                JSCalendarProperty::Kind => JSCalendarParticipantKind::from_str(value)
                    .ok()
                    .map(JSCalendarValue::ParticipantKind),
                JSCalendarProperty::ParticipationStatus => {
                    JSCalendarParticipationStatus::from_str(value)
                        .ok()
                        .map(JSCalendarValue::ParticipationStatus)
                }
                JSCalendarProperty::Privacy => JSCalendarPrivacy::from_str(value)
                    .ok()
                    .map(JSCalendarValue::Privacy),
                JSCalendarProperty::Progress => JSCalendarProgress::from_str(value)
                    .ok()
                    .map(JSCalendarValue::Progress),
                JSCalendarProperty::RelativeTo => JSCalendarRelativeTo::from_str(value)
                    .ok()
                    .map(JSCalendarValue::RelativeTo),
                JSCalendarProperty::ScheduleAgent => JSCalendarScheduleAgent::from_str(value)
                    .ok()
                    .map(JSCalendarValue::ScheduleAgent),
                JSCalendarProperty::Status => JSCalendarEventStatus::from_str(value)
                    .ok()
                    .map(JSCalendarValue::EventStatus),
                JSCalendarProperty::Rel => {
                    LinkRelation::parse(value.as_bytes()).map(JSCalendarValue::LinkRelation)
                }
                JSCalendarProperty::Frequency => {
                    ICalendarFrequency::parse(value.as_bytes()).map(JSCalendarValue::Frequency)
                }
                JSCalendarProperty::FirstDayOfWeek | JSCalendarProperty::Day => {
                    ICalendarWeekday::parse(value.as_bytes()).map(JSCalendarValue::Weekday)
                }
                JSCalendarProperty::Skip => {
                    ICalendarSkip::parse(value.as_bytes()).map(JSCalendarValue::Skip)
                }
                JSCalendarProperty::ByMonth => {
                    ICalendarMonth::parse(value.as_bytes()).map(JSCalendarValue::Month)
                }
                JSCalendarProperty::Method => {
                    ICalendarMethod::parse(value.as_bytes()).map(JSCalendarValue::Method)
                }
                _ => None,
            }
        } else {
            None
        }
    }

    fn to_cow(&self) -> Cow<'static, str> {
        match self {
            JSCalendarValue::Type(v) => v.as_str().into(),
            JSCalendarValue::DateTime(v) => v.to_rfc3339().into(),
            JSCalendarValue::Duration(v) => v.to_string().into(),
            JSCalendarValue::AlertAction(v) => v.as_str().into(),
            JSCalendarValue::FreeBusyStatus(v) => v.as_str().into(),
            JSCalendarValue::ParticipantKind(v) => v.as_str().into(),
            JSCalendarValue::ParticipationStatus(v) => v.as_str().into(),
            JSCalendarValue::Privacy(v) => v.as_str().into(),
            JSCalendarValue::Progress(v) => v.as_str().into(),
            JSCalendarValue::RelativeTo(v) => v.as_str().into(),
            JSCalendarValue::ScheduleAgent(v) => v.as_str().into(),
            JSCalendarValue::EventStatus(v) => v.as_str().into(),
            JSCalendarValue::LinkRelation(v) => v.as_str().into(),
            JSCalendarValue::Frequency(v) => v.as_js_str().into(),
            JSCalendarValue::CalendarScale(v) => v.as_js_str().into(),
            JSCalendarValue::Skip(v) => v.as_js_str().into(),
            JSCalendarValue::Weekday(v) => v.as_js_str().into(),
            JSCalendarValue::Month(v) => v.to_string().into(),
            JSCalendarValue::Method(v) => v.as_js_str().into(),
        }
    }
}

impl jmap_tools::Property for JSCalendarProperty {
    fn try_parse(key: Option<&Key<'_, Self>>, value: &str) -> Option<Self> {
        let Some(Key::Property(key)) = key else {
            return JSCalendarProperty::from_str(value).ok();
        };

        match key.patch_or_prop() {
            JSCalendarProperty::RecurrenceOverrides => DateTime::parse_rfc3339(value).map(|dt| {
                JSCalendarProperty::DateTime(JSCalendarDateTime {
                    timestamp: dt.to_timestamp_local(),
                    is_local: true,
                })
            }),
            JSCalendarProperty::Display => JSCalendarLinkDisplay::from_str(value)
                .ok()
                .map(JSCalendarProperty::LinkDisplay),
            JSCalendarProperty::Features => JSCalendarVirtualLocationFeature::from_str(value)
                .ok()
                .map(JSCalendarProperty::VirtualLocationFeature),
            JSCalendarProperty::Roles => JSCalendarParticipantRole::from_str(value)
                .ok()
                .map(JSCalendarProperty::ParticipantRole),
            JSCalendarProperty::Relation => JSCalendarRelation::from_str(value)
                .ok()
                .map(JSCalendarProperty::RelationValue),
            JSCalendarProperty::ConvertedProperties => {
                JSCalendarProperty::Pointer(JsonPointer::parse(value)).into()
            }
            JSCalendarProperty::DateTime(_) if value.contains('/') => {
                JSCalendarProperty::Pointer(JsonPointer::parse(value)).into()
            }
            _ => JSCalendarProperty::from_str(value).ok(),
        }
    }

    fn to_cow(&self) -> Cow<'static, str> {
        self.to_string()
    }
}

impl JSCalendarProperty {
    fn patch_or_prop(&self) -> &JSCalendarProperty {
        if let JSCalendarProperty::Pointer(ptr) = self
            && let Some(JsonPointerItem::Key(Key::Property(prop))) = ptr.last()
        {
            prop
        } else {
            self
        }
    }
}
