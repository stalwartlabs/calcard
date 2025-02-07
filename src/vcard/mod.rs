use crate::tokenizer::Token;

pub mod parser;
pub mod writer;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct VCard {
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
    Begin,
    End,
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
            "BEGIN" => VCardProperty::Begin,
            "END" => VCardProperty::End,
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

impl VCardProperty {
    pub fn as_str(&self) -> &str {
        match self {
            VCardProperty::Source => "SOURCE",
            VCardProperty::Kind => "KIND",
            VCardProperty::Xml => "XML",
            VCardProperty::Fn => "FN",
            VCardProperty::N => "N",
            VCardProperty::Nickname => "NICKNAME",
            VCardProperty::Photo => "PHOTO",
            VCardProperty::Bday => "BDAY",
            VCardProperty::Anniversary => "ANNIVERSARY",
            VCardProperty::Gender => "GENDER",
            VCardProperty::Adr => "ADR",
            VCardProperty::Tel => "TEL",
            VCardProperty::Email => "EMAIL",
            VCardProperty::Impp => "IMPP",
            VCardProperty::Lang => "LANG",
            VCardProperty::Tz => "TZ",
            VCardProperty::Geo => "GEO",
            VCardProperty::Title => "TITLE",
            VCardProperty::Role => "ROLE",
            VCardProperty::Logo => "LOGO",
            VCardProperty::Org => "ORG",
            VCardProperty::Member => "MEMBER",
            VCardProperty::Related => "RELATED",
            VCardProperty::Categories => "CATEGORIES",
            VCardProperty::Note => "NOTE",
            VCardProperty::Prodid => "PRODID",
            VCardProperty::Rev => "REV",
            VCardProperty::Sound => "SOUND",
            VCardProperty::Uid => "UID",
            VCardProperty::Clientpidmap => "CLIENTPIDMAP",
            VCardProperty::Url => "URL",
            VCardProperty::Version => "VERSION",
            VCardProperty::Key => "KEY",
            VCardProperty::Fburl => "FBURL",
            VCardProperty::Caladruri => "CALADRURI",
            VCardProperty::Caluri => "CALURI",
            VCardProperty::Birthplace => "BIRTHPLACE",
            VCardProperty::Deathplace => "DEATHPLACE",
            VCardProperty::Deathdate => "DEATHDATE",
            VCardProperty::Expertise => "EXPERTISE",
            VCardProperty::Hobby => "HOBBY",
            VCardProperty::Interest => "INTEREST",
            VCardProperty::OrgDirectory => "ORG-DIRECTORY",
            VCardProperty::ContactUri => "CONTACT-URI",
            VCardProperty::Created => "CREATED",
            VCardProperty::Gramgender => "GRAMGENDER",
            VCardProperty::Language => "LANGUAGE",
            VCardProperty::Pronouns => "PRONOUNS",
            VCardProperty::Socialprofile => "SOCIALPROFILE",
            VCardProperty::Jsprop => "JSPROP",
            VCardProperty::Begin => "BEGIN",
            VCardProperty::End => "END",
            VCardProperty::Other(ref s) => s,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum VCardValue {
    Text(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    PartialDateTime(VCardPartialDateTime),
    Binary(VCardBinary),
    Sex(VCardSex),
    GramGender(VCardGramGender),
    Kind(VCardKind),
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct VCardBinary {
    pub content_type: Option<String>,
    pub data: Vec<u8>,
}

impl Eq for VCardValue {}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct VCardPartialDateTime {
    pub year: Option<u16>,
    pub month: Option<u16>,
    pub day: Option<u16>,
    pub hour: Option<u16>,
    pub minute: Option<u16>,
    pub second: Option<u16>,
    pub tz_hour: Option<u16>,
    pub tz_minute: Option<u16>,
    pub tz_minus: bool,
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

impl VCardValueType {
    pub fn as_str(&self) -> &str {
        match self {
            VCardValueType::Boolean => "BOOLEAN",
            VCardValueType::Date => "DATE",
            VCardValueType::DateAndOrTime => "DATE-AND-OR-TIME",
            VCardValueType::DateTime => "DATE-TIME",
            VCardValueType::Float => "FLOAT",
            VCardValueType::Integer => "INTEGER",
            VCardValueType::LanguageTag => "LANGUAGE-TAG",
            VCardValueType::Text => "TEXT",
            VCardValueType::Time => "TIME",
            VCardValueType::Timestamp => "TIMESTAMP",
            VCardValueType::Uri => "URI",
            VCardValueType::UtcOffset => "UTC-OFFSET",
            VCardValueType::Other(ref s) => s,
        }
    }
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

impl VCardCalendarScale {
    pub fn as_str(&self) -> &str {
        match self {
            VCardCalendarScale::Gregorian => "GREGORIAN",
            VCardCalendarScale::Chinese => "CHINESE",
            VCardCalendarScale::IslamicCivil => "ISLAMIC-CIVIL",
            VCardCalendarScale::Hebrew => "HEBREW",
            VCardCalendarScale::Ethiopic => "ETHIOPIC",
            VCardCalendarScale::Other(ref s) => s,
        }
    }
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

impl VCardLevel {
    pub fn as_str(&self) -> &str {
        match self {
            VCardLevel::Beginner => "BEGINNER",
            VCardLevel::Average => "AVERAGE",
            VCardLevel::Expert => "EXPERT",
            VCardLevel::High => "HIGH",
            VCardLevel::Medium => "MEDIUM",
            VCardLevel::Low => "LOW",
        }
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

impl VCardPhonetic {
    pub fn as_str(&self) -> &str {
        match self {
            VCardPhonetic::Ipa => "IPA",
            VCardPhonetic::Jyut => "JYUT",
            VCardPhonetic::Piny => "PINY",
            VCardPhonetic::Script => "SCRIPT",
            VCardPhonetic::Other(ref s) => s,
        }
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

impl VCardType {
    pub fn as_str(&self) -> &str {
        match self {
            VCardType::Work => "WORK",
            VCardType::Home => "HOME",
            VCardType::Billing => "BILLING",
            VCardType::Delivery => "DELIVERY",
            VCardType::Contact => "CONTACT",
            VCardType::Acquaintance => "ACQUAINTANCE",
            VCardType::Friend => "FRIEND",
            VCardType::Met => "MET",
            VCardType::CoWorker => "CO-WORKER",
            VCardType::Colleague => "COLLEAGUE",
            VCardType::CoResident => "CO-RESIDENT",
            VCardType::Neighbor => "NEIGHBOR",
            VCardType::Child => "CHILD",
            VCardType::Parent => "PARENT",
            VCardType::Sibling => "SIBLING",
            VCardType::Spouse => "SPOUSE",
            VCardType::Kin => "KIN",
            VCardType::Muse => "MUSE",
            VCardType::Crush => "CRUSH",
            VCardType::Date => "DATE",
            VCardType::Sweetheart => "SWEETHEART",
            VCardType::Me => "ME",
            VCardType::Agent => "AGENT",
            VCardType::Emergency => "EMERGENCY",
            VCardType::Text => "TEXT",
            VCardType::Voice => "VOICE",
            VCardType::Fax => "FAX",
            VCardType::Cell => "CELL",
            VCardType::Video => "VIDEO",
            VCardType::Pager => "PAGER",
            VCardType::Textphone => "TEXTPHONE",
            VCardType::MainNumber => "MAIN-NUMBER",
            VCardType::Other(ref s) => s,
        }
    }
}

impl From<Token<'_>> for VCardType {
    fn from(token: Token<'_>) -> Self {
        VCardType::try_from(token.text.as_ref())
            .unwrap_or_else(|_| VCardType::Other(token.into_string()))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

impl VCardGramGender {
    pub fn as_str(&self) -> &str {
        match self {
            VCardGramGender::Animate => "ANIMATE",
            VCardGramGender::Common => "COMMON",
            VCardGramGender::Feminine => "FEMININE",
            VCardGramGender::Inanimate => "INANIMATE",
            VCardGramGender::Masculine => "MASCULINE",
            VCardGramGender::Neuter => "NEUTER",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VCardSex {
    Male,
    Female,
    Other,
    NoneOrNotApplicable,
    Unknown,
}

impl TryFrom<&[u8]> for VCardSex {
    type Error = ();
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        hashify::tiny_map_ignore_case!(value,
            "M" => VCardSex::Male,
            "F" => VCardSex::Female,
            "O" => VCardSex::Other,
            "N" => VCardSex::NoneOrNotApplicable,
            "U" => VCardSex::Unknown,
        )
        .ok_or(())
    }
}

impl VCardSex {
    pub fn as_str(&self) -> &str {
        match self {
            VCardSex::Male => "M",
            VCardSex::Female => "F",
            VCardSex::Other => "O",
            VCardSex::NoneOrNotApplicable => "N",
            VCardSex::Unknown => "U",
        }
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

impl VCardKind {
    pub fn as_str(&self) -> &str {
        match self {
            VCardKind::Individual => "INDIVIDUAL",
            VCardKind::Group => "GROUP",
            VCardKind::Org => "ORG",
            VCardKind::Location => "LOCATION",
            VCardKind::Application => "APPLICATION",
            VCardKind::Device => "DEVICE",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ValueSeparator {
    None,
    Comma,
    Semicolon,
    Eof,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ValueType {
    Vcard(VCardValueType),
    Kind,
    Sex,
    GramGender,
}

impl VCardProperty {
    // Returns the default value type and whether the property is multi-valued.
    pub(crate) fn default_types(&self) -> (ValueType, ValueSeparator) {
        match self {
            VCardProperty::Source => (ValueType::Vcard(VCardValueType::Uri), ValueSeparator::None),
            VCardProperty::Kind => (ValueType::Kind, ValueSeparator::None),
            VCardProperty::Xml => (ValueType::Vcard(VCardValueType::Text), ValueSeparator::None),
            VCardProperty::Fn => (ValueType::Vcard(VCardValueType::Text), ValueSeparator::None),
            VCardProperty::N => (
                ValueType::Vcard(VCardValueType::Text),
                ValueSeparator::Semicolon,
            ),
            VCardProperty::Nickname => (
                ValueType::Vcard(VCardValueType::Text),
                ValueSeparator::Comma,
            ),
            VCardProperty::Photo => (ValueType::Vcard(VCardValueType::Uri), ValueSeparator::None),
            VCardProperty::Bday => (
                ValueType::Vcard(VCardValueType::DateAndOrTime),
                ValueSeparator::None,
            ),
            VCardProperty::Anniversary => (
                ValueType::Vcard(VCardValueType::DateAndOrTime),
                ValueSeparator::None,
            ),
            VCardProperty::Gender => (ValueType::Sex, ValueSeparator::Semicolon),
            VCardProperty::Adr => (
                ValueType::Vcard(VCardValueType::Text),
                ValueSeparator::Semicolon,
            ),
            VCardProperty::Tel => (ValueType::Vcard(VCardValueType::Text), ValueSeparator::None),
            VCardProperty::Email => (ValueType::Vcard(VCardValueType::Text), ValueSeparator::None),
            VCardProperty::Impp => (ValueType::Vcard(VCardValueType::Uri), ValueSeparator::None),
            VCardProperty::Lang => (ValueType::Vcard(VCardValueType::Text), ValueSeparator::None),
            VCardProperty::Tz => (
                ValueType::Vcard(VCardValueType::UtcOffset),
                ValueSeparator::None,
            ),
            VCardProperty::Geo => (ValueType::Vcard(VCardValueType::Uri), ValueSeparator::None),
            VCardProperty::Title => (ValueType::Vcard(VCardValueType::Text), ValueSeparator::None),
            VCardProperty::Role => (ValueType::Vcard(VCardValueType::Text), ValueSeparator::None),
            VCardProperty::Logo => (ValueType::Vcard(VCardValueType::Uri), ValueSeparator::None),
            VCardProperty::Org => (
                ValueType::Vcard(VCardValueType::Text),
                ValueSeparator::Semicolon,
            ),
            VCardProperty::Member => (ValueType::Vcard(VCardValueType::Uri), ValueSeparator::None),
            VCardProperty::Related => (ValueType::Vcard(VCardValueType::Uri), ValueSeparator::None),
            VCardProperty::Categories => {
                (ValueType::Vcard(VCardValueType::Uri), ValueSeparator::Comma)
            }
            VCardProperty::Note => (ValueType::Vcard(VCardValueType::Text), ValueSeparator::None),
            VCardProperty::Prodid => (ValueType::Vcard(VCardValueType::Text), ValueSeparator::None),
            VCardProperty::Rev => (
                ValueType::Vcard(VCardValueType::Timestamp),
                ValueSeparator::None,
            ),
            VCardProperty::Sound => (ValueType::Vcard(VCardValueType::Uri), ValueSeparator::None),
            VCardProperty::Uid => (ValueType::Vcard(VCardValueType::Uri), ValueSeparator::None),
            VCardProperty::Clientpidmap => (
                ValueType::Vcard(VCardValueType::Text),
                ValueSeparator::Semicolon,
            ),
            VCardProperty::Url => (ValueType::Vcard(VCardValueType::Uri), ValueSeparator::None),
            VCardProperty::Version => {
                (ValueType::Vcard(VCardValueType::Text), ValueSeparator::None)
            }
            VCardProperty::Key => (ValueType::Vcard(VCardValueType::Uri), ValueSeparator::None),
            VCardProperty::Fburl => (ValueType::Vcard(VCardValueType::Uri), ValueSeparator::None),
            VCardProperty::Caladruri => {
                (ValueType::Vcard(VCardValueType::Uri), ValueSeparator::None)
            }
            VCardProperty::Caluri => (ValueType::Vcard(VCardValueType::Uri), ValueSeparator::None),
            VCardProperty::Birthplace => {
                (ValueType::Vcard(VCardValueType::Text), ValueSeparator::None)
            }
            VCardProperty::Deathplace => {
                (ValueType::Vcard(VCardValueType::Text), ValueSeparator::None)
            }
            VCardProperty::Deathdate => (
                ValueType::Vcard(VCardValueType::DateAndOrTime),
                ValueSeparator::None,
            ),
            VCardProperty::Expertise => {
                (ValueType::Vcard(VCardValueType::Text), ValueSeparator::None)
            }
            VCardProperty::Hobby => (ValueType::Vcard(VCardValueType::Text), ValueSeparator::None),
            VCardProperty::Interest => {
                (ValueType::Vcard(VCardValueType::Text), ValueSeparator::None)
            }
            VCardProperty::OrgDirectory => {
                (ValueType::Vcard(VCardValueType::Uri), ValueSeparator::None)
            }
            VCardProperty::ContactUri => {
                (ValueType::Vcard(VCardValueType::Uri), ValueSeparator::None)
            }
            VCardProperty::Created => (
                ValueType::Vcard(VCardValueType::Timestamp),
                ValueSeparator::None,
            ),
            VCardProperty::Gramgender => (ValueType::GramGender, ValueSeparator::None),
            VCardProperty::Language => (
                ValueType::Vcard(VCardValueType::LanguageTag),
                ValueSeparator::None,
            ),
            VCardProperty::Pronouns => {
                (ValueType::Vcard(VCardValueType::Text), ValueSeparator::None)
            }
            VCardProperty::Socialprofile => {
                (ValueType::Vcard(VCardValueType::Uri), ValueSeparator::None)
            }
            VCardProperty::Jsprop => (ValueType::Vcard(VCardValueType::Text), ValueSeparator::None),
            VCardProperty::Other(_) => (
                ValueType::Vcard(VCardValueType::Text),
                ValueSeparator::Semicolon,
            ),
            VCardProperty::Begin | VCardProperty::End => {
                (ValueType::Vcard(VCardValueType::Text), ValueSeparator::Eof)
            }
        }
    }
}
