use crate::tokenizer::Token;

pub mod parser;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VCardParameter {
    Language,    // [RFC6350, Section 5.1]
    Value,       // [RFC6350, Section 5.2]
    Pref,        // [RFC6350, Section 5.3]
    Altid,       // [RFC6350, Section 5.4]
    Pid,         // [RFC6350, Section 5.5]
    Type,        // [RFC6350, Section 5.6]
    Mediatype,   // [RFC6350, Section 5.7]
    Calscale,    // [RFC6350, Section 5.8]
    SortAs,      // [RFC6350, Section 5.9]
    Geo,         // [RFC6350, Section 5.10]
    Tz,          // [RFC6350, Section 5.11]
    Index,       // [RFC6715, Section 3.1]
    Level,       // [RFC6715, Section 3.2]
    Group,       // [RFC7095, Section 8.1]
    Cc,          // [RFC8605, Section 3.1]
    Author,      // [RFC9554, Section 4.1]
    AuthorName,  // [RFC9554, Section 4.2]
    Created,     // [RFC9554, Section 4.3]
    Derived,     // [RFC9554, Section 4.4]
    Label,       // [RFC6350, Section 6.3.1][RFC9554, Section 4.5]
    Phonetic,    // [RFC9554, Section 4.6]
    PropId,      // [RFC9554, Section 4.7]
    Script,      // [RFC9554, Section 4.8]
    ServiceType, // [RFC9554, Section 4.9]
    Username,    // [RFC9554, Section 4.10]
    Jsptr,       // [RFC9555, Section 3.3.2]
    Other(String),
}

impl TryFrom<&[u8]> for VCardParameter {
    type Error = ();
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        hashify::tiny_map_ignore_case!(value,
            "LANGUAGE" => VCardParameter::Language,
            "VALUE" => VCardParameter::Value,
            "PREF" => VCardParameter::Pref,
            "ALTID" => VCardParameter::Altid,
            "PID" => VCardParameter::Pid,
            "TYPE" => VCardParameter::Type,
            "MEDIATYPE" => VCardParameter::Mediatype,
            "CALSCALE" => VCardParameter::Calscale,
            "SORT-AS" => VCardParameter::SortAs,
            "GEO" => VCardParameter::Geo,
            "TZ" => VCardParameter::Tz,
            "INDEX" => VCardParameter::Index,
            "LEVEL" => VCardParameter::Level,
            "GROUP" => VCardParameter::Group,
            "CC" => VCardParameter::Cc,
            "AUTHOR" => VCardParameter::Author,
            "AUTHOR-NAME" => VCardParameter::AuthorName,
            "CREATED" => VCardParameter::Created,
            "DERIVED" => VCardParameter::Derived,
            "LABEL" => VCardParameter::Label,
            "PHONETIC" => VCardParameter::Phonetic,
            "PROP-ID" => VCardParameter::PropId,
            "SCRIPT" => VCardParameter::Script,
            "SERVICE-TYPE" => VCardParameter::ServiceType,
            "USERNAME" => VCardParameter::Username,
            "JSPTR" => VCardParameter::Jsptr,
        )
        .ok_or(())
    }
}

pub enum VCardValueDataTypes {
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
    Unknown,       // [RFC7095, Section 8.2]
    Uri,           // [RFC6350, Section 4.2]
    UtcOffset,     // [RFC6350, Section 4.7]
    Other(String),
}

impl From<Token<'_>> for VCardValueDataTypes {
    fn from(token: Token<'_>) -> Self {
        hashify::tiny_map_ignore_case!(token.text.as_ref(),
            "BOOLEAN" => VCardValueDataTypes::Boolean,
            "DATE" => VCardValueDataTypes::Date,
            "DATE-AND-OR-TIME" => VCardValueDataTypes::DateAndOrTime,
            "DATE-TIME" => VCardValueDataTypes::DateTime,
            "FLOAT" => VCardValueDataTypes::Float,
            "INTEGER" => VCardValueDataTypes::Integer,
            "LANGUAGE-TAG" => VCardValueDataTypes::LanguageTag,
            "TEXT" => VCardValueDataTypes::Text,
            "TIME" => VCardValueDataTypes::Time,
            "TIMESTAMP" => VCardValueDataTypes::Timestamp,
            "UNKNOWN" => VCardValueDataTypes::Unknown,
            "URI" => VCardValueDataTypes::Uri,
            "UTC-OFFSET" => VCardValueDataTypes::UtcOffset,
        )
        .unwrap_or_else(|| VCardValueDataTypes::Other(token.into_string()))
    }
}

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

pub enum VCardNPhonetic {
    Ipa,    // [RFC9554, Section 4.6]
    Jyut,   // [RFC9554, Section 4.6]
    Piny,   // [RFC9554, Section 4.6]
    Script, // [RFC9554, Section 4.6]
}

impl TryFrom<&[u8]> for VCardNPhonetic {
    type Error = ();
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        hashify::tiny_map_ignore_case!(value,
            "ipa" => VCardNPhonetic::Ipa,
            "jyut" => VCardNPhonetic::Jyut,
            "piny" => VCardNPhonetic::Piny,
            "script" => VCardNPhonetic::Script,
        )
        .ok_or(())
    }
}

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
