/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

use crate::{
    icalendar::ICalendarDuration,
    jscalendar::{
        JSCalendar, JSCalendarAlertAction, JSCalendarDateTime, JSCalendarEventStatus,
        JSCalendarFreeBusyStatus, JSCalendarLinkDisplay, JSCalendarParticipantKind,
        JSCalendarParticipantRole, JSCalendarParticipationStatus, JSCalendarPrivacy,
        JSCalendarProgress, JSCalendarProperty, JSCalendarRelation, JSCalendarRelativeTo,
        JSCalendarScheduleAgent, JSCalendarType, JSCalendarValue, JSCalendarVirtualLocationFeature,
    },
};
use chrono::DateTime;
use jmap_tools::{Element, Key, Value};
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
            match prop {
                JSCalendarProperty::Type => JSCalendarType::from_str(value)
                    .ok()
                    .map(JSCalendarValue::Type),
                JSCalendarProperty::Created
                | JSCalendarProperty::Updated
                | JSCalendarProperty::Acknowledged
                | JSCalendarProperty::ScheduleUpdated
                | JSCalendarProperty::When => DateTime::parse_from_rfc3339(value)
                    .map(|dt| {
                        JSCalendarValue::DateTime(JSCalendarDateTime {
                            timestamp: dt.timestamp(),
                            is_local: false,
                        })
                    })
                    .ok(),
                JSCalendarProperty::Due
                | JSCalendarProperty::RecurrenceId
                | JSCalendarProperty::Start
                | JSCalendarProperty::Until => DateTime::parse_from_rfc3339(value)
                    .map(|dt| {
                        JSCalendarValue::DateTime(JSCalendarDateTime {
                            timestamp: dt.timestamp(),
                            is_local: true,
                        })
                    })
                    .ok(),
                JSCalendarProperty::Duration
                | JSCalendarProperty::EstimatedDuration
                | JSCalendarProperty::Offset => ICalendarDuration::try_from(value.as_bytes())
                    .ok()
                    .map(JSCalendarValue::Duration),
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

                _ => None,
            }
        } else {
            None
        }
    }

    fn to_cow(&self) -> Cow<'_, str> {
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
        }
    }
}

impl jmap_tools::Property for JSCalendarProperty {
    fn try_parse(key: Option<&Key<'_, Self>>, value: &str) -> Option<Self> {
        match key {
            Some(Key::Property(JSCalendarProperty::RecurrenceOverrides)) => {
                DateTime::parse_from_rfc3339(value)
                    .map(|dt| {
                        JSCalendarProperty::DateTime(JSCalendarDateTime {
                            timestamp: dt.timestamp(),
                            is_local: false,
                        })
                    })
                    .ok()
            }
            Some(Key::Property(JSCalendarProperty::Display)) => {
                JSCalendarLinkDisplay::from_str(value)
                    .ok()
                    .map(JSCalendarProperty::LinkDisplay)
            }
            Some(Key::Property(JSCalendarProperty::Features)) => {
                JSCalendarVirtualLocationFeature::from_str(value)
                    .ok()
                    .map(JSCalendarProperty::VirtualLocationFeature)
            }
            Some(Key::Property(JSCalendarProperty::Roles)) => {
                JSCalendarParticipantRole::from_str(value)
                    .ok()
                    .map(JSCalendarProperty::ParticipantRole)
            }
            Some(Key::Property(JSCalendarProperty::Relation)) => {
                JSCalendarRelation::from_str(value)
                    .ok()
                    .map(JSCalendarProperty::RelationValue)
            }
            _ => JSCalendarProperty::from_str(value).ok(),
        }
    }

    fn to_cow(&self) -> Cow<'static, str> {
        self.to_string()
    }
}
