/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

use crate::common::ArchivedLinkRelation;

use super::*;

impl ArchivedICalendarProperty {
    // Returns the default value type and whether the property is multi-valued.
    pub(crate) fn default_types(&self) -> (ArchivedValueType, ValueSeparator) {
        match self {
            ArchivedICalendarProperty::Calscale => {
                (ArchivedValueType::CalendarScale, ValueSeparator::None)
            }
            ArchivedICalendarProperty::Method => (ArchivedValueType::Method, ValueSeparator::None),
            ArchivedICalendarProperty::Prodid => (
                ArchivedValueType::Ical(ArchivedICalendarValueType::Text),
                ValueSeparator::None,
            ),
            ArchivedICalendarProperty::Version => (
                ArchivedValueType::Ical(ArchivedICalendarValueType::Text),
                ValueSeparator::None,
            ),
            ArchivedICalendarProperty::Attach => (
                ArchivedValueType::Ical(ArchivedICalendarValueType::Uri),
                ValueSeparator::None,
            ),
            ArchivedICalendarProperty::Categories => (
                ArchivedValueType::Ical(ArchivedICalendarValueType::Text),
                ValueSeparator::Comma,
            ),
            ArchivedICalendarProperty::Class => {
                (ArchivedValueType::Classification, ValueSeparator::None)
            }
            ArchivedICalendarProperty::Comment => (
                ArchivedValueType::Ical(ArchivedICalendarValueType::Text),
                ValueSeparator::None,
            ),
            ArchivedICalendarProperty::Description => (
                ArchivedValueType::Ical(ArchivedICalendarValueType::Text),
                ValueSeparator::None,
            ),
            ArchivedICalendarProperty::Geo => (
                ArchivedValueType::Ical(ArchivedICalendarValueType::Float),
                ValueSeparator::Semicolon,
            ),
            ArchivedICalendarProperty::Location => (
                ArchivedValueType::Ical(ArchivedICalendarValueType::Text),
                ValueSeparator::None,
            ),
            ArchivedICalendarProperty::PercentComplete => (
                ArchivedValueType::Ical(ArchivedICalendarValueType::Integer),
                ValueSeparator::None,
            ),
            ArchivedICalendarProperty::Priority => (
                ArchivedValueType::Ical(ArchivedICalendarValueType::Integer),
                ValueSeparator::None,
            ),
            ArchivedICalendarProperty::Resources => (
                ArchivedValueType::Ical(ArchivedICalendarValueType::Text),
                ValueSeparator::Comma,
            ),
            ArchivedICalendarProperty::Status => (ArchivedValueType::Status, ValueSeparator::None),
            ArchivedICalendarProperty::Summary => (
                ArchivedValueType::Ical(ArchivedICalendarValueType::Text),
                ValueSeparator::None,
            ),
            ArchivedICalendarProperty::Completed => (
                ArchivedValueType::Ical(ArchivedICalendarValueType::DateTime),
                ValueSeparator::None,
            ),
            ArchivedICalendarProperty::Dtend => (
                ArchivedValueType::Ical(ArchivedICalendarValueType::DateTime),
                ValueSeparator::None,
            ),
            ArchivedICalendarProperty::Due => (
                ArchivedValueType::Ical(ArchivedICalendarValueType::DateTime),
                ValueSeparator::None,
            ),
            ArchivedICalendarProperty::Dtstart => (
                ArchivedValueType::Ical(ArchivedICalendarValueType::DateTime),
                ValueSeparator::None,
            ),
            ArchivedICalendarProperty::Duration => (
                ArchivedValueType::Ical(ArchivedICalendarValueType::Duration),
                ValueSeparator::None,
            ),
            ArchivedICalendarProperty::Freebusy => (
                ArchivedValueType::Ical(ArchivedICalendarValueType::Period),
                ValueSeparator::Comma,
            ),
            ArchivedICalendarProperty::Transp => {
                (ArchivedValueType::Transparency, ValueSeparator::None)
            }
            ArchivedICalendarProperty::Tzid => (
                ArchivedValueType::Ical(ArchivedICalendarValueType::Text),
                ValueSeparator::None,
            ),
            ArchivedICalendarProperty::Tzname => (
                ArchivedValueType::Ical(ArchivedICalendarValueType::Text),
                ValueSeparator::None,
            ),
            ArchivedICalendarProperty::Tzoffsetfrom => (
                ArchivedValueType::Ical(ArchivedICalendarValueType::UtcOffset),
                ValueSeparator::None,
            ),
            ArchivedICalendarProperty::Tzoffsetto => (
                ArchivedValueType::Ical(ArchivedICalendarValueType::UtcOffset),
                ValueSeparator::None,
            ),
            ArchivedICalendarProperty::Tzurl => (
                ArchivedValueType::Ical(ArchivedICalendarValueType::Uri),
                ValueSeparator::None,
            ),
            ArchivedICalendarProperty::Attendee => (
                ArchivedValueType::Ical(ArchivedICalendarValueType::Uri),
                ValueSeparator::None,
            ),
            ArchivedICalendarProperty::Contact => (
                ArchivedValueType::Ical(ArchivedICalendarValueType::Text),
                ValueSeparator::None,
            ),
            ArchivedICalendarProperty::Organizer => (
                ArchivedValueType::Ical(ArchivedICalendarValueType::Uri),
                ValueSeparator::None,
            ),
            ArchivedICalendarProperty::RecurrenceId => (
                ArchivedValueType::Ical(ArchivedICalendarValueType::DateTime),
                ValueSeparator::None,
            ),
            ArchivedICalendarProperty::RelatedTo => (
                ArchivedValueType::Ical(ArchivedICalendarValueType::Text),
                ValueSeparator::None,
            ),
            ArchivedICalendarProperty::Url => (
                ArchivedValueType::Ical(ArchivedICalendarValueType::Uri),
                ValueSeparator::None,
            ),
            ArchivedICalendarProperty::Uid => (
                ArchivedValueType::Ical(ArchivedICalendarValueType::Text),
                ValueSeparator::None,
            ),
            ArchivedICalendarProperty::Exdate => (
                ArchivedValueType::Ical(ArchivedICalendarValueType::DateTime),
                ValueSeparator::Comma,
            ),
            ArchivedICalendarProperty::Exrule => (
                ArchivedValueType::Ical(ArchivedICalendarValueType::Recur),
                ValueSeparator::None,
            ),
            ArchivedICalendarProperty::Rdate => (
                ArchivedValueType::Ical(ArchivedICalendarValueType::DateTime),
                ValueSeparator::Comma,
            ),
            ArchivedICalendarProperty::Rrule => (
                ArchivedValueType::Ical(ArchivedICalendarValueType::Recur),
                ValueSeparator::None,
            ),
            ArchivedICalendarProperty::Action => (ArchivedValueType::Action, ValueSeparator::None),
            ArchivedICalendarProperty::Repeat => (
                ArchivedValueType::Ical(ArchivedICalendarValueType::Integer),
                ValueSeparator::None,
            ),
            ArchivedICalendarProperty::Trigger => (
                ArchivedValueType::Ical(ArchivedICalendarValueType::Duration),
                ValueSeparator::None,
            ),
            ArchivedICalendarProperty::Created => (
                ArchivedValueType::Ical(ArchivedICalendarValueType::DateTime),
                ValueSeparator::None,
            ),
            ArchivedICalendarProperty::Dtstamp => (
                ArchivedValueType::Ical(ArchivedICalendarValueType::DateTime),
                ValueSeparator::None,
            ),
            ArchivedICalendarProperty::LastModified => (
                ArchivedValueType::Ical(ArchivedICalendarValueType::DateTime),
                ValueSeparator::None,
            ),
            ArchivedICalendarProperty::Sequence => (
                ArchivedValueType::Ical(ArchivedICalendarValueType::Integer),
                ValueSeparator::None,
            ),
            ArchivedICalendarProperty::RequestStatus => (
                ArchivedValueType::Ical(ArchivedICalendarValueType::Text),
                ValueSeparator::Semicolon,
            ),
            ArchivedICalendarProperty::Xml => (
                ArchivedValueType::Ical(ArchivedICalendarValueType::Text),
                ValueSeparator::None,
            ),
            ArchivedICalendarProperty::Tzuntil => (
                ArchivedValueType::Ical(ArchivedICalendarValueType::DateTime),
                ValueSeparator::None,
            ),
            ArchivedICalendarProperty::TzidAliasOf => (
                ArchivedValueType::Ical(ArchivedICalendarValueType::Text),
                ValueSeparator::None,
            ),
            ArchivedICalendarProperty::Busytype => {
                (ArchivedValueType::BusyType, ValueSeparator::None)
            }
            ArchivedICalendarProperty::Name => (
                ArchivedValueType::Ical(ArchivedICalendarValueType::Text),
                ValueSeparator::None,
            ),
            ArchivedICalendarProperty::RefreshInterval => (
                ArchivedValueType::Ical(ArchivedICalendarValueType::Duration),
                ValueSeparator::None,
            ),
            ArchivedICalendarProperty::Source => (
                ArchivedValueType::Ical(ArchivedICalendarValueType::Uri),
                ValueSeparator::None,
            ),
            ArchivedICalendarProperty::Color => (
                ArchivedValueType::Ical(ArchivedICalendarValueType::Text),
                ValueSeparator::None,
            ),
            ArchivedICalendarProperty::Image => (
                ArchivedValueType::Ical(ArchivedICalendarValueType::Uri),
                ValueSeparator::None,
            ),
            ArchivedICalendarProperty::Conference => (
                ArchivedValueType::Ical(ArchivedICalendarValueType::Uri),
                ValueSeparator::None,
            ),
            ArchivedICalendarProperty::CalendarAddress => (
                ArchivedValueType::Ical(ArchivedICalendarValueType::Uri),
                ValueSeparator::None,
            ),
            ArchivedICalendarProperty::LocationType => (
                ArchivedValueType::Ical(ArchivedICalendarValueType::Text),
                ValueSeparator::Comma,
            ),
            ArchivedICalendarProperty::ParticipantType => {
                (ArchivedValueType::ParticipantType, ValueSeparator::None)
            }
            ArchivedICalendarProperty::ResourceType => {
                (ArchivedValueType::ResourceType, ValueSeparator::None)
            }
            ArchivedICalendarProperty::StructuredData => (
                ArchivedValueType::Ical(ArchivedICalendarValueType::Text),
                ValueSeparator::None,
            ),
            ArchivedICalendarProperty::StyledDescription => (
                ArchivedValueType::Ical(ArchivedICalendarValueType::Text),
                ValueSeparator::None,
            ),
            ArchivedICalendarProperty::Acknowledged => (
                ArchivedValueType::Ical(ArchivedICalendarValueType::DateTime),
                ValueSeparator::None,
            ),
            ArchivedICalendarProperty::Proximity => {
                (ArchivedValueType::Proximity, ValueSeparator::None)
            }
            ArchivedICalendarProperty::Concept => (
                ArchivedValueType::Ical(ArchivedICalendarValueType::Uri),
                ValueSeparator::None,
            ),
            ArchivedICalendarProperty::Link => (
                ArchivedValueType::Ical(ArchivedICalendarValueType::Uri),
                ValueSeparator::None,
            ),
            ArchivedICalendarProperty::Refid => (
                ArchivedValueType::Ical(ArchivedICalendarValueType::Text),
                ValueSeparator::None,
            ),
            ArchivedICalendarProperty::Begin => (
                ArchivedValueType::Ical(ArchivedICalendarValueType::Text),
                ValueSeparator::None,
            ),
            ArchivedICalendarProperty::End => (
                ArchivedValueType::Ical(ArchivedICalendarValueType::Text),
                ValueSeparator::None,
            ),
            ArchivedICalendarProperty::Other(_) => (
                ArchivedValueType::Ical(ArchivedICalendarValueType::Text),
                ValueSeparator::Semicolon,
            ),
            ArchivedICalendarProperty::Coordinates => (
                ArchivedValueType::Ical(ArchivedICalendarValueType::Uri),
                ValueSeparator::None,
            ),
            ArchivedICalendarProperty::ShowWithoutTime => (
                ArchivedValueType::Ical(ArchivedICalendarValueType::Boolean),
                ValueSeparator::None,
            ),
            ArchivedICalendarProperty::Jsid | ArchivedICalendarProperty::Jsprop => (
                ArchivedValueType::Ical(ArchivedICalendarValueType::Text),
                ValueSeparator::None,
            ),
            ArchivedICalendarProperty::EstimatedDuration => (
                ArchivedValueType::Ical(ArchivedICalendarValueType::Duration),
                ValueSeparator::None,
            ),
            ArchivedICalendarProperty::Reason => (
                ArchivedValueType::Ical(ArchivedICalendarValueType::Uri),
                ValueSeparator::None,
            ),
            ArchivedICalendarProperty::Substate => (
                ArchivedValueType::Ical(ArchivedICalendarValueType::Text),
                ValueSeparator::None,
            ),
            ArchivedICalendarProperty::TaskMode => (
                ArchivedValueType::Ical(ArchivedICalendarValueType::Text),
                ValueSeparator::None,
            ),
        }
    }
}

impl IanaString for ArchivedICalendarFrequency {
    fn as_str(&self) -> &'static str {
        match self {
            ArchivedICalendarFrequency::Secondly => "SECONDLY",
            ArchivedICalendarFrequency::Minutely => "MINUTELY",
            ArchivedICalendarFrequency::Hourly => "HOURLY",
            ArchivedICalendarFrequency::Daily => "DAILY",
            ArchivedICalendarFrequency::Weekly => "WEEKLY",
            ArchivedICalendarFrequency::Monthly => "MONTHLY",
            ArchivedICalendarFrequency::Yearly => "YEARLY",
        }
    }
}

impl ArchivedICalendarSkip {
    pub fn as_str(&self) -> &'static str {
        match self {
            ArchivedICalendarSkip::Omit => "OMIT",
            ArchivedICalendarSkip::Backward => "BACKWARD",
            ArchivedICalendarSkip::Forward => "FORWARD",
        }
    }
}

impl ArchivedICalendarWeekday {
    pub fn as_str(&self) -> &'static str {
        match self {
            ArchivedICalendarWeekday::Sunday => "SU",
            ArchivedICalendarWeekday::Monday => "MO",
            ArchivedICalendarWeekday::Tuesday => "TU",
            ArchivedICalendarWeekday::Wednesday => "WE",
            ArchivedICalendarWeekday::Thursday => "TH",
            ArchivedICalendarWeekday::Friday => "FR",
            ArchivedICalendarWeekday::Saturday => "SA",
        }
    }
}

impl ArchivedICalendarAction {
    pub fn as_str(&self) -> &'static str {
        match self {
            ArchivedICalendarAction::Audio => "AUDIO",
            ArchivedICalendarAction::Display => "DISPLAY",
            ArchivedICalendarAction::Email => "EMAIL",
            ArchivedICalendarAction::Procedure => "PROCEDURE",
        }
    }
}

impl ArchivedICalendarUserTypes {
    pub fn as_str(&self) -> &'static str {
        match self {
            ArchivedICalendarUserTypes::Individual => "INDIVIDUAL",
            ArchivedICalendarUserTypes::Group => "GROUP",
            ArchivedICalendarUserTypes::Resource => "RESOURCE",
            ArchivedICalendarUserTypes::Room => "ROOM",
            ArchivedICalendarUserTypes::Unknown => "UNKNOWN",
        }
    }
}

impl ArchivedICalendarClassification {
    pub fn as_str(&self) -> &'static str {
        match self {
            ArchivedICalendarClassification::Public => "PUBLIC",
            ArchivedICalendarClassification::Private => "PRIVATE",
            ArchivedICalendarClassification::Confidential => "CONFIDENTIAL",
        }
    }
}

impl ArchivedICalendarComponentType {
    pub fn as_str(&self) -> &str {
        match self {
            ArchivedICalendarComponentType::VCalendar => "VCALENDAR",
            ArchivedICalendarComponentType::VEvent => "VEVENT",
            ArchivedICalendarComponentType::VTodo => "VTODO",
            ArchivedICalendarComponentType::VJournal => "VJOURNAL",
            ArchivedICalendarComponentType::VFreebusy => "VFREEBUSY",
            ArchivedICalendarComponentType::VTimezone => "VTIMEZONE",
            ArchivedICalendarComponentType::VAlarm => "VALARM",
            ArchivedICalendarComponentType::Standard => "STANDARD",
            ArchivedICalendarComponentType::Daylight => "DAYLIGHT",
            ArchivedICalendarComponentType::VAvailability => "VAVAILABILITY",
            ArchivedICalendarComponentType::Available => "AVAILABLE",
            ArchivedICalendarComponentType::Participant => "PARTICIPANT",
            ArchivedICalendarComponentType::VLocation => "VLOCATION",
            ArchivedICalendarComponentType::VResource => "VRESOURCE",
            ArchivedICalendarComponentType::VStatus => "VSTATUS",
            ArchivedICalendarComponentType::Other(name) => name.as_str(),
        }
    }
}

impl ArchivedICalendarDisplayType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ArchivedICalendarDisplayType::Badge => "BADGE",
            ArchivedICalendarDisplayType::Graphic => "GRAPHIC",
            ArchivedICalendarDisplayType::Fullsize => "FULLSIZE",
            ArchivedICalendarDisplayType::Thumbnail => "THUMBNAIL",
        }
    }
}

impl ArchivedICalendarFeatureType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ArchivedICalendarFeatureType::Audio => "AUDIO",
            ArchivedICalendarFeatureType::Chat => "CHAT",
            ArchivedICalendarFeatureType::Feed => "FEED",
            ArchivedICalendarFeatureType::Moderator => "MODERATOR",
            ArchivedICalendarFeatureType::Phone => "PHONE",
            ArchivedICalendarFeatureType::Screen => "SCREEN",
            ArchivedICalendarFeatureType::Video => "VIDEO",
        }
    }
}

impl ArchivedICalendarFreeBusyType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ArchivedICalendarFreeBusyType::Free => "FREE",
            ArchivedICalendarFreeBusyType::Busy => "BUSY",
            ArchivedICalendarFreeBusyType::BusyUnavailable => "BUSY-UNAVAILABLE",
            ArchivedICalendarFreeBusyType::BusyTentative => "BUSY-TENTATIVE",
        }
    }
}

impl ArchivedICalendarMethod {
    pub fn as_str(&self) -> &'static str {
        match self {
            ArchivedICalendarMethod::Publish => "PUBLISH",
            ArchivedICalendarMethod::Request => "REQUEST",
            ArchivedICalendarMethod::Reply => "REPLY",
            ArchivedICalendarMethod::Add => "ADD",
            ArchivedICalendarMethod::Cancel => "CANCEL",
            ArchivedICalendarMethod::Refresh => "REFRESH",
            ArchivedICalendarMethod::Counter => "COUNTER",
            ArchivedICalendarMethod::Declinecounter => "DECLINECOUNTER",
        }
    }
}

impl ArchivedICalendarParticipantType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ArchivedICalendarParticipantType::Active => "ACTIVE",
            ArchivedICalendarParticipantType::Inactive => "INACTIVE",
            ArchivedICalendarParticipantType::Sponsor => "SPONSOR",
            ArchivedICalendarParticipantType::Contact => "CONTACT",
            ArchivedICalendarParticipantType::BookingContact => "BOOKING-CONTACT",
            ArchivedICalendarParticipantType::EmergencyContact => "EMERGENCY-CONTACT",
            ArchivedICalendarParticipantType::PublicityContact => "PUBLICITY-CONTACT",
            ArchivedICalendarParticipantType::PlannerContact => "PLANNER-CONTACT",
            ArchivedICalendarParticipantType::Performer => "PERFORMER",
            ArchivedICalendarParticipantType::Speaker => "SPEAKER",
        }
    }
}

impl ArchivedICalendarParticipationRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            ArchivedICalendarParticipationRole::Chair => "CHAIR",
            ArchivedICalendarParticipationRole::ReqParticipant => "REQ-PARTICIPANT",
            ArchivedICalendarParticipationRole::OptParticipant => "OPT-PARTICIPANT",
            ArchivedICalendarParticipationRole::NonParticipant => "NON-PARTICIPANT",
            ArchivedICalendarParticipationRole::Owner => "OWNER",
        }
    }
}

impl ArchivedICalendarParticipationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            ArchivedICalendarParticipationStatus::NeedsAction => "NEEDS-ACTION",
            ArchivedICalendarParticipationStatus::Accepted => "ACCEPTED",
            ArchivedICalendarParticipationStatus::Declined => "DECLINED",
            ArchivedICalendarParticipationStatus::Tentative => "TENTATIVE",
            ArchivedICalendarParticipationStatus::Delegated => "DELEGATED",
            ArchivedICalendarParticipationStatus::Completed => "COMPLETED",
            ArchivedICalendarParticipationStatus::InProcess => "IN-PROCESS",
            ArchivedICalendarParticipationStatus::Failed => "FAILED",
        }
    }
}

impl ArchivedICalendarStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            ArchivedICalendarStatus::Tentative => "TENTATIVE",
            ArchivedICalendarStatus::Confirmed => "CONFIRMED",
            ArchivedICalendarStatus::Cancelled => "CANCELLED",
            ArchivedICalendarStatus::NeedsAction => "NEEDS-ACTION",
            ArchivedICalendarStatus::Completed => "COMPLETED",
            ArchivedICalendarStatus::InProcess => "IN-PROCESS",
            ArchivedICalendarStatus::Draft => "DRAFT",
            ArchivedICalendarStatus::Final => "FINAL",
            ArchivedICalendarStatus::Failed => "FAILED",
            ArchivedICalendarStatus::Pending => "PENDING",
        }
    }
}

impl ArchivedICalendarProperty {
    pub fn as_str(&self) -> &str {
        match self {
            ArchivedICalendarProperty::Calscale => "CALSCALE",
            ArchivedICalendarProperty::Method => "METHOD",
            ArchivedICalendarProperty::Prodid => "PRODID",
            ArchivedICalendarProperty::Version => "VERSION",
            ArchivedICalendarProperty::Attach => "ATTACH",
            ArchivedICalendarProperty::Categories => "CATEGORIES",
            ArchivedICalendarProperty::Class => "CLASS",
            ArchivedICalendarProperty::Comment => "COMMENT",
            ArchivedICalendarProperty::Description => "DESCRIPTION",
            ArchivedICalendarProperty::Geo => "GEO",
            ArchivedICalendarProperty::Location => "LOCATION",
            ArchivedICalendarProperty::PercentComplete => "PERCENT-COMPLETE",
            ArchivedICalendarProperty::Priority => "PRIORITY",
            ArchivedICalendarProperty::Resources => "RESOURCES",
            ArchivedICalendarProperty::Status => "STATUS",
            ArchivedICalendarProperty::Summary => "SUMMARY",
            ArchivedICalendarProperty::Completed => "COMPLETED",
            ArchivedICalendarProperty::Dtend => "DTEND",
            ArchivedICalendarProperty::Due => "DUE",
            ArchivedICalendarProperty::Dtstart => "DTSTART",
            ArchivedICalendarProperty::Duration => "DURATION",
            ArchivedICalendarProperty::Freebusy => "FREEBUSY",
            ArchivedICalendarProperty::Transp => "TRANSP",
            ArchivedICalendarProperty::Tzid => "TZID",
            ArchivedICalendarProperty::Tzname => "TZNAME",
            ArchivedICalendarProperty::Tzoffsetfrom => "TZOFFSETFROM",
            ArchivedICalendarProperty::Tzoffsetto => "TZOFFSETTO",
            ArchivedICalendarProperty::Tzurl => "TZURL",
            ArchivedICalendarProperty::Attendee => "ATTENDEE",
            ArchivedICalendarProperty::Contact => "CONTACT",
            ArchivedICalendarProperty::Organizer => "ORGANIZER",
            ArchivedICalendarProperty::RecurrenceId => "RECURRENCE-ID",
            ArchivedICalendarProperty::RelatedTo => "RELATED-TO",
            ArchivedICalendarProperty::Url => "URL",
            ArchivedICalendarProperty::Uid => "UID",
            ArchivedICalendarProperty::Exdate => "EXDATE",
            ArchivedICalendarProperty::Exrule => "EXRULE",
            ArchivedICalendarProperty::Rdate => "RDATE",
            ArchivedICalendarProperty::Rrule => "RRULE",
            ArchivedICalendarProperty::Action => "ACTION",
            ArchivedICalendarProperty::Repeat => "REPEAT",
            ArchivedICalendarProperty::Trigger => "TRIGGER",
            ArchivedICalendarProperty::Created => "CREATED",
            ArchivedICalendarProperty::Dtstamp => "DTSTAMP",
            ArchivedICalendarProperty::LastModified => "LAST-MODIFIED",
            ArchivedICalendarProperty::Sequence => "SEQUENCE",
            ArchivedICalendarProperty::RequestStatus => "REQUEST-STATUS",
            ArchivedICalendarProperty::Xml => "XML",
            ArchivedICalendarProperty::Tzuntil => "TZUNTIL",
            ArchivedICalendarProperty::TzidAliasOf => "TZID-ALIAS-OF",
            ArchivedICalendarProperty::Busytype => "BUSYTYPE",
            ArchivedICalendarProperty::Name => "NAME",
            ArchivedICalendarProperty::RefreshInterval => "REFRESH-INTERVAL",
            ArchivedICalendarProperty::Source => "SOURCE",
            ArchivedICalendarProperty::Color => "COLOR",
            ArchivedICalendarProperty::Image => "IMAGE",
            ArchivedICalendarProperty::Conference => "CONFERENCE",
            ArchivedICalendarProperty::CalendarAddress => "CALENDAR-ADDRESS",
            ArchivedICalendarProperty::LocationType => "LOCATION-TYPE",
            ArchivedICalendarProperty::ParticipantType => "PARTICIPANT-TYPE",
            ArchivedICalendarProperty::ResourceType => "RESOURCE-TYPE",
            ArchivedICalendarProperty::StructuredData => "STRUCTURED-DATA",
            ArchivedICalendarProperty::StyledDescription => "STYLED-DESCRIPTION",
            ArchivedICalendarProperty::Acknowledged => "ACKNOWLEDGED",
            ArchivedICalendarProperty::Proximity => "PROXIMITY",
            ArchivedICalendarProperty::Concept => "CONCEPT",
            ArchivedICalendarProperty::Link => "LINK",
            ArchivedICalendarProperty::Refid => "REFID",
            ArchivedICalendarProperty::Begin => "BEGIN",
            ArchivedICalendarProperty::End => "END",
            ArchivedICalendarProperty::Coordinates => "COORDINATES",
            ArchivedICalendarProperty::ShowWithoutTime => "SHOW-WITHOUT-TIME",
            ArchivedICalendarProperty::Jsid => "JSID",
            ArchivedICalendarProperty::Jsprop => "JSPROP",
            ArchivedICalendarProperty::EstimatedDuration => "ESTIMATED-DURATION",
            ArchivedICalendarProperty::Reason => "REASON",
            ArchivedICalendarProperty::Substate => "SUBSTATE",
            ArchivedICalendarProperty::TaskMode => "TASK-MODE",
            ArchivedICalendarProperty::Other(value) => value.as_str(),
        }
    }
}

impl ArchivedICalendarProximityValue {
    pub fn as_str(&self) -> &'static str {
        match self {
            ArchivedICalendarProximityValue::Arrive => "ARRIVE",
            ArchivedICalendarProximityValue::Depart => "DEPART",
            ArchivedICalendarProximityValue::Connect => "CONNECT",
            ArchivedICalendarProximityValue::Disconnect => "DISCONNECT",
        }
    }
}

impl ArchivedICalendarRelationshipType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ArchivedICalendarRelationshipType::Child => "CHILD",
            ArchivedICalendarRelationshipType::Parent => "PARENT",
            ArchivedICalendarRelationshipType::Sibling => "SIBLING",
            ArchivedICalendarRelationshipType::Snooze => "SNOOZE",
            ArchivedICalendarRelationshipType::Concept => "CONCEPT",
            ArchivedICalendarRelationshipType::DependsOn => "DEPENDS-ON",
            ArchivedICalendarRelationshipType::Finishtofinish => "FINISHTOFINISH",
            ArchivedICalendarRelationshipType::Finishtostart => "FINISHTOSTART",
            ArchivedICalendarRelationshipType::First => "FIRST",
            ArchivedICalendarRelationshipType::Next => "NEXT",
            ArchivedICalendarRelationshipType::Refid => "REFID",
            ArchivedICalendarRelationshipType::Starttofinish => "STARTTOFINISH",
            ArchivedICalendarRelationshipType::Starttostart => "STARTTOSTART",
        }
    }
}

impl ArchivedICalendarResourceType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ArchivedICalendarResourceType::Projector => "PROJECTOR",
            ArchivedICalendarResourceType::Room => "ROOM",
            ArchivedICalendarResourceType::RemoteConferenceAudio => "REMOTE-CONFERENCE-AUDIO",
            ArchivedICalendarResourceType::RemoteConferenceVideo => "REMOTE-CONFERENCE-VIDEO",
        }
    }
}

impl ArchivedICalendarScheduleAgentValue {
    pub fn as_str(&self) -> &'static str {
        match self {
            ArchivedICalendarScheduleAgentValue::Server => "SERVER",
            ArchivedICalendarScheduleAgentValue::Client => "CLIENT",
            ArchivedICalendarScheduleAgentValue::None => "NONE",
        }
    }
}

impl ArchivedICalendarScheduleForceSendValue {
    pub fn as_str(&self) -> &'static str {
        match self {
            ArchivedICalendarScheduleForceSendValue::Request => "REQUEST",
            ArchivedICalendarScheduleForceSendValue::Reply => "REPLY",
        }
    }
}

impl ArchivedICalendarValueType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ArchivedICalendarValueType::Binary => "BINARY",
            ArchivedICalendarValueType::Boolean => "BOOLEAN",
            ArchivedICalendarValueType::CalAddress => "CAL-ADDRESS",
            ArchivedICalendarValueType::Date => "DATE",
            ArchivedICalendarValueType::DateTime => "DATE-TIME",
            ArchivedICalendarValueType::Duration => "DURATION",
            ArchivedICalendarValueType::Float => "FLOAT",
            ArchivedICalendarValueType::Integer => "INTEGER",
            ArchivedICalendarValueType::Period => "PERIOD",
            ArchivedICalendarValueType::Recur => "RECUR",
            ArchivedICalendarValueType::Text => "TEXT",
            ArchivedICalendarValueType::Time => "TIME",
            ArchivedICalendarValueType::Unknown => "UNKNOWN",
            ArchivedICalendarValueType::Uri => "URI",
            ArchivedICalendarValueType::UtcOffset => "UTC-OFFSET",
            ArchivedICalendarValueType::XmlReference => "XML-REFERENCE",
            ArchivedICalendarValueType::Uid => "UID",
        }
    }
}

impl ArchivedICalendarTransparency {
    pub fn as_str(&self) -> &'static str {
        match self {
            ArchivedICalendarTransparency::Opaque => "OPAQUE",
            ArchivedICalendarTransparency::Transparent => "TRANSPARENT",
        }
    }
}

impl ArchivedLinkRelation {
    pub fn as_str(&self) -> &'static str {
        match self {
            ArchivedLinkRelation::About => "about",
            ArchivedLinkRelation::Acl => "acl",
            ArchivedLinkRelation::Alternate => "alternate",
            ArchivedLinkRelation::Amphtml => "amphtml",
            ArchivedLinkRelation::ApiCatalog => "api-catalog",
            ArchivedLinkRelation::Appendix => "appendix",
            ArchivedLinkRelation::AppleTouchIcon => "apple-touch-icon",
            ArchivedLinkRelation::AppleTouchStartupImage => "apple-touch-startup-image",
            ArchivedLinkRelation::Archives => "archives",
            ArchivedLinkRelation::Author => "author",
            ArchivedLinkRelation::BlockedBy => "blocked-by",
            ArchivedLinkRelation::Bookmark => "bookmark",
            ArchivedLinkRelation::C2paManifest => "c2pa-manifest",
            ArchivedLinkRelation::Canonical => "canonical",
            ArchivedLinkRelation::Chapter => "chapter",
            ArchivedLinkRelation::CiteAs => "cite-as",
            ArchivedLinkRelation::Collection => "collection",
            ArchivedLinkRelation::CompressionDictionary => "compression-dictionary",
            ArchivedLinkRelation::Contents => "contents",
            ArchivedLinkRelation::Convertedfrom => "convertedfrom",
            ArchivedLinkRelation::Copyright => "copyright",
            ArchivedLinkRelation::CreateForm => "create-form",
            ArchivedLinkRelation::Current => "current",
            ArchivedLinkRelation::Deprecation => "deprecation",
            ArchivedLinkRelation::Describedby => "describedby",
            ArchivedLinkRelation::Describes => "describes",
            ArchivedLinkRelation::Disclosure => "disclosure",
            ArchivedLinkRelation::DnsPrefetch => "dns-prefetch",
            ArchivedLinkRelation::Duplicate => "duplicate",
            ArchivedLinkRelation::Edit => "edit",
            ArchivedLinkRelation::EditForm => "edit-form",
            ArchivedLinkRelation::EditMedia => "edit-media",
            ArchivedLinkRelation::Enclosure => "enclosure",
            ArchivedLinkRelation::External => "external",
            ArchivedLinkRelation::First => "first",
            ArchivedLinkRelation::Geofeed => "geofeed",
            ArchivedLinkRelation::Glossary => "glossary",
            ArchivedLinkRelation::Help => "help",
            ArchivedLinkRelation::Hosts => "hosts",
            ArchivedLinkRelation::Hub => "hub",
            ArchivedLinkRelation::IceServer => "ice-server",
            ArchivedLinkRelation::Icon => "icon",
            ArchivedLinkRelation::Index => "index",
            ArchivedLinkRelation::Intervalafter => "intervalafter",
            ArchivedLinkRelation::Intervalbefore => "intervalbefore",
            ArchivedLinkRelation::Intervalcontains => "intervalcontains",
            ArchivedLinkRelation::Intervaldisjoint => "intervaldisjoint",
            ArchivedLinkRelation::Intervalduring => "intervalduring",
            ArchivedLinkRelation::Intervalequals => "intervalequals",
            ArchivedLinkRelation::Intervalfinishedby => "intervalfinishedby",
            ArchivedLinkRelation::Intervalfinishes => "intervalfinishes",
            ArchivedLinkRelation::Intervalin => "intervalin",
            ArchivedLinkRelation::Intervalmeets => "intervalmeets",
            ArchivedLinkRelation::Intervalmetby => "intervalmetby",
            ArchivedLinkRelation::Intervaloverlappedby => "intervaloverlappedby",
            ArchivedLinkRelation::Intervaloverlaps => "intervaloverlaps",
            ArchivedLinkRelation::Intervalstartedby => "intervalstartedby",
            ArchivedLinkRelation::Intervalstarts => "intervalstarts",
            ArchivedLinkRelation::Item => "item",
            ArchivedLinkRelation::Last => "last",
            ArchivedLinkRelation::LatestVersion => "latest-version",
            ArchivedLinkRelation::License => "license",
            ArchivedLinkRelation::Linkset => "linkset",
            ArchivedLinkRelation::Lrdd => "lrdd",
            ArchivedLinkRelation::Manifest => "manifest",
            ArchivedLinkRelation::MaskIcon => "mask-icon",
            ArchivedLinkRelation::Me => "me",
            ArchivedLinkRelation::MediaFeed => "media-feed",
            ArchivedLinkRelation::Memento => "memento",
            ArchivedLinkRelation::Micropub => "micropub",
            ArchivedLinkRelation::Modulepreload => "modulepreload",
            ArchivedLinkRelation::Monitor => "monitor",
            ArchivedLinkRelation::MonitorGroup => "monitor-group",
            ArchivedLinkRelation::Next => "next",
            ArchivedLinkRelation::NextArchive => "next-archive",
            ArchivedLinkRelation::Nofollow => "nofollow",
            ArchivedLinkRelation::Noopener => "noopener",
            ArchivedLinkRelation::Noreferrer => "noreferrer",
            ArchivedLinkRelation::Opener => "opener",
            ArchivedLinkRelation::Openid2LocalId => "openid2.local_id",
            ArchivedLinkRelation::Openid2Provider => "openid2.provider",
            ArchivedLinkRelation::Original => "original",
            ArchivedLinkRelation::P3pv1 => "p3pv1",
            ArchivedLinkRelation::Payment => "payment",
            ArchivedLinkRelation::Pingback => "pingback",
            ArchivedLinkRelation::Preconnect => "preconnect",
            ArchivedLinkRelation::PredecessorVersion => "predecessor-version",
            ArchivedLinkRelation::Prefetch => "prefetch",
            ArchivedLinkRelation::Preload => "preload",
            ArchivedLinkRelation::Prerender => "prerender",
            ArchivedLinkRelation::Prev => "prev",
            ArchivedLinkRelation::Preview => "preview",
            ArchivedLinkRelation::Previous => "previous",
            ArchivedLinkRelation::PrevArchive => "prev-archive",
            ArchivedLinkRelation::PrivacyPolicy => "privacy-policy",
            ArchivedLinkRelation::Profile => "profile",
            ArchivedLinkRelation::Publication => "publication",
            ArchivedLinkRelation::RdapActive => "rdap-active",
            ArchivedLinkRelation::RdapBottom => "rdap-bottom",
            ArchivedLinkRelation::RdapDown => "rdap-down",
            ArchivedLinkRelation::RdapTop => "rdap-top",
            ArchivedLinkRelation::RdapUp => "rdap-up",
            ArchivedLinkRelation::Related => "related",
            ArchivedLinkRelation::Restconf => "restconf",
            ArchivedLinkRelation::Replies => "replies",
            ArchivedLinkRelation::Ruleinput => "ruleinput",
            ArchivedLinkRelation::Search => "search",
            ArchivedLinkRelation::Section => "section",
            ArchivedLinkRelation::Self_ => "self",
            ArchivedLinkRelation::Service => "service",
            ArchivedLinkRelation::ServiceDesc => "service-desc",
            ArchivedLinkRelation::ServiceDoc => "service-doc",
            ArchivedLinkRelation::ServiceMeta => "service-meta",
            ArchivedLinkRelation::SipTrunkingCapability => "sip-trunking-capability",
            ArchivedLinkRelation::Sponsored => "sponsored",
            ArchivedLinkRelation::Start => "start",
            ArchivedLinkRelation::Status => "status",
            ArchivedLinkRelation::Stylesheet => "stylesheet",
            ArchivedLinkRelation::Subsection => "subsection",
            ArchivedLinkRelation::SuccessorVersion => "successor-version",
            ArchivedLinkRelation::Sunset => "sunset",
            ArchivedLinkRelation::Tag => "tag",
            ArchivedLinkRelation::TermsOfService => "terms-of-service",
            ArchivedLinkRelation::Timegate => "timegate",
            ArchivedLinkRelation::Timemap => "timemap",
            ArchivedLinkRelation::Type => "type",
            ArchivedLinkRelation::Ugc => "ugc",
            ArchivedLinkRelation::Up => "up",
            ArchivedLinkRelation::VersionHistory => "version-history",
            ArchivedLinkRelation::Via => "via",
            ArchivedLinkRelation::Webmention => "webmention",
            ArchivedLinkRelation::WorkingCopy => "working-copy",
            ArchivedLinkRelation::WorkingCopyOf => "working-copy-of",
        }
    }
}

impl ArchivedICalendarRelated {
    pub fn as_str(&self) -> &'static str {
        match self {
            ArchivedICalendarRelated::Start => "START",
            ArchivedICalendarRelated::End => "END",
        }
    }
}

#[derive(Debug)]
pub(crate) enum ArchivedValueType {
    Ical(ArchivedICalendarValueType),
    CalendarScale,
    Method,
    Classification,
    Status,
    Transparency,
    Action,
    BusyType,
    ParticipantType,
    ResourceType,
    Proximity,
}
impl ArchivedValueType {
    pub fn unwrap_ical(self) -> ArchivedICalendarValueType {
        match self {
            ArchivedValueType::Ical(v) => v,
            _ => ArchivedICalendarValueType::Text,
        }
    }
}
