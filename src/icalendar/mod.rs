use crate::common::Data;

pub mod parser;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ICalendar {}

pub enum ICalendarActions {
    Audio,     // [RFC5545, Section 3.8.6.1]
    Display,   // [RFC5545, Section 3.8.6.1]
    Email,     // [RFC5545, Section 3.8.6.1]
    Procedure, // [RFC2445, Section 4.8.6.1]
}

impl TryFrom<&[u8]> for ICalendarActions {
    type Error = ();
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        hashify::tiny_map_ignore_case!(value,
            "AUDIO" => ICalendarActions::Audio,
            "DISPLAY" => ICalendarActions::Display,
            "EMAIL" => ICalendarActions::Email,
            "PROCEDURE" => ICalendarActions::Procedure,
        )
        .ok_or(())
    }
}
pub enum ICalendarUserTypes {
    Individual, // [RFC5545, Section 3.2.3]
    Group,      // [RFC5545, Section 3.2.3]
    Resource,   // [RFC5545, Section 3.2.3]
    Room,       // [RFC5545, Section 3.2.3]
    Unknown,    // [RFC5545, Section 3.2.3]
    Other(String),
}

impl TryFrom<&[u8]> for ICalendarUserTypes {
    type Error = ();
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        hashify::tiny_map_ignore_case!(value,
            "INDIVIDUAL" => ICalendarUserTypes::Individual,
            "GROUP" => ICalendarUserTypes::Group,
            "RESOURCE" => ICalendarUserTypes::Resource,
            "ROOM" => ICalendarUserTypes::Room,
            "UNKNOWN" => ICalendarUserTypes::Unknown,
        )
        .ok_or(())
    }
}
pub enum ICalendarClassifications {
    Public,       // [RFC5545, Section 3.8.1.3]
    Private,      // [RFC5545, Section 3.8.1.3]
    Confidential, // [RFC5545, Section 3.8.1.3]
}

impl TryFrom<&[u8]> for ICalendarClassifications {
    type Error = ();
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        hashify::tiny_map_ignore_case!(value,
            "PUBLIC" => ICalendarClassifications::Public,
            "PRIVATE" => ICalendarClassifications::Private,
            "CONFIDENTIAL" => ICalendarClassifications::Confidential,
        )
        .ok_or(())
    }
}
pub enum ICalendarComponents {
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

impl TryFrom<&[u8]> for ICalendarComponents {
    type Error = ();
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        hashify::tiny_map_ignore_case!(value,
            "VCALENDAR" => ICalendarComponents::VCalendar,
            "VEVENT" => ICalendarComponents::VEvent,
            "VTODO" => ICalendarComponents::VTodo,
            "VJOURNAL" => ICalendarComponents::VJournal,
            "VFREEBUSY" => ICalendarComponents::VFreebusy,
            "VTIMEZONE" => ICalendarComponents::VTimezone,
            "VALARM" => ICalendarComponents::VAlarm,
            "STANDARD" => ICalendarComponents::Standard,
            "DAYLIGHT" => ICalendarComponents::Daylight,
            "VAVAILABILITY" => ICalendarComponents::VAvailability,
            "AVAILABLE" => ICalendarComponents::Available,
            "PARTICIPANT" => ICalendarComponents::Participant,
            "VLOCATION" => ICalendarComponents::VLocation,
            "VRESOURCE" => ICalendarComponents::VResource,
        )
        .ok_or(())
    }
}
pub enum ICalendarDisplayTypes {
    Badge,     // [RFC7986, Section 6.1]
    Graphic,   // [RFC7986, Section 6.1]
    Fullsize,  // [RFC7986, Section 6.1]
    Thumbnail, // [RFC7986, Section 6.1]
    Other(String),
}

impl TryFrom<&[u8]> for ICalendarDisplayTypes {
    type Error = ();
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        hashify::tiny_map_ignore_case!(value,
            "BADGE" => ICalendarDisplayTypes::Badge,
            "GRAPHIC" => ICalendarDisplayTypes::Graphic,
            "FULLSIZE" => ICalendarDisplayTypes::Fullsize,
            "THUMBNAIL" => ICalendarDisplayTypes::Thumbnail,
        )
        .ok_or(())
    }
}
pub enum ICalendarFeatureTypes {
    Audio,     // [RFC7986, Section 6.3]
    Chat,      // [RFC7986, Section 6.3]
    Feed,      // [RFC7986, Section 6.3]
    Moderator, // [RFC7986, Section 6.3]
    Phone,     // [RFC7986, Section 6.3]
    Screen,    // [RFC7986, Section 6.3]
    Video,     // [RFC7986, Section 6.3]
    Other(String),
}

impl TryFrom<&[u8]> for ICalendarFeatureTypes {
    type Error = ();
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        hashify::tiny_map_ignore_case!(value,
            "AUDIO" => ICalendarFeatureTypes::Audio,
            "CHAT" => ICalendarFeatureTypes::Chat,
            "FEED" => ICalendarFeatureTypes::Feed,
            "MODERATOR" => ICalendarFeatureTypes::Moderator,
            "PHONE" => ICalendarFeatureTypes::Phone,
            "SCREEN" => ICalendarFeatureTypes::Screen,
            "VIDEO" => ICalendarFeatureTypes::Video,
        )
        .ok_or(())
    }
}
pub enum ICalendarFreeBusyTimeTypes {
    Free,            // [RFC5545, Section 3.2.9]
    Busy,            // [RFC5545, Section 3.2.9]
    BusyUnavailable, // [RFC5545, Section 3.2.9]
    BusyTentative,   // [RFC5545, Section 3.2.9]
    Other(String),
}

impl TryFrom<&[u8]> for ICalendarFreeBusyTimeTypes {
    type Error = ();
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        hashify::tiny_map_ignore_case!(value,
            "FREE" => ICalendarFreeBusyTimeTypes::Free,
            "BUSY" => ICalendarFreeBusyTimeTypes::Busy,
            "BUSY-UNAVAILABLE" => ICalendarFreeBusyTimeTypes::BusyUnavailable,
            "BUSY-TENTATIVE" => ICalendarFreeBusyTimeTypes::BusyTentative,
        )
        .ok_or(())
    }
}
pub enum ICalendarMethods {
    Publish,        // [RFC5546]
    Request,        // [RFC5546]
    Reply,          // [RFC5546]
    Add,            // [RFC5546]
    Cancel,         // [RFC5546]
    Refresh,        // [RFC5546]
    Counter,        // [RFC5546]
    Declinecounter, // [RFC5546]
}

impl TryFrom<&[u8]> for ICalendarMethods {
    type Error = ();
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        hashify::tiny_map_ignore_case!(value,
            "PUBLISH" => ICalendarMethods::Publish,
            "REQUEST" => ICalendarMethods::Request,
            "REPLY" => ICalendarMethods::Reply,
            "ADD" => ICalendarMethods::Add,
            "CANCEL" => ICalendarMethods::Cancel,
            "REFRESH" => ICalendarMethods::Refresh,
            "COUNTER" => ICalendarMethods::Counter,
            "DECLINECOUNTER" => ICalendarMethods::Declinecounter,
        )
        .ok_or(())
    }
}
pub enum ICalendarParameter {
    Altrep(Uri),                                         // [RFC5545, Section 3.2.1]
    Cn(String),                                          // [RFC5545, Section 3.2.2]
    Cutype(ICalendarUserTypes),                          // [RFC5545, Section 3.2.3]
    DelegatedFrom(Vec<Uri>),                             // [RFC5545, Section 3.2.4]
    DelegatedTo(Vec<Uri>),                               // [RFC5545, Section 3.2.5]
    Dir(Uri),                                            // [RFC5545, Section 3.2.6]
    Fmttype(String),                                     // [RFC5545, Section 3.2.8]
    Fbtype(ICalendarFreeBusyTimeTypes),                  // [RFC5545, Section 3.2.9]
    Language(String),                                    // [RFC5545, Section 3.2.10]
    Member(Vec<Uri>),                                    // [RFC5545, Section 3.2.11]
    Partstat(ICalendarParticipationStatuses),            // [RFC5545, Section 3.2.12]
    Range,                                               // [RFC5545, Section 3.2.13]
    Related(Related),                                    // [RFC5545, Section 3.2.14]
    Reltype(ICalendarRelationshipTypes),                 // [RFC5545, Section 3.2.15]
    Role(ICalendarParticipationRoles),                   // [RFC5545, Section 3.2.16]
    Rsvp(bool),                                          // [RFC5545, Section 3.2.17]
    ScheduleAgent(ICalendarScheduleAgentValues),         // [RFC6638, Section 7.1]
    ScheduleForceSend(ICalendarScheduleForceSendValues), // [RFC6638, Section 7.2]
    ScheduleStatus(String),                              // [RFC6638, Section 7.3]
    SentBy(Uri),                                         // [RFC5545, Section 3.2.18]
    Tzid(String),                                        // [RFC5545, Section 3.2.19]
    Value(ICalendarValueType),                           // [RFC5545, Section 3.2.20]
    Display(Vec<ICalendarDisplayTypes>),                 // [RFC7986, Section 6.1]
    Email(String),                                       // [RFC7986, Section 6.2]
    Feature(Vec<ICalendarFeatureTypes>),                 // [RFC7986, Section 6.3]
    Label(String),                                       // [RFC7986, Section 6.4]
    Size(u64),                                           // [RFC8607, Section 4.1]
    Filename(String),                                    // [RFC8607, Section 4.2]
    ManagedId(String),                                   // [RFC8607, Section 4.3]
    Order(u64),                                          // [RFC9073, Section 5.1]
    Schema(Uri),                                         // [RFC9073, Section 5.2]
    Derived(bool),                                       // [RFC9073, Section 5.3]
    Gap(ICalendarDuration),                              // [RFC9253, Section 6.2]
    Linkrel(Uri),                                        // [RFC9253, Section 6.1]
    Other(Vec<String>),
}

pub enum Related {
    Start,
    End,
}

pub struct ICalendarDuration {}

pub enum Uri {
    Data(Data),
    Location(String),
}

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
}

impl TryFrom<&[u8]> for ICalendarParticipantTypes {
    type Error = ();
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        hashify::tiny_map_ignore_case!(value,
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
        .ok_or(())
    }
}
pub enum ICalendarParticipationRoles {
    Chair,          // [RFC5545, Section 3.2.16]
    ReqParticipant, // [RFC5545, Section 3.2.16]
    OptParticipant, // [RFC5545, Section 3.2.16]
    NonParticipant, // [RFC5545, Section 3.2.16]
    Other(String),
}

impl TryFrom<&[u8]> for ICalendarParticipationRoles {
    type Error = ();
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        hashify::tiny_map_ignore_case!(value,
            "CHAIR" => ICalendarParticipationRoles::Chair,
            "REQ-PARTICIPANT" => ICalendarParticipationRoles::ReqParticipant,
            "OPT-PARTICIPANT" => ICalendarParticipationRoles::OptParticipant,
            "NON-PARTICIPANT" => ICalendarParticipationRoles::NonParticipant,
        )
        .ok_or(())
    }
}
pub enum ICalendarParticipationStatuses {
    NeedsAction, // [RFC5545, Section 3.2.12]
    Accepted,    // [RFC5545, Section 3.2.12]
    Declined,    // [RFC5545, Section 3.2.12]
    Tentative,   // [RFC5545, Section 3.2.12]
    Delegated,   // [RFC5545, Section 3.2.12]
    Completed,   // [RFC5545, Section 3.2.12]
    InProcess,   // [RFC5545, Section 3.2.12]
    Other(String),
}

impl TryFrom<&[u8]> for ICalendarParticipationStatuses {
    type Error = ();
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        hashify::tiny_map_ignore_case!(value,
            "NEEDS-ACTION" => ICalendarParticipationStatuses::NeedsAction,
            "ACCEPTED" => ICalendarParticipationStatuses::Accepted,
            "DECLINED" => ICalendarParticipationStatuses::Declined,
            "TENTATIVE" => ICalendarParticipationStatuses::Tentative,
            "DELEGATED" => ICalendarParticipationStatuses::Delegated,
            "COMPLETED" => ICalendarParticipationStatuses::Completed,
            "IN-PROCESS" => ICalendarParticipationStatuses::InProcess,
        )
        .ok_or(())
    }
}
pub enum ICalendarProperties {
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
}

impl TryFrom<&[u8]> for ICalendarProperties {
    type Error = ();
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        hashify::tiny_map_ignore_case!(value,
            "CALSCALE" => ICalendarProperties::Calscale,
            "METHOD" => ICalendarProperties::Method,
            "PRODID" => ICalendarProperties::Prodid,
            "VERSION" => ICalendarProperties::Version,
            "ATTACH" => ICalendarProperties::Attach,
            "CATEGORIES" => ICalendarProperties::Categories,
            "CLASS" => ICalendarProperties::Class,
            "COMMENT" => ICalendarProperties::Comment,
            "DESCRIPTION" => ICalendarProperties::Description,
            "GEO" => ICalendarProperties::Geo,
            "LOCATION" => ICalendarProperties::Location,
            "PERCENT-COMPLETE" => ICalendarProperties::PercentComplete,
            "PRIORITY" => ICalendarProperties::Priority,
            "RESOURCES" => ICalendarProperties::Resources,
            "STATUS" => ICalendarProperties::Status,
            "SUMMARY" => ICalendarProperties::Summary,
            "COMPLETED" => ICalendarProperties::Completed,
            "DTEND" => ICalendarProperties::Dtend,
            "DUE" => ICalendarProperties::Due,
            "DTSTART" => ICalendarProperties::Dtstart,
            "DURATION" => ICalendarProperties::Duration,
            "FREEBUSY" => ICalendarProperties::Freebusy,
            "TRANSP" => ICalendarProperties::Transp,
            "TZID" => ICalendarProperties::Tzid,
            "TZNAME" => ICalendarProperties::Tzname,
            "TZOFFSETFROM" => ICalendarProperties::Tzoffsetfrom,
            "TZOFFSETTO" => ICalendarProperties::Tzoffsetto,
            "TZURL" => ICalendarProperties::Tzurl,
            "ATTENDEE" => ICalendarProperties::Attendee,
            "CONTACT" => ICalendarProperties::Contact,
            "ORGANIZER" => ICalendarProperties::Organizer,
            "RECURRENCE-ID" => ICalendarProperties::RecurrenceId,
            "RELATED-TO" => ICalendarProperties::RelatedTo,
            "URL" => ICalendarProperties::Url,
            "UID" => ICalendarProperties::Uid,
            "EXDATE" => ICalendarProperties::Exdate,
            "EXRULE" => ICalendarProperties::Exrule,
            "RDATE" => ICalendarProperties::Rdate,
            "RRULE" => ICalendarProperties::Rrule,
            "ACTION" => ICalendarProperties::Action,
            "REPEAT" => ICalendarProperties::Repeat,
            "TRIGGER" => ICalendarProperties::Trigger,
            "CREATED" => ICalendarProperties::Created,
            "DTSTAMP" => ICalendarProperties::Dtstamp,
            "LAST-MODIFIED" => ICalendarProperties::LastModified,
            "SEQUENCE" => ICalendarProperties::Sequence,
            "REQUEST-STATUS" => ICalendarProperties::RequestStatus,
            "XML" => ICalendarProperties::Xml,
            "TZUNTIL" => ICalendarProperties::Tzuntil,
            "TZID-ALIAS-OF" => ICalendarProperties::TzidAliasOf,
            "BUSYTYPE" => ICalendarProperties::Busytype,
            "NAME" => ICalendarProperties::Name,
            "REFRESH-INTERVAL" => ICalendarProperties::RefreshInterval,
            "SOURCE" => ICalendarProperties::Source,
            "COLOR" => ICalendarProperties::Color,
            "IMAGE" => ICalendarProperties::Image,
            "CONFERENCE" => ICalendarProperties::Conference,
            "CALENDAR-ADDRESS" => ICalendarProperties::CalendarAddress,
            "LOCATION-TYPE" => ICalendarProperties::LocationType,
            "PARTICIPANT-TYPE" => ICalendarProperties::ParticipantType,
            "RESOURCE-TYPE" => ICalendarProperties::ResourceType,
            "STRUCTURED-DATA" => ICalendarProperties::StructuredData,
            "STYLED-DESCRIPTION" => ICalendarProperties::StyledDescription,
            "ACKNOWLEDGED" => ICalendarProperties::Acknowledged,
            "PROXIMITY" => ICalendarProperties::Proximity,
            "CONCEPT" => ICalendarProperties::Concept,
            "LINK" => ICalendarProperties::Link,
            "REFID" => ICalendarProperties::Refid,
        )
        .ok_or(())
    }
}
pub enum ICalendarProximityValues {
    Arrive,     // [RFC9074, Section 8.1]
    Depart,     // [RFC9074, Section 8.1]
    Connect,    // [RFC9074, Section 8.1]
    Disconnect, // [RFC9074, Section 8.1]
}

impl TryFrom<&[u8]> for ICalendarProximityValues {
    type Error = ();
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        hashify::tiny_map_ignore_case!(value,
            "ARRIVE" => ICalendarProximityValues::Arrive,
            "DEPART" => ICalendarProximityValues::Depart,
            "CONNECT" => ICalendarProximityValues::Connect,
            "DISCONNECT" => ICalendarProximityValues::Disconnect,
        )
        .ok_or(())
    }
}
pub enum ICalendarRelationshipTypes {
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

impl TryFrom<&[u8]> for ICalendarRelationshipTypes {
    type Error = ();
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        hashify::tiny_map_ignore_case!(value,
            "CHILD" => ICalendarRelationshipTypes::Child,
            "PARENT" => ICalendarRelationshipTypes::Parent,
            "SIBLING" => ICalendarRelationshipTypes::Sibling,
            "SNOOZE" => ICalendarRelationshipTypes::Snooze,
            "CONCEPT" => ICalendarRelationshipTypes::Concept,
            "DEPENDS-ON" => ICalendarRelationshipTypes::DependsOn,
            "FINISHTOFINISH" => ICalendarRelationshipTypes::Finishtofinish,
            "FINISHTOSTART" => ICalendarRelationshipTypes::Finishtostart,
            "FIRST" => ICalendarRelationshipTypes::First,
            "NEXT" => ICalendarRelationshipTypes::Next,
            "REFID" => ICalendarRelationshipTypes::Refid,
            "STARTTOFINISH" => ICalendarRelationshipTypes::Starttofinish,
            "STARTTOSTART" => ICalendarRelationshipTypes::Starttostart,
        )
        .ok_or(())
    }
}
pub enum ICalendarResourceTypes {
    Projector,             // [RFC9073, Section 6.3]
    Room,                  // [RFC9073, Section 6.3]
    RemoteConferenceAudio, // [RFC9073, Section 6.3]
    RemoteConferenceVideo, // [RFC9073, Section 6.3]
}

impl TryFrom<&[u8]> for ICalendarResourceTypes {
    type Error = ();
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        hashify::tiny_map_ignore_case!(value,
            "PROJECTOR" => ICalendarResourceTypes::Projector,
            "ROOM" => ICalendarResourceTypes::Room,
            "REMOTE-CONFERENCE-AUDIO" => ICalendarResourceTypes::RemoteConferenceAudio,
            "REMOTE-CONFERENCE-VIDEO" => ICalendarResourceTypes::RemoteConferenceVideo,
        )
        .ok_or(())
    }
}
pub enum ICalendarScheduleAgentValues {
    Server, // [RFC6638, Section 7.1]
    Client, // [RFC6638, Section 7.1]
    None,   // [RFC6638, Section 7.1]
    Other(String),
}

impl TryFrom<&[u8]> for ICalendarScheduleAgentValues {
    type Error = ();
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        hashify::tiny_map_ignore_case!(value,
            "SERVER" => ICalendarScheduleAgentValues::Server,
            "CLIENT" => ICalendarScheduleAgentValues::Client,
            "NONE" => ICalendarScheduleAgentValues::None,
        )
        .ok_or(())
    }
}
pub enum ICalendarScheduleForceSendValues {
    Request, // [RFC6638, Section 7.2]
    Reply,   // [RFC6638, Section 7.2]
    Other(String),
}

impl TryFrom<&[u8]> for ICalendarScheduleForceSendValues {
    type Error = ();
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        hashify::tiny_map_ignore_case!(value,
            "REQUEST" => ICalendarScheduleForceSendValues::Request,
            "REPLY" => ICalendarScheduleForceSendValues::Reply,
        )
        .ok_or(())
    }
}
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

impl TryFrom<&[u8]> for ICalendarValueType {
    type Error = ();
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
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
        .ok_or(())
    }
}
