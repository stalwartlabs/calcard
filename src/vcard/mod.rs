use crate::tokenizer::Token;

pub mod parser;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct VCard {
    pub kind: Option<VCardKind>,
    pub entries: Vec<VCardEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VCardEntry {
    group: Option<String>,
    name: VCardProperty,
    params: Vec<VCardParameter>,
    values: Vec<VCardValue>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum VCardProperty {
    Source,        // [RFC6350, Section 6.1.3]
    Kind,          // [RFC6350, Section 6.1.4]
    Xml,           // [RFC6350, Section 6.1.5]
    Fn,            // [RFC6350, Section 6.2.1]
    N,             // [RFC6350, Section 6.2.2][RFC9554, Section 2.2]
    Nickname,      // [RFC6350, Section 6.2.3]
    Photo,         // [RFC6350, Section 6.2.4]
    Bday,          // [RFC6350, Section 6.2.5]
    Anniversary,   // [RFC6350, Section 6.2.6]
    Gender,        // [RFC6350, Section 6.2.7]
    Adr,           // [RFC6350, Section 6.3.1][RFC9554, Section 2.1]
    Tel,           // [RFC6350, Section 6.4.1]
    Email,         // [RFC6350, Section 6.4.2]
    Impp,          // [RFC6350, Section 6.4.3]
    Lang,          // [RFC6350, Section 6.4.4]
    Tz,            // [RFC6350, Section 6.5.1]
    Geo,           // [RFC6350, Section 6.5.2]
    Title,         // [RFC6350, Section 6.6.1]
    Role,          // [RFC6350, Section 6.6.2]
    Logo,          // [RFC6350, Section 6.6.3]
    Org,           // [RFC6350, Section 6.6.4]
    Member,        // [RFC6350, Section 6.6.5]
    Related,       // [RFC6350, Section 6.6.6]
    Categories,    // [RFC6350, Section 6.7.1]
    Note,          // [RFC6350, Section 6.7.2]
    Prodid,        // [RFC6350, Section 6.7.3]
    Rev,           // [RFC6350, Section 6.7.4]
    Sound,         // [RFC6350, Section 6.7.5]
    Uid,           // [RFC6350, Section 6.7.6]
    Clientpidmap,  // [RFC6350, Section 6.7.7]
    Url,           // [RFC6350, Section 6.7.8]
    Version,       // [RFC6350, Section 6.7.9]
    Key,           // [RFC6350, Section 6.8.1]
    Fburl,         // [RFC6350, Section 6.9.1]
    Caladruri,     // [RFC6350, Section 6.9.2]
    Caluri,        // [RFC6350, Section 6.9.3]
    Birthplace,    // [RFC6474, Section 2.1]
    Deathplace,    // [RFC6474, Section 2.2]
    Deathdate,     // [RFC6474, Section 2.3]
    Expertise,     // [RFC6715, Section 2.1]
    Hobby,         // [RFC6715, Section 2.2]
    Interest,      // [RFC6715, Section 2.3]
    OrgDirectory,  // [RFC6715, Section 2.4][RFC Errata 3341]
    ContactUri,    // [RFC8605, Section 2.1]
    Created,       // [RFC9554, Section 3.1]
    Gramgender,    // [RFC9554, Section 3.2]
    Language,      // [RFC9554, Section 3.3]
    Pronouns,      // [RFC9554, Section 3.4]
    Socialprofile, // [RFC9554, Section 3.5]
    Jsprop,        // [RFC9555, Section 3.2.1]
    Other(String),
}

impl TryFrom<&[u8]> for VCardProperty {
    type Error = ();
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        hashify::tiny_map_ignore_case!(value,
            "SOURCE" => VCardProperty::Source,
            "KIND" => VCardProperty::Kind,
            "XML" => VCardProperty::Xml,
            "FN" => VCardProperty::Fn,
            "N" => VCardProperty::N,
            "NICKNAME" => VCardProperty::Nickname,
            "PHOTO" => VCardProperty::Photo,
            "BDAY" => VCardProperty::Bday,
            "ANNIVERSARY" => VCardProperty::Anniversary,
            "GENDER" => VCardProperty::Gender,
            "ADR" => VCardProperty::Adr,
            "TEL" => VCardProperty::Tel,
            "EMAIL" => VCardProperty::Email,
            "IMPP" => VCardProperty::Impp,
            "LANG" => VCardProperty::Lang,
            "TZ" => VCardProperty::Tz,
            "GEO" => VCardProperty::Geo,
            "TITLE" => VCardProperty::Title,
            "ROLE" => VCardProperty::Role,
            "LOGO" => VCardProperty::Logo,
            "ORG" => VCardProperty::Org,
            "MEMBER" => VCardProperty::Member,
            "RELATED" => VCardProperty::Related,
            "CATEGORIES" => VCardProperty::Categories,
            "NOTE" => VCardProperty::Note,
            "PRODID" => VCardProperty::Prodid,
            "REV" => VCardProperty::Rev,
            "SOUND" => VCardProperty::Sound,
            "UID" => VCardProperty::Uid,
            "CLIENTPIDMAP" => VCardProperty::Clientpidmap,
            "URL" => VCardProperty::Url,
            "VERSION" => VCardProperty::Version,
            "KEY" => VCardProperty::Key,
            "FBURL" => VCardProperty::Fburl,
            "CALADRURI" => VCardProperty::Caladruri,
            "CALURI" => VCardProperty::Caluri,
            "BIRTHPLACE" => VCardProperty::Birthplace,
            "DEATHPLACE" => VCardProperty::Deathplace,
            "DEATHDATE" => VCardProperty::Deathdate,
            "EXPERTISE" => VCardProperty::Expertise,
            "HOBBY" => VCardProperty::Hobby,
            "INTEREST" => VCardProperty::Interest,
            "ORG-DIRECTORY" => VCardProperty::OrgDirectory,
            "CONTACT-URI" => VCardProperty::ContactUri,
            "CREATED" => VCardProperty::Created,
            "GRAMGENDER" => VCardProperty::Gramgender,
            "LANGUAGE" => VCardProperty::Language,
            "PRONOUNS" => VCardProperty::Pronouns,
            "SOCIALPROFILE" => VCardProperty::Socialprofile,
            "JSPROP" => VCardProperty::Jsprop,
        )
        .ok_or(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum VCardValue {
    Text(String),
    Uri(UriType),
    Date(VCardPartialDateTime),
    Time(VCardPartialDateTime),
    DateTime(VCardPartialDateTime),
    DateAndOrTime(VCardPartialDateTime),
    Timestamp(i64),
    Boolean(bool),
    Float(f64),
    UtcOffset(i16),
    Integer(i64),
    LanguageTag(String),
    Other(OtherValue),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OtherValue {
    pub typ_: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UriType {
    Text(String),
    Data(Vec<u8>),
}

impl Eq for VCardValue {}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct VCardPartialDateTime {
    year: Option<u16>,
    month: Option<u16>,
    day: Option<u16>,
    hour: Option<u16>,
    minute: Option<u16>,
    second: Option<u16>,
    timezone: Option<i16>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VCardParameter {
    Language(String),             // [RFC6350, Section 5.1]
    Value(Vec<VCardValueType>),   // [RFC6350, Section 5.2]
    Pref(u32),                    // [RFC6350, Section 5.3]
    Altid(String),                // [RFC6350, Section 5.4]
    Pid(Vec<String>),             // [RFC6350, Section 5.5]
    Type(Vec<VCardType>),         // [RFC6350, Section 5.6]
    Mediatype(String),            // [RFC6350, Section 5.7]
    Calscale(VCardCalendarScale), // [RFC6350, Section 5.8]
    SortAs(String),               // [RFC6350, Section 5.9]
    Geo(String),                  // [RFC6350, Section 5.10]
    Tz(String),                   // [RFC6350, Section 5.11]
    Index(u32),                   // [RFC6715, Section 3.1]
    Level(VCardLevel),            // [RFC6715, Section 3.2]
    Group(String),                // [RFC7095, Section 8.1]
    Cc(String),                   // [RFC8605, Section 3.1]
    Author(String),               // [RFC9554, Section 4.1]
    AuthorName(String),           // [RFC9554, Section 4.2]
    Created(i64),                 // [RFC9554, Section 4.3]
    Derived(bool),                // [RFC9554, Section 4.4]
    Label(String),                // [RFC6350, Section 6.3.1][RFC9554, Section 4.5]
    Phonetic(VCardPhonetic),      // [RFC9554, Section 4.6]
    PropId(String),               // [RFC9554, Section 4.7]
    Script(String),               // [RFC9554, Section 4.8]
    ServiceType(String),          // [RFC9554, Section 4.9]
    Username(String),             // [RFC9554, Section 4.10]
    Jsptr(String),                // [RFC9555, Section 3.3.2]
    Other(Vec<String>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VCardValueType {
    Boolean,       // [RFC6350, Section 4.4]
    Date,          // [RFC6350, Section 4.3.1]
    DateAndOrTime, // [RFC6350, Section 4.3.4]
    DateTime,      // [RFC6350, Section 4.3.3]
    Float,         // [RFC6350, Section 4.6]
    Integer,       // [RFC6350, Section 4.5]
    LanguageTag,   // [RFC6350, Section 4.8]
    Text,          // [RFC6350, Section 4.1]
    Time,          // [RFC6350, Section 4.3.2]
    Timestamp,     // [RFC6350, Section 4.3.5]
    Uri,           // [RFC6350, Section 4.2]
    UtcOffset,     // [RFC6350, Section 4.7]
    Other(String),
}

impl From<Token<'_>> for VCardValueType {
    fn from(token: Token<'_>) -> Self {
        hashify::tiny_map_ignore_case!(token.text.as_ref(),
            "BOOLEAN" => VCardValueType::Boolean,
            "DATE" => VCardValueType::Date,
            "DATE-AND-OR-TIME" => VCardValueType::DateAndOrTime,
            "DATE-TIME" => VCardValueType::DateTime,
            "FLOAT" => VCardValueType::Float,
            "INTEGER" => VCardValueType::Integer,
            "LANGUAGE-TAG" => VCardValueType::LanguageTag,
            "TEXT" => VCardValueType::Text,
            "TIME" => VCardValueType::Time,
            "TIMESTAMP" => VCardValueType::Timestamp,
            "URI" => VCardValueType::Uri,
            "UTC-OFFSET" => VCardValueType::UtcOffset,
        )
        .unwrap_or_else(|| VCardValueType::Other(token.into_string()))
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum VCardCalendarScale {
    #[default]
    Gregorian,
    Chinese,
    IslamicCivil,
    Hebrew,
    Ethiopic,
    Other(String),
}

impl From<Token<'_>> for VCardCalendarScale {
    fn from(token: Token<'_>) -> Self {
        hashify::tiny_map_ignore_case!(token.text.as_ref(),
            "gregorian" => VCardCalendarScale::Gregorian,
            "chinese" => VCardCalendarScale::Chinese,
            "islamic-civil" => VCardCalendarScale::IslamicCivil,
            "hebrew" => VCardCalendarScale::Hebrew,
            "ethiopic" => VCardCalendarScale::Ethiopic,
        )
        .unwrap_or_else(|| VCardCalendarScale::Other(token.into_string()))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VCardLevel {
    Beginner, // [RFC6715, Section 3.2]
    Average,  // [RFC6715, Section 3.2]
    Expert,   // [RFC6715, Section 3.2]
    High,     // [RFC6715, Section 3.2]
    Medium,   // [RFC6715, Section 3.2]
    Low,      // [RFC6715, Section 3.2]
}

impl TryFrom<&[u8]> for VCardLevel {
    type Error = ();
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        hashify::tiny_map_ignore_case!(value,
            "beginner" => VCardLevel::Beginner,
            "average" => VCardLevel::Average,
            "expert" => VCardLevel::Expert,
            "high" => VCardLevel::High,
            "medium" => VCardLevel::Medium,
            "low" => VCardLevel::Low,
        )
        .ok_or(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VCardPhonetic {
    Ipa,    // [RFC9554, Section 4.6]
    Jyut,   // [RFC9554, Section 4.6]
    Piny,   // [RFC9554, Section 4.6]
    Script, // [RFC9554, Section 4.6]
    Other(String),
}

impl From<Token<'_>> for VCardPhonetic {
    fn from(token: Token<'_>) -> Self {
        hashify::tiny_map_ignore_case!(token.text.as_ref(),
            "ipa" => VCardPhonetic::Ipa,
            "jyut" => VCardPhonetic::Jyut,
            "piny" => VCardPhonetic::Piny,
            "script" => VCardPhonetic::Script,
        )
        .unwrap_or_else(|| VCardPhonetic::Other(token.into_string()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VCardType {
    Work,         // [RFC6350, Section 5.6]
    Home,         // [RFC6350, Section 5.6]
    Billing,      // [RFC9554, Section 5.1]
    Delivery,     // [RFC9554, Section 5.2]
    Contact,      // [RFC6350, Section 6.6.6]
    Acquaintance, // [RFC6350, Section 6.6.6]
    Friend,       // [RFC6350, Section 6.6.6]
    Met,          // [RFC6350, Section 6.6.6]
    CoWorker,     // [RFC6350, Section 6.6.6]
    Colleague,    // [RFC6350, Section 6.6.6]
    CoResident,   // [RFC6350, Section 6.6.6]
    Neighbor,     // [RFC6350, Section 6.6.6]
    Child,        // [RFC6350, Section 6.6.6]
    Parent,       // [RFC6350, Section 6.6.6]
    Sibling,      // [RFC6350, Section 6.6.6]
    Spouse,       // [RFC6350, Section 6.6.6]
    Kin,          // [RFC6350, Section 6.6.6]
    Muse,         // [RFC6350, Section 6.6.6]
    Crush,        // [RFC6350, Section 6.6.6]
    Date,         // [RFC6350, Section 6.6.6]
    Sweetheart,   // [RFC6350, Section 6.6.6]
    Me,           // [RFC6350, Section 6.6.6]
    Agent,        // [RFC6350, Section 6.6.6]
    Emergency,    // [RFC6350, Section 6.6.6]
    Text,         // [RFC6350, Section 6.4.1]
    Voice,        // [RFC6350, Section 6.4.1]
    Fax,          // [RFC6350, Section 6.4.1]
    Cell,         // [RFC6350, Section 6.4.1]
    Video,        // [RFC6350, Section 6.4.1]
    Pager,        // [RFC6350, Section 6.4.1]
    Textphone,    // [RFC6350, Section 6.4.1]
    MainNumber,   // [RFC7852]
    Other(String),
}

impl TryFrom<&[u8]> for VCardType {
    type Error = ();
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        hashify::tiny_map_ignore_case!(value,
            "work" => VCardType::Work,
            "home" => VCardType::Home,
            "billing" => VCardType::Billing,
            "delivery" => VCardType::Delivery,
            "contact" => VCardType::Contact,
            "acquaintance" => VCardType::Acquaintance,
            "friend" => VCardType::Friend,
            "met" => VCardType::Met,
            "co-worker" => VCardType::CoWorker,
            "colleague" => VCardType::Colleague,
            "co-resident" => VCardType::CoResident,
            "neighbor" => VCardType::Neighbor,
            "child" => VCardType::Child,
            "parent" => VCardType::Parent,
            "sibling" => VCardType::Sibling,
            "spouse" => VCardType::Spouse,
            "kin" => VCardType::Kin,
            "muse" => VCardType::Muse,
            "crush" => VCardType::Crush,
            "date" => VCardType::Date,
            "sweetheart" => VCardType::Sweetheart,
            "me" => VCardType::Me,
            "agent" => VCardType::Agent,
            "emergency" => VCardType::Emergency,
            "text" => VCardType::Text,
            "voice" => VCardType::Voice,
            "fax" => VCardType::Fax,
            "cell" => VCardType::Cell,
            "video" => VCardType::Video,
            "pager" => VCardType::Pager,
            "textphone" => VCardType::Textphone,
            "main-number" => VCardType::MainNumber,
        )
        .ok_or(())
    }
}

impl From<Token<'_>> for VCardType {
    fn from(token: Token<'_>) -> Self {
        VCardType::try_from(token.text.as_ref())
            .unwrap_or_else(|_| VCardType::Other(token.into_string()))
    }
}

pub enum VCardGramGender {
    Animate,   // [RFC9554, Section 3.2]
    Common,    // [RFC9554, Section 3.2]
    Feminine,  // [RFC9554, Section 3.2]
    Inanimate, // [RFC9554, Section 3.2]
    Masculine, // [RFC9554, Section 3.2]
    Neuter,    // [RFC9554, Section 3.2]
}

impl TryFrom<&[u8]> for VCardGramGender {
    type Error = ();
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        hashify::tiny_map_ignore_case!(value,
            "animate" => VCardGramGender::Animate,
            "common" => VCardGramGender::Common,
            "feminine" => VCardGramGender::Feminine,
            "inanimate" => VCardGramGender::Inanimate,
            "masculine" => VCardGramGender::Masculine,
            "neuter" => VCardGramGender::Neuter,
        )
        .ok_or(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VCardKind {
    Individual,  // [RFC6350, Section 6.1.4]
    Group,       // [RFC6350, Section 6.1.4]
    Org,         // [RFC6350, Section 6.1.4]
    Location,    // [RFC6350, Section 6.1.4]
    Application, // [RFC6473, Section 3]
    Device,      // [RFC6869, Section 3]
}

impl TryFrom<&[u8]> for VCardKind {
    type Error = ();
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        hashify::tiny_map_ignore_case!(value,
            "individual" => VCardKind::Individual,
            "group" => VCardKind::Group,
            "org" => VCardKind::Org,
            "location" => VCardKind::Location,
            "application" => VCardKind::Application,
            "device" => VCardKind::Device,
        )
        .ok_or(())
    }
}

pub(crate) enum ValueSeparator {
    None,
    Comma,
    Semicolon,
}

impl VCardProperty {
    // Returns the default value type and whether the property is multi-valued.
    pub(crate) fn default_types(&self) -> (VCardValueType, ValueSeparator) {
        match self {
            VCardProperty::Source => (VCardValueType::Uri, ValueSeparator::None),
            VCardProperty::Kind => (VCardValueType::Text, ValueSeparator::None),
            VCardProperty::Xml => (VCardValueType::Text, ValueSeparator::None),
            VCardProperty::Fn => (VCardValueType::Text, ValueSeparator::None),
            VCardProperty::N => (VCardValueType::Text, ValueSeparator::Semicolon),
            VCardProperty::Nickname => (VCardValueType::Text, ValueSeparator::Comma),
            VCardProperty::Photo => (VCardValueType::Uri, ValueSeparator::None),
            VCardProperty::Bday => (VCardValueType::DateAndOrTime, ValueSeparator::None),
            VCardProperty::Anniversary => (VCardValueType::DateAndOrTime, ValueSeparator::None),
            VCardProperty::Gender => (VCardValueType::Text, ValueSeparator::Semicolon),
            VCardProperty::Adr => (VCardValueType::Text, ValueSeparator::Semicolon),
            VCardProperty::Tel => (VCardValueType::Text, ValueSeparator::None),
            VCardProperty::Email => (VCardValueType::Text, ValueSeparator::None),
            VCardProperty::Impp => (VCardValueType::Uri, ValueSeparator::None),
            VCardProperty::Lang => (VCardValueType::Text, ValueSeparator::None),
            VCardProperty::Tz => (VCardValueType::UtcOffset, ValueSeparator::None),
            VCardProperty::Geo => (VCardValueType::Uri, ValueSeparator::None),
            VCardProperty::Title => (VCardValueType::Text, ValueSeparator::None),
            VCardProperty::Role => (VCardValueType::Text, ValueSeparator::None),
            VCardProperty::Logo => (VCardValueType::Uri, ValueSeparator::None),
            VCardProperty::Org => (VCardValueType::Text, ValueSeparator::Semicolon),
            VCardProperty::Member => (VCardValueType::Uri, ValueSeparator::None),
            VCardProperty::Related => (VCardValueType::Uri, ValueSeparator::None),
            VCardProperty::Categories => (VCardValueType::Uri, ValueSeparator::Comma),
            VCardProperty::Note => (VCardValueType::Text, ValueSeparator::None),
            VCardProperty::Prodid => (VCardValueType::Text, ValueSeparator::None),
            VCardProperty::Rev => (VCardValueType::Timestamp, ValueSeparator::None),
            VCardProperty::Sound => (VCardValueType::Uri, ValueSeparator::None),
            VCardProperty::Uid => (VCardValueType::Uri, ValueSeparator::None),
            VCardProperty::Clientpidmap => (VCardValueType::Text, ValueSeparator::Semicolon),
            VCardProperty::Url => (VCardValueType::Uri, ValueSeparator::None),
            VCardProperty::Version => (VCardValueType::Text, ValueSeparator::None),
            VCardProperty::Key => (VCardValueType::Uri, ValueSeparator::None),
            VCardProperty::Fburl => (VCardValueType::Uri, ValueSeparator::None),
            VCardProperty::Caladruri => (VCardValueType::Uri, ValueSeparator::None),
            VCardProperty::Caluri => (VCardValueType::Uri, ValueSeparator::None),
            VCardProperty::Birthplace => (VCardValueType::Text, ValueSeparator::None),
            VCardProperty::Deathplace => (VCardValueType::Text, ValueSeparator::None),
            VCardProperty::Deathdate => (VCardValueType::DateAndOrTime, ValueSeparator::None),
            VCardProperty::Expertise => (VCardValueType::Text, ValueSeparator::None),
            VCardProperty::Hobby => (VCardValueType::Text, ValueSeparator::None),
            VCardProperty::Interest => (VCardValueType::Text, ValueSeparator::None),
            VCardProperty::OrgDirectory => (VCardValueType::Uri, ValueSeparator::None),
            VCardProperty::ContactUri => (VCardValueType::Uri, ValueSeparator::None),
            VCardProperty::Created => (VCardValueType::Timestamp, ValueSeparator::None),
            VCardProperty::Gramgender => (VCardValueType::Text, ValueSeparator::None),
            VCardProperty::Language => (VCardValueType::LanguageTag, ValueSeparator::None),
            VCardProperty::Pronouns => (VCardValueType::Text, ValueSeparator::None),
            VCardProperty::Socialprofile => (VCardValueType::Uri, ValueSeparator::None),
            VCardProperty::Jsprop => (VCardValueType::Text, ValueSeparator::None),
            VCardProperty::Other(_) => (VCardValueType::Text, ValueSeparator::Semicolon),
        }
    }
}
