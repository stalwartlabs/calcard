/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

use crate::{
    common::jsprop::{property_names, text::PointerProperty},
    jscalendar::*,
};
use jmap_tools::{JsonPointer, JsonPointerItem, Key};
use std::str::FromStr;

impl<I: JSCalendarId> PointerProperty for JSCalendarProperty<I> {
    fn into_pointer(self) -> Result<JsonPointer<Self>, Self> {
        match self {
            JSCalendarProperty::Pointer(pointer) => Ok(pointer),
            property => Err(property),
        }
    }
}

impl<I: JSCalendarId> FromStr for JSCalendarProperty<I> {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        JSCalendarProperty::parse_name(s)
            .or_else(|| JSCalendarProperty::parse_metadata_selector(s))
            .ok_or(())
    }
}

impl<I: JSCalendarId> JSCalendarProperty<I> {
    property_names!(
        JSCalendarProperty,
        "@type" => Type,
        "acknowledged" => Acknowledged,
        "action" => Action,
        "alerts" => Alerts,
        "baseEventId" => BaseEventId,
        "byDay" => ByDay,
        "byHour" => ByHour,
        "byMinute" => ByMinute,
        "byMonth" => ByMonth,
        "byMonthDay" => ByMonthDay,
        "bySecond" => BySecond,
        "bySetPosition" => BySetPosition,
        "byWeekNo" => ByWeekNo,
        "byYearDay" => ByYearDay,
        "calendarAddress" => CalendarAddress,
        "calendarIds" => CalendarIds,
        "categories" => Categories,
        "color" => Color,
        "contentType" => ContentType,
        "coordinates" => Coordinates,
        "count" => Count,
        "created" => Created,
        "day" => Day,
        "delegatedFrom" => DelegatedFrom,
        "delegatedTo" => DelegatedTo,
        "description" => Description,
        "descriptionContentType" => DescriptionContentType,
        "display" => Display,
        "due" => Due,
        "duration" => Duration,
        "email" => Email,
        "entries" => Entries,
        "estimatedDuration" => EstimatedDuration,
        "excluded" => Excluded,
        "expectReply" => ExpectReply,
        "features" => Features,
        "firstDayOfWeek" => FirstDayOfWeek,
        "freeBusyStatus" => FreeBusyStatus,
        "frequency" => Frequency,
        "hideAttendees" => HideAttendees,
        "href" => Href,
        "id" => Id,
        "interval" => Interval,
        "invitedBy" => InvitedBy,
        "isDraft" => IsDraft,
        "isOrigin" => IsOrigin,
        "keywords" => Keywords,
        "kind" => Kind,
        "links" => Links,
        "locale" => Locale,
        "locations" => Locations,
        "locationTypes" => LocationTypes,
        "mayInviteOthers" => MayInviteOthers,
        "mayInviteSelf" => MayInviteSelf,
        "memberOf" => MemberOf,
        "method" => Method,
        "name" => Name,
        "nthOfPeriod" => NthOfPeriod,
        "offset" => Offset,
        "participants" => Participants,
        "participationComment" => ParticipationComment,
        "participationStatus" => ParticipationStatus,
        "percentComplete" => PercentComplete,
        "priority" => Priority,
        "privacy" => Privacy,
        "prodId" => ProdId,
        "progress" => Progress,
        "recurrenceId" => RecurrenceId,
        "recurrenceIdTimeZone" => RecurrenceIdTimeZone,
        "recurrenceOverrides" => RecurrenceOverrides,
        "rel" => Rel,
        "relatedTo" => RelatedTo,
        "relation" => Relation,
        "relativeTo" => RelativeTo,
        "replyTo" => ReplyTo,
        "requestStatus" => RequestStatus,
        "roles" => Roles,
        "rscale" => Rscale,
        "sentBy" => SentBy,
        "scheduleAgent" => ScheduleAgent,
        "scheduleForceSend" => ScheduleForceSend,
        "scheduleSequence" => ScheduleSequence,
        "scheduleStatus" => ScheduleStatus,
        "scheduleUpdated" => ScheduleUpdated,
        "sendTo" => SendTo,
        "sequence" => Sequence,
        "showWithoutTime" => ShowWithoutTime,
        "size" => Size,
        "skip" => Skip,
        "source" => Source,
        "start" => Start,
        "status" => Status,
        "timeZone" => TimeZone,
        "title" => Title,
        "trigger" => Trigger,
        "uid" => Uid,
        "until" => Until,
        "updated" => Updated,
        "uri" => Uri,
        "useDefaultAlerts" => UseDefaultAlerts,
        "utcEnd" => UtcEnd,
        "utcStart" => UtcStart,
        "version" => Version,
        "virtualLocations" => VirtualLocations,
        "when" => When,
        "endTimeZone" => EndTimeZone,
        "mainLocationId" => MainLocationId,
        "organizerCalendarAddress" => OrganizerCalendarAddress,
        "recurrenceRule" => RecurrenceRule,
        "properties" => Properties,
        "components" => Components,
        "valueType" => ValueType,
        "convertedProperties" => ConvertedProperties,
        "parameters" => Parameters,
        "iCalendar" => ICalendar,
        "blobId" => BlobId,
    );

    fn parse_metadata_selector(s: &str) -> Option<Self> {
        s.split_once('/')
            .and_then(|(root, _)| JSCalendarProperty::<I>::parse_name(root))
            .filter(JSCalendarProperty::is_metadata)
            .map(|_| JSCalendarProperty::Pointer(JsonPointer::parse(s)))
    }

    pub(crate) fn is_metadata(&self) -> bool {
        match self {
            JSCalendarProperty::Metadata | JSCalendarProperty::PrivateMetadata => true,
            JSCalendarProperty::Pointer(pointer) => {
                JSCalendarProperty::is_metadata_pointer(pointer)
            }
            _ => false,
        }
    }

    pub(crate) fn sets_misplaced_excluded(pointer: &JsonPointer<Self>) -> bool {
        let items = match pointer.as_slice() {
            [JsonPointerItem::Root, items @ ..] => items,
            items => items,
        };
        let Some((JsonPointerItem::Key(Key::Property(JSCalendarProperty::Excluded)), parents)) =
            items.split_last()
        else {
            return false;
        };
        if matches!(
            parents,
            [
                JsonPointerItem::Key(Key::Property(JSCalendarProperty::RecurrenceOverrides)),
                JsonPointerItem::Key(Key::Property(JSCalendarProperty::DateTime(_)))
            ]
        ) {
            return false;
        }
        let mut is_member_key = false;
        parents.iter().all(|item| {
            let is_structural = is_member_key
                || matches!(
                    item,
                    JsonPointerItem::Key(Key::Property(_)) | JsonPointerItem::Number(_)
                );
            is_member_key = matches!(
                item,
                JsonPointerItem::Key(Key::Property(property))
                    if property.is_member_map() || *property == JSCalendarProperty::RelatedTo
            );
            is_structural
        })
    }

    pub(crate) fn is_metadata_pointer(pointer: &JsonPointer<Self>) -> bool {
        pointer
            .first()
            .and_then(JsonPointerItem::as_property_key)
            .is_some_and(JSCalendarProperty::is_metadata)
    }

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
            JSCalendarProperty::Version => "version",
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
            JSCalendarProperty::ICalendar => "iCalendar",
            JSCalendarProperty::BlobId => "blobId",
            JSCalendarProperty::Metadata => "metadata",
            JSCalendarProperty::PrivateMetadata => "privateMetadata",
            JSCalendarProperty::LinkDisplay(v) => v.as_str(),
            JSCalendarProperty::VirtualLocationFeature(v) => v.as_str(),
            JSCalendarProperty::ParticipantRole(v) => v.as_str(),
            JSCalendarProperty::RelationValue(v) => v.as_str(),
            JSCalendarProperty::LinkRelation(v) => v.as_str(),
            JSCalendarProperty::DateTime(dt) => return dt.to_rfc3339().into(),
            JSCalendarProperty::Pointer(pointer) => return Cow::Owned(pointer.to_string()),
            JSCalendarProperty::IdValue(id) => return id.to_string().into(),
            JSCalendarProperty::IdReference(s) => return format!("#{}", s).into(),
        }
        .into()
    }

    #[inline]
    pub(crate) fn static_name(&self) -> Option<&'static str> {
        match self {
            JSCalendarProperty::DateTime(_)
            | JSCalendarProperty::Pointer(_)
            | JSCalendarProperty::IdValue(_)
            | JSCalendarProperty::IdReference(_) => None,
            JSCalendarProperty::LinkRelation(v) => Some(v.as_str()),
            property => match property.to_string() {
                Cow::Borrowed(name) => Some(name),
                Cow::Owned(_) => None,
            },
        }
    }

    #[inline]
    pub(crate) fn has_data(&self) -> bool {
        match self {
            JSCalendarProperty::DateTime(_)
            | JSCalendarProperty::LinkDisplay(_)
            | JSCalendarProperty::VirtualLocationFeature(_)
            | JSCalendarProperty::ParticipantRole(_)
            | JSCalendarProperty::RelationValue(_)
            | JSCalendarProperty::LinkRelation(_)
            | JSCalendarProperty::Pointer(_)
            | JSCalendarProperty::IdValue(_)
            | JSCalendarProperty::IdReference(_) => true,
            JSCalendarProperty::Id
            | JSCalendarProperty::BaseEventId
            | JSCalendarProperty::CalendarIds
            | JSCalendarProperty::IsDraft
            | JSCalendarProperty::IsOrigin
            | JSCalendarProperty::UtcStart
            | JSCalendarProperty::UtcEnd
            | JSCalendarProperty::UseDefaultAlerts
            | JSCalendarProperty::MayInviteSelf
            | JSCalendarProperty::MayInviteOthers
            | JSCalendarProperty::HideAttendees
            | JSCalendarProperty::BlobId
            | JSCalendarProperty::Metadata
            | JSCalendarProperty::PrivateMetadata
            | JSCalendarProperty::Type
            | JSCalendarProperty::Acknowledged
            | JSCalendarProperty::Action
            | JSCalendarProperty::Alerts
            | JSCalendarProperty::ByDay
            | JSCalendarProperty::ByHour
            | JSCalendarProperty::ByMinute
            | JSCalendarProperty::ByMonth
            | JSCalendarProperty::ByMonthDay
            | JSCalendarProperty::BySecond
            | JSCalendarProperty::BySetPosition
            | JSCalendarProperty::ByWeekNo
            | JSCalendarProperty::ByYearDay
            | JSCalendarProperty::CalendarAddress
            | JSCalendarProperty::Categories
            | JSCalendarProperty::Color
            | JSCalendarProperty::ContentType
            | JSCalendarProperty::Coordinates
            | JSCalendarProperty::Count
            | JSCalendarProperty::Created
            | JSCalendarProperty::Day
            | JSCalendarProperty::DelegatedFrom
            | JSCalendarProperty::DelegatedTo
            | JSCalendarProperty::Description
            | JSCalendarProperty::DescriptionContentType
            | JSCalendarProperty::Display
            | JSCalendarProperty::Due
            | JSCalendarProperty::Duration
            | JSCalendarProperty::Email
            | JSCalendarProperty::Entries
            | JSCalendarProperty::EstimatedDuration
            | JSCalendarProperty::Excluded
            | JSCalendarProperty::ExpectReply
            | JSCalendarProperty::Features
            | JSCalendarProperty::FirstDayOfWeek
            | JSCalendarProperty::FreeBusyStatus
            | JSCalendarProperty::Frequency
            | JSCalendarProperty::Href
            | JSCalendarProperty::Interval
            | JSCalendarProperty::InvitedBy
            | JSCalendarProperty::Keywords
            | JSCalendarProperty::Kind
            | JSCalendarProperty::Links
            | JSCalendarProperty::Locale
            | JSCalendarProperty::Locations
            | JSCalendarProperty::LocationTypes
            | JSCalendarProperty::MemberOf
            | JSCalendarProperty::Method
            | JSCalendarProperty::Name
            | JSCalendarProperty::NthOfPeriod
            | JSCalendarProperty::Offset
            | JSCalendarProperty::Participants
            | JSCalendarProperty::ParticipationComment
            | JSCalendarProperty::ParticipationStatus
            | JSCalendarProperty::PercentComplete
            | JSCalendarProperty::Priority
            | JSCalendarProperty::Privacy
            | JSCalendarProperty::ProdId
            | JSCalendarProperty::Progress
            | JSCalendarProperty::RecurrenceId
            | JSCalendarProperty::RecurrenceIdTimeZone
            | JSCalendarProperty::RecurrenceOverrides
            | JSCalendarProperty::Rel
            | JSCalendarProperty::RelatedTo
            | JSCalendarProperty::Relation
            | JSCalendarProperty::RelativeTo
            | JSCalendarProperty::ReplyTo
            | JSCalendarProperty::RequestStatus
            | JSCalendarProperty::Roles
            | JSCalendarProperty::Rscale
            | JSCalendarProperty::SentBy
            | JSCalendarProperty::ScheduleAgent
            | JSCalendarProperty::ScheduleForceSend
            | JSCalendarProperty::ScheduleSequence
            | JSCalendarProperty::ScheduleStatus
            | JSCalendarProperty::ScheduleUpdated
            | JSCalendarProperty::SendTo
            | JSCalendarProperty::Sequence
            | JSCalendarProperty::ShowWithoutTime
            | JSCalendarProperty::Size
            | JSCalendarProperty::Skip
            | JSCalendarProperty::Source
            | JSCalendarProperty::Start
            | JSCalendarProperty::Status
            | JSCalendarProperty::TimeZone
            | JSCalendarProperty::Title
            | JSCalendarProperty::Trigger
            | JSCalendarProperty::Uid
            | JSCalendarProperty::Until
            | JSCalendarProperty::Updated
            | JSCalendarProperty::Uri
            | JSCalendarProperty::Version
            | JSCalendarProperty::VirtualLocations
            | JSCalendarProperty::When
            | JSCalendarProperty::EndTimeZone
            | JSCalendarProperty::MainLocationId
            | JSCalendarProperty::OrganizerCalendarAddress
            | JSCalendarProperty::RecurrenceRule
            | JSCalendarProperty::ICalendar
            | JSCalendarProperty::Properties
            | JSCalendarProperty::Parameters
            | JSCalendarProperty::ConvertedProperties
            | JSCalendarProperty::ValueType
            | JSCalendarProperty::Components => false,
        }
    }

    pub(crate) fn is_forbidden_override_patch(&self) -> bool {
        matches!(
            self,
            JSCalendarProperty::Type
                | JSCalendarProperty::Method
                | JSCalendarProperty::OrganizerCalendarAddress
                | JSCalendarProperty::Privacy
                | JSCalendarProperty::ProdId
                | JSCalendarProperty::RecurrenceId
                | JSCalendarProperty::RecurrenceIdTimeZone
                | JSCalendarProperty::SentBy
                | JSCalendarProperty::Uid
                | JSCalendarProperty::RecurrenceOverrides
                | JSCalendarProperty::RecurrenceRule
        )
    }

    pub(crate) fn is_forbidden_override_pointer(pointer: &JsonPointer<Self>) -> bool {
        let mut items = pointer
            .iter()
            .filter(|item| !matches!(item, JsonPointerItem::Root));

        match (items.next(), items.next(), items.next(), items.next()) {
            (
                Some(JsonPointerItem::Key(Key::Property(
                    JSCalendarProperty::RecurrenceOverrides | JSCalendarProperty::RecurrenceRule,
                ))),
                ..,
            ) => true,
            (Some(JsonPointerItem::Key(Key::Property(property))), None, ..) => {
                property.is_forbidden_override_patch()
            }
            (
                Some(JsonPointerItem::Key(Key::Property(JSCalendarProperty::Participants))),
                Some(_),
                Some(JsonPointerItem::Key(Key::Property(JSCalendarProperty::CalendarAddress))),
                None,
            ) => true,
            _ => false,
        }
    }
}

impl FromStr for JSCalendarType {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        hashify::map!(s.as_bytes(), JSCalendarType,
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
        .copied()
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
        hashify::map!(s.as_bytes(), JSCalendarAlertAction,
            "display" => JSCalendarAlertAction::Display,
            "email" => JSCalendarAlertAction::Email
        )
        .copied()
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
        hashify::map!(s.as_bytes(), JSCalendarLinkDisplay,
            "badge" => JSCalendarLinkDisplay::Badge,
            "graphic" => JSCalendarLinkDisplay::Graphic,
            "fullsize" => JSCalendarLinkDisplay::Fullsize,
            "thumbnail" => JSCalendarLinkDisplay::Thumbnail
        )
        .copied()
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
        hashify::map!(s.as_bytes(), JSCalendarVirtualLocationFeature,
            "audio" => JSCalendarVirtualLocationFeature::Audio,
            "chat" => JSCalendarVirtualLocationFeature::Chat,
            "feed" => JSCalendarVirtualLocationFeature::Feed,
            "moderator" => JSCalendarVirtualLocationFeature::Moderator,
            "phone" => JSCalendarVirtualLocationFeature::Phone,
            "screen" => JSCalendarVirtualLocationFeature::Screen,
            "video" => JSCalendarVirtualLocationFeature::Video
        )
        .copied()
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
        hashify::map!(s.as_bytes(), JSCalendarFreeBusyStatus,
            "free" => JSCalendarFreeBusyStatus::Free,
            "busy" => JSCalendarFreeBusyStatus::Busy
        )
        .copied()
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
        hashify::map!(s.as_bytes(), JSCalendarParticipantKind,
            "individual" => JSCalendarParticipantKind::Individual,
            "group" => JSCalendarParticipantKind::Group,
            "resource" => JSCalendarParticipantKind::Resource,
            "location" => JSCalendarParticipantKind::Location
        )
        .copied()
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
        hashify::map!(s.as_bytes(), JSCalendarParticipationStatus,
            "needs-action" => JSCalendarParticipationStatus::NeedsAction,
            "accepted" => JSCalendarParticipationStatus::Accepted,
            "declined" => JSCalendarParticipationStatus::Declined,
            "tentative" => JSCalendarParticipationStatus::Tentative,
            "delegated" => JSCalendarParticipationStatus::Delegated
        )
        .copied()
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
        hashify::map!(s.as_bytes(), JSCalendarPrivacy,
            "public" => JSCalendarPrivacy::Public,
            "private" => JSCalendarPrivacy::Private,
            "secret" => JSCalendarPrivacy::Secret
        )
        .copied()
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
        hashify::map!(s.as_bytes(), JSCalendarProgress,
            "needs-action" => JSCalendarProgress::NeedsAction,
            "in-process" => JSCalendarProgress::InProcess,
            "completed" => JSCalendarProgress::Completed,
            "failed" => JSCalendarProgress::Failed,
            "cancelled" => JSCalendarProgress::Cancelled,
        )
        .copied()
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

impl FromStr for JSCalendarRelation {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        hashify::map!(s.as_bytes(), JSCalendarRelation,
            "first" => JSCalendarRelation::First,
            "next" => JSCalendarRelation::Next,
            "child" => JSCalendarRelation::Child,
            "parent" => JSCalendarRelation::Parent,
            "snooze" => JSCalendarRelation::Snooze,
        )
        .copied()
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
        hashify::map!(s.as_bytes(), JSCalendarRelativeTo,
            "start" => JSCalendarRelativeTo::Start,
            "end" => JSCalendarRelativeTo::End
        )
        .copied()
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
        hashify::map!(s.as_bytes(), JSCalendarParticipantRole,
            "owner" => JSCalendarParticipantRole::Owner,
            "optional" => JSCalendarParticipantRole::Optional,
            "informational" => JSCalendarParticipantRole::Informational,
            "chair" => JSCalendarParticipantRole::Chair,
            "required" => JSCalendarParticipantRole::Required,
        )
        .copied()
        .ok_or(())
    }
}

impl JSCalendarParticipantRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            JSCalendarParticipantRole::Owner => "owner",
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
        hashify::map!(s.as_bytes(), JSCalendarScheduleAgent,
            "server" => JSCalendarScheduleAgent::Server,
            "client" => JSCalendarScheduleAgent::Client,
            "none" => JSCalendarScheduleAgent::None
        )
        .copied()
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
        hashify::map!(s.as_bytes(), JSCalendarEventStatus,
            "confirmed" => JSCalendarEventStatus::Confirmed,
            "cancelled" => JSCalendarEventStatus::Cancelled,
            "tentative" => JSCalendarEventStatus::Tentative
        )
        .copied()
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
