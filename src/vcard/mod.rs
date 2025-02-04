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

pub enum VCardParameters {
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
}

impl TryFrom<&[u8]> for VCardParameters {
    type Error = ();
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        hashify::tiny_map_ignore_case!(value,
            "LANGUAGE" => VCardParameters::Language,
            "VALUE" => VCardParameters::Value,
            "PREF" => VCardParameters::Pref,
            "ALTID" => VCardParameters::Altid,
            "PID" => VCardParameters::Pid,
            "TYPE" => VCardParameters::Type,
            "MEDIATYPE" => VCardParameters::Mediatype,
            "CALSCALE" => VCardParameters::Calscale,
            "SORT-AS" => VCardParameters::SortAs,
            "GEO" => VCardParameters::Geo,
            "TZ" => VCardParameters::Tz,
            "INDEX" => VCardParameters::Index,
            "LEVEL" => VCardParameters::Level,
            "GROUP" => VCardParameters::Group,
            "CC" => VCardParameters::Cc,
            "AUTHOR" => VCardParameters::Author,
            "AUTHOR-NAME" => VCardParameters::AuthorName,
            "CREATED" => VCardParameters::Created,
            "DERIVED" => VCardParameters::Derived,
            "LABEL" => VCardParameters::Label,
            "PHONETIC" => VCardParameters::Phonetic,
            "PROP-ID" => VCardParameters::PropId,
            "SCRIPT" => VCardParameters::Script,
            "SERVICE-TYPE" => VCardParameters::ServiceType,
            "USERNAME" => VCardParameters::Username,
            "JSPTR" => VCardParameters::Jsptr,
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
}

impl TryFrom<&[u8]> for VCardValueDataTypes {
    type Error = ();
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        hashify::tiny_map_ignore_case!(value,
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
        .ok_or(())
    }
}

pub enum VCardAdrType {
    Billing,  // [RFC9554, Section 5.1]
    Delivery, // [RFC9554, Section 5.2]
}

impl TryFrom<&[u8]> for VCardAdrType {
    type Error = ();
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        hashify::tiny_map_ignore_case!(value,
            "billing" => VCardAdrType::Billing,
            "delivery" => VCardAdrType::Delivery,
        )
        .ok_or(())
    }
}

pub enum VCardCalscale {
    Gregorian, // [RFC6350, Section 5.8]
}

impl TryFrom<&[u8]> for VCardCalscale {
    type Error = ();
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        hashify::tiny_map_ignore_case!(value,
            "gregorian" => VCardCalscale::Gregorian,
        )
        .ok_or(())
    }
}

pub enum VCardExpertiseLevel {
    Beginner, // [RFC6715, Section 3.2]
    Average,  // [RFC6715, Section 3.2]
    Expert,   // [RFC6715, Section 3.2]
}

impl TryFrom<&[u8]> for VCardExpertiseLevel {
    type Error = ();
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        hashify::tiny_map_ignore_case!(value,
            "beginner" => VCardExpertiseLevel::Beginner,
            "average" => VCardExpertiseLevel::Average,
            "expert" => VCardExpertiseLevel::Expert,
        )
        .ok_or(())
    }
}

pub enum VCardHobbyLevel {
    High,   // [RFC6715, Section 3.2]
    Medium, // [RFC6715, Section 3.2]
    Low,    // [RFC6715, Section 3.2]
}

impl TryFrom<&[u8]> for VCardHobbyLevel {
    type Error = ();
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        hashify::tiny_map_ignore_case!(value,
            "high" => VCardHobbyLevel::High,
            "medium" => VCardHobbyLevel::Medium,
            "low" => VCardHobbyLevel::Low,
        )
        .ok_or(())
    }
}

pub enum VCardInterestLevel {
    High,   // [RFC6715, Section 3.2]
    Medium, // [RFC6715, Section 3.2]
    Low,    // [RFC6715, Section 3.2]
}

impl TryFrom<&[u8]> for VCardInterestLevel {
    type Error = ();
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        hashify::tiny_map_ignore_case!(value,
            "high" => VCardInterestLevel::High,
            "medium" => VCardInterestLevel::Medium,
            "low" => VCardInterestLevel::Low,
        )
        .ok_or(())
    }
}

pub enum VCardLocationType {
    Work, // [RFC6350, Section 5.6]
    Home, // [RFC6350, Section 5.6]
}

impl TryFrom<&[u8]> for VCardLocationType {
    type Error = ();
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        hashify::tiny_map_ignore_case!(value,
            "work" => VCardLocationType::Work,
            "home" => VCardLocationType::Home,
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

pub enum VCardRelatedType {
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
}

impl TryFrom<&[u8]> for VCardRelatedType {
    type Error = ();
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        hashify::tiny_map_ignore_case!(value,
            "contact" => VCardRelatedType::Contact,
            "acquaintance" => VCardRelatedType::Acquaintance,
            "friend" => VCardRelatedType::Friend,
            "met" => VCardRelatedType::Met,
            "co-worker" => VCardRelatedType::CoWorker,
            "colleague" => VCardRelatedType::Colleague,
            "co-resident" => VCardRelatedType::CoResident,
            "neighbor" => VCardRelatedType::Neighbor,
            "child" => VCardRelatedType::Child,
            "parent" => VCardRelatedType::Parent,
            "sibling" => VCardRelatedType::Sibling,
            "spouse" => VCardRelatedType::Spouse,
            "kin" => VCardRelatedType::Kin,
            "muse" => VCardRelatedType::Muse,
            "crush" => VCardRelatedType::Crush,
            "date" => VCardRelatedType::Date,
            "sweetheart" => VCardRelatedType::Sweetheart,
            "me" => VCardRelatedType::Me,
            "agent" => VCardRelatedType::Agent,
            "emergency" => VCardRelatedType::Emergency,
        )
        .ok_or(())
    }
}

pub enum VCardTelType {
    Text,       // [RFC6350, Section 6.4.1]
    Voice,      // [RFC6350, Section 6.4.1]
    Fax,        // [RFC6350, Section 6.4.1]
    Cell,       // [RFC6350, Section 6.4.1]
    Video,      // [RFC6350, Section 6.4.1]
    Pager,      // [RFC6350, Section 6.4.1]
    Textphone,  // [RFC6350, Section 6.4.1]
    MainNumber, // [RFC7852]
}

impl TryFrom<&[u8]> for VCardTelType {
    type Error = ();
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        hashify::tiny_map_ignore_case!(value,
            "text" => VCardTelType::Text,
            "voice" => VCardTelType::Voice,
            "fax" => VCardTelType::Fax,
            "cell" => VCardTelType::Cell,
            "video" => VCardTelType::Video,
            "pager" => VCardTelType::Pager,
            "textphone" => VCardTelType::Textphone,
            "main-number" => VCardTelType::MainNumber,
        )
        .ok_or(())
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
