/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

use crate::jscalendar::*;
use std::str::FromStr;

impl<I: JSCalendarId> FromStr for JSCalendarProperty<I> {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        hashify::tiny_map!(s.as_bytes(),
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
            "iCalComponent" => JSCalendarProperty::ICalComponent,
            "blobId" => JSCalendarProperty::BlobId,
        )
        .ok_or(())
    }
}

impl<I: JSCalendarId> JSCalendarProperty<I> {
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
            JSCalendarProperty::ICalComponent => "iCalComponent",
            JSCalendarProperty::BlobId => "blobId",
            JSCalendarProperty::LinkDisplay(v) => v.as_str(),
            JSCalendarProperty::VirtualLocationFeature(v) => v.as_str(),
            JSCalendarProperty::ParticipantRole(v) => v.as_str(),
            JSCalendarProperty::RelationValue(v) => v.as_str(),
            JSCalendarProperty::LinkRelation(v) => return v.as_str().to_string().into(),
            JSCalendarProperty::DateTime(dt) => return dt.to_rfc3339().into(),
            JSCalendarProperty::Pointer(pointer) => return Cow::Owned(pointer.to_string()),
            JSCalendarProperty::IdValue(id) => return id.to_string().into(),
            JSCalendarProperty::IdReference(s) => return format!("#{}", s).into(),
        }
        .into()
    }
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
            "AbsoluteTrigger" => JSCalendarType::AbsoluteTrigger,
            "OffsetTrigger" => JSCalendarType::OffsetTrigger,
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
            JSCalendarType::OffsetTrigger => "OffsetTrigger",
            JSCalendarType::AbsoluteTrigger => "AbsoluteTrigger",
        }
    }
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

impl FromStr for JSCalendarProgress {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        hashify::tiny_map!(s.as_bytes(),
            "needs-action" => JSCalendarProgress::NeedsAction,
            "in-process" => JSCalendarProgress::InProcess,
            "completed" => JSCalendarProgress::Completed,
            "failed" => JSCalendarProgress::Failed,
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
        }
    }
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

impl ICalendarFrequency {
    pub fn as_js_str(&self) -> &'static str {
        match self {
            ICalendarFrequency::Secondly => "secondly",
            ICalendarFrequency::Minutely => "minutely",
            ICalendarFrequency::Hourly => "hourly",
            ICalendarFrequency::Daily => "daily",
            ICalendarFrequency::Weekly => "weekly",
            ICalendarFrequency::Monthly => "monthly",
            ICalendarFrequency::Yearly => "yearly",
        }
    }
}

impl ICalendarSkip {
    pub fn as_js_str(&self) -> &'static str {
        match self {
            ICalendarSkip::Omit => "omit",
            ICalendarSkip::Backward => "backward",
            ICalendarSkip::Forward => "forward",
        }
    }
}

impl ICalendarWeekday {
    pub fn as_js_str(&self) -> &'static str {
        match self {
            ICalendarWeekday::Sunday => "su",
            ICalendarWeekday::Monday => "mo",
            ICalendarWeekday::Tuesday => "tu",
            ICalendarWeekday::Wednesday => "we",
            ICalendarWeekday::Thursday => "th",
            ICalendarWeekday::Friday => "fr",
            ICalendarWeekday::Saturday => "sa",
        }
    }
}

impl ICalendarMethod {
    pub fn as_js_str(&self) -> &'static str {
        match self {
            ICalendarMethod::Publish => "publish",
            ICalendarMethod::Request => "request",
            ICalendarMethod::Reply => "reply",
            ICalendarMethod::Add => "add",
            ICalendarMethod::Cancel => "cancel",
            ICalendarMethod::Refresh => "refresh",
            ICalendarMethod::Counter => "counter",
            ICalendarMethod::Declinecounter => "declinecounter",
        }
    }
}
