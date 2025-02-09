use crate::{
    common::{Data, PartialDateTime},
    Token,
};

pub mod parser;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ICalendar {
    pub component_type: ICalendarComponentType,
    pub entries: Vec<ICalendarEntry>,
    pub components: Vec<ICalendar>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ICalendarEntry {
    name: ICalendarProperty,
    params: Vec<ICalendarParameter>,
    values: Vec<ICalendarValue>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ICalendarValue {
    Binary(Vec<u8>),
    Boolean(bool),
    Uri(Uri),
    PartialDateTime(PartialDateTime),
    Duration(ICalendarDuration),
    RecurrenceRule(ICalendarRecurrenceRule),
    Float(f64),
    Integer(i64),
    Text(String),
}

impl Eq for ICalendarValue {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ICalendarRecurrenceRule {
    /*freq: ICalendarFrequency,
    until: Option<ICalendarDateTime>,
    count: Option<u64>,
    interval: Option<u64>,
    bysecond: Option<Vec<u64>>,
    byminute: Option<Vec<u64>>,
    byhour: Option<Vec<u64>>,
    byday: Option<Vec<ICalendarWeekday>>,
    bymonthday: Option<Vec<i64>>,
    byyearday: Option<Vec<i64>>,
    byweekno: Option<Vec<i64>>,
    bymonth: Option<Vec<i64>>,
    bysetpos: Option<Vec<i64>>,
    wkst: Option<ICalendarWeekday>,*/
}

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ICalendarActions {
    Audio,     // [RFC5545, Section 3.8.6.1]
    Display,   // [RFC5545, Section 3.8.6.1]
    Email,     // [RFC5545, Section 3.8.6.1]
    Procedure, // [RFC2445, Section 4.8.6.1]
    Other(String),
}

impl From<Token<'_>> for ICalendarActions {
    fn from(token: Token<'_>) -> Self {
        hashify::tiny_map_ignore_case!(token.text.as_ref(),
            "AUDIO" => ICalendarActions::Audio,
            "DISPLAY" => ICalendarActions::Display,
            "EMAIL" => ICalendarActions::Email,
            "PROCEDURE" => ICalendarActions::Procedure,
        )
        .unwrap_or_else(|| Self::Other(token.into_string()))
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ICalendarUserTypes {
    Individual, // [RFC5545, Section 3.2.3]
    Group,      // [RFC5545, Section 3.2.3]
    Resource,   // [RFC5545, Section 3.2.3]
    Room,       // [RFC5545, Section 3.2.3]
    Unknown,    // [RFC5545, Section 3.2.3]
    Other(String),
}

impl From<Token<'_>> for ICalendarUserTypes {
    fn from(token: Token<'_>) -> Self {
        hashify::tiny_map_ignore_case!(token.text.as_ref(),
            "INDIVIDUAL" => ICalendarUserTypes::Individual,
            "GROUP" => ICalendarUserTypes::Group,
            "RESOURCE" => ICalendarUserTypes::Resource,
            "ROOM" => ICalendarUserTypes::Room,
            "UNKNOWN" => ICalendarUserTypes::Unknown,
        )
        .unwrap_or_else(|| Self::Other(token.into_string()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ICalendarClassifications {
    Public,       // [RFC5545, Section 3.8.1.3]
    Private,      // [RFC5545, Section 3.8.1.3]
    Confidential, // [RFC5545, Section 3.8.1.3]
    Other(String),
}

impl From<Token<'_>> for ICalendarClassifications {
    fn from(token: Token<'_>) -> Self {
        hashify::tiny_map_ignore_case!(token.text.as_ref(),
            "PUBLIC" => ICalendarClassifications::Public,
            "PRIVATE" => ICalendarClassifications::Private,
            "CONFIDENTIAL" => ICalendarClassifications::Confidential,
        )
        .unwrap_or_else(|| Self::Other(token.into_string()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ICalendarComponentType {
    VCalendar,     // [RFC5545, Section 3.4]
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

/*impl From<Token<'_>> for ICalendarComponentType {
    fn from(token: Token<'_>) -> Self {
        hashify::tiny_map_ignore_case!(token.text.as_ref(),
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
        )
        .unwrap_or_else(|| Self::Other(token.into_string()))
    }
}*/

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ICalendarDisplayType {
    Badge,     // [RFC7986, Section 6.1]
    Graphic,   // [RFC7986, Section 6.1]
    Fullsize,  // [RFC7986, Section 6.1]
    Thumbnail, // [RFC7986, Section 6.1]
    Other(String),
}

impl From<Token<'_>> for ICalendarDisplayType {
    fn from(token: Token<'_>) -> Self {
        hashify::tiny_map_ignore_case!(token.text.as_ref(),
            "BADGE" => ICalendarDisplayType::Badge,
            "GRAPHIC" => ICalendarDisplayType::Graphic,
            "FULLSIZE" => ICalendarDisplayType::Fullsize,
            "THUMBNAIL" => ICalendarDisplayType::Thumbnail,
        )
        .unwrap_or_else(|| Self::Other(token.into_string()))
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ICalendarFeatureType {
    Audio,     // [RFC7986, Section 6.3]
    Chat,      // [RFC7986, Section 6.3]
    Feed,      // [RFC7986, Section 6.3]
    Moderator, // [RFC7986, Section 6.3]
    Phone,     // [RFC7986, Section 6.3]
    Screen,    // [RFC7986, Section 6.3]
    Video,     // [RFC7986, Section 6.3]
    Other(String),
}

impl From<Token<'_>> for ICalendarFeatureType {
    fn from(token: Token<'_>) -> Self {
        hashify::tiny_map_ignore_case!(token.text.as_ref(),
            "AUDIO" => ICalendarFeatureType::Audio,
            "CHAT" => ICalendarFeatureType::Chat,
            "FEED" => ICalendarFeatureType::Feed,
            "MODERATOR" => ICalendarFeatureType::Moderator,
            "PHONE" => ICalendarFeatureType::Phone,
            "SCREEN" => ICalendarFeatureType::Screen,
            "VIDEO" => ICalendarFeatureType::Video,
        )
        .unwrap_or_else(|| Self::Other(token.into_string()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ICalendarFreeBusyTimeType {
    Free,            // [RFC5545, Section 3.2.9]
    Busy,            // [RFC5545, Section 3.2.9]
    BusyUnavailable, // [RFC5545, Section 3.2.9]
    BusyTentative,   // [RFC5545, Section 3.2.9]
    Other(String),
}

impl From<Token<'_>> for ICalendarFreeBusyTimeType {
    fn from(token: Token<'_>) -> Self {
        hashify::tiny_map_ignore_case!(token.text.as_ref(),
            "FREE" => ICalendarFreeBusyTimeType::Free,
            "BUSY" => ICalendarFreeBusyTimeType::Busy,
            "BUSY-UNAVAILABLE" => ICalendarFreeBusyTimeType::BusyUnavailable,
            "BUSY-TENTATIVE" => ICalendarFreeBusyTimeType::BusyTentative,
        )
        .unwrap_or_else(|| Self::Other(token.into_string()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ICalendarMethods {
    Publish,        // [RFC5546]
    Request,        // [RFC5546]
    Reply,          // [RFC5546]
    Add,            // [RFC5546]
    Cancel,         // [RFC5546]
    Refresh,        // [RFC5546]
    Counter,        // [RFC5546]
    Declinecounter, // [RFC5546]
    Other(String),
}

impl From<Token<'_>> for ICalendarMethods {
    fn from(token: Token<'_>) -> Self {
        hashify::tiny_map_ignore_case!(token.text.as_ref(),
            "PUBLISH" => ICalendarMethods::Publish,
            "REQUEST" => ICalendarMethods::Request,
            "REPLY" => ICalendarMethods::Reply,
            "ADD" => ICalendarMethods::Add,
            "CANCEL" => ICalendarMethods::Cancel,
            "REFRESH" => ICalendarMethods::Refresh,
            "COUNTER" => ICalendarMethods::Counter,
            "DECLINECOUNTER" => ICalendarMethods::Declinecounter,
        )
        .unwrap_or_else(|| Self::Other(token.into_string()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ICalendarParameter {
    Altrep(Uri),                                        // [RFC5545, Section 3.2.1]
    Cn(String),                                         // [RFC5545, Section 3.2.2]
    Cutype(ICalendarUserTypes),                         // [RFC5545, Section 3.2.3]
    DelegatedFrom(Vec<Uri>),                            // [RFC5545, Section 3.2.4]
    DelegatedTo(Vec<Uri>),                              // [RFC5545, Section 3.2.5]
    Dir(Uri),                                           // [RFC5545, Section 3.2.6]
    Fmttype(String),                                    // [RFC5545, Section 3.2.8]
    Fbtype(ICalendarFreeBusyTimeType),                  // [RFC5545, Section 3.2.9]
    Language(String),                                   // [RFC5545, Section 3.2.10]
    Member(Vec<Uri>),                                   // [RFC5545, Section 3.2.11]
    Partstat(ICalendarParticipationStatus),             // [RFC5545, Section 3.2.12]
    Range,                                              // [RFC5545, Section 3.2.13]
    Related(Related),                                   // [RFC5545, Section 3.2.14]
    Reltype(ICalendarRelationshipType),                 // [RFC5545, Section 3.2.15]
    Role(ICalendarParticipationRole),                   // [RFC5545, Section 3.2.16]
    Rsvp(bool),                                         // [RFC5545, Section 3.2.17]
    ScheduleAgent(ICalendarScheduleAgentValue),         // [RFC6638, Section 7.1]
    ScheduleForceSend(ICalendarScheduleForceSendValue), // [RFC6638, Section 7.2]
    ScheduleStatus(String),                             // [RFC6638, Section 7.3]
    SentBy(Uri),                                        // [RFC5545, Section 3.2.18]
    Tzid(String),                                       // [RFC5545, Section 3.2.19]
    Value(ICalendarValueType),                          // [RFC5545, Section 3.2.20]
    Display(Vec<ICalendarDisplayType>),                 // [RFC7986, Section 6.1]
    Email(String),                                      // [RFC7986, Section 6.2]
    Feature(Vec<ICalendarFeatureType>),                 // [RFC7986, Section 6.3]
    Label(String),                                      // [RFC7986, Section 6.4]
    Size(u64),                                          // [RFC8607, Section 4.1]
    Filename(String),                                   // [RFC8607, Section 4.2]
    ManagedId(String),                                  // [RFC8607, Section 4.3]
    Order(u64),                                         // [RFC9073, Section 5.1]
    Schema(Uri),                                        // [RFC9073, Section 5.2]
    Derived(bool),                                      // [RFC9073, Section 5.3]
    Gap(ICalendarDuration),                             // [RFC9253, Section 6.2]
    Linkrel(Uri),                                       // [RFC9253, Section 6.1]
    Other(Vec<String>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ICalendarDuration {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Uri {
    Data(Data),
    Location(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Related {
    Start,
    End,
}

impl TryFrom<&[u8]> for Related {
    type Error = ();

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        hashify::tiny_map_ignore_case!(value,
            "START" => Related::Start,
            "END" => Related::End,
        )
        .ok_or(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ICalendarParticipantTypes {
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
    Other(String),
}

impl From<Token<'_>> for ICalendarParticipantTypes {
    fn from(token: Token<'_>) -> Self {
        hashify::tiny_map_ignore_case!(token.text.as_ref(),
            "ACTIVE" => ICalendarParticipantTypes::Active,
            "INACTIVE" => ICalendarParticipantTypes::Inactive,
            "SPONSOR" => ICalendarParticipantTypes::Sponsor,
            "CONTACT" => ICalendarParticipantTypes::Contact,
            "BOOKING-CONTACT" => ICalendarParticipantTypes::BookingContact,
            "EMERGENCY-CONTACT" => ICalendarParticipantTypes::EmergencyContact,
            "PUBLICITY-CONTACT" => ICalendarParticipantTypes::PublicityContact,
            "PLANNER-CONTACT" => ICalendarParticipantTypes::PlannerContact,
            "PERFORMER" => ICalendarParticipantTypes::Performer,
            "SPEAKER" => ICalendarParticipantTypes::Speaker,
        )
        .unwrap_or_else(|| Self::Other(token.into_string()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ICalendarParticipationRole {
    Chair,          // [RFC5545, Section 3.2.16]
    ReqParticipant, // [RFC5545, Section 3.2.16]
    OptParticipant, // [RFC5545, Section 3.2.16]
    NonParticipant, // [RFC5545, Section 3.2.16]
    Other(String),
}

impl From<Token<'_>> for ICalendarParticipationRole {
    fn from(token: Token<'_>) -> Self {
        hashify::tiny_map_ignore_case!(token.text.as_ref(),
            "CHAIR" => ICalendarParticipationRole::Chair,
            "REQ-PARTICIPANT" => ICalendarParticipationRole::ReqParticipant,
            "OPT-PARTICIPANT" => ICalendarParticipationRole::OptParticipant,
            "NON-PARTICIPANT" => ICalendarParticipationRole::NonParticipant,
        )
        .unwrap_or_else(|| Self::Other(token.into_string()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ICalendarParticipationStatus {
    NeedsAction, // [RFC5545, Section 3.2.12]
    Accepted,    // [RFC5545, Section 3.2.12]
    Declined,    // [RFC5545, Section 3.2.12]
    Tentative,   // [RFC5545, Section 3.2.12]
    Delegated,   // [RFC5545, Section 3.2.12]
    Completed,   // [RFC5545, Section 3.2.12]
    InProcess,   // [RFC5545, Section 3.2.12]
    Other(String),
}

impl From<Token<'_>> for ICalendarParticipationStatus {
    fn from(token: Token<'_>) -> Self {
        hashify::tiny_map_ignore_case!(token.text.as_ref(),
            "NEEDS-ACTION" => ICalendarParticipationStatus::NeedsAction,
            "ACCEPTED" => ICalendarParticipationStatus::Accepted,
            "DECLINED" => ICalendarParticipationStatus::Declined,
            "TENTATIVE" => ICalendarParticipationStatus::Tentative,
            "DELEGATED" => ICalendarParticipationStatus::Delegated,
            "COMPLETED" => ICalendarParticipationStatus::Completed,
            "IN-PROCESS" => ICalendarParticipationStatus::InProcess,
        )
        .unwrap_or_else(|| Self::Other(token.into_string()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ICalendarProperty {
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
    Other(String),
}

impl From<Token<'_>> for ICalendarProperty {
    fn from(token: Token<'_>) -> Self {
        hashify::tiny_map_ignore_case!(token.text.as_ref(),
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
        )
        .unwrap_or_else(|| Self::Other(token.into_string()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ICalendarProximityValues {
    Arrive,     // [RFC9074, Section 8.1]
    Depart,     // [RFC9074, Section 8.1]
    Connect,    // [RFC9074, Section 8.1]
    Disconnect, // [RFC9074, Section 8.1]
    Other(String),
}

impl From<Token<'_>> for ICalendarProximityValues {
    fn from(token: Token<'_>) -> Self {
        hashify::tiny_map_ignore_case!(token.text.as_ref(),
            "ARRIVE" => ICalendarProximityValues::Arrive,
            "DEPART" => ICalendarProximityValues::Depart,
            "CONNECT" => ICalendarProximityValues::Connect,
            "DISCONNECT" => ICalendarProximityValues::Disconnect,
        )
        .unwrap_or_else(|| Self::Other(token.into_string()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
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
    Other(String),
}

impl From<Token<'_>> for ICalendarRelationshipType {
    fn from(token: Token<'_>) -> Self {
        hashify::tiny_map_ignore_case!(token.text.as_ref(),
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
        .unwrap_or_else(|| Self::Other(token.into_string()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ICalendarResourceTypes {
    Projector,             // [RFC9073, Section 6.3]
    Room,                  // [RFC9073, Section 6.3]
    RemoteConferenceAudio, // [RFC9073, Section 6.3]
    RemoteConferenceVideo, // [RFC9073, Section 6.3]
    Other(String),
}

impl From<Token<'_>> for ICalendarResourceTypes {
    fn from(token: Token<'_>) -> Self {
        hashify::tiny_map_ignore_case!(token.text.as_ref(),
            "PROJECTOR" => ICalendarResourceTypes::Projector,
            "ROOM" => ICalendarResourceTypes::Room,
            "REMOTE-CONFERENCE-AUDIO" => ICalendarResourceTypes::RemoteConferenceAudio,
            "REMOTE-CONFERENCE-VIDEO" => ICalendarResourceTypes::RemoteConferenceVideo,
        )
        .unwrap_or_else(|| Self::Other(token.into_string()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ICalendarScheduleAgentValue {
    Server, // [RFC6638, Section 7.1]
    Client, // [RFC6638, Section 7.1]
    None,   // [RFC6638, Section 7.1]
    Other(String),
}

impl From<Token<'_>> for ICalendarScheduleAgentValue {
    fn from(token: Token<'_>) -> Self {
        hashify::tiny_map_ignore_case!(token.text.as_ref(),
            "SERVER" => ICalendarScheduleAgentValue::Server,
            "CLIENT" => ICalendarScheduleAgentValue::Client,
            "NONE" => ICalendarScheduleAgentValue::None,
        )
        .unwrap_or_else(|| Self::Other(token.into_string()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ICalendarScheduleForceSendValue {
    Request, // [RFC6638, Section 7.2]
    Reply,   // [RFC6638, Section 7.2]
    Other(String),
}

impl From<Token<'_>> for ICalendarScheduleForceSendValue {
    fn from(token: Token<'_>) -> Self {
        hashify::tiny_map_ignore_case!(token.text.as_ref(),
            "REQUEST" => ICalendarScheduleForceSendValue::Request,
            "REPLY" => ICalendarScheduleForceSendValue::Reply,
        )
        .unwrap_or_else(|| Self::Other(token.into_string()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
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
    Other(String),
}

impl From<Token<'_>> for ICalendarValueType {
    fn from(token: Token<'_>) -> Self {
        hashify::tiny_map_ignore_case!(token.text.as_ref(),
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
        .unwrap_or_else(|| Self::Other(token.into_string()))
    }
}
