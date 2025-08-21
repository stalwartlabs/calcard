/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

use super::*;

impl ArchivedVCardProperty {
    pub fn as_str(&self) -> &str {
        match self {
            ArchivedVCardProperty::Source => "SOURCE",
            ArchivedVCardProperty::Kind => "KIND",
            ArchivedVCardProperty::Xml => "XML",
            ArchivedVCardProperty::Fn => "FN",
            ArchivedVCardProperty::N => "N",
            ArchivedVCardProperty::Nickname => "NICKNAME",
            ArchivedVCardProperty::Photo => "PHOTO",
            ArchivedVCardProperty::Bday => "BDAY",
            ArchivedVCardProperty::Anniversary => "ANNIVERSARY",
            ArchivedVCardProperty::Gender => "GENDER",
            ArchivedVCardProperty::Adr => "ADR",
            ArchivedVCardProperty::Tel => "TEL",
            ArchivedVCardProperty::Email => "EMAIL",
            ArchivedVCardProperty::Impp => "IMPP",
            ArchivedVCardProperty::Lang => "LANG",
            ArchivedVCardProperty::Tz => "TZ",
            ArchivedVCardProperty::Geo => "GEO",
            ArchivedVCardProperty::Title => "TITLE",
            ArchivedVCardProperty::Role => "ROLE",
            ArchivedVCardProperty::Logo => "LOGO",
            ArchivedVCardProperty::Org => "ORG",
            ArchivedVCardProperty::Member => "MEMBER",
            ArchivedVCardProperty::Related => "RELATED",
            ArchivedVCardProperty::Categories => "CATEGORIES",
            ArchivedVCardProperty::Note => "NOTE",
            ArchivedVCardProperty::Prodid => "PRODID",
            ArchivedVCardProperty::Rev => "REV",
            ArchivedVCardProperty::Sound => "SOUND",
            ArchivedVCardProperty::Uid => "UID",
            ArchivedVCardProperty::Clientpidmap => "CLIENTPIDMAP",
            ArchivedVCardProperty::Url => "URL",
            ArchivedVCardProperty::Version => "VERSION",
            ArchivedVCardProperty::Key => "KEY",
            ArchivedVCardProperty::Fburl => "FBURL",
            ArchivedVCardProperty::Caladruri => "CALADRURI",
            ArchivedVCardProperty::Caluri => "CALURI",
            ArchivedVCardProperty::Birthplace => "BIRTHPLACE",
            ArchivedVCardProperty::Deathplace => "DEATHPLACE",
            ArchivedVCardProperty::Deathdate => "DEATHDATE",
            ArchivedVCardProperty::Expertise => "EXPERTISE",
            ArchivedVCardProperty::Hobby => "HOBBY",
            ArchivedVCardProperty::Interest => "INTEREST",
            ArchivedVCardProperty::OrgDirectory => "ORG-DIRECTORY",
            ArchivedVCardProperty::ContactUri => "CONTACT-URI",
            ArchivedVCardProperty::Created => "CREATED",
            ArchivedVCardProperty::Gramgender => "GRAMGENDER",
            ArchivedVCardProperty::Language => "LANGUAGE",
            ArchivedVCardProperty::Pronouns => "PRONOUNS",
            ArchivedVCardProperty::Socialprofile => "SOCIALPROFILE",
            ArchivedVCardProperty::Jsprop => "JSPROP",
            ArchivedVCardProperty::Begin => "BEGIN",
            ArchivedVCardProperty::End => "END",
            ArchivedVCardProperty::Other(s) => s.as_str(),
        }
    }

    // Returns the default value type and whether the property is multi-valued.
    pub(crate) fn default_types(&self) -> (ArchivedValueType, ValueSeparator) {
        match self {
            ArchivedVCardProperty::Source => (
                ArchivedValueType::Vcard(ArchivedVCardValueType::Uri),
                ValueSeparator::None,
            ),
            ArchivedVCardProperty::Kind => (ArchivedValueType::Kind, ValueSeparator::None),
            ArchivedVCardProperty::Xml => (
                ArchivedValueType::Vcard(ArchivedVCardValueType::Text),
                ValueSeparator::None,
            ),
            ArchivedVCardProperty::Fn => (
                ArchivedValueType::Vcard(ArchivedVCardValueType::Text),
                ValueSeparator::None,
            ),
            ArchivedVCardProperty::N => (
                ArchivedValueType::Vcard(ArchivedVCardValueType::Text),
                ValueSeparator::SemicolonAndComma,
            ),
            ArchivedVCardProperty::Nickname => (
                ArchivedValueType::Vcard(ArchivedVCardValueType::Text),
                ValueSeparator::Comma,
            ),
            ArchivedVCardProperty::Photo => (
                ArchivedValueType::Vcard(ArchivedVCardValueType::Uri),
                ValueSeparator::None,
            ),
            ArchivedVCardProperty::Bday => (
                ArchivedValueType::Vcard(ArchivedVCardValueType::DateAndOrTime),
                ValueSeparator::None,
            ),
            ArchivedVCardProperty::Anniversary => (
                ArchivedValueType::Vcard(ArchivedVCardValueType::DateAndOrTime),
                ValueSeparator::None,
            ),
            ArchivedVCardProperty::Gender => (ArchivedValueType::Sex, ValueSeparator::Semicolon),
            ArchivedVCardProperty::Adr => (
                ArchivedValueType::Vcard(ArchivedVCardValueType::Text),
                ValueSeparator::SemicolonAndComma,
            ),
            ArchivedVCardProperty::Tel => (
                ArchivedValueType::Vcard(ArchivedVCardValueType::Text),
                ValueSeparator::None,
            ),
            ArchivedVCardProperty::Email => (
                ArchivedValueType::Vcard(ArchivedVCardValueType::Text),
                ValueSeparator::None,
            ),
            ArchivedVCardProperty::Impp => (
                ArchivedValueType::Vcard(ArchivedVCardValueType::Uri),
                ValueSeparator::None,
            ),
            ArchivedVCardProperty::Lang => (
                ArchivedValueType::Vcard(ArchivedVCardValueType::Text),
                ValueSeparator::None,
            ),
            ArchivedVCardProperty::Tz => (
                ArchivedValueType::Vcard(ArchivedVCardValueType::UtcOffset),
                ValueSeparator::None,
            ),
            ArchivedVCardProperty::Geo => (
                ArchivedValueType::Vcard(ArchivedVCardValueType::Uri),
                ValueSeparator::None,
            ),
            ArchivedVCardProperty::Title => (
                ArchivedValueType::Vcard(ArchivedVCardValueType::Text),
                ValueSeparator::None,
            ),
            ArchivedVCardProperty::Role => (
                ArchivedValueType::Vcard(ArchivedVCardValueType::Text),
                ValueSeparator::None,
            ),
            ArchivedVCardProperty::Logo => (
                ArchivedValueType::Vcard(ArchivedVCardValueType::Uri),
                ValueSeparator::None,
            ),
            ArchivedVCardProperty::Org => (
                ArchivedValueType::Vcard(ArchivedVCardValueType::Text),
                ValueSeparator::Semicolon,
            ),
            ArchivedVCardProperty::Member => (
                ArchivedValueType::Vcard(ArchivedVCardValueType::Uri),
                ValueSeparator::None,
            ),
            ArchivedVCardProperty::Related => (
                ArchivedValueType::Vcard(ArchivedVCardValueType::Uri),
                ValueSeparator::None,
            ),
            ArchivedVCardProperty::Categories => (
                ArchivedValueType::Vcard(ArchivedVCardValueType::Text),
                ValueSeparator::Comma,
            ),
            ArchivedVCardProperty::Note => (
                ArchivedValueType::Vcard(ArchivedVCardValueType::Text),
                ValueSeparator::None,
            ),
            ArchivedVCardProperty::Prodid => (
                ArchivedValueType::Vcard(ArchivedVCardValueType::Text),
                ValueSeparator::None,
            ),
            ArchivedVCardProperty::Rev => (
                ArchivedValueType::Vcard(ArchivedVCardValueType::Timestamp),
                ValueSeparator::None,
            ),
            ArchivedVCardProperty::Sound => (
                ArchivedValueType::Vcard(ArchivedVCardValueType::Uri),
                ValueSeparator::None,
            ),
            ArchivedVCardProperty::Uid => (
                ArchivedValueType::Vcard(ArchivedVCardValueType::Uri),
                ValueSeparator::None,
            ),
            ArchivedVCardProperty::Clientpidmap => (
                ArchivedValueType::Vcard(ArchivedVCardValueType::Text),
                ValueSeparator::Semicolon,
            ),
            ArchivedVCardProperty::Url => (
                ArchivedValueType::Vcard(ArchivedVCardValueType::Uri),
                ValueSeparator::None,
            ),
            ArchivedVCardProperty::Version => (
                ArchivedValueType::Vcard(ArchivedVCardValueType::Text),
                ValueSeparator::None,
            ),
            ArchivedVCardProperty::Key => (
                ArchivedValueType::Vcard(ArchivedVCardValueType::Uri),
                ValueSeparator::None,
            ),
            ArchivedVCardProperty::Fburl => (
                ArchivedValueType::Vcard(ArchivedVCardValueType::Uri),
                ValueSeparator::None,
            ),
            ArchivedVCardProperty::Caladruri => (
                ArchivedValueType::Vcard(ArchivedVCardValueType::Uri),
                ValueSeparator::None,
            ),
            ArchivedVCardProperty::Caluri => (
                ArchivedValueType::Vcard(ArchivedVCardValueType::Uri),
                ValueSeparator::None,
            ),
            ArchivedVCardProperty::Birthplace => (
                ArchivedValueType::Vcard(ArchivedVCardValueType::Text),
                ValueSeparator::None,
            ),
            ArchivedVCardProperty::Deathplace => (
                ArchivedValueType::Vcard(ArchivedVCardValueType::Text),
                ValueSeparator::None,
            ),
            ArchivedVCardProperty::Deathdate => (
                ArchivedValueType::Vcard(ArchivedVCardValueType::DateAndOrTime),
                ValueSeparator::None,
            ),
            ArchivedVCardProperty::Expertise => (
                ArchivedValueType::Vcard(ArchivedVCardValueType::Text),
                ValueSeparator::None,
            ),
            ArchivedVCardProperty::Hobby => (
                ArchivedValueType::Vcard(ArchivedVCardValueType::Text),
                ValueSeparator::None,
            ),
            ArchivedVCardProperty::Interest => (
                ArchivedValueType::Vcard(ArchivedVCardValueType::Text),
                ValueSeparator::None,
            ),
            ArchivedVCardProperty::OrgDirectory => (
                ArchivedValueType::Vcard(ArchivedVCardValueType::Uri),
                ValueSeparator::None,
            ),
            ArchivedVCardProperty::ContactUri => (
                ArchivedValueType::Vcard(ArchivedVCardValueType::Uri),
                ValueSeparator::None,
            ),
            ArchivedVCardProperty::Created => (
                ArchivedValueType::Vcard(ArchivedVCardValueType::Timestamp),
                ValueSeparator::None,
            ),
            ArchivedVCardProperty::Gramgender => {
                (ArchivedValueType::GramGender, ValueSeparator::None)
            }
            ArchivedVCardProperty::Language => (
                ArchivedValueType::Vcard(ArchivedVCardValueType::LanguageTag),
                ValueSeparator::None,
            ),
            ArchivedVCardProperty::Pronouns => (
                ArchivedValueType::Vcard(ArchivedVCardValueType::Text),
                ValueSeparator::None,
            ),
            ArchivedVCardProperty::Socialprofile => (
                ArchivedValueType::Vcard(ArchivedVCardValueType::Uri),
                ValueSeparator::None,
            ),
            ArchivedVCardProperty::Jsprop => (
                ArchivedValueType::Vcard(ArchivedVCardValueType::Text),
                ValueSeparator::None,
            ),
            ArchivedVCardProperty::Other(_) => (
                ArchivedValueType::Vcard(ArchivedVCardValueType::Text),
                ValueSeparator::None,
            ),
            ArchivedVCardProperty::Begin | ArchivedVCardProperty::End => (
                ArchivedValueType::Vcard(ArchivedVCardValueType::Text),
                ValueSeparator::Skip,
            ),
        }
    }
}

impl ArchivedVCardValueType {
    pub fn as_str(&self) -> &str {
        match self {
            ArchivedVCardValueType::Boolean => "BOOLEAN",
            ArchivedVCardValueType::Date => "DATE",
            ArchivedVCardValueType::DateAndOrTime => "DATE-AND-OR-TIME",
            ArchivedVCardValueType::DateTime => "DATE-TIME",
            ArchivedVCardValueType::Float => "FLOAT",
            ArchivedVCardValueType::Integer => "INTEGER",
            ArchivedVCardValueType::LanguageTag => "LANGUAGE-TAG",
            ArchivedVCardValueType::Text => "TEXT",
            ArchivedVCardValueType::Time => "TIME",
            ArchivedVCardValueType::Timestamp => "TIMESTAMP",
            ArchivedVCardValueType::Uri => "URI",
            ArchivedVCardValueType::UtcOffset => "UTC-OFFSET",
            ArchivedVCardValueType::Other(s) => s.as_str(),
        }
    }
}

impl AsRef<str> for ArchivedVCardValueType {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl ArchivedVCardLevel {
    pub fn as_str(&self) -> &str {
        match self {
            ArchivedVCardLevel::Beginner => "BEGINNER",
            ArchivedVCardLevel::Average => "AVERAGE",
            ArchivedVCardLevel::Expert => "EXPERT",
            ArchivedVCardLevel::High => "HIGH",
            ArchivedVCardLevel::Medium => "MEDIUM",
            ArchivedVCardLevel::Low => "LOW",
        }
    }
}

impl AsRef<str> for ArchivedVCardLevel {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl ArchivedVCardPhonetic {
    pub fn as_str(&self) -> &str {
        match self {
            ArchivedVCardPhonetic::Ipa => "IPA",
            ArchivedVCardPhonetic::Jyut => "JYUT",
            ArchivedVCardPhonetic::Piny => "PINY",
            ArchivedVCardPhonetic::Script => "SCRIPT",
            ArchivedVCardPhonetic::Other(s) => s.as_str(),
        }
    }
}

impl AsRef<str> for ArchivedVCardPhonetic {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl ArchivedVCardType {
    pub fn as_str(&self) -> &str {
        match self {
            ArchivedVCardType::Work => "WORK",
            ArchivedVCardType::Home => "HOME",
            ArchivedVCardType::Billing => "BILLING",
            ArchivedVCardType::Delivery => "DELIVERY",
            ArchivedVCardType::Contact => "CONTACT",
            ArchivedVCardType::Acquaintance => "ACQUAINTANCE",
            ArchivedVCardType::Friend => "FRIEND",
            ArchivedVCardType::Met => "MET",
            ArchivedVCardType::CoWorker => "CO-WORKER",
            ArchivedVCardType::Colleague => "COLLEAGUE",
            ArchivedVCardType::CoResident => "CO-RESIDENT",
            ArchivedVCardType::Neighbor => "NEIGHBOR",
            ArchivedVCardType::Child => "CHILD",
            ArchivedVCardType::Parent => "PARENT",
            ArchivedVCardType::Sibling => "SIBLING",
            ArchivedVCardType::Spouse => "SPOUSE",
            ArchivedVCardType::Kin => "KIN",
            ArchivedVCardType::Muse => "MUSE",
            ArchivedVCardType::Crush => "CRUSH",
            ArchivedVCardType::Date => "DATE",
            ArchivedVCardType::Sweetheart => "SWEETHEART",
            ArchivedVCardType::Me => "ME",
            ArchivedVCardType::Agent => "AGENT",
            ArchivedVCardType::Emergency => "EMERGENCY",
            ArchivedVCardType::Text => "TEXT",
            ArchivedVCardType::Voice => "VOICE",
            ArchivedVCardType::Fax => "FAX",
            ArchivedVCardType::Cell => "CELL",
            ArchivedVCardType::Video => "VIDEO",
            ArchivedVCardType::Pager => "PAGER",
            ArchivedVCardType::Textphone => "TEXTPHONE",
            ArchivedVCardType::MainNumber => "MAIN-NUMBER",
            ArchivedVCardType::Other(s) => s.as_str(),
        }
    }
}

impl AsRef<str> for ArchivedVCardType {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl ArchivedVCardGramGender {
    pub fn as_str(&self) -> &str {
        match self {
            ArchivedVCardGramGender::Animate => "ANIMATE",
            ArchivedVCardGramGender::Common => "COMMON",
            ArchivedVCardGramGender::Feminine => "FEMININE",
            ArchivedVCardGramGender::Inanimate => "INANIMATE",
            ArchivedVCardGramGender::Masculine => "MASCULINE",
            ArchivedVCardGramGender::Neuter => "NEUTER",
        }
    }
}

impl ArchivedVCardSex {
    pub fn as_str(&self) -> &str {
        match self {
            ArchivedVCardSex::Male => "M",
            ArchivedVCardSex::Female => "F",
            ArchivedVCardSex::Other => "O",
            ArchivedVCardSex::NoneOrNotApplicable => "N",
            ArchivedVCardSex::Unknown => "U",
        }
    }
}

impl ArchivedVCardKind {
    pub fn as_str(&self) -> &str {
        match self {
            ArchivedVCardKind::Individual => "INDIVIDUAL",
            ArchivedVCardKind::Group => "GROUP",
            ArchivedVCardKind::Org => "ORG",
            ArchivedVCardKind::Location => "LOCATION",
            ArchivedVCardKind::Application => "APPLICATION",
            ArchivedVCardKind::Device => "DEVICE",
        }
    }
}

#[derive(Debug)]

pub(crate) enum ArchivedValueType {
    Vcard(ArchivedVCardValueType),
    Kind,
    Sex,
    GramGender,
}

impl ArchivedValueType {
    pub fn unwrap_vcard(self) -> ArchivedVCardValueType {
        match self {
            ArchivedValueType::Vcard(v) => v,
            _ => ArchivedVCardValueType::Text,
        }
    }
}
