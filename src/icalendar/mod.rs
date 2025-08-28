/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

use crate::common::{
    CalendarScale, Data, IanaParse, IanaString, IanaType, LinkRelation, PartialDateTime,
};
use std::hash::{Hash, Hasher};

pub mod builder;
pub mod dates;
pub mod parser;
pub mod timezone;
pub mod types;
pub mod utils;
pub mod writer;

#[cfg(feature = "rkyv")]
pub mod rkyv_timezone;
#[cfg(feature = "rkyv")]
pub mod rkyv_types;
#[cfg(feature = "rkyv")]
pub mod rkyv_utils;
#[cfg(feature = "rkyv")]
pub mod rkyv_writer;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize)
)]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Serialize, rkyv::Deserialize, rkyv::Archive)
)]
#[cfg_attr(feature = "rkyv", rkyv(compare(PartialEq)))]
pub struct ICalendar {
    pub components: Vec<ICalendarComponent>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize)
)]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Serialize, rkyv::Deserialize, rkyv::Archive)
)]
#[cfg_attr(feature = "rkyv", rkyv(compare(PartialEq)))]
pub struct ICalendarComponent {
    pub component_type: ICalendarComponentType,
    pub entries: Vec<ICalendarEntry>,
    pub component_ids: Vec<u16>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize)
)]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Serialize, rkyv::Deserialize, rkyv::Archive)
)]
#[cfg_attr(feature = "rkyv", rkyv(compare(PartialEq)))]
pub struct ICalendarEntry {
    pub name: ICalendarProperty,
    pub params: Vec<ICalendarParameter>,
    pub values: Vec<ICalendarValue>,
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize)
)]
#[cfg_attr(feature = "serde", serde(tag = "type", content = "data"))]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Serialize, rkyv::Deserialize, rkyv::Archive)
)]
#[cfg_attr(feature = "rkyv", rkyv(compare(PartialEq)))]
pub enum ICalendarValue {
    Binary(Vec<u8>),
    Boolean(bool),
    Uri(Uri),
    PartialDateTime(Box<PartialDateTime>),
    Duration(ICalendarDuration),
    RecurrenceRule(Box<ICalendarRecurrenceRule>),
    Period(ICalendarPeriod),
    Float(f64),
    Integer(i64),
    Text(String),
    CalendarScale(CalendarScale),
    Method(ICalendarMethod),
    Classification(ICalendarClassification),
    Status(ICalendarStatus),
    Transparency(ICalendarTransparency),
    Action(ICalendarAction),
    BusyType(ICalendarFreeBusyType),
    ParticipantType(ICalendarParticipantType),
    ResourceType(ICalendarResourceType),
    Proximity(ICalendarProximityValue),
}

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize)
)]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Serialize, rkyv::Deserialize, rkyv::Archive)
)]
#[cfg_attr(feature = "rkyv", rkyv(compare(PartialEq), derive(Debug)))]
pub struct ICalendarRecurrenceRule {
    pub freq: ICalendarFrequency,
    pub until: Option<PartialDateTime>,
    pub count: Option<u32>,
    pub interval: Option<u16>,
    pub bysecond: Vec<u8>,
    pub byminute: Vec<u8>,
    pub byhour: Vec<u8>,
    pub byday: Vec<ICalendarDay>,
    pub bymonthday: Vec<i8>,
    pub byyearday: Vec<i16>,
    pub byweekno: Vec<i8>,
    pub bymonth: Vec<ICalendarMonth>,
    pub bysetpos: Vec<i32>,
    pub wkst: Option<ICalendarWeekday>,
    pub rscale: Option<CalendarScale>,
    pub skip: Option<ICalendarSkip>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize)
)]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Serialize, rkyv::Deserialize, rkyv::Archive)
)]
#[cfg_attr(feature = "rkyv", rkyv(compare(PartialEq), derive(Debug)))]
#[repr(u8)]
pub enum ICalendarSkip {
    Omit,
    Backward,
    Forward,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize)
)]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Serialize, rkyv::Deserialize, rkyv::Archive)
)]
#[cfg_attr(feature = "rkyv", rkyv(compare(PartialEq), derive(Debug)))]
pub struct ICalendarDay {
    pub ordwk: Option<i16>,
    pub weekday: ICalendarWeekday,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize)
)]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Serialize, rkyv::Deserialize, rkyv::Archive)
)]
#[cfg_attr(feature = "rkyv", rkyv(compare(PartialEq), derive(Debug)))]
#[repr(transparent)]
pub struct ICalendarMonth(i8);

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize)
)]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Serialize, rkyv::Deserialize, rkyv::Archive)
)]
#[cfg_attr(feature = "rkyv", rkyv(compare(PartialEq), derive(Debug)))]
#[repr(u8)]
pub enum ICalendarFrequency {
    Yearly = 0,
    Monthly = 1,
    Weekly = 2,
    #[default]
    Daily = 3,
    Hourly = 4,
    Minutely = 5,
    Secondly = 6,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize)
)]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Serialize, rkyv::Deserialize, rkyv::Archive)
)]
#[cfg_attr(feature = "rkyv", rkyv(compare(PartialEq), derive(Debug)))]
pub enum ICalendarWeekday {
    Sunday,
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize)
)]
#[cfg_attr(feature = "serde", serde(tag = "type", content = "data"))]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Serialize, rkyv::Deserialize, rkyv::Archive)
)]
#[cfg_attr(feature = "rkyv", rkyv(compare(PartialEq), derive(Debug)))]
pub enum ICalendarPeriod {
    Range {
        start: PartialDateTime,
        end: PartialDateTime,
    },
    Duration {
        start: PartialDateTime,
        duration: ICalendarDuration,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize)
)]
#[cfg_attr(feature = "serde", serde(tag = "type", content = "data"))]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Serialize, rkyv::Deserialize, rkyv::Archive)
)]
#[cfg_attr(feature = "rkyv", rkyv(compare(PartialEq), derive(Debug)))]
pub enum ICalendarAction {
    Audio,     // [RFC5545, Section 3.8.6.1]
    Display,   // [RFC5545, Section 3.8.6.1]
    Email,     // [RFC5545, Section 3.8.6.1]
    Procedure, // [RFC2445, Section 4.8.6.1]
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize)
)]
#[cfg_attr(feature = "serde", serde(tag = "type", content = "data"))]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Serialize, rkyv::Deserialize, rkyv::Archive)
)]
#[cfg_attr(feature = "rkyv", rkyv(compare(PartialEq), derive(Debug)))]
pub enum ICalendarUserTypes {
    Individual, // [RFC5545, Section 3.2.3]
    Group,      // [RFC5545, Section 3.2.3]
    Resource,   // [RFC5545, Section 3.2.3]
    Room,       // [RFC5545, Section 3.2.3]
    Unknown,    // [RFC5545, Section 3.2.3]
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize)
)]
#[cfg_attr(feature = "serde", serde(tag = "type", content = "data"))]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Serialize, rkyv::Deserialize, rkyv::Archive)
)]
#[cfg_attr(feature = "rkyv", rkyv(compare(PartialEq), derive(Debug)))]
pub enum ICalendarClassification {
    Public,       // [RFC5545, Section 3.8.1.3]
    Private,      // [RFC5545, Section 3.8.1.3]
    Confidential, // [RFC5545, Section 3.8.1.3]
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize)
)]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Serialize, rkyv::Deserialize, rkyv::Archive)
)]
#[cfg_attr(feature = "rkyv", rkyv(compare(PartialEq), derive(Debug)))]
pub enum ICalendarComponentType {
    Other(String),
    #[default]
    VCalendar, // [RFC5545, Section 3.4]
    VEvent,        // [RFC5545, Section 3.6.1]
    VTodo,         // [RFC5545, Section 3.6.2]
    VJournal,      // [RFC5545, Section 3.6.3]
    VFreebusy,     // [RFC5545, Section 3.6.4]
    VTimezone,     // [RFC5545, Section 3.6.5]
    VAlarm,        // [RFC5545, Section 3.6.6]
    Standard,      // [RFC5545, Section 3.6.5]
    Daylight,      // [RFC5545, Section 3.6.5]
    VAvailability, // [RFC7953, Section 3.1]
    Available,     // [RFC7953, Section 3.1]
    Participant,   // [RFC9073, Section 7.1]
    VLocation,     // [RFC9073, Section 7.2] [RFC Errata 7381]
    VResource,     // [RFC9073, Section 7.3]
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize)
)]
#[cfg_attr(feature = "serde", serde(tag = "type", content = "data"))]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Serialize, rkyv::Deserialize, rkyv::Archive)
)]
#[cfg_attr(feature = "rkyv", rkyv(compare(PartialEq), derive(Debug)))]
pub enum ICalendarDisplayType {
    Badge,     // [RFC7986, Section 6.1]
    Graphic,   // [RFC7986, Section 6.1]
    Fullsize,  // [RFC7986, Section 6.1]
    Thumbnail, // [RFC7986, Section 6.1]
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize)
)]
#[cfg_attr(feature = "serde", serde(tag = "type", content = "data"))]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Serialize, rkyv::Deserialize, rkyv::Archive)
)]
#[cfg_attr(feature = "rkyv", rkyv(compare(PartialEq), derive(Debug)))]
pub enum ICalendarFeatureType {
    Audio,     // [RFC7986, Section 6.3]
    Chat,      // [RFC7986, Section 6.3]
    Feed,      // [RFC7986, Section 6.3]
    Moderator, // [RFC7986, Section 6.3]
    Phone,     // [RFC7986, Section 6.3]
    Screen,    // [RFC7986, Section 6.3]
    Video,     // [RFC7986, Section 6.3]
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize)
)]
#[cfg_attr(feature = "serde", serde(tag = "type", content = "data"))]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Serialize, rkyv::Deserialize, rkyv::Archive)
)]
#[cfg_attr(feature = "rkyv", rkyv(compare(PartialEq), derive(Debug)))]
pub enum ICalendarFreeBusyType {
    Free,            // [RFC5545, Section 3.2.9]
    Busy,            // [RFC5545, Section 3.2.9]
    BusyUnavailable, // [RFC5545, Section 3.2.9]
    BusyTentative,   // [RFC5545, Section 3.2.9]
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize)
)]
#[cfg_attr(feature = "serde", serde(tag = "type", content = "data"))]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Serialize, rkyv::Deserialize, rkyv::Archive)
)]
#[cfg_attr(feature = "rkyv", rkyv(compare(PartialEq), derive(Debug)))]
pub enum ICalendarMethod {
    Publish,        // [RFC5546]
    Request,        // [RFC5546]
    Reply,          // [RFC5546]
    Add,            // [RFC5546]
    Cancel,         // [RFC5546]
    Refresh,        // [RFC5546]
    Counter,        // [RFC5546]
    Declinecounter, // [RFC5546]
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize)
)]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Serialize, rkyv::Deserialize, rkyv::Archive)
)]
#[cfg_attr(feature = "rkyv", rkyv(compare(PartialEq)))]
pub struct ICalendarParameter {
    pub name: ICalendarParameterName,
    pub value: ICalendarParameterValue,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize)
)]
#[cfg_attr(feature = "serde", serde(tag = "type", content = "data"))]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Serialize, rkyv::Deserialize, rkyv::Archive)
)]
#[cfg_attr(feature = "rkyv", rkyv(compare(PartialEq)))]
pub enum ICalendarParameterValue {
    Text(String),
    Integer(u64),
    Bool(bool),
    Uri(Uri),
    Cutype(ICalendarUserTypes),
    Fbtype(ICalendarFreeBusyType),
    Partstat(ICalendarParticipationStatus),
    Related(Related),
    Reltype(ICalendarRelationshipType),
    Role(ICalendarParticipationRole),
    ScheduleAgent(ICalendarScheduleAgentValue),
    ScheduleForceSend(ICalendarScheduleForceSendValue),
    Value(ICalendarValueType),
    Display(ICalendarDisplayType),
    Feature(ICalendarFeatureType),
    Duration(ICalendarDuration),
    Linkrel(LinkRelation),
    Null,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize)
)]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Serialize, rkyv::Deserialize, rkyv::Archive)
)]
#[cfg_attr(feature = "rkyv", rkyv(compare(PartialEq), derive(PartialEq)))]
pub enum ICalendarParameterName {
    Other(String),
    Altrep,            // [RFC5545, Section 3.2.1]
    Cn,                // [RFC5545, Section 3.2.2]
    Cutype,            // [RFC5545, Section 3.2.3]
    DelegatedFrom,     // [RFC5545, Section 3.2.4]
    DelegatedTo,       // [RFC5545, Section 3.2.5]
    Dir,               // [RFC5545, Section 3.2.6]
    Fmttype,           // [RFC5545, Section 3.2.8]
    Fbtype,            // [RFC5545, Section 3.2.9]
    Language,          // [RFC5545, Section 3.2.10]
    Member,            // [RFC5545, Section 3.2.11]
    Partstat,          // [RFC5545, Section 3.2.12]
    Range,             // [RFC5545, Section 3.2.13]
    Related,           // [RFC5545, Section 3.2.14]
    Reltype,           // [RFC5545, Section 3.2.15]
    Role,              // [RFC5545, Section 3.2.16]
    Rsvp,              // [RFC5545, Section 3.2.17]
    ScheduleAgent,     // [RFC6638, Section 7.1]
    ScheduleForceSend, // [RFC6638, Section 7.2]
    ScheduleStatus,    // [RFC6638, Section 7.3]
    SentBy,            // [RFC5545, Section 3.2.18]
    Tzid,              // [RFC5545, Section 3.2.19]
    Value,             // [RFC5545, Section 3.2.20]
    Display,           // [RFC7986, Section 6.1]
    Email,             // [RFC7986, Section 6.2]
    Feature,           // [RFC7986, Section 6.3]
    Label,             // [RFC7986, Section 6.4]
    Size,              // [RFC8607, Section 4.1]
    Filename,          // [RFC8607, Section 4.2]
    ManagedId,         // [RFC8607, Section 4.3]
    Order,             // [RFC9073, Section 5.1]
    Schema,            // [RFC9073, Section 5.2]
    Derived,           // [RFC9073, Section 5.3]
    Gap,               // [RFC9253, Section 6.2]
    Linkrel,           // [RFC9253, Section 6.1]
    Jsptr,             // draft-ietf-calext-jscalendar-icalendar
    Jsid,              // draft-ietf-calext-jscalendar-icalendar
}

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize)
)]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Serialize, rkyv::Deserialize, rkyv::Archive)
)]
#[cfg_attr(feature = "rkyv", rkyv(compare(PartialEq), derive(Debug)))]
pub struct ICalendarDuration {
    pub neg: bool,
    pub weeks: u32,
    pub days: u32,
    pub hours: u32,
    pub minutes: u32,
    pub seconds: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize)
)]
#[cfg_attr(feature = "serde", serde(tag = "type", content = "data"))]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Serialize, rkyv::Deserialize, rkyv::Archive)
)]
#[cfg_attr(feature = "rkyv", rkyv(compare(PartialEq), derive(Debug)))]
pub enum Uri {
    Data(Data),
    Location(String),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize)
)]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Serialize, rkyv::Deserialize, rkyv::Archive)
)]
#[cfg_attr(feature = "rkyv", rkyv(compare(PartialEq), derive(Debug)))]
pub enum Related {
    Start,
    End,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize)
)]
#[cfg_attr(feature = "serde", serde(tag = "type", content = "data"))]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Serialize, rkyv::Deserialize, rkyv::Archive)
)]
#[cfg_attr(feature = "rkyv", rkyv(compare(PartialEq), derive(Debug)))]
pub enum ICalendarParticipantType {
    Active,           // [RFC9073, Section 6.2]
    Inactive,         // [RFC9073, Section 6.2]
    Sponsor,          // [RFC9073, Section 6.2]
    Contact,          // [RFC9073, Section 6.2]
    BookingContact,   // [RFC9073, Section 6.2]
    EmergencyContact, // [RFC9073, Section 6.2]
    PublicityContact, // [RFC9073, Section 6.2]
    PlannerContact,   // [RFC9073, Section 6.2]
    Performer,        // [RFC9073, Section 6.2]
    Speaker,          // [RFC9073, Section 6.2]
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize)
)]
#[cfg_attr(feature = "serde", serde(tag = "type", content = "data"))]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Serialize, rkyv::Deserialize, rkyv::Archive)
)]
#[cfg_attr(feature = "rkyv", rkyv(compare(PartialEq), derive(Debug)))]
pub enum ICalendarParticipationRole {
    Chair,          // [RFC5545, Section 3.2.16]
    ReqParticipant, // [RFC5545, Section 3.2.16]
    OptParticipant, // [RFC5545, Section 3.2.16]
    NonParticipant, // [RFC5545, Section 3.2.16]
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize)
)]
#[cfg_attr(feature = "serde", serde(tag = "type", content = "data"))]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Serialize, rkyv::Deserialize, rkyv::Archive)
)]
#[cfg_attr(feature = "rkyv", rkyv(compare(PartialEq), derive(Debug)))]
pub enum ICalendarStatus {
    Tentative,   // [RFC5545, Section 3.8.1]
    Confirmed,   // [RFC5545, Section 3.8.1]
    Cancelled,   // [RFC5545, Section 3.8.1]
    NeedsAction, // [RFC5545, Section 3.8.1]
    Completed,   // [RFC5545, Section 3.8.1]
    InProcess,   // [RFC5545, Section 3.8.1]
    Draft,       // [RFC5545, Section 3.8.1]
    Final,       // [RFC5545, Section 3.8.1]
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize)
)]
#[cfg_attr(feature = "serde", serde(tag = "type", content = "data"))]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Serialize, rkyv::Deserialize, rkyv::Archive)
)]
#[cfg_attr(feature = "rkyv", rkyv(compare(PartialEq), derive(Debug)))]
pub enum ICalendarParticipationStatus {
    NeedsAction, // [RFC5545, Section 3.2.12]
    Accepted,    // [RFC5545, Section 3.2.12]
    Declined,    // [RFC5545, Section 3.2.12]
    Tentative,   // [RFC5545, Section 3.2.12]
    Delegated,   // [RFC5545, Section 3.2.12]
    Completed,   // [RFC5545, Section 3.2.12]
    InProcess,   // [RFC5545, Section 3.2.12]
    Failed,      // ical-tasks
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize)
)]
#[cfg_attr(feature = "serde", serde(tag = "type", content = "data"))]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Serialize, rkyv::Deserialize, rkyv::Archive)
)]
#[cfg_attr(feature = "rkyv", rkyv(compare(PartialEq), derive(Debug)))]
pub enum ICalendarProperty {
    Other(String),
    Calscale,          // [RFC5545, Section 3.7.1]
    Method,            // [RFC5545, Section 3.7.2]
    Prodid,            // [RFC5545, Section 3.7.3]
    Version,           // [RFC5545, Section 3.7.4]
    Attach,            // [RFC5545, Section 3.8.1.1]
    Categories,        // [RFC5545, Section 3.8.1.2] [RFC7986, Section 5.6]
    Class,             // [RFC5545, Section 3.8.1.3]
    Comment,           // [RFC5545, Section 3.8.1.4]
    Description,       // [RFC5545, Section 3.8.1.5] [RFC7986, Section 5.2]
    Geo,               // [RFC5545, Section 3.8.1.6]
    Location,          // [RFC5545, Section 3.8.1.7]
    PercentComplete,   // [RFC5545, Section 3.8.1.8]
    Priority,          // [RFC5545, Section 3.8.1.9]
    Resources,         // [RFC5545, Section 3.8.1.10]
    Status,            // [RFC5545, Section 3.8.1.11]
    Summary,           // [RFC5545, Section 3.8.1.12]
    Completed,         // [RFC5545, Section 3.8.2.1]
    Dtend,             // [RFC5545, Section 3.8.2.2]
    Due,               // [RFC5545, Section 3.8.2.3]
    Dtstart,           // [RFC5545, Section 3.8.2.4]
    Duration,          // [RFC5545, Section 3.8.2.5]
    Freebusy,          // [RFC5545, Section 3.8.2.6]
    Transp,            // [RFC5545, Section 3.8.2.7]
    Tzid,              // [RFC5545, Section 3.8.3.1]
    Tzname,            // [RFC5545, Section 3.8.3.2]
    Tzoffsetfrom,      // [RFC5545, Section 3.8.3.3]
    Tzoffsetto,        // [RFC5545, Section 3.8.3.4]
    Tzurl,             // [RFC5545, Section 3.8.3.5]
    Attendee,          // [RFC5545, Section 3.8.4.1]
    Contact,           // [RFC5545, Section 3.8.4.2]
    Organizer,         // [RFC5545, Section 3.8.4.3]
    RecurrenceId,      // [RFC5545, Section 3.8.4.4]
    RelatedTo,         // [RFC5545, Section 3.8.4.5] [RFC9253, Section 9.1]
    Url,               // [RFC5545, Section 3.8.4.6] [RFC7986, Section 5.5]
    Uid,               // [RFC5545, Section 3.8.4.7] [RFC7986, Section 5.3]
    Exdate,            // [RFC5545, Section 3.8.5.1]
    Exrule,            // [RFC2445, Section 4.8.5.2]
    Rdate,             // [RFC5545, Section 3.8.5.2]
    Rrule,             // [RFC5545, Section 3.8.5.3]
    Action,            // [RFC5545, Section 3.8.6.1]
    Repeat,            // [RFC5545, Section 3.8.6.2]
    Trigger,           // [RFC5545, Section 3.8.6.3]
    Created,           // [RFC5545, Section 3.8.7.1]
    Dtstamp,           // [RFC5545, Section 3.8.7.2]
    LastModified,      // [RFC5545, Section 3.8.7.3] [RFC7986, Section 5.4]
    Sequence,          // [RFC5545, Section 3.8.7.4]
    RequestStatus,     // [RFC5545, Section 3.8.8.3]
    Xml,               // [RFC6321, Section 4.2]
    Tzuntil,           // [RFC7808, Section 7.1]
    TzidAliasOf,       // [RFC7808, Section 7.2]
    Busytype,          // [RFC7953, Section 3.2]
    Name,              // [RFC7986, Section 5.1]
    RefreshInterval,   // [RFC7986, Section 5.7]
    Source,            // [RFC7986, Section 5.8]
    Color,             // [RFC7986, Section 5.9]
    Image,             // [RFC7986, Section 5.10]
    Conference,        // [RFC7986, Section 5.11]
    CalendarAddress,   // [RFC9073, Section 6.4]
    LocationType,      // [RFC9073, Section 6.1]
    ParticipantType,   // [RFC9073, Section 6.2]
    ResourceType,      // [RFC9073, Section 6.3]
    StructuredData,    // [RFC9073, Section 6.6]
    StyledDescription, // [RFC9073, Section 6.5]
    Acknowledged,      // [RFC9074, Section 6.1]
    Proximity,         // [RFC9074, Section 8.1]
    Concept,           // [RFC9253, Section 8.1]
    Link,              // [RFC9253, Section 8.2]
    Refid,             // [RFC9253, Section 8.3]
    Coordinates,       // draft-ietf-calext-icalendar-jscalendar-extensions
    ShowWithoutTime,   // draft-ietf-calext-icalendar-jscalendar-extensions
    Jsid,              // draft-ietf-calext-jscalendar-icalendar
    Jsprop,            // draft-ietf-calext-jscalendar-icalendar
    Begin,
    End,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize)
)]
#[cfg_attr(feature = "serde", serde(tag = "type", content = "data"))]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Serialize, rkyv::Deserialize, rkyv::Archive)
)]
#[cfg_attr(feature = "rkyv", rkyv(compare(PartialEq), derive(Debug)))]
pub enum ICalendarProximityValue {
    Arrive,     // [RFC9074, Section 8.1]
    Depart,     // [RFC9074, Section 8.1]
    Connect,    // [RFC9074, Section 8.1]
    Disconnect, // [RFC9074, Section 8.1]
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize)
)]
#[cfg_attr(feature = "serde", serde(tag = "type", content = "data"))]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Serialize, rkyv::Deserialize, rkyv::Archive)
)]
#[cfg_attr(feature = "rkyv", rkyv(compare(PartialEq), derive(Debug)))]
pub enum ICalendarRelationshipType {
    Child,          // [RFC5545, Section 3.2.15]
    Parent,         // [RFC5545, Section 3.2.15]
    Sibling,        // [RFC5545, Section 3.2.15]
    Snooze,         // [RFC9074, Section 7.1]
    Concept,        // [RFC9253, Section 5]
    DependsOn,      // [RFC9253, Section 5]
    Finishtofinish, // [RFC9253, Section 4]
    Finishtostart,  // [RFC9253, Section 4]
    First,          // [RFC9253, Section 5]
    Next,           // [RFC9253, Section 5]
    Refid,          // [RFC9253, Section 5]
    Starttofinish,  // [RFC9253, Section 4]
    Starttostart,   // [RFC9253, Section 4]
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize)
)]
#[cfg_attr(feature = "serde", serde(tag = "type", content = "data"))]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Serialize, rkyv::Deserialize, rkyv::Archive)
)]
#[cfg_attr(feature = "rkyv", rkyv(compare(PartialEq), derive(Debug)))]
pub enum ICalendarResourceType {
    Projector,             // [RFC9073, Section 6.3]
    Room,                  // [RFC9073, Section 6.3]
    RemoteConferenceAudio, // [RFC9073, Section 6.3]
    RemoteConferenceVideo, // [RFC9073, Section 6.3]
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize)
)]
#[cfg_attr(feature = "serde", serde(tag = "type", content = "data"))]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Serialize, rkyv::Deserialize, rkyv::Archive)
)]
#[cfg_attr(feature = "rkyv", rkyv(compare(PartialEq), derive(Debug)))]
pub enum ICalendarScheduleAgentValue {
    Server, // [RFC6638, Section 7.1]
    Client, // [RFC6638, Section 7.1]
    None,   // [RFC6638, Section 7.1]
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize)
)]
#[cfg_attr(feature = "serde", serde(tag = "type", content = "data"))]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Serialize, rkyv::Deserialize, rkyv::Archive)
)]
#[cfg_attr(feature = "rkyv", rkyv(compare(PartialEq), derive(Debug)))]
pub enum ICalendarScheduleForceSendValue {
    Request, // [RFC6638, Section 7.2]
    Reply,   // [RFC6638, Section 7.2]
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize)
)]
#[cfg_attr(feature = "serde", serde(tag = "type", content = "data"))]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Serialize, rkyv::Deserialize, rkyv::Archive)
)]
#[cfg_attr(feature = "rkyv", rkyv(compare(PartialEq), derive(Debug)))]
pub enum ICalendarValueType {
    Binary,       // [RFC5545, Section 3.3.1]
    Boolean,      // [RFC5545, Section 3.3.2]
    CalAddress,   // [RFC5545, Section 3.3.3]
    Date,         // [RFC5545, Section 3.3.4]
    DateTime,     // [RFC5545, Section 3.3.5]
    Duration,     // [RFC5545, Section 3.3.6]
    Float,        // [RFC5545, Section 3.3.7]
    Integer,      // [RFC5545, Section 3.3.8]
    Period,       // [RFC5545, Section 3.3.9]
    Recur,        // [RFC5545, Section 3.3.10]
    Text,         // [RFC5545, Section 3.3.11]
    Time,         // [RFC5545, Section 3.3.12]
    Unknown,      // [RFC7265, Section 5]
    Uri,          // [RFC5545, Section 3.3.13]
    UtcOffset,    // [RFC5545, Section 3.3.14]
    XmlReference, // [RFC9253, Section 7]
    Uid,          // [RFC9253, Section 7]
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize)
)]
#[cfg_attr(feature = "serde", serde(tag = "type", content = "data"))]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Serialize, rkyv::Deserialize, rkyv::Archive)
)]
#[cfg_attr(feature = "rkyv", rkyv(compare(PartialEq), derive(Debug)))]
pub enum ICalendarTransparency {
    Opaque,
    Transparent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ValueSeparator {
    None,
    Comma,
    Semicolon,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ValueType {
    Ical(ICalendarValueType),
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
