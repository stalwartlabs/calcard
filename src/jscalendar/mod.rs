/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

pub mod export;
pub mod import;
pub mod parser;

use crate::{
    common::{IanaString, LinkRelation},
    icalendar::ICalendarDuration,
};
use jmap_tools::{JsonPointer, Value};
use mail_parser::DateTime;
use serde::Serialize;
use std::{borrow::Cow, str::FromStr};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
#[repr(transparent)]
pub struct JSCalendar<'x>(pub Value<'x, JSCalendarProperty, JSCalendarValue>);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum JSCalendarValue {
    Type(JSCalendarType),
    DateTime(JSCalendarDateTime),
    Duration(ICalendarDuration),
    AlertAction(JSCalendarAlertAction),
    FreeBusyStatus(JSCalendarFreeBusyStatus),
    ParticipantKind(JSCalendarParticipantKind),
    ParticipationStatus(JSCalendarParticipationStatus),
    Privacy(JSCalendarPrivacy),
    Progress(JSCalendarProgress),
    RelativeTo(JSCalendarRelativeTo),
    ScheduleAgent(JSCalendarScheduleAgent),
    EventStatus(JSCalendarEventStatus),
    LinkRelation(LinkRelation),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct JSCalendarDateTime {
    pub timestamp: i64,
    pub is_local: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum JSCalendarProperty {
    Type,
    Acknowledged,
    Action,
    Alerts,
    BaseEventId,
    ByDay,
    ByHour,
    ByMinute,
    ByMonth,
    ByMonthDay,
    BySecond,
    BySetPosition,
    ByWeekNo,
    ByYearDay,
    CalendarAddress,
    CalendarIds,
    Categories,
    Color,
    ContentType,
    Coordinates,
    Count,
    Created,
    Day,
    DelegatedFrom,
    DelegatedTo,
    Description,
    DescriptionContentType,
    Display,
    Due,
    Duration,
    Email,
    Entries,
    EstimatedDuration,
    Excluded,
    ExpectReply,
    Features,
    FirstDayOfWeek,
    FreeBusyStatus,
    Frequency,
    HideAttendees,
    Href,
    Id,
    Interval,
    InvitedBy,
    IsDraft,
    IsOrigin,
    Keywords,
    Kind,
    Links,
    Locale,
    Localizations,
    Locations,
    LocationTypes,
    MayInviteOthers,
    MayInviteSelf,
    MemberOf,
    Method,
    Name,
    NthOfPeriod,
    Offset,
    Participants,
    ParticipationComment,
    ParticipationStatus,
    PercentComplete,
    Priority,
    Privacy,
    ProdId,
    Progress,
    RecurrenceId,
    RecurrenceIdTimeZone,
    RecurrenceOverrides,
    Rel,
    RelatedTo,
    Relation,
    RelativeTo,
    ReplyTo,
    RequestStatus,
    Roles,
    Rscale,
    SentBy,
    ScheduleAgent,
    ScheduleForceSend,
    ScheduleSequence,
    ScheduleStatus,
    ScheduleUpdated,
    SendTo,
    Sequence,
    ShowWithoutTime,
    Size,
    Skip,
    Source,
    Start,
    Status,
    TimeZone,
    Title,
    Trigger,
    Uid,
    Until,
    Updated,
    Uri,
    UseDefaultAlerts,
    UtcEnd,
    UtcStart,
    VirtualLocations,
    When,
    EndTimeZone,
    MainLocationId,
    OrganizerCalendarAddress,
    RecurrenceRule,
    ICalComponent,
    Properties,
    Parameters,
    ConvertedProperties,
    ValueType,
    Components,
    DateTime(JSCalendarDateTime),
    LinkDisplay(JSCalendarLinkDisplay),
    VirtualLocationFeature(JSCalendarVirtualLocationFeature),
    ParticipantRole(JSCalendarParticipantRole),
    RelationValue(JSCalendarRelation),
    LinkRelation(LinkRelation),
    Pointer(JsonPointer<JSCalendarProperty>),
}

impl FromStr for JSCalendarProperty {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        hashify::map!(s.as_bytes(), JSCalendarProperty,
            "@type" => JSCalendarProperty::Type,
            "acknowledged" => JSCalendarProperty::Acknowledged,
            "action" => JSCalendarProperty::Action,
            "alerts" => JSCalendarProperty::Alerts,
            "baseEventId" => JSCalendarProperty::BaseEventId,
            "byDay" => JSCalendarProperty::ByDay,
            "byHour" => JSCalendarProperty::ByHour,
            "byMinute" => JSCalendarProperty::ByMinute,
            "byMonth" => JSCalendarProperty::ByMonth,
            "byMonthDay" => JSCalendarProperty::ByMonthDay,
            "bySecond" => JSCalendarProperty::BySecond,
            "bySetPosition" => JSCalendarProperty::BySetPosition,
            "byWeekNo" => JSCalendarProperty::ByWeekNo,
            "byYearDay" => JSCalendarProperty::ByYearDay,
            "calendarAddress" => JSCalendarProperty::CalendarAddress,
            "calendarIds" => JSCalendarProperty::CalendarIds,
            "categories" => JSCalendarProperty::Categories,
            "color" => JSCalendarProperty::Color,
            "contentType" => JSCalendarProperty::ContentType,
            "coordinates" => JSCalendarProperty::Coordinates,
            "count" => JSCalendarProperty::Count,
            "created" => JSCalendarProperty::Created,
            "day" => JSCalendarProperty::Day,
            "delegatedFrom" => JSCalendarProperty::DelegatedFrom,
            "delegatedTo" => JSCalendarProperty::DelegatedTo,
            "description" => JSCalendarProperty::Description,
            "descriptionContentType" => JSCalendarProperty::DescriptionContentType,
            "display" => JSCalendarProperty::Display,
            "due" => JSCalendarProperty::Due,
            "duration" => JSCalendarProperty::Duration,
            "email" => JSCalendarProperty::Email,
            "entries" => JSCalendarProperty::Entries,
            "estimatedDuration" => JSCalendarProperty::EstimatedDuration,
            "excluded" => JSCalendarProperty::Excluded,
            "expectReply" => JSCalendarProperty::ExpectReply,
            "features" => JSCalendarProperty::Features,
            "firstDayOfWeek" => JSCalendarProperty::FirstDayOfWeek,
            "freeBusyStatus" => JSCalendarProperty::FreeBusyStatus,
            "frequency" => JSCalendarProperty::Frequency,
            "hideAttendees" => JSCalendarProperty::HideAttendees,
            "href" => JSCalendarProperty::Href,
            "id" => JSCalendarProperty::Id,
            "interval" => JSCalendarProperty::Interval,
            "invitedBy" => JSCalendarProperty::InvitedBy,
            "isDraft" => JSCalendarProperty::IsDraft,
            "isOrigin" => JSCalendarProperty::IsOrigin,
            "keywords" => JSCalendarProperty::Keywords,
            "kind" => JSCalendarProperty::Kind,
            "links" => JSCalendarProperty::Links,
            "locale" => JSCalendarProperty::Locale,
            "localizations" => JSCalendarProperty::Localizations,
            "locations" => JSCalendarProperty::Locations,
            "locationTypes" => JSCalendarProperty::LocationTypes,
            "mayInviteOthers" => JSCalendarProperty::MayInviteOthers,
            "mayInviteSelf" => JSCalendarProperty::MayInviteSelf,
            "memberOf" => JSCalendarProperty::MemberOf,
            "method" => JSCalendarProperty::Method,
            "name" => JSCalendarProperty::Name,
            "nthOfPeriod" => JSCalendarProperty::NthOfPeriod,
            "offset" => JSCalendarProperty::Offset,
            "participants" => JSCalendarProperty::Participants,
            "participationComment" => JSCalendarProperty::ParticipationComment,
            "participationStatus" => JSCalendarProperty::ParticipationStatus,
            "percentComplete" => JSCalendarProperty::PercentComplete,
            "priority" => JSCalendarProperty::Priority,
            "privacy" => JSCalendarProperty::Privacy,
            "prodId" => JSCalendarProperty::ProdId,
            "progress" => JSCalendarProperty::Progress,
            "recurrenceId" => JSCalendarProperty::RecurrenceId,
            "recurrenceIdTimeZone" => JSCalendarProperty::RecurrenceIdTimeZone,
            "recurrenceOverrides" => JSCalendarProperty::RecurrenceOverrides,
            "rel" => JSCalendarProperty::Rel,
            "relatedTo" => JSCalendarProperty::RelatedTo,
            "relation" => JSCalendarProperty::Relation,
            "relativeTo" => JSCalendarProperty::RelativeTo,
            "replyTo" => JSCalendarProperty::ReplyTo,
            "requestStatus" => JSCalendarProperty::RequestStatus,
            "roles" => JSCalendarProperty::Roles,
            "rscale" => JSCalendarProperty::Rscale,
            "sentBy" => JSCalendarProperty::SentBy,
            "scheduleAgent" => JSCalendarProperty::ScheduleAgent,
            "scheduleForceSend" => JSCalendarProperty::ScheduleForceSend,
            "scheduleSequence" => JSCalendarProperty::ScheduleSequence,
            "scheduleStatus" => JSCalendarProperty::ScheduleStatus,
            "scheduleUpdated" => JSCalendarProperty::ScheduleUpdated,
            "sendTo" => JSCalendarProperty::SendTo,
            "sequence" => JSCalendarProperty::Sequence,
            "showWithoutTime" => JSCalendarProperty::ShowWithoutTime,
            "size" => JSCalendarProperty::Size,
            "skip" => JSCalendarProperty::Skip,
            "source" => JSCalendarProperty::Source,
            "start" => JSCalendarProperty::Start,
            "status" => JSCalendarProperty::Status,
            "timeZone" => JSCalendarProperty::TimeZone,
            "title" => JSCalendarProperty::Title,
            "trigger" => JSCalendarProperty::Trigger,
            "uid" => JSCalendarProperty::Uid,
            "until" => JSCalendarProperty::Until,
            "updated" => JSCalendarProperty::Updated,
            "uri" => JSCalendarProperty::Uri,
            "useDefaultAlerts" => JSCalendarProperty::UseDefaultAlerts,
            "utcEnd" => JSCalendarProperty::UtcEnd,
            "utcStart" => JSCalendarProperty::UtcStart,
            "virtualLocations" => JSCalendarProperty::VirtualLocations,
            "when" => JSCalendarProperty::When,
            "endTimeZone" => JSCalendarProperty::EndTimeZone,
            "mainLocationId" => JSCalendarProperty::MainLocationId,
            "organizerCalendarAddress" => JSCalendarProperty::OrganizerCalendarAddress,
            "recurrenceRule" => JSCalendarProperty::RecurrenceRule,
            "properties" => JSCalendarProperty::Properties,
            "components" => JSCalendarProperty::Components,
            "valueType" => JSCalendarProperty::ValueType,
            "convertedProperties" => JSCalendarProperty::ConvertedProperties,
            "parameters" => JSCalendarProperty::Parameters,
            "ICalComponent" => JSCalendarProperty::ICalComponent
        )
        .cloned()
        .ok_or(())
    }
}

impl JSCalendarProperty {
    pub fn to_string(&self) -> Cow<'static, str> {
        match self {
            JSCalendarProperty::Type => "@type",
            JSCalendarProperty::Acknowledged => "acknowledged",
            JSCalendarProperty::Action => "action",
            JSCalendarProperty::Alerts => "alerts",
            JSCalendarProperty::BaseEventId => "baseEventId",
            JSCalendarProperty::ByDay => "byDay",
            JSCalendarProperty::ByHour => "byHour",
            JSCalendarProperty::ByMinute => "byMinute",
            JSCalendarProperty::ByMonth => "byMonth",
            JSCalendarProperty::ByMonthDay => "byMonthDay",
            JSCalendarProperty::BySecond => "bySecond",
            JSCalendarProperty::BySetPosition => "bySetPosition",
            JSCalendarProperty::ByWeekNo => "byWeekNo",
            JSCalendarProperty::ByYearDay => "byYearDay",
            JSCalendarProperty::CalendarAddress => "calendarAddress",
            JSCalendarProperty::CalendarIds => "calendarIds",
            JSCalendarProperty::Categories => "categories",
            JSCalendarProperty::Color => "color",
            JSCalendarProperty::ContentType => "contentType",
            JSCalendarProperty::Coordinates => "coordinates",
            JSCalendarProperty::Count => "count",
            JSCalendarProperty::Created => "created",
            JSCalendarProperty::Day => "day",
            JSCalendarProperty::DelegatedFrom => "delegatedFrom",
            JSCalendarProperty::DelegatedTo => "delegatedTo",
            JSCalendarProperty::Description => "description",
            JSCalendarProperty::DescriptionContentType => "descriptionContentType",
            JSCalendarProperty::Display => "display",
            JSCalendarProperty::Due => "due",
            JSCalendarProperty::Duration => "duration",
            JSCalendarProperty::Email => "email",
            JSCalendarProperty::Entries => "entries",
            JSCalendarProperty::EstimatedDuration => "estimatedDuration",
            JSCalendarProperty::Excluded => "excluded",
            JSCalendarProperty::ExpectReply => "expectReply",
            JSCalendarProperty::Features => "features",
            JSCalendarProperty::FirstDayOfWeek => "firstDayOfWeek",
            JSCalendarProperty::FreeBusyStatus => "freeBusyStatus",
            JSCalendarProperty::Frequency => "frequency",
            JSCalendarProperty::HideAttendees => "hideAttendees",
            JSCalendarProperty::Href => "href",
            JSCalendarProperty::Id => "id",
            JSCalendarProperty::Interval => "interval",
            JSCalendarProperty::InvitedBy => "invitedBy",
            JSCalendarProperty::IsDraft => "isDraft",
            JSCalendarProperty::IsOrigin => "isOrigin",
            JSCalendarProperty::Keywords => "keywords",
            JSCalendarProperty::Kind => "kind",
            JSCalendarProperty::Links => "links",
            JSCalendarProperty::Locale => "locale",
            JSCalendarProperty::Localizations => "localizations",
            JSCalendarProperty::Locations => "locations",
            JSCalendarProperty::LocationTypes => "locationTypes",
            JSCalendarProperty::MayInviteOthers => "mayInviteOthers",
            JSCalendarProperty::MayInviteSelf => "mayInviteSelf",
            JSCalendarProperty::MemberOf => "memberOf",
            JSCalendarProperty::Method => "method",
            JSCalendarProperty::Name => "name",
            JSCalendarProperty::NthOfPeriod => "nthOfPeriod",
            JSCalendarProperty::Offset => "offset",
            JSCalendarProperty::Participants => "participants",
            JSCalendarProperty::ParticipationComment => "participationComment",
            JSCalendarProperty::ParticipationStatus => "participationStatus",
            JSCalendarProperty::PercentComplete => "percentComplete",
            JSCalendarProperty::Priority => "priority",
            JSCalendarProperty::Privacy => "privacy",
            JSCalendarProperty::ProdId => "prodId",
            JSCalendarProperty::Progress => "progress",
            JSCalendarProperty::RecurrenceId => "recurrenceId",
            JSCalendarProperty::RecurrenceIdTimeZone => "recurrenceIdTimeZone",
            JSCalendarProperty::RecurrenceOverrides => "recurrenceOverrides",
            JSCalendarProperty::Rel => "rel",
            JSCalendarProperty::RelatedTo => "relatedTo",
            JSCalendarProperty::Relation => "relation",
            JSCalendarProperty::RelativeTo => "relativeTo",
            JSCalendarProperty::ReplyTo => "replyTo",
            JSCalendarProperty::RequestStatus => "requestStatus",
            JSCalendarProperty::Roles => "roles",
            JSCalendarProperty::Rscale => "rscale",
            JSCalendarProperty::SentBy => "sentBy",
            JSCalendarProperty::ScheduleAgent => "scheduleAgent",
            JSCalendarProperty::ScheduleForceSend => "scheduleForceSend",
            JSCalendarProperty::ScheduleSequence => "scheduleSequence",
            JSCalendarProperty::ScheduleStatus => "scheduleStatus",
            JSCalendarProperty::ScheduleUpdated => "scheduleUpdated",
            JSCalendarProperty::SendTo => "sendTo",
            JSCalendarProperty::Sequence => "sequence",
            JSCalendarProperty::ShowWithoutTime => "showWithoutTime",
            JSCalendarProperty::Size => "size",
            JSCalendarProperty::Skip => "skip",
            JSCalendarProperty::Source => "source",
            JSCalendarProperty::Start => "start",
            JSCalendarProperty::Status => "status",
            JSCalendarProperty::TimeZone => "timeZone",
            JSCalendarProperty::Title => "title",
            JSCalendarProperty::Trigger => "trigger",
            JSCalendarProperty::Uid => "uid",
            JSCalendarProperty::Until => "until",
            JSCalendarProperty::Updated => "updated",
            JSCalendarProperty::Uri => "uri",
            JSCalendarProperty::UseDefaultAlerts => "useDefaultAlerts",
            JSCalendarProperty::UtcEnd => "utcEnd",
            JSCalendarProperty::UtcStart => "utcStart",
            JSCalendarProperty::VirtualLocations => "virtualLocations",
            JSCalendarProperty::When => "when",
            JSCalendarProperty::EndTimeZone => "endTimeZone",
            JSCalendarProperty::MainLocationId => "mainLocationId",
            JSCalendarProperty::OrganizerCalendarAddress => "organizerCalendarAddress",
            JSCalendarProperty::RecurrenceRule => "recurrenceRule",
            JSCalendarProperty::Properties => "properties",
            JSCalendarProperty::Components => "components",
            JSCalendarProperty::ValueType => "valueType",
            JSCalendarProperty::ConvertedProperties => "convertedProperties",
            JSCalendarProperty::Parameters => "parameters",
            JSCalendarProperty::ICalComponent => "ICalComponent",
            JSCalendarProperty::LinkDisplay(v) => v.as_str(),
            JSCalendarProperty::VirtualLocationFeature(v) => v.as_str(),
            JSCalendarProperty::ParticipantRole(v) => v.as_str(),
            JSCalendarProperty::RelationValue(v) => v.as_str(),
            JSCalendarProperty::LinkRelation(v) => return v.as_str().to_string().into(),
            JSCalendarProperty::DateTime(dt) => return dt.to_rfc3339().into(),
            JSCalendarProperty::Pointer(pointer) => return Cow::Owned(pointer.to_string()),
        }
        .into()
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum JSCalendarType {
    #[default]
    Event,
    Task,
    Group,
    Alert,
    Boolean,
    Duration,
    Id,
    Int,
    LocalDateTime,
    Link,
    Location,
    NDay,
    Number,
    Participant,
    PatchObject,
    RecurrenceRule,
    Relation,
    SignedDuration,
    String,
    TimeZone,
    TimeZoneId,
    TimeZoneRule,
    UnsignedInt,
    UTCDateTime,
    VirtualLocation,
    ICalComponent,
}

impl FromStr for JSCalendarType {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        hashify::tiny_map!(s.as_bytes(),
            "Event" => JSCalendarType::Event,
            "Task" => JSCalendarType::Task,
            "Group" => JSCalendarType::Group,
            "Alert" => JSCalendarType::Alert,
            "Boolean" => JSCalendarType::Boolean,
            "Duration" => JSCalendarType::Duration,
            "Id" => JSCalendarType::Id,
            "Int" => JSCalendarType::Int,
            "LocalDateTime" => JSCalendarType::LocalDateTime,
            "Link" => JSCalendarType::Link,
            "Location" => JSCalendarType::Location,
            "NDay" => JSCalendarType::NDay,
            "Number" => JSCalendarType::Number,
            "Participant" => JSCalendarType::Participant,
            "PatchObject" => JSCalendarType::PatchObject,
            "RecurrenceRule" => JSCalendarType::RecurrenceRule,
            "Relation" => JSCalendarType::Relation,
            "SignedDuration" => JSCalendarType::SignedDuration,
            "String" => JSCalendarType::String,
            "TimeZone" => JSCalendarType::TimeZone,
            "TimeZoneId" => JSCalendarType::TimeZoneId,
            "TimeZoneRule" => JSCalendarType::TimeZoneRule,
            "UnsignedInt" => JSCalendarType::UnsignedInt,
            "UTCDateTime" => JSCalendarType::UTCDateTime,
            "VirtualLocation" => JSCalendarType::VirtualLocation,
            "ICalComponent" => JSCalendarType::ICalComponent,
        )
        .ok_or(())
    }
}

impl JSCalendarType {
    pub fn as_str(&self) -> &'static str {
        match self {
            JSCalendarType::Event => "Event",
            JSCalendarType::Task => "Task",
            JSCalendarType::Group => "Group",
            JSCalendarType::Alert => "Alert",
            JSCalendarType::Boolean => "Boolean",
            JSCalendarType::Duration => "Duration",
            JSCalendarType::Id => "Id",
            JSCalendarType::Int => "Int",
            JSCalendarType::LocalDateTime => "LocalDateTime",
            JSCalendarType::Link => "Link",
            JSCalendarType::Location => "Location",
            JSCalendarType::NDay => "NDay",
            JSCalendarType::Number => "Number",
            JSCalendarType::Participant => "Participant",
            JSCalendarType::PatchObject => "PatchObject",
            JSCalendarType::RecurrenceRule => "RecurrenceRule",
            JSCalendarType::Relation => "Relation",
            JSCalendarType::SignedDuration => "SignedDuration",
            JSCalendarType::String => "String",
            JSCalendarType::TimeZone => "TimeZone",
            JSCalendarType::TimeZoneId => "TimeZoneId",
            JSCalendarType::TimeZoneRule => "TimeZoneRule",
            JSCalendarType::UnsignedInt => "UnsignedInt",
            JSCalendarType::UTCDateTime => "UTCDateTime",
            JSCalendarType::VirtualLocation => "VirtualLocation",
            JSCalendarType::ICalComponent => "ICalComponent",
        }
    }
}

// JSCalendar Enum Values for action (Context: Alert)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum JSCalendarAlertAction {
    Display,
    Email,
}

impl FromStr for JSCalendarAlertAction {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        hashify::tiny_map!(s.as_bytes(),
            "display" => JSCalendarAlertAction::Display,
            "email" => JSCalendarAlertAction::Email
        )
        .ok_or(())
    }
}

impl JSCalendarAlertAction {
    pub fn as_str(&self) -> &'static str {
        match self {
            JSCalendarAlertAction::Display => "display",
            JSCalendarAlertAction::Email => "email",
        }
    }
}

// JSCalendar Enum Values for display (Context: Link)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum JSCalendarLinkDisplay {
    Badge,
    Graphic,
    Fullsize,
    Thumbnail,
}

impl FromStr for JSCalendarLinkDisplay {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        hashify::tiny_map!(s.as_bytes(),
            "badge" => JSCalendarLinkDisplay::Badge,
            "graphic" => JSCalendarLinkDisplay::Graphic,
            "fullsize" => JSCalendarLinkDisplay::Fullsize,
            "thumbnail" => JSCalendarLinkDisplay::Thumbnail
        )
        .ok_or(())
    }
}

impl JSCalendarLinkDisplay {
    pub fn as_str(&self) -> &'static str {
        match self {
            JSCalendarLinkDisplay::Badge => "badge",
            JSCalendarLinkDisplay::Graphic => "graphic",
            JSCalendarLinkDisplay::Fullsize => "fullsize",
            JSCalendarLinkDisplay::Thumbnail => "thumbnail",
        }
    }
}

// JSCalendar Enum Values for features (Context: VirtualLocation)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum JSCalendarVirtualLocationFeature {
    Audio,
    Chat,
    Feed,
    Moderator,
    Phone,
    Screen,
    Video,
}

impl FromStr for JSCalendarVirtualLocationFeature {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        hashify::tiny_map!(s.as_bytes(),
            "audio" => JSCalendarVirtualLocationFeature::Audio,
            "chat" => JSCalendarVirtualLocationFeature::Chat,
            "feed" => JSCalendarVirtualLocationFeature::Feed,
            "moderator" => JSCalendarVirtualLocationFeature::Moderator,
            "phone" => JSCalendarVirtualLocationFeature::Phone,
            "screen" => JSCalendarVirtualLocationFeature::Screen,
            "video" => JSCalendarVirtualLocationFeature::Video
        )
        .ok_or(())
    }
}

impl JSCalendarVirtualLocationFeature {
    pub fn as_str(&self) -> &'static str {
        match self {
            JSCalendarVirtualLocationFeature::Audio => "audio",
            JSCalendarVirtualLocationFeature::Chat => "chat",
            JSCalendarVirtualLocationFeature::Feed => "feed",
            JSCalendarVirtualLocationFeature::Moderator => "moderator",
            JSCalendarVirtualLocationFeature::Phone => "phone",
            JSCalendarVirtualLocationFeature::Screen => "screen",
            JSCalendarVirtualLocationFeature::Video => "video",
        }
    }
}

// JSCalendar Enum Values for freeBusyStatus (Context: Event, Task)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum JSCalendarFreeBusyStatus {
    Free,
    Busy,
}

impl FromStr for JSCalendarFreeBusyStatus {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        hashify::tiny_map!(s.as_bytes(),
            "free" => JSCalendarFreeBusyStatus::Free,
            "busy" => JSCalendarFreeBusyStatus::Busy
        )
        .ok_or(())
    }
}

impl JSCalendarFreeBusyStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            JSCalendarFreeBusyStatus::Free => "free",
            JSCalendarFreeBusyStatus::Busy => "busy",
        }
    }
}

// JSCalendar Enum Values for kind (Context: Participant)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum JSCalendarParticipantKind {
    Individual,
    Group,
    Resource,
    Location,
}

impl FromStr for JSCalendarParticipantKind {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        hashify::tiny_map!(s.as_bytes(),
            "individual" => JSCalendarParticipantKind::Individual,
            "group" => JSCalendarParticipantKind::Group,
            "resource" => JSCalendarParticipantKind::Resource,
            "location" => JSCalendarParticipantKind::Location
        )
        .ok_or(())
    }
}

impl JSCalendarParticipantKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            JSCalendarParticipantKind::Individual => "individual",
            JSCalendarParticipantKind::Group => "group",
            JSCalendarParticipantKind::Resource => "resource",
            JSCalendarParticipantKind::Location => "location",
        }
    }
}

// JSCalendar Enum Values for participationStatus (Context: Participant)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum JSCalendarParticipationStatus {
    NeedsAction,
    Accepted,
    Declined,
    Tentative,
    Delegated,
}

impl FromStr for JSCalendarParticipationStatus {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        hashify::tiny_map!(s.as_bytes(),
            "needs-action" => JSCalendarParticipationStatus::NeedsAction,
            "accepted" => JSCalendarParticipationStatus::Accepted,
            "declined" => JSCalendarParticipationStatus::Declined,
            "tentative" => JSCalendarParticipationStatus::Tentative,
            "delegated" => JSCalendarParticipationStatus::Delegated
        )
        .ok_or(())
    }
}

impl JSCalendarParticipationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            JSCalendarParticipationStatus::NeedsAction => "needs-action",
            JSCalendarParticipationStatus::Accepted => "accepted",
            JSCalendarParticipationStatus::Declined => "declined",
            JSCalendarParticipationStatus::Tentative => "tentative",
            JSCalendarParticipationStatus::Delegated => "delegated",
        }
    }
}

// JSCalendar Enum Values for privacy (Context: Event, Task)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum JSCalendarPrivacy {
    Public,
    Private,
    Secret,
}

impl FromStr for JSCalendarPrivacy {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        hashify::tiny_map!(s.as_bytes(),
            "public" => JSCalendarPrivacy::Public,
            "private" => JSCalendarPrivacy::Private,
            "secret" => JSCalendarPrivacy::Secret
        )
        .ok_or(())
    }
}

impl JSCalendarPrivacy {
    pub fn as_str(&self) -> &'static str {
        match self {
            JSCalendarPrivacy::Public => "public",
            JSCalendarPrivacy::Private => "private",
            JSCalendarPrivacy::Secret => "secret",
        }
    }
}

// JSCalendar Enum Values for progress (Context: Task, Participant)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum JSCalendarProgress {
    NeedsAction,
    InProcess,
    Completed,
    Failed,
    Cancelled,
}

impl FromStr for JSCalendarProgress {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        hashify::tiny_map!(s.as_bytes(),
            "needs-action" => JSCalendarProgress::NeedsAction,
            "in-process" => JSCalendarProgress::InProcess,
            "completed" => JSCalendarProgress::Completed,
            "failed" => JSCalendarProgress::Failed,
            "cancelled" => JSCalendarProgress::Cancelled
        )
        .ok_or(())
    }
}

impl JSCalendarProgress {
    pub fn as_str(&self) -> &'static str {
        match self {
            JSCalendarProgress::NeedsAction => "needs-action",
            JSCalendarProgress::InProcess => "in-process",
            JSCalendarProgress::Completed => "completed",
            JSCalendarProgress::Failed => "failed",
            JSCalendarProgress::Cancelled => "cancelled",
        }
    }
}

// JSCalendar Enum Values for relation (Context: Relation)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum JSCalendarRelation {
    First,
    Next,
    Child,
    Parent,
    Snooze,
}

impl FromStr for JSCalendarRelation {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        hashify::tiny_map!(s.as_bytes(),
            "first" => JSCalendarRelation::First,
            "next" => JSCalendarRelation::Next,
            "child" => JSCalendarRelation::Child,
            "parent" => JSCalendarRelation::Parent,
            "snooze" => JSCalendarRelation::Snooze,
        )
        .ok_or(())
    }
}

impl JSCalendarRelation {
    pub fn as_str(&self) -> &'static str {
        match self {
            JSCalendarRelation::First => "first",
            JSCalendarRelation::Next => "next",
            JSCalendarRelation::Child => "child",
            JSCalendarRelation::Parent => "parent",
            JSCalendarRelation::Snooze => "snooze",
        }
    }
}

// JSCalendar Enum Values for relativeTo (Context: OffsetTrigger, Location)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum JSCalendarRelativeTo {
    Start,
    End,
}

impl FromStr for JSCalendarRelativeTo {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        hashify::tiny_map!(s.as_bytes(),
            "start" => JSCalendarRelativeTo::Start,
            "end" => JSCalendarRelativeTo::End
        )
        .ok_or(())
    }
}

impl JSCalendarRelativeTo {
    pub fn as_str(&self) -> &'static str {
        match self {
            JSCalendarRelativeTo::Start => "start",
            JSCalendarRelativeTo::End => "end",
        }
    }
}

// JSCalendar Enum Values for roles (Context: Participant)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum JSCalendarParticipantRole {
    Owner,
    Attendee,
    Optional,
    Informational,
    Chair,
    Required,
}

impl FromStr for JSCalendarParticipantRole {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        hashify::tiny_map!(s.as_bytes(),
            "owner" => JSCalendarParticipantRole::Owner,
            "attendee" => JSCalendarParticipantRole::Attendee,
            "optional" => JSCalendarParticipantRole::Optional,
            "informational" => JSCalendarParticipantRole::Informational,
            "chair" => JSCalendarParticipantRole::Chair,
            "required" => JSCalendarParticipantRole::Required,
        )
        .ok_or(())
    }
}

impl JSCalendarParticipantRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            JSCalendarParticipantRole::Owner => "owner",
            JSCalendarParticipantRole::Attendee => "attendee",
            JSCalendarParticipantRole::Optional => "optional",
            JSCalendarParticipantRole::Informational => "informational",
            JSCalendarParticipantRole::Chair => "chair",
            JSCalendarParticipantRole::Required => "required",
        }
    }
}

// JSCalendar Enum Values for scheduleAgent (Context: Participant)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum JSCalendarScheduleAgent {
    Server,
    Client,
    None,
}

impl FromStr for JSCalendarScheduleAgent {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        hashify::tiny_map!(s.as_bytes(),
            "server" => JSCalendarScheduleAgent::Server,
            "client" => JSCalendarScheduleAgent::Client,
            "none" => JSCalendarScheduleAgent::None
        )
        .ok_or(())
    }
}

impl JSCalendarScheduleAgent {
    pub fn as_str(&self) -> &'static str {
        match self {
            JSCalendarScheduleAgent::Server => "server",
            JSCalendarScheduleAgent::Client => "client",
            JSCalendarScheduleAgent::None => "none",
        }
    }
}

// JSCalendar Enum Values for status (Context: Event)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum JSCalendarEventStatus {
    Confirmed,
    Cancelled,
    Tentative,
}

impl FromStr for JSCalendarEventStatus {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        hashify::tiny_map!(s.as_bytes(),
            "confirmed" => JSCalendarEventStatus::Confirmed,
            "cancelled" => JSCalendarEventStatus::Cancelled,
            "tentative" => JSCalendarEventStatus::Tentative
        )
        .ok_or(())
    }
}

impl JSCalendarEventStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            JSCalendarEventStatus::Confirmed => "confirmed",
            JSCalendarEventStatus::Cancelled => "cancelled",
            JSCalendarEventStatus::Tentative => "tentative",
        }
    }
}

impl JSCalendarDateTime {
    pub fn new(timestamp: i64, is_local: bool) -> Self {
        Self {
            timestamp,
            is_local,
        }
    }

    pub fn to_rfc3339(&self) -> String {
        let dt = DateTime::from_timestamp(self.timestamp);
        if !self.is_local {
            dt.to_rfc3339()
        } else {
            format!(
                "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}",
                dt.year, dt.month, dt.day, dt.hour, dt.minute, dt.second,
            )
        }
    }
}
