/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

pub mod export;
pub mod import;
pub mod parser;
pub mod types;

use crate::{
    common::{CalendarScale, IanaString, LinkRelation},
    icalendar::{
        ICalendarDuration, ICalendarFrequency, ICalendarMethod, ICalendarMonth, ICalendarSkip,
        ICalendarWeekday,
    },
};
use jmap_tools::{JsonPointer, Value};
use mail_parser::DateTime;
use serde::Serialize;
use std::borrow::Cow;

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
    Frequency(ICalendarFrequency),
    CalendarScale(CalendarScale),
    Skip(ICalendarSkip),
    Weekday(ICalendarWeekday),
    Month(ICalendarMonth),
    Method(ICalendarMethod),
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
    ThisAndFuture,
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

// JSCalendar Enum Values for action (Context: Alert)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum JSCalendarAlertAction {
    Display,
    Email,
}

// JSCalendar Enum Values for display (Context: Link)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum JSCalendarLinkDisplay {
    Badge,
    Graphic,
    Fullsize,
    Thumbnail,
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

// JSCalendar Enum Values for freeBusyStatus (Context: Event, Task)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum JSCalendarFreeBusyStatus {
    Free,
    Busy,
}

// JSCalendar Enum Values for kind (Context: Participant)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum JSCalendarParticipantKind {
    Individual,
    Group,
    Resource,
    Location,
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

// JSCalendar Enum Values for privacy (Context: Event, Task)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum JSCalendarPrivacy {
    Public,
    Private,
    Secret,
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

// JSCalendar Enum Values for relation (Context: Relation)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum JSCalendarRelation {
    First,
    Next,
    Child,
    Parent,
    Snooze,
}

// JSCalendar Enum Values for relativeTo (Context: OffsetTrigger, Location)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum JSCalendarRelativeTo {
    Start,
    End,
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

// JSCalendar Enum Values for scheduleAgent (Context: Participant)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum JSCalendarScheduleAgent {
    Server,
    Client,
    None,
}

// JSCalendar Enum Values for status (Context: Event)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum JSCalendarEventStatus {
    Confirmed,
    Cancelled,
    Tentative,
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

// 7f1e1965-ae73-4454-b088-232c90730ce2
static JSCAL_NAMESPACE: uuid::Uuid = uuid::Uuid::from_bytes([
    127, 30, 25, 101, 174, 115, 68, 84, 176, 136, 35, 44, 144, 115, 12, 226,
]);

#[inline]
pub(crate) fn uuid5(text: &str) -> String {
    uuid::Uuid::new_v5(&JSCAL_NAMESPACE, text.as_bytes())
        .hyphenated()
        .to_string()
}
