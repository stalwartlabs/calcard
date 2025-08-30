/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

use crate::{Entry, Parser};

use super::*;

impl IanaParse for ICalendarProperty {
    fn parse(value: &[u8]) -> Option<Self> {
        hashify::tiny_map_ignore_case!(value,
            "CALSCALE" => ICalendarProperty::Calscale,
            "METHOD" => ICalendarProperty::Method,
            "PRODID" => ICalendarProperty::Prodid,
            "VERSION" => ICalendarProperty::Version,
            "ATTACH" => ICalendarProperty::Attach,
            "CATEGORIES" => ICalendarProperty::Categories,
            "CLASS" => ICalendarProperty::Class,
            "COMMENT" => ICalendarProperty::Comment,
            "DESCRIPTION" => ICalendarProperty::Description,
            "GEO" => ICalendarProperty::Geo,
            "LOCATION" => ICalendarProperty::Location,
            "PERCENT-COMPLETE" => ICalendarProperty::PercentComplete,
            "PRIORITY" => ICalendarProperty::Priority,
            "RESOURCES" => ICalendarProperty::Resources,
            "STATUS" => ICalendarProperty::Status,
            "SUMMARY" => ICalendarProperty::Summary,
            "COMPLETED" => ICalendarProperty::Completed,
            "DTEND" => ICalendarProperty::Dtend,
            "DUE" => ICalendarProperty::Due,
            "DTSTART" => ICalendarProperty::Dtstart,
            "DURATION" => ICalendarProperty::Duration,
            "FREEBUSY" => ICalendarProperty::Freebusy,
            "TRANSP" => ICalendarProperty::Transp,
            "TZID" => ICalendarProperty::Tzid,
            "TZNAME" => ICalendarProperty::Tzname,
            "TZOFFSETFROM" => ICalendarProperty::Tzoffsetfrom,
            "TZOFFSETTO" => ICalendarProperty::Tzoffsetto,
            "TZURL" => ICalendarProperty::Tzurl,
            "ATTENDEE" => ICalendarProperty::Attendee,
            "CONTACT" => ICalendarProperty::Contact,
            "ORGANIZER" => ICalendarProperty::Organizer,
            "RECURRENCE-ID" => ICalendarProperty::RecurrenceId,
            "RELATED-TO" => ICalendarProperty::RelatedTo,
            "URL" => ICalendarProperty::Url,
            "UID" => ICalendarProperty::Uid,
            "EXDATE" => ICalendarProperty::Exdate,
            "EXRULE" => ICalendarProperty::Exrule,
            "RDATE" => ICalendarProperty::Rdate,
            "RRULE" => ICalendarProperty::Rrule,
            "ACTION" => ICalendarProperty::Action,
            "REPEAT" => ICalendarProperty::Repeat,
            "TRIGGER" => ICalendarProperty::Trigger,
            "CREATED" => ICalendarProperty::Created,
            "DTSTAMP" => ICalendarProperty::Dtstamp,
            "LAST-MODIFIED" => ICalendarProperty::LastModified,
            "SEQUENCE" => ICalendarProperty::Sequence,
            "REQUEST-STATUS" => ICalendarProperty::RequestStatus,
            "XML" => ICalendarProperty::Xml,
            "TZUNTIL" => ICalendarProperty::Tzuntil,
            "TZID-ALIAS-OF" => ICalendarProperty::TzidAliasOf,
            "BUSYTYPE" => ICalendarProperty::Busytype,
            "NAME" => ICalendarProperty::Name,
            "REFRESH-INTERVAL" => ICalendarProperty::RefreshInterval,
            "SOURCE" => ICalendarProperty::Source,
            "COLOR" => ICalendarProperty::Color,
            "IMAGE" => ICalendarProperty::Image,
            "CONFERENCE" => ICalendarProperty::Conference,
            "CALENDAR-ADDRESS" => ICalendarProperty::CalendarAddress,
            "LOCATION-TYPE" => ICalendarProperty::LocationType,
            "PARTICIPANT-TYPE" => ICalendarProperty::ParticipantType,
            "RESOURCE-TYPE" => ICalendarProperty::ResourceType,
            "STRUCTURED-DATA" => ICalendarProperty::StructuredData,
            "STYLED-DESCRIPTION" => ICalendarProperty::StyledDescription,
            "ACKNOWLEDGED" => ICalendarProperty::Acknowledged,
            "PROXIMITY" => ICalendarProperty::Proximity,
            "CONCEPT" => ICalendarProperty::Concept,
            "LINK" => ICalendarProperty::Link,
            "REFID" => ICalendarProperty::Refid,
            "COORDINATES" => ICalendarProperty::Coordinates,
            "SHOW-WITHOUT-TIME" => ICalendarProperty::ShowWithoutTime,
            "JSID" => ICalendarProperty::Jsid,
            "JSPROP" => ICalendarProperty::Jsprop,
            "BEGIN" => ICalendarProperty::Begin,
            "END" => ICalendarProperty::End,
            "ESTIMATED-DURATION" => ICalendarProperty::EstimatedDuration,
            "REASON" => ICalendarProperty::Reason,
            "SUBSTATE" => ICalendarProperty::Substate,
            "TASK-MODE" => ICalendarProperty::TaskMode,
        )
    }
}

impl ICalendarProperty {
    pub fn as_str(&self) -> &str {
        match self {
            ICalendarProperty::Calscale => "CALSCALE",
            ICalendarProperty::Method => "METHOD",
            ICalendarProperty::Prodid => "PRODID",
            ICalendarProperty::Version => "VERSION",
            ICalendarProperty::Attach => "ATTACH",
            ICalendarProperty::Categories => "CATEGORIES",
            ICalendarProperty::Class => "CLASS",
            ICalendarProperty::Comment => "COMMENT",
            ICalendarProperty::Description => "DESCRIPTION",
            ICalendarProperty::Geo => "GEO",
            ICalendarProperty::Location => "LOCATION",
            ICalendarProperty::PercentComplete => "PERCENT-COMPLETE",
            ICalendarProperty::Priority => "PRIORITY",
            ICalendarProperty::Resources => "RESOURCES",
            ICalendarProperty::Status => "STATUS",
            ICalendarProperty::Summary => "SUMMARY",
            ICalendarProperty::Completed => "COMPLETED",
            ICalendarProperty::Dtend => "DTEND",
            ICalendarProperty::Due => "DUE",
            ICalendarProperty::Dtstart => "DTSTART",
            ICalendarProperty::Duration => "DURATION",
            ICalendarProperty::Freebusy => "FREEBUSY",
            ICalendarProperty::Transp => "TRANSP",
            ICalendarProperty::Tzid => "TZID",
            ICalendarProperty::Tzname => "TZNAME",
            ICalendarProperty::Tzoffsetfrom => "TZOFFSETFROM",
            ICalendarProperty::Tzoffsetto => "TZOFFSETTO",
            ICalendarProperty::Tzurl => "TZURL",
            ICalendarProperty::Attendee => "ATTENDEE",
            ICalendarProperty::Contact => "CONTACT",
            ICalendarProperty::Organizer => "ORGANIZER",
            ICalendarProperty::RecurrenceId => "RECURRENCE-ID",
            ICalendarProperty::RelatedTo => "RELATED-TO",
            ICalendarProperty::Url => "URL",
            ICalendarProperty::Uid => "UID",
            ICalendarProperty::Exdate => "EXDATE",
            ICalendarProperty::Exrule => "EXRULE",
            ICalendarProperty::Rdate => "RDATE",
            ICalendarProperty::Rrule => "RRULE",
            ICalendarProperty::Action => "ACTION",
            ICalendarProperty::Repeat => "REPEAT",
            ICalendarProperty::Trigger => "TRIGGER",
            ICalendarProperty::Created => "CREATED",
            ICalendarProperty::Dtstamp => "DTSTAMP",
            ICalendarProperty::LastModified => "LAST-MODIFIED",
            ICalendarProperty::Sequence => "SEQUENCE",
            ICalendarProperty::RequestStatus => "REQUEST-STATUS",
            ICalendarProperty::Xml => "XML",
            ICalendarProperty::Tzuntil => "TZUNTIL",
            ICalendarProperty::TzidAliasOf => "TZID-ALIAS-OF",
            ICalendarProperty::Busytype => "BUSYTYPE",
            ICalendarProperty::Name => "NAME",
            ICalendarProperty::RefreshInterval => "REFRESH-INTERVAL",
            ICalendarProperty::Source => "SOURCE",
            ICalendarProperty::Color => "COLOR",
            ICalendarProperty::Image => "IMAGE",
            ICalendarProperty::Conference => "CONFERENCE",
            ICalendarProperty::CalendarAddress => "CALENDAR-ADDRESS",
            ICalendarProperty::LocationType => "LOCATION-TYPE",
            ICalendarProperty::ParticipantType => "PARTICIPANT-TYPE",
            ICalendarProperty::ResourceType => "RESOURCE-TYPE",
            ICalendarProperty::StructuredData => "STRUCTURED-DATA",
            ICalendarProperty::StyledDescription => "STYLED-DESCRIPTION",
            ICalendarProperty::Acknowledged => "ACKNOWLEDGED",
            ICalendarProperty::Proximity => "PROXIMITY",
            ICalendarProperty::Concept => "CONCEPT",
            ICalendarProperty::Link => "LINK",
            ICalendarProperty::Refid => "REFID",
            ICalendarProperty::Begin => "BEGIN",
            ICalendarProperty::End => "END",
            ICalendarProperty::Coordinates => "COORDINATES",
            ICalendarProperty::ShowWithoutTime => "SHOW-WITHOUT-TIME",
            ICalendarProperty::Jsid => "JSID",
            ICalendarProperty::Jsprop => "JSPROP",
            ICalendarProperty::EstimatedDuration => "ESTIMATED-DURATION",
            ICalendarProperty::Reason => "REASON",
            ICalendarProperty::Substate => "SUBSTATE",
            ICalendarProperty::TaskMode => "TASK-MODE",
            ICalendarProperty::Other(s) => s.as_str(),
        }
    }
}

impl ICalendarParameterName {
    pub fn try_parse(input: &[u8]) -> Option<Self> {
        hashify::tiny_map_ignore_case!(input,
                b"ALTREP" => ICalendarParameterName::Altrep,
                b"CN" => ICalendarParameterName::Cn,
                b"CUTYPE" => ICalendarParameterName::Cutype,
                b"DELEGATED-FROM" => ICalendarParameterName::DelegatedFrom,
                b"DELEGATED-TO" => ICalendarParameterName::DelegatedTo,
                b"DIR" => ICalendarParameterName::Dir,
                b"FMTTYPE" => ICalendarParameterName::Fmttype,
                b"FBTYPE" => ICalendarParameterName::Fbtype,
                b"LANGUAGE" => ICalendarParameterName::Language,
                b"MEMBER" => ICalendarParameterName::Member,
                b"PARTSTAT" => ICalendarParameterName::Partstat,
                b"RANGE" => ICalendarParameterName::Range,
                b"RELATED" => ICalendarParameterName::Related,
                b"RELTYPE" => ICalendarParameterName::Reltype,
                b"ROLE" => ICalendarParameterName::Role,
                b"RSVP" => ICalendarParameterName::Rsvp,
                b"SCHEDULE-AGENT" => ICalendarParameterName::ScheduleAgent,
                b"SCHEDULE-FORCE-SEND" => ICalendarParameterName::ScheduleForceSend,
                b"SCHEDULE-STATUS" => ICalendarParameterName::ScheduleStatus,
                b"SENT-BY" => ICalendarParameterName::SentBy,
                b"TZID" => ICalendarParameterName::Tzid,
                b"VALUE" => ICalendarParameterName::Value,
                b"DISPLAY" => ICalendarParameterName::Display,
                b"EMAIL" => ICalendarParameterName::Email,
                b"FEATURE" => ICalendarParameterName::Feature,
                b"LABEL" => ICalendarParameterName::Label,
                b"SIZE" => ICalendarParameterName::Size,
                b"FILENAME" => ICalendarParameterName::Filename,
                b"MANAGED-ID" => ICalendarParameterName::ManagedId,
                b"ORDER" => ICalendarParameterName::Order,
                b"SCHEMA" => ICalendarParameterName::Schema,
                b"DERIVED" => ICalendarParameterName::Derived,
                b"GAP" => ICalendarParameterName::Gap,
                b"LINKREL" => ICalendarParameterName::Linkrel,
                b"JSPTR" => ICalendarParameterName::Jsptr,
                b"JSID" => ICalendarParameterName::Jsid,
        )
    }

    pub fn as_str(&self) -> &str {
        match self {
            ICalendarParameterName::Altrep => "ALTREP",
            ICalendarParameterName::Cn => "CN",
            ICalendarParameterName::Cutype => "CUTYPE",
            ICalendarParameterName::DelegatedFrom => "DELEGATED-FROM",
            ICalendarParameterName::DelegatedTo => "DELEGATED-TO",
            ICalendarParameterName::Dir => "DIR",
            ICalendarParameterName::Fmttype => "FMTTYPE",
            ICalendarParameterName::Fbtype => "FBTYPE",
            ICalendarParameterName::Language => "LANGUAGE",
            ICalendarParameterName::Member => "MEMBER",
            ICalendarParameterName::Partstat => "PARTSTAT",
            ICalendarParameterName::Range => "RANGE",
            ICalendarParameterName::Related => "RELATED",
            ICalendarParameterName::Reltype => "RELTYPE",
            ICalendarParameterName::Role => "ROLE",
            ICalendarParameterName::Rsvp => "RSVP",
            ICalendarParameterName::ScheduleAgent => "SCHEDULE-AGENT",
            ICalendarParameterName::ScheduleForceSend => "SCHEDULE-FORCE-SEND",
            ICalendarParameterName::ScheduleStatus => "SCHEDULE-STATUS",
            ICalendarParameterName::SentBy => "SENT-BY",
            ICalendarParameterName::Tzid => "TZID",
            ICalendarParameterName::Value => "VALUE",
            ICalendarParameterName::Display => "DISPLAY",
            ICalendarParameterName::Email => "EMAIL",
            ICalendarParameterName::Feature => "FEATURE",
            ICalendarParameterName::Label => "LABEL",
            ICalendarParameterName::Size => "SIZE",
            ICalendarParameterName::Filename => "FILENAME",
            ICalendarParameterName::ManagedId => "MANAGED-ID",
            ICalendarParameterName::Order => "ORDER",
            ICalendarParameterName::Schema => "SCHEMA",
            ICalendarParameterName::Derived => "DERIVED",
            ICalendarParameterName::Gap => "GAP",
            ICalendarParameterName::Linkrel => "LINKREL",
            ICalendarParameterName::Jsptr => "JSPTR",
            ICalendarParameterName::Jsid => "JSID",
            ICalendarParameterName::Other(name) => name.as_str(),
        }
    }

    pub fn parse(input: &str) -> Self {
        Self::try_parse(input.as_bytes())
            .unwrap_or_else(|| ICalendarParameterName::Other(input.to_string()))
    }
}

impl IanaParse for ICalendarFrequency {
    fn parse(value: &[u8]) -> Option<Self> {
        hashify::tiny_map_ignore_case!(value,
            b"SECONDLY" => ICalendarFrequency::Secondly,
            b"MINUTELY" => ICalendarFrequency::Minutely,
            b"HOURLY" => ICalendarFrequency::Hourly,
            b"DAILY" => ICalendarFrequency::Daily,
            b"WEEKLY" => ICalendarFrequency::Weekly,
            b"MONTHLY" => ICalendarFrequency::Monthly,
            b"YEARLY" => ICalendarFrequency::Yearly,
        )
    }
}

impl IanaString for ICalendarFrequency {
    fn as_str(&self) -> &'static str {
        match self {
            ICalendarFrequency::Secondly => "SECONDLY",
            ICalendarFrequency::Minutely => "MINUTELY",
            ICalendarFrequency::Hourly => "HOURLY",
            ICalendarFrequency::Daily => "DAILY",
            ICalendarFrequency::Weekly => "WEEKLY",
            ICalendarFrequency::Monthly => "MONTHLY",
            ICalendarFrequency::Yearly => "YEARLY",
        }
    }
}

impl IanaParse for ICalendarSkip {
    fn parse(value: &[u8]) -> Option<Self> {
        hashify::tiny_map_ignore_case!(value,
            b"OMIT" => ICalendarSkip::Omit,
            b"BACKWARD" => ICalendarSkip::Backward,
            b"FORWARD" => ICalendarSkip::Forward,
        )
    }
}

impl IanaString for ICalendarSkip {
    fn as_str(&self) -> &'static str {
        match self {
            ICalendarSkip::Omit => "OMIT",
            ICalendarSkip::Backward => "BACKWARD",
            ICalendarSkip::Forward => "FORWARD",
        }
    }
}

impl IanaParse for ICalendarWeekday {
    fn parse(value: &[u8]) -> Option<Self> {
        hashify::tiny_map_ignore_case!(value,
            b"SU" => ICalendarWeekday::Sunday,
            b"MO" => ICalendarWeekday::Monday,
            b"TU" => ICalendarWeekday::Tuesday,
            b"WE" => ICalendarWeekday::Wednesday,
            b"TH" => ICalendarWeekday::Thursday,
            b"FR" => ICalendarWeekday::Friday,
            b"SA" => ICalendarWeekday::Saturday,
        )
    }
}

impl IanaString for ICalendarWeekday {
    fn as_str(&self) -> &'static str {
        match self {
            ICalendarWeekday::Sunday => "SU",
            ICalendarWeekday::Monday => "MO",
            ICalendarWeekday::Tuesday => "TU",
            ICalendarWeekday::Wednesday => "WE",
            ICalendarWeekday::Thursday => "TH",
            ICalendarWeekday::Friday => "FR",
            ICalendarWeekday::Saturday => "SA",
        }
    }
}

impl From<ICalendarWeekday> for chrono::Weekday {
    fn from(value: ICalendarWeekday) -> Self {
        match value {
            ICalendarWeekday::Sunday => chrono::Weekday::Sun,
            ICalendarWeekday::Monday => chrono::Weekday::Mon,
            ICalendarWeekday::Tuesday => chrono::Weekday::Tue,
            ICalendarWeekday::Wednesday => chrono::Weekday::Wed,
            ICalendarWeekday::Thursday => chrono::Weekday::Thu,
            ICalendarWeekday::Friday => chrono::Weekday::Fri,
            ICalendarWeekday::Saturday => chrono::Weekday::Sat,
        }
    }
}

impl IanaParse for ICalendarAction {
    fn parse(value: &[u8]) -> Option<Self> {
        hashify::tiny_map_ignore_case!(value,
            "AUDIO" => ICalendarAction::Audio,
            "DISPLAY" => ICalendarAction::Display,
            "EMAIL" => ICalendarAction::Email,
            "PROCEDURE" => ICalendarAction::Procedure,
        )
    }
}

impl IanaString for ICalendarAction {
    fn as_str(&self) -> &'static str {
        match self {
            ICalendarAction::Audio => "AUDIO",
            ICalendarAction::Display => "DISPLAY",
            ICalendarAction::Email => "EMAIL",
            ICalendarAction::Procedure => "PROCEDURE",
        }
    }
}

impl IanaParse for ICalendarUserTypes {
    fn parse(value: &[u8]) -> Option<Self> {
        hashify::tiny_map_ignore_case!(value,
            "INDIVIDUAL" => ICalendarUserTypes::Individual,
            "GROUP" => ICalendarUserTypes::Group,
            "RESOURCE" => ICalendarUserTypes::Resource,
            "ROOM" => ICalendarUserTypes::Room,
            "UNKNOWN" => ICalendarUserTypes::Unknown,
        )
    }
}

impl IanaString for ICalendarUserTypes {
    fn as_str(&self) -> &'static str {
        match self {
            ICalendarUserTypes::Individual => "INDIVIDUAL",
            ICalendarUserTypes::Group => "GROUP",
            ICalendarUserTypes::Resource => "RESOURCE",
            ICalendarUserTypes::Room => "ROOM",
            ICalendarUserTypes::Unknown => "UNKNOWN",
        }
    }
}

impl IanaParse for ICalendarClassification {
    fn parse(value: &[u8]) -> Option<Self> {
        hashify::tiny_map_ignore_case!(value,
            "PUBLIC" => ICalendarClassification::Public,
            "PRIVATE" => ICalendarClassification::Private,
            "CONFIDENTIAL" => ICalendarClassification::Confidential,
        )
    }
}

impl IanaString for ICalendarClassification {
    fn as_str(&self) -> &'static str {
        match self {
            ICalendarClassification::Public => "PUBLIC",
            ICalendarClassification::Private => "PRIVATE",
            ICalendarClassification::Confidential => "CONFIDENTIAL",
        }
    }
}

impl IanaParse for ICalendarComponentType {
    fn parse(value: &[u8]) -> Option<Self> {
        hashify::tiny_map_ignore_case!(value,
            "VCALENDAR" => ICalendarComponentType::VCalendar,
            "VEVENT" => ICalendarComponentType::VEvent,
            "VTODO" => ICalendarComponentType::VTodo,
            "VJOURNAL" => ICalendarComponentType::VJournal,
            "VFREEBUSY" => ICalendarComponentType::VFreebusy,
            "VTIMEZONE" => ICalendarComponentType::VTimezone,
            "VALARM" => ICalendarComponentType::VAlarm,
            "STANDARD" => ICalendarComponentType::Standard,
            "DAYLIGHT" => ICalendarComponentType::Daylight,
            "VAVAILABILITY" => ICalendarComponentType::VAvailability,
            "AVAILABLE" => ICalendarComponentType::Available,
            "PARTICIPANT" => ICalendarComponentType::Participant,
            "VLOCATION" => ICalendarComponentType::VLocation,
            "VRESOURCE" => ICalendarComponentType::VResource,
            "VSTATUS" => ICalendarComponentType::VStatus
        )
    }
}

impl ICalendarComponentType {
    pub fn as_str(&self) -> &str {
        match self {
            ICalendarComponentType::VCalendar => "VCALENDAR",
            ICalendarComponentType::VEvent => "VEVENT",
            ICalendarComponentType::VTodo => "VTODO",
            ICalendarComponentType::VJournal => "VJOURNAL",
            ICalendarComponentType::VFreebusy => "VFREEBUSY",
            ICalendarComponentType::VTimezone => "VTIMEZONE",
            ICalendarComponentType::VAlarm => "VALARM",
            ICalendarComponentType::Standard => "STANDARD",
            ICalendarComponentType::Daylight => "DAYLIGHT",
            ICalendarComponentType::VAvailability => "VAVAILABILITY",
            ICalendarComponentType::Available => "AVAILABLE",
            ICalendarComponentType::Participant => "PARTICIPANT",
            ICalendarComponentType::VLocation => "VLOCATION",
            ICalendarComponentType::VResource => "VRESOURCE",
            ICalendarComponentType::VStatus => "VSTATUS",
            ICalendarComponentType::Other(s) => s.as_str(),
        }
    }
}

impl IanaParse for ICalendarDisplayType {
    fn parse(value: &[u8]) -> Option<Self> {
        hashify::tiny_map_ignore_case!(value,
            "BADGE" => ICalendarDisplayType::Badge,
            "GRAPHIC" => ICalendarDisplayType::Graphic,
            "FULLSIZE" => ICalendarDisplayType::Fullsize,
            "THUMBNAIL" => ICalendarDisplayType::Thumbnail,
        )
    }
}

impl IanaString for ICalendarDisplayType {
    fn as_str(&self) -> &'static str {
        match self {
            ICalendarDisplayType::Badge => "BADGE",
            ICalendarDisplayType::Graphic => "GRAPHIC",
            ICalendarDisplayType::Fullsize => "FULLSIZE",
            ICalendarDisplayType::Thumbnail => "THUMBNAIL",
        }
    }
}

impl IanaParse for ICalendarFeatureType {
    fn parse(value: &[u8]) -> Option<Self> {
        hashify::tiny_map_ignore_case!(value,
            "AUDIO" => ICalendarFeatureType::Audio,
            "CHAT" => ICalendarFeatureType::Chat,
            "FEED" => ICalendarFeatureType::Feed,
            "MODERATOR" => ICalendarFeatureType::Moderator,
            "PHONE" => ICalendarFeatureType::Phone,
            "SCREEN" => ICalendarFeatureType::Screen,
            "VIDEO" => ICalendarFeatureType::Video,
        )
    }
}

impl IanaString for ICalendarFeatureType {
    fn as_str(&self) -> &'static str {
        match self {
            ICalendarFeatureType::Audio => "AUDIO",
            ICalendarFeatureType::Chat => "CHAT",
            ICalendarFeatureType::Feed => "FEED",
            ICalendarFeatureType::Moderator => "MODERATOR",
            ICalendarFeatureType::Phone => "PHONE",
            ICalendarFeatureType::Screen => "SCREEN",
            ICalendarFeatureType::Video => "VIDEO",
        }
    }
}

impl IanaParse for ICalendarFreeBusyType {
    fn parse(value: &[u8]) -> Option<Self> {
        hashify::tiny_map_ignore_case!(value,
            "FREE" => ICalendarFreeBusyType::Free,
            "BUSY" => ICalendarFreeBusyType::Busy,
            "BUSY-UNAVAILABLE" => ICalendarFreeBusyType::BusyUnavailable,
            "BUSY-TENTATIVE" => ICalendarFreeBusyType::BusyTentative,
        )
    }
}

impl IanaString for ICalendarFreeBusyType {
    fn as_str(&self) -> &'static str {
        match self {
            ICalendarFreeBusyType::Free => "FREE",
            ICalendarFreeBusyType::Busy => "BUSY",
            ICalendarFreeBusyType::BusyUnavailable => "BUSY-UNAVAILABLE",
            ICalendarFreeBusyType::BusyTentative => "BUSY-TENTATIVE",
        }
    }
}

impl IanaParse for ICalendarMethod {
    fn parse(value: &[u8]) -> Option<Self> {
        hashify::tiny_map_ignore_case!(value,
            "PUBLISH" => ICalendarMethod::Publish,
            "REQUEST" => ICalendarMethod::Request,
            "REPLY" => ICalendarMethod::Reply,
            "ADD" => ICalendarMethod::Add,
            "CANCEL" => ICalendarMethod::Cancel,
            "REFRESH" => ICalendarMethod::Refresh,
            "COUNTER" => ICalendarMethod::Counter,
            "DECLINECOUNTER" => ICalendarMethod::Declinecounter,
        )
    }
}

impl IanaString for ICalendarMethod {
    fn as_str(&self) -> &'static str {
        match self {
            ICalendarMethod::Publish => "PUBLISH",
            ICalendarMethod::Request => "REQUEST",
            ICalendarMethod::Reply => "REPLY",
            ICalendarMethod::Add => "ADD",
            ICalendarMethod::Cancel => "CANCEL",
            ICalendarMethod::Refresh => "REFRESH",
            ICalendarMethod::Counter => "COUNTER",
            ICalendarMethod::Declinecounter => "DECLINECOUNTER",
        }
    }
}

impl IanaParse for ICalendarRelated {
    fn parse(value: &[u8]) -> Option<Self> {
        hashify::tiny_map_ignore_case!(value,
            "START" => ICalendarRelated::Start,
            "END" => ICalendarRelated::End,
        )
    }
}

impl IanaString for ICalendarRelated {
    fn as_str(&self) -> &'static str {
        match self {
            ICalendarRelated::Start => "START",
            ICalendarRelated::End => "END",
        }
    }
}

impl IanaParse for ICalendarParticipantType {
    fn parse(value: &[u8]) -> Option<Self> {
        hashify::tiny_map_ignore_case!(value,
            "ACTIVE" => ICalendarParticipantType::Active,
            "INACTIVE" => ICalendarParticipantType::Inactive,
            "SPONSOR" => ICalendarParticipantType::Sponsor,
            "CONTACT" => ICalendarParticipantType::Contact,
            "BOOKING-CONTACT" => ICalendarParticipantType::BookingContact,
            "EMERGENCY-CONTACT" => ICalendarParticipantType::EmergencyContact,
            "PUBLICITY-CONTACT" => ICalendarParticipantType::PublicityContact,
            "PLANNER-CONTACT" => ICalendarParticipantType::PlannerContact,
            "PERFORMER" => ICalendarParticipantType::Performer,
            "SPEAKER" => ICalendarParticipantType::Speaker,
        )
    }
}

impl IanaString for ICalendarParticipantType {
    fn as_str(&self) -> &'static str {
        match self {
            ICalendarParticipantType::Active => "ACTIVE",
            ICalendarParticipantType::Inactive => "INACTIVE",
            ICalendarParticipantType::Sponsor => "SPONSOR",
            ICalendarParticipantType::Contact => "CONTACT",
            ICalendarParticipantType::BookingContact => "BOOKING-CONTACT",
            ICalendarParticipantType::EmergencyContact => "EMERGENCY-CONTACT",
            ICalendarParticipantType::PublicityContact => "PUBLICITY-CONTACT",
            ICalendarParticipantType::PlannerContact => "PLANNER-CONTACT",
            ICalendarParticipantType::Performer => "PERFORMER",
            ICalendarParticipantType::Speaker => "SPEAKER",
        }
    }
}

impl IanaParse for ICalendarParticipationRole {
    fn parse(value: &[u8]) -> Option<Self> {
        hashify::tiny_map_ignore_case!(value,
            "CHAIR" => ICalendarParticipationRole::Chair,
            "REQ-PARTICIPANT" => ICalendarParticipationRole::ReqParticipant,
            "OPT-PARTICIPANT" => ICalendarParticipationRole::OptParticipant,
            "NON-PARTICIPANT" => ICalendarParticipationRole::NonParticipant,
        )
    }
}

impl IanaString for ICalendarParticipationRole {
    fn as_str(&self) -> &'static str {
        match self {
            ICalendarParticipationRole::Chair => "CHAIR",
            ICalendarParticipationRole::ReqParticipant => "REQ-PARTICIPANT",
            ICalendarParticipationRole::OptParticipant => "OPT-PARTICIPANT",
            ICalendarParticipationRole::NonParticipant => "NON-PARTICIPANT",
        }
    }
}

impl IanaParse for ICalendarStatus {
    fn parse(value: &[u8]) -> Option<Self> {
        hashify::tiny_map_ignore_case!(value,
            "TENTATIVE" => ICalendarStatus::Tentative,
            "CONFIRMED" => ICalendarStatus::Confirmed,
            "CANCELLED" => ICalendarStatus::Cancelled,
            "NEEDS-ACTION" => ICalendarStatus::NeedsAction,
            "COMPLETED" => ICalendarStatus::Completed,
            "IN-PROCESS" => ICalendarStatus::InProcess,
            "DRAFT" => ICalendarStatus::Draft,
            "FINAL" => ICalendarStatus::Final,
            "FAILED" => ICalendarStatus::Failed,
            "PENDING" => ICalendarStatus::Pending
        )
    }
}

impl IanaString for ICalendarStatus {
    fn as_str(&self) -> &'static str {
        match self {
            ICalendarStatus::Tentative => "TENTATIVE",
            ICalendarStatus::Confirmed => "CONFIRMED",
            ICalendarStatus::Cancelled => "CANCELLED",
            ICalendarStatus::NeedsAction => "NEEDS-ACTION",
            ICalendarStatus::Completed => "COMPLETED",
            ICalendarStatus::InProcess => "IN-PROCESS",
            ICalendarStatus::Draft => "DRAFT",
            ICalendarStatus::Final => "FINAL",
            ICalendarStatus::Failed => "FAILED",
            ICalendarStatus::Pending => "PENDING",
        }
    }
}

impl IanaParse for ICalendarParticipationStatus {
    fn parse(value: &[u8]) -> Option<Self> {
        hashify::tiny_map_ignore_case!(value,
            "NEEDS-ACTION" => ICalendarParticipationStatus::NeedsAction,
            "ACCEPTED" => ICalendarParticipationStatus::Accepted,
            "DECLINED" => ICalendarParticipationStatus::Declined,
            "TENTATIVE" => ICalendarParticipationStatus::Tentative,
            "DELEGATED" => ICalendarParticipationStatus::Delegated,
            "COMPLETED" => ICalendarParticipationStatus::Completed,
            "IN-PROCESS" => ICalendarParticipationStatus::InProcess,
            "FAILED" => ICalendarParticipationStatus::Failed
        )
    }
}

impl IanaString for ICalendarParticipationStatus {
    fn as_str(&self) -> &'static str {
        match self {
            ICalendarParticipationStatus::NeedsAction => "NEEDS-ACTION",
            ICalendarParticipationStatus::Accepted => "ACCEPTED",
            ICalendarParticipationStatus::Declined => "DECLINED",
            ICalendarParticipationStatus::Tentative => "TENTATIVE",
            ICalendarParticipationStatus::Delegated => "DELEGATED",
            ICalendarParticipationStatus::Completed => "COMPLETED",
            ICalendarParticipationStatus::InProcess => "IN-PROCESS",
            ICalendarParticipationStatus::Failed => "FAILED",
        }
    }
}

impl IanaParse for ICalendarProximityValue {
    fn parse(value: &[u8]) -> Option<Self> {
        hashify::tiny_map_ignore_case!(value,
            "ARRIVE" => ICalendarProximityValue::Arrive,
            "DEPART" => ICalendarProximityValue::Depart,
            "CONNECT" => ICalendarProximityValue::Connect,
            "DISCONNECT" => ICalendarProximityValue::Disconnect,
        )
    }
}

impl IanaString for ICalendarProximityValue {
    fn as_str(&self) -> &'static str {
        match self {
            ICalendarProximityValue::Arrive => "ARRIVE",
            ICalendarProximityValue::Depart => "DEPART",
            ICalendarProximityValue::Connect => "CONNECT",
            ICalendarProximityValue::Disconnect => "DISCONNECT",
        }
    }
}

impl IanaParse for ICalendarRelationshipType {
    fn parse(value: &[u8]) -> Option<Self> {
        hashify::tiny_map_ignore_case!(value,
            "CHILD" => ICalendarRelationshipType::Child,
            "PARENT" => ICalendarRelationshipType::Parent,
            "SIBLING" => ICalendarRelationshipType::Sibling,
            "SNOOZE" => ICalendarRelationshipType::Snooze,
            "CONCEPT" => ICalendarRelationshipType::Concept,
            "DEPENDS-ON" => ICalendarRelationshipType::DependsOn,
            "FINISHTOFINISH" => ICalendarRelationshipType::Finishtofinish,
            "FINISHTOSTART" => ICalendarRelationshipType::Finishtostart,
            "FIRST" => ICalendarRelationshipType::First,
            "NEXT" => ICalendarRelationshipType::Next,
            "REFID" => ICalendarRelationshipType::Refid,
            "STARTTOFINISH" => ICalendarRelationshipType::Starttofinish,
            "STARTTOSTART" => ICalendarRelationshipType::Starttostart,
        )
    }
}

impl IanaString for ICalendarRelationshipType {
    fn as_str(&self) -> &'static str {
        match self {
            ICalendarRelationshipType::Child => "CHILD",
            ICalendarRelationshipType::Parent => "PARENT",
            ICalendarRelationshipType::Sibling => "SIBLING",
            ICalendarRelationshipType::Snooze => "SNOOZE",
            ICalendarRelationshipType::Concept => "CONCEPT",
            ICalendarRelationshipType::DependsOn => "DEPENDS-ON",
            ICalendarRelationshipType::Finishtofinish => "FINISHTOFINISH",
            ICalendarRelationshipType::Finishtostart => "FINISHTOSTART",
            ICalendarRelationshipType::First => "FIRST",
            ICalendarRelationshipType::Next => "NEXT",
            ICalendarRelationshipType::Refid => "REFID",
            ICalendarRelationshipType::Starttofinish => "STARTTOFINISH",
            ICalendarRelationshipType::Starttostart => "STARTTOSTART",
        }
    }
}

impl IanaParse for ICalendarResourceType {
    fn parse(value: &[u8]) -> Option<Self> {
        hashify::tiny_map_ignore_case!(value,
            "PROJECTOR" => ICalendarResourceType::Projector,
            "ROOM" => ICalendarResourceType::Room,
            "REMOTE-CONFERENCE-AUDIO" => ICalendarResourceType::RemoteConferenceAudio,
            "REMOTE-CONFERENCE-VIDEO" => ICalendarResourceType::RemoteConferenceVideo,
        )
    }
}

impl IanaString for ICalendarResourceType {
    fn as_str(&self) -> &'static str {
        match self {
            ICalendarResourceType::Projector => "PROJECTOR",
            ICalendarResourceType::Room => "ROOM",
            ICalendarResourceType::RemoteConferenceAudio => "REMOTE-CONFERENCE-AUDIO",
            ICalendarResourceType::RemoteConferenceVideo => "REMOTE-CONFERENCE-VIDEO",
        }
    }
}

impl IanaParse for ICalendarScheduleAgentValue {
    fn parse(value: &[u8]) -> Option<Self> {
        hashify::tiny_map_ignore_case!(value,
            "SERVER" => ICalendarScheduleAgentValue::Server,
            "CLIENT" => ICalendarScheduleAgentValue::Client,
            "NONE" => ICalendarScheduleAgentValue::None,
        )
    }
}

impl IanaString for ICalendarScheduleAgentValue {
    fn as_str(&self) -> &'static str {
        match self {
            ICalendarScheduleAgentValue::Server => "SERVER",
            ICalendarScheduleAgentValue::Client => "CLIENT",
            ICalendarScheduleAgentValue::None => "NONE",
        }
    }
}

impl IanaParse for ICalendarScheduleForceSendValue {
    fn parse(value: &[u8]) -> Option<Self> {
        hashify::tiny_map_ignore_case!(value,
            "REQUEST" => ICalendarScheduleForceSendValue::Request,
            "REPLY" => ICalendarScheduleForceSendValue::Reply,
        )
    }
}

impl IanaString for ICalendarScheduleForceSendValue {
    fn as_str(&self) -> &'static str {
        match self {
            ICalendarScheduleForceSendValue::Request => "REQUEST",
            ICalendarScheduleForceSendValue::Reply => "REPLY",
        }
    }
}

impl IanaParse for ICalendarValueType {
    fn parse(value: &[u8]) -> Option<Self> {
        hashify::tiny_map_ignore_case!(value,
            "BINARY" => ICalendarValueType::Binary,
            "BOOLEAN" => ICalendarValueType::Boolean,
            "CAL-ADDRESS" => ICalendarValueType::CalAddress,
            "DATE" => ICalendarValueType::Date,
            "DATE-TIME" => ICalendarValueType::DateTime,
            "DURATION" => ICalendarValueType::Duration,
            "FLOAT" => ICalendarValueType::Float,
            "INTEGER" => ICalendarValueType::Integer,
            "PERIOD" => ICalendarValueType::Period,
            "RECUR" => ICalendarValueType::Recur,
            "TEXT" => ICalendarValueType::Text,
            "TIME" => ICalendarValueType::Time,
            "UNKNOWN" => ICalendarValueType::Unknown,
            "URI" => ICalendarValueType::Uri,
            "UTC-OFFSET" => ICalendarValueType::UtcOffset,
            "XML-REFERENCE" => ICalendarValueType::XmlReference,
            "UID" => ICalendarValueType::Uid,
        )
    }
}

impl IanaString for ICalendarValueType {
    fn as_str(&self) -> &'static str {
        match self {
            ICalendarValueType::Binary => "BINARY",
            ICalendarValueType::Boolean => "BOOLEAN",
            ICalendarValueType::CalAddress => "CAL-ADDRESS",
            ICalendarValueType::Date => "DATE",
            ICalendarValueType::DateTime => "DATE-TIME",
            ICalendarValueType::Duration => "DURATION",
            ICalendarValueType::Float => "FLOAT",
            ICalendarValueType::Integer => "INTEGER",
            ICalendarValueType::Period => "PERIOD",
            ICalendarValueType::Recur => "RECUR",
            ICalendarValueType::Text => "TEXT",
            ICalendarValueType::Time => "TIME",
            ICalendarValueType::Unknown => "UNKNOWN",
            ICalendarValueType::Uri => "URI",
            ICalendarValueType::UtcOffset => "UTC-OFFSET",
            ICalendarValueType::XmlReference => "XML-REFERENCE",
            ICalendarValueType::Uid => "UID",
        }
    }
}

impl IanaParse for ICalendarTransparency {
    fn parse(value: &[u8]) -> Option<Self> {
        hashify::tiny_map_ignore_case!(value,
            "OPAQUE" => ICalendarTransparency::Opaque,
            "TRANSPARENT" => ICalendarTransparency::Transparent,
        )
    }
}

impl IanaString for ICalendarTransparency {
    fn as_str(&self) -> &'static str {
        match self {
            ICalendarTransparency::Opaque => "OPAQUE",
            ICalendarTransparency::Transparent => "TRANSPARENT",
        }
    }
}

impl ICalendarProperty {
    // Returns the default value type and whether the property is multi-valued.
    pub(crate) fn default_types(&self) -> (ValueType, ValueSeparator) {
        match self {
            ICalendarProperty::Calscale => (ValueType::CalendarScale, ValueSeparator::None),
            ICalendarProperty::Method => (ValueType::Method, ValueSeparator::None),
            ICalendarProperty::Prodid => (
                ValueType::Ical(ICalendarValueType::Text),
                ValueSeparator::None,
            ),
            ICalendarProperty::Version => (
                ValueType::Ical(ICalendarValueType::Text),
                ValueSeparator::None,
            ),
            ICalendarProperty::Attach => (
                ValueType::Ical(ICalendarValueType::Uri),
                ValueSeparator::None,
            ),
            ICalendarProperty::Categories => (
                ValueType::Ical(ICalendarValueType::Text),
                ValueSeparator::Comma,
            ),
            ICalendarProperty::Class => (ValueType::Classification, ValueSeparator::None),
            ICalendarProperty::Comment => (
                ValueType::Ical(ICalendarValueType::Text),
                ValueSeparator::None,
            ),
            ICalendarProperty::Description => (
                ValueType::Ical(ICalendarValueType::Text),
                ValueSeparator::None,
            ),
            ICalendarProperty::Geo => (
                ValueType::Ical(ICalendarValueType::Float),
                ValueSeparator::Semicolon,
            ),
            ICalendarProperty::Location => (
                ValueType::Ical(ICalendarValueType::Text),
                ValueSeparator::None,
            ),
            ICalendarProperty::PercentComplete => (
                ValueType::Ical(ICalendarValueType::Integer),
                ValueSeparator::None,
            ),
            ICalendarProperty::Priority => (
                ValueType::Ical(ICalendarValueType::Integer),
                ValueSeparator::None,
            ),
            ICalendarProperty::Resources => (
                ValueType::Ical(ICalendarValueType::Text),
                ValueSeparator::Comma,
            ),
            ICalendarProperty::Status => (ValueType::Status, ValueSeparator::None),
            ICalendarProperty::Summary => (
                ValueType::Ical(ICalendarValueType::Text),
                ValueSeparator::None,
            ),
            ICalendarProperty::Completed => (
                ValueType::Ical(ICalendarValueType::DateTime),
                ValueSeparator::None,
            ),
            ICalendarProperty::Dtend => (
                ValueType::Ical(ICalendarValueType::DateTime),
                ValueSeparator::None,
            ),
            ICalendarProperty::Due => (
                ValueType::Ical(ICalendarValueType::DateTime),
                ValueSeparator::None,
            ),
            ICalendarProperty::Dtstart => (
                ValueType::Ical(ICalendarValueType::DateTime),
                ValueSeparator::None,
            ),
            ICalendarProperty::Duration => (
                ValueType::Ical(ICalendarValueType::Duration),
                ValueSeparator::None,
            ),
            ICalendarProperty::Freebusy => (
                ValueType::Ical(ICalendarValueType::Period),
                ValueSeparator::None,
            ),
            ICalendarProperty::Transp => (ValueType::Transparency, ValueSeparator::None),
            ICalendarProperty::Tzid => (
                ValueType::Ical(ICalendarValueType::Text),
                ValueSeparator::None,
            ),
            ICalendarProperty::Tzname => (
                ValueType::Ical(ICalendarValueType::Text),
                ValueSeparator::None,
            ),
            ICalendarProperty::Tzoffsetfrom => (
                ValueType::Ical(ICalendarValueType::UtcOffset),
                ValueSeparator::None,
            ),
            ICalendarProperty::Tzoffsetto => (
                ValueType::Ical(ICalendarValueType::UtcOffset),
                ValueSeparator::None,
            ),
            ICalendarProperty::Tzurl => (
                ValueType::Ical(ICalendarValueType::Uri),
                ValueSeparator::None,
            ),
            ICalendarProperty::Attendee => (
                ValueType::Ical(ICalendarValueType::Uri),
                ValueSeparator::None,
            ),
            ICalendarProperty::Contact => (
                ValueType::Ical(ICalendarValueType::Text),
                ValueSeparator::None,
            ),
            ICalendarProperty::Organizer => (
                ValueType::Ical(ICalendarValueType::Uri),
                ValueSeparator::None,
            ),
            ICalendarProperty::RecurrenceId => (
                ValueType::Ical(ICalendarValueType::DateTime),
                ValueSeparator::None,
            ),
            ICalendarProperty::RelatedTo => (
                ValueType::Ical(ICalendarValueType::Text),
                ValueSeparator::None,
            ),
            ICalendarProperty::Url => (
                ValueType::Ical(ICalendarValueType::Uri),
                ValueSeparator::None,
            ),
            ICalendarProperty::Uid => (
                ValueType::Ical(ICalendarValueType::Text),
                ValueSeparator::None,
            ),
            ICalendarProperty::Exdate => (
                ValueType::Ical(ICalendarValueType::DateTime),
                ValueSeparator::Comma,
            ),
            ICalendarProperty::Exrule => (
                ValueType::Ical(ICalendarValueType::Recur),
                ValueSeparator::None,
            ),
            ICalendarProperty::Rdate => (
                ValueType::Ical(ICalendarValueType::DateTime),
                ValueSeparator::Comma,
            ),
            ICalendarProperty::Rrule => (
                ValueType::Ical(ICalendarValueType::Recur),
                ValueSeparator::None,
            ),
            ICalendarProperty::Action => (ValueType::Action, ValueSeparator::None),
            ICalendarProperty::Repeat => (
                ValueType::Ical(ICalendarValueType::Integer),
                ValueSeparator::None,
            ),
            ICalendarProperty::Trigger => (
                ValueType::Ical(ICalendarValueType::Duration),
                ValueSeparator::None,
            ),
            ICalendarProperty::Created => (
                ValueType::Ical(ICalendarValueType::DateTime),
                ValueSeparator::None,
            ),
            ICalendarProperty::Dtstamp => (
                ValueType::Ical(ICalendarValueType::DateTime),
                ValueSeparator::None,
            ),
            ICalendarProperty::LastModified => (
                ValueType::Ical(ICalendarValueType::DateTime),
                ValueSeparator::None,
            ),
            ICalendarProperty::Sequence => (
                ValueType::Ical(ICalendarValueType::Integer),
                ValueSeparator::None,
            ),
            ICalendarProperty::RequestStatus => (
                ValueType::Ical(ICalendarValueType::Text),
                ValueSeparator::Semicolon,
            ),
            ICalendarProperty::Xml => (
                ValueType::Ical(ICalendarValueType::Text),
                ValueSeparator::None,
            ),
            ICalendarProperty::Tzuntil => (
                ValueType::Ical(ICalendarValueType::DateTime),
                ValueSeparator::None,
            ),
            ICalendarProperty::TzidAliasOf => (
                ValueType::Ical(ICalendarValueType::Text),
                ValueSeparator::None,
            ),
            ICalendarProperty::Busytype => (ValueType::BusyType, ValueSeparator::None),
            ICalendarProperty::Name => (
                ValueType::Ical(ICalendarValueType::Text),
                ValueSeparator::None,
            ),
            ICalendarProperty::RefreshInterval => (
                ValueType::Ical(ICalendarValueType::Duration),
                ValueSeparator::None,
            ),
            ICalendarProperty::Source => (
                ValueType::Ical(ICalendarValueType::Uri),
                ValueSeparator::None,
            ),
            ICalendarProperty::Color => (
                ValueType::Ical(ICalendarValueType::Text),
                ValueSeparator::None,
            ),
            ICalendarProperty::Image => (
                ValueType::Ical(ICalendarValueType::Uri),
                ValueSeparator::None,
            ),
            ICalendarProperty::Conference => (
                ValueType::Ical(ICalendarValueType::Uri),
                ValueSeparator::None,
            ),
            ICalendarProperty::CalendarAddress => (
                ValueType::Ical(ICalendarValueType::Uri),
                ValueSeparator::None,
            ),
            ICalendarProperty::LocationType => (
                ValueType::Ical(ICalendarValueType::Text),
                ValueSeparator::Comma,
            ),
            ICalendarProperty::ParticipantType => {
                (ValueType::ParticipantType, ValueSeparator::None)
            }
            ICalendarProperty::ResourceType => (ValueType::ResourceType, ValueSeparator::None),
            ICalendarProperty::StructuredData => (
                ValueType::Ical(ICalendarValueType::Text),
                ValueSeparator::None,
            ),
            ICalendarProperty::StyledDescription => (
                ValueType::Ical(ICalendarValueType::Text),
                ValueSeparator::None,
            ),
            ICalendarProperty::Acknowledged => (
                ValueType::Ical(ICalendarValueType::DateTime),
                ValueSeparator::None,
            ),
            ICalendarProperty::Proximity => (ValueType::Proximity, ValueSeparator::None),
            ICalendarProperty::Concept => (
                ValueType::Ical(ICalendarValueType::Uri),
                ValueSeparator::None,
            ),
            ICalendarProperty::Link => (
                ValueType::Ical(ICalendarValueType::Uri),
                ValueSeparator::None,
            ),
            ICalendarProperty::Refid => (
                ValueType::Ical(ICalendarValueType::Text),
                ValueSeparator::None,
            ),
            ICalendarProperty::Begin => (
                ValueType::Ical(ICalendarValueType::Text),
                ValueSeparator::None,
            ),
            ICalendarProperty::End => (
                ValueType::Ical(ICalendarValueType::Text),
                ValueSeparator::None,
            ),
            ICalendarProperty::Other(_) => (
                ValueType::Ical(ICalendarValueType::Text),
                ValueSeparator::Semicolon,
            ),
            ICalendarProperty::Coordinates => (
                ValueType::Ical(ICalendarValueType::Uri),
                ValueSeparator::None,
            ),
            ICalendarProperty::ShowWithoutTime => (
                ValueType::Ical(ICalendarValueType::Boolean),
                ValueSeparator::None,
            ),
            ICalendarProperty::Jsid | ICalendarProperty::Jsprop => (
                ValueType::Ical(ICalendarValueType::Text),
                ValueSeparator::None,
            ),
            ICalendarProperty::EstimatedDuration => (
                ValueType::Ical(ICalendarValueType::Duration),
                ValueSeparator::None,
            ),
            ICalendarProperty::Reason => (
                ValueType::Ical(ICalendarValueType::Uri),
                ValueSeparator::None,
            ),
            ICalendarProperty::Substate => (
                ValueType::Ical(ICalendarValueType::Text),
                ValueSeparator::None,
            ),
            ICalendarProperty::TaskMode => (
                ValueType::Ical(ICalendarValueType::Text),
                ValueSeparator::None,
            ),
        }
    }
}

impl ValueType {
    pub fn unwrap_ical(self) -> ICalendarValueType {
        match self {
            ValueType::Ical(value) => value,
            _ => ICalendarValueType::Text,
        }
    }
}

impl ICalendar {
    pub fn parse(value: impl AsRef<str>) -> Result<Self, Entry> {
        let mut parser = Parser::new(value.as_ref());
        match parser.entry() {
            Entry::ICalendar(icalendar) => Ok(icalendar),
            other => Err(other),
        }
    }
}

impl Hash for ICalendarValue {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            ICalendarValue::Binary(value) => {
                state.write_u8(0);
                value.hash(state);
            }
            ICalendarValue::Boolean(value) => {
                state.write_u8(1);
                value.hash(state);
            }
            ICalendarValue::Uri(value) => {
                state.write_u8(2);
                value.hash(state);
            }
            ICalendarValue::PartialDateTime(value) => {
                state.write_u8(3);
                value.hash(state);
            }
            ICalendarValue::Duration(value) => {
                state.write_u8(4);
                value.hash(state);
            }
            ICalendarValue::RecurrenceRule(value) => {
                state.write_u8(5);
                value.hash(state);
            }
            ICalendarValue::Period(value) => {
                state.write_u8(6);
                value.hash(state);
            }
            ICalendarValue::Float(value) => {
                state.write_u8(7);
                value.to_bits().hash(state);
            }
            ICalendarValue::Integer(value) => {
                state.write_u8(8);
                value.hash(state);
            }
            ICalendarValue::Text(value) => {
                state.write_u8(9);
                value.hash(state);
            }
            ICalendarValue::CalendarScale(value) => {
                state.write_u8(10);
                value.hash(state);
            }
            ICalendarValue::Method(value) => {
                state.write_u8(11);
                value.hash(state);
            }
            ICalendarValue::Classification(value) => {
                state.write_u8(12);
                value.hash(state);
            }
            ICalendarValue::Status(value) => {
                state.write_u8(13);
                value.hash(state);
            }
            ICalendarValue::Transparency(value) => {
                state.write_u8(14);
                value.hash(state);
            }
            ICalendarValue::Action(value) => {
                state.write_u8(15);
                value.hash(state);
            }
            ICalendarValue::BusyType(value) => {
                state.write_u8(16);
                value.hash(state);
            }
            ICalendarValue::ParticipantType(value) => {
                state.write_u8(17);
                value.hash(state);
            }
            ICalendarValue::ResourceType(value) => {
                state.write_u8(18);
                value.hash(state);
            }
            ICalendarValue::Proximity(value) => {
                state.write_u8(19);
                value.hash(state);
            }
        }
    }
}

impl Eq for ICalendarValue {}

#[allow(clippy::derive_ord_xor_partial_ord)]
impl Ord for ICalendarValue {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap_or(std::cmp::Ordering::Equal)
    }
}
