/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

use std::borrow::Cow;

use super::*;

impl IanaParse for VCardProperty {
    fn parse(value: &[u8]) -> Option<Self> {
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
            VCardProperty::Other(v) => v.as_str(),
        }
    }
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
                ValueSeparator::SemicolonAndComma,
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
                ValueSeparator::SemicolonAndComma,
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
            VCardProperty::Categories => (
                ValueType::Vcard(VCardValueType::Text),
                ValueSeparator::Comma,
            ),
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
            VCardProperty::Other(_) => {
                (ValueType::Vcard(VCardValueType::Text), ValueSeparator::None)
            }
            VCardProperty::Begin | VCardProperty::End => {
                (ValueType::Vcard(VCardValueType::Text), ValueSeparator::Skip)
            }
        }
    }
}

impl Eq for VCardValue {}

impl IanaString for VCardValueType {
    fn as_str(&self) -> &'static str {
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
        }
    }
}

impl IanaParse for VCardValueType {
    fn parse(value: &[u8]) -> Option<Self> {
        hashify::tiny_map_ignore_case!(value,
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
    }
}

impl IanaParse for VCardLevel {
    fn parse(value: &[u8]) -> Option<Self> {
        hashify::tiny_map_ignore_case!(value,
            "beginner" => VCardLevel::Beginner,
            "average" => VCardLevel::Average,
            "expert" => VCardLevel::Expert,
            "high" => VCardLevel::High,
            "medium" => VCardLevel::Medium,
            "low" => VCardLevel::Low,
        )
    }
}

impl IanaString for VCardLevel {
    fn as_str(&self) -> &'static str {
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

impl IanaParse for VCardPhonetic {
    fn parse(value: &[u8]) -> Option<Self> {
        hashify::tiny_map_ignore_case!(value,
            "ipa" => VCardPhonetic::Ipa,
            "jyut" => VCardPhonetic::Jyut,
            "piny" => VCardPhonetic::Piny,
            "script" => VCardPhonetic::Script,
        )
    }
}

impl IanaString for VCardPhonetic {
    fn as_str(&self) -> &'static str {
        match self {
            VCardPhonetic::Ipa => "IPA",
            VCardPhonetic::Jyut => "JYUT",
            VCardPhonetic::Piny => "PINY",
            VCardPhonetic::Script => "SCRIPT",
        }
    }
}

impl IanaParse for VCardType {
    fn parse(value: &[u8]) -> Option<Self> {
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
    }
}

impl IanaString for VCardType {
    fn as_str(&self) -> &'static str {
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
        }
    }
}

impl IanaParse for VCardGramGender {
    fn parse(value: &[u8]) -> Option<Self> {
        hashify::tiny_map_ignore_case!(value,
            "animate" => VCardGramGender::Animate,
            "common" => VCardGramGender::Common,
            "feminine" => VCardGramGender::Feminine,
            "inanimate" => VCardGramGender::Inanimate,
            "masculine" => VCardGramGender::Masculine,
            "neuter" => VCardGramGender::Neuter,
        )
    }
}

impl IanaString for VCardGramGender {
    fn as_str(&self) -> &'static str {
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

impl IanaParse for VCardSex {
    fn parse(value: &[u8]) -> Option<Self> {
        hashify::tiny_map_ignore_case!(value,
            "M" => VCardSex::Male,
            "F" => VCardSex::Female,
            "O" => VCardSex::Other,
            "N" => VCardSex::NoneOrNotApplicable,
            "U" => VCardSex::Unknown,
        )
    }
}

impl IanaString for VCardSex {
    fn as_str(&self) -> &'static str {
        match self {
            VCardSex::Male => "M",
            VCardSex::Female => "F",
            VCardSex::Other => "O",
            VCardSex::NoneOrNotApplicable => "N",
            VCardSex::Unknown => "U",
        }
    }
}

impl IanaParse for VCardKind {
    fn parse(value: &[u8]) -> Option<Self> {
        hashify::tiny_map_ignore_case!(value,
            "individual" => VCardKind::Individual,
            "group" => VCardKind::Group,
            "org" => VCardKind::Org,
            "location" => VCardKind::Location,
            "application" => VCardKind::Application,
            "device" => VCardKind::Device,
        )
    }
}

impl IanaString for VCardKind {
    fn as_str(&self) -> &'static str {
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

impl VCardParameterName {
    pub fn try_parse(input: &[u8]) -> Option<Self> {
        hashify::tiny_map_ignore_case!(input,
            b"LANGUAGE" => VCardParameterName::Language,
            b"VALUE" => VCardParameterName::Value,
            b"PREF" => VCardParameterName::Pref,
            b"ALTID" => VCardParameterName::Altid,
            b"PID" => VCardParameterName::Pid,
            b"TYPE" => VCardParameterName::Type,
            b"MEDIATYPE" => VCardParameterName::Mediatype,
            b"CALSCALE" => VCardParameterName::Calscale,
            b"SORT-AS" => VCardParameterName::SortAs,
            b"GEO" => VCardParameterName::Geo,
            b"TZ" => VCardParameterName::Tz,
            b"INDEX" => VCardParameterName::Index,
            b"LEVEL" => VCardParameterName::Level,
            b"GROUP" => VCardParameterName::Group,
            b"CC" => VCardParameterName::Cc,
            b"AUTHOR" => VCardParameterName::Author,
            b"AUTHOR-NAME" => VCardParameterName::AuthorName,
            b"CREATED" => VCardParameterName::Created,
            b"DERIVED" => VCardParameterName::Derived,
            b"LABEL" => VCardParameterName::Label,
            b"PHONETIC" => VCardParameterName::Phonetic,
            b"PROP-ID" => VCardParameterName::PropId,
            b"SCRIPT" => VCardParameterName::Script,
            b"SERVICE-TYPE" => VCardParameterName::ServiceType,
            b"USERNAME" => VCardParameterName::Username,
            b"JSPTR" => VCardParameterName::Jsptr,
            b"JSCOMPS" => VCardParameterName::Jscomps,
        )
    }

    pub fn parse(input: &str) -> Self {
        Self::try_parse(input.as_bytes()).unwrap_or_else(|| VCardParameterName::Other(input.into()))
    }

    pub fn as_str(&self) -> &str {
        match self {
            VCardParameterName::Language => "LANGUAGE",
            VCardParameterName::Value => "VALUE",
            VCardParameterName::Pref => "PREF",
            VCardParameterName::Altid => "ALTID",
            VCardParameterName::Pid => "PID",
            VCardParameterName::Type => "TYPE",
            VCardParameterName::Mediatype => "MEDIATYPE",
            VCardParameterName::Calscale => "CALSCALE",
            VCardParameterName::SortAs => "SORT-AS",
            VCardParameterName::Geo => "GEO",
            VCardParameterName::Tz => "TZ",
            VCardParameterName::Index => "INDEX",
            VCardParameterName::Level => "LEVEL",
            VCardParameterName::Group => "GROUP",
            VCardParameterName::Cc => "CC",
            VCardParameterName::Author => "AUTHOR",
            VCardParameterName::AuthorName => "AUTHOR-NAME",
            VCardParameterName::Created => "CREATED",
            VCardParameterName::Derived => "DERIVED",
            VCardParameterName::Label => "LABEL",
            VCardParameterName::Phonetic => "PHONETIC",
            VCardParameterName::PropId => "PROP-ID",
            VCardParameterName::Script => "SCRIPT",
            VCardParameterName::ServiceType => "SERVICE-TYPE",
            VCardParameterName::Username => "USERNAME",
            VCardParameterName::Jsptr => "JSPTR",
            VCardParameterName::Jscomps => "JSCOMPS",
            VCardParameterName::Other(name) => name,
        }
    }

    pub fn into_string(self) -> Cow<'static, str> {
        match self {
            VCardParameterName::Language => "LANGUAGE".into(),
            VCardParameterName::Value => "VALUE".into(),
            VCardParameterName::Pref => "PREF".into(),
            VCardParameterName::Altid => "ALTID".into(),
            VCardParameterName::Pid => "PID".into(),
            VCardParameterName::Type => "TYPE".into(),
            VCardParameterName::Mediatype => "MEDIATYPE".into(),
            VCardParameterName::Calscale => "CALSCALE".into(),
            VCardParameterName::SortAs => "SORT-AS".into(),
            VCardParameterName::Geo => "GEO".into(),
            VCardParameterName::Tz => "TZ".into(),
            VCardParameterName::Index => "INDEX".into(),
            VCardParameterName::Level => "LEVEL".into(),
            VCardParameterName::Group => "GROUP".into(),
            VCardParameterName::Cc => "CC".into(),
            VCardParameterName::Author => "AUTHOR".into(),
            VCardParameterName::AuthorName => "AUTHOR-NAME".into(),
            VCardParameterName::Created => "CREATED".into(),
            VCardParameterName::Derived => "DERIVED".into(),
            VCardParameterName::Label => "LABEL".into(),
            VCardParameterName::Phonetic => "PHONETIC".into(),
            VCardParameterName::PropId => "PROP-ID".into(),
            VCardParameterName::Script => "SCRIPT".into(),
            VCardParameterName::ServiceType => "SERVICE-TYPE".into(),
            VCardParameterName::Username => "USERNAME".into(),
            VCardParameterName::Jsptr => "JSPTR".into(),
            VCardParameterName::Jscomps => "JSCOMPS".into(),
            VCardParameterName::Other(name) => name.into(),
        }
    }
}

impl VCardVersion {
    pub fn try_parse(input: &str) -> Option<Self> {
        hashify::tiny_map!(input.as_bytes(),
            b"4.0" => VCardVersion::V4_0,
            b"3.0" => VCardVersion::V3_0,
            b"2.1" => VCardVersion::V2_1,
            b"2.0" => VCardVersion::V2_0,
        )
    }
}
