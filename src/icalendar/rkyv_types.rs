/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

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
                ValueSeparator::None,
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
        }
    }
}

impl ArchivedICalendarFrequency {
    pub fn as_str(&self) -> &str {
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

impl ArchivedICalendarWeekday {
    pub fn as_str(&self) -> &str {
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
    pub fn as_str(&self) -> &str {
        match self {
            ArchivedICalendarAction::Audio => "AUDIO",
            ArchivedICalendarAction::Display => "DISPLAY",
            ArchivedICalendarAction::Email => "EMAIL",
            ArchivedICalendarAction::Procedure => "PROCEDURE",
            ArchivedICalendarAction::Other(value) => value,
        }
    }
}

impl ArchivedICalendarUserTypes {
    pub fn as_str(&self) -> &str {
        match self {
            ArchivedICalendarUserTypes::Individual => "INDIVIDUAL",
            ArchivedICalendarUserTypes::Group => "GROUP",
            ArchivedICalendarUserTypes::Resource => "RESOURCE",
            ArchivedICalendarUserTypes::Room => "ROOM",
            ArchivedICalendarUserTypes::Unknown => "UNKNOWN",
            ArchivedICalendarUserTypes::Other(value) => value,
        }
    }
}

impl ArchivedICalendarClassification {
    pub fn as_str(&self) -> &str {
        match self {
            ArchivedICalendarClassification::Public => "PUBLIC",
            ArchivedICalendarClassification::Private => "PRIVATE",
            ArchivedICalendarClassification::Confidential => "CONFIDENTIAL",
            ArchivedICalendarClassification::Other(value) => value,
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
            ArchivedICalendarComponentType::Other(name) => name.as_str(),
        }
    }

    pub fn has_time_ranges(&self) -> bool {
        matches!(
            self,
            ArchivedICalendarComponentType::VEvent
                | ArchivedICalendarComponentType::VTodo
                | ArchivedICalendarComponentType::VJournal
                | ArchivedICalendarComponentType::VFreebusy
        )
    }
}

impl ArchivedICalendarDisplayType {
    pub fn as_str(&self) -> &str {
        match self {
            ArchivedICalendarDisplayType::Badge => "BADGE",
            ArchivedICalendarDisplayType::Graphic => "GRAPHIC",
            ArchivedICalendarDisplayType::Fullsize => "FULLSIZE",
            ArchivedICalendarDisplayType::Thumbnail => "THUMBNAIL",
            ArchivedICalendarDisplayType::Other(value) => value,
        }
    }
}

impl ArchivedICalendarFeatureType {
    pub fn as_str(&self) -> &str {
        match self {
            ArchivedICalendarFeatureType::Audio => "AUDIO",
            ArchivedICalendarFeatureType::Chat => "CHAT",
            ArchivedICalendarFeatureType::Feed => "FEED",
            ArchivedICalendarFeatureType::Moderator => "MODERATOR",
            ArchivedICalendarFeatureType::Phone => "PHONE",
            ArchivedICalendarFeatureType::Screen => "SCREEN",
            ArchivedICalendarFeatureType::Video => "VIDEO",
            ArchivedICalendarFeatureType::Other(value) => value,
        }
    }
}

impl ArchivedICalendarFreeBusyType {
    pub fn as_str(&self) -> &str {
        match self {
            ArchivedICalendarFreeBusyType::Free => "FREE",
            ArchivedICalendarFreeBusyType::Busy => "BUSY",
            ArchivedICalendarFreeBusyType::BusyUnavailable => "BUSY-UNAVAILABLE",
            ArchivedICalendarFreeBusyType::BusyTentative => "BUSY-TENTATIVE",
            ArchivedICalendarFreeBusyType::Other(value) => value,
        }
    }
}

impl ArchivedICalendarMethod {
    pub fn as_str(&self) -> &str {
        match self {
            ArchivedICalendarMethod::Publish => "PUBLISH",
            ArchivedICalendarMethod::Request => "REQUEST",
            ArchivedICalendarMethod::Reply => "REPLY",
            ArchivedICalendarMethod::Add => "ADD",
            ArchivedICalendarMethod::Cancel => "CANCEL",
            ArchivedICalendarMethod::Refresh => "REFRESH",
            ArchivedICalendarMethod::Counter => "COUNTER",
            ArchivedICalendarMethod::Declinecounter => "DECLINECOUNTER",
            ArchivedICalendarMethod::Other(value) => value,
        }
    }
}

impl ArchivedICalendarParticipantType {
    pub fn as_str(&self) -> &str {
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
            ArchivedICalendarParticipantType::Other(value) => value,
        }
    }
}

impl ArchivedICalendarParticipationRole {
    pub fn as_str(&self) -> &str {
        match self {
            ArchivedICalendarParticipationRole::Chair => "CHAIR",
            ArchivedICalendarParticipationRole::ReqParticipant => "REQ-PARTICIPANT",
            ArchivedICalendarParticipationRole::OptParticipant => "OPT-PARTICIPANT",
            ArchivedICalendarParticipationRole::NonParticipant => "NON-PARTICIPANT",
            ArchivedICalendarParticipationRole::Other(value) => value,
        }
    }
}

impl ArchivedICalendarParticipationStatus {
    pub fn as_str(&self) -> &str {
        match self {
            ArchivedICalendarParticipationStatus::NeedsAction => "NEEDS-ACTION",
            ArchivedICalendarParticipationStatus::Accepted => "ACCEPTED",
            ArchivedICalendarParticipationStatus::Declined => "DECLINED",
            ArchivedICalendarParticipationStatus::Tentative => "TENTATIVE",
            ArchivedICalendarParticipationStatus::Delegated => "DELEGATED",
            ArchivedICalendarParticipationStatus::Completed => "COMPLETED",
            ArchivedICalendarParticipationStatus::InProcess => "IN-PROCESS",
            ArchivedICalendarParticipationStatus::Other(value) => value,
        }
    }
}

impl ArchivedICalendarStatus {
    pub fn as_str(&self) -> &str {
        match self {
            ArchivedICalendarStatus::Tentative => "TENTATIVE",
            ArchivedICalendarStatus::Confirmed => "CONFIRMED",
            ArchivedICalendarStatus::Cancelled => "CANCELLED",
            ArchivedICalendarStatus::NeedsAction => "NEEDS-ACTION",
            ArchivedICalendarStatus::Completed => "COMPLETED",
            ArchivedICalendarStatus::InProcess => "IN-PROCESS",
            ArchivedICalendarStatus::Draft => "DRAFT",
            ArchivedICalendarStatus::Final => "FINAL",
            ArchivedICalendarStatus::Other(value) => value,
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
            ArchivedICalendarProperty::Other(value) => value,
        }
    }
}

impl ArchivedICalendarProximityValue {
    pub fn as_str(&self) -> &str {
        match self {
            ArchivedICalendarProximityValue::Arrive => "ARRIVE",
            ArchivedICalendarProximityValue::Depart => "DEPART",
            ArchivedICalendarProximityValue::Connect => "CONNECT",
            ArchivedICalendarProximityValue::Disconnect => "DISCONNECT",
            ArchivedICalendarProximityValue::Other(value) => value,
        }
    }
}

impl ArchivedICalendarRelationshipType {
    pub fn as_str(&self) -> &str {
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
            ArchivedICalendarRelationshipType::Other(value) => value,
        }
    }
}

impl ArchivedICalendarResourceType {
    pub fn as_str(&self) -> &str {
        match self {
            ArchivedICalendarResourceType::Projector => "PROJECTOR",
            ArchivedICalendarResourceType::Room => "ROOM",
            ArchivedICalendarResourceType::RemoteConferenceAudio => "REMOTE-CONFERENCE-AUDIO",
            ArchivedICalendarResourceType::RemoteConferenceVideo => "REMOTE-CONFERENCE-VIDEO",
            ArchivedICalendarResourceType::Other(value) => value,
        }
    }
}

impl ArchivedICalendarScheduleAgentValue {
    pub fn as_str(&self) -> &str {
        match self {
            ArchivedICalendarScheduleAgentValue::Server => "SERVER",
            ArchivedICalendarScheduleAgentValue::Client => "CLIENT",
            ArchivedICalendarScheduleAgentValue::None => "NONE",
            ArchivedICalendarScheduleAgentValue::Other(value) => value,
        }
    }
}

impl ArchivedICalendarScheduleForceSendValue {
    pub fn as_str(&self) -> &str {
        match self {
            ArchivedICalendarScheduleForceSendValue::Request => "REQUEST",
            ArchivedICalendarScheduleForceSendValue::Reply => "REPLY",
            ArchivedICalendarScheduleForceSendValue::Other(value) => value,
        }
    }
}

impl ArchivedICalendarValueType {
    pub fn as_str(&self) -> &str {
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
            ArchivedICalendarValueType::Other(value) => value,
        }
    }
}

impl ArchivedICalendarTransparency {
    pub fn as_str(&self) -> &str {
        match self {
            ArchivedICalendarTransparency::Opaque => "OPAQUE",
            ArchivedICalendarTransparency::Transparent => "TRANSPARENT",
            ArchivedICalendarTransparency::Other(value) => value,
        }
    }
}

impl ArchivedRelated {
    pub fn as_str(&self) -> &str {
        match self {
            ArchivedRelated::Start => "START",
            ArchivedRelated::End => "END",
        }
    }
}

impl AsRef<str> for ArchivedICalendarFrequency {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl AsRef<str> for ArchivedICalendarWeekday {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl AsRef<str> for ArchivedICalendarAction {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl AsRef<str> for ArchivedICalendarUserTypes {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl AsRef<str> for ArchivedICalendarClassification {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl AsRef<str> for ArchivedICalendarComponentType {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl AsRef<str> for ArchivedICalendarDisplayType {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl AsRef<str> for ArchivedICalendarFeatureType {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl AsRef<str> for ArchivedICalendarFreeBusyType {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl AsRef<str> for ArchivedICalendarMethod {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl AsRef<str> for ArchivedRelated {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl AsRef<str> for ArchivedICalendarParticipantType {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl AsRef<str> for ArchivedICalendarParticipationRole {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl AsRef<str> for ArchivedICalendarParticipationStatus {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl AsRef<str> for ArchivedICalendarProperty {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl AsRef<str> for ArchivedICalendarProximityValue {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl AsRef<str> for ArchivedICalendarRelationshipType {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl AsRef<str> for ArchivedICalendarResourceType {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl AsRef<str> for ArchivedICalendarScheduleAgentValue {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl AsRef<str> for ArchivedICalendarScheduleForceSendValue {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl AsRef<str> for ArchivedICalendarValueType {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl AsRef<str> for ArchivedICalendarTransparency {
    fn as_ref(&self) -> &str {
        self.as_str()
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
