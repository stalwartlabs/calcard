/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

use crate::{
    Entry, Parser,
    common::{CalendarScale, Data, IanaParse, IanaString, IanaType, PartialDateTime},
};

pub mod builder;
pub(crate) mod media_type;
pub mod parser;
pub mod types;
pub mod utils;
pub mod writer;

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
pub struct VCard {
    pub entries: Vec<VCardEntry>,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum VCardVersion {
    V2_0 = 20,
    V2_1 = 21,
    V3_0 = 30,
    #[default]
    V4_0 = 40,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize)
)]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Serialize, rkyv::Deserialize, rkyv::Archive)
)]
#[cfg_attr(feature = "rkyv", rkyv(compare(PartialEq)))]
pub struct VCardEntry {
    pub group: Option<String>,
    pub name: VCardProperty,
    pub params: Vec<VCardParameter>,
    pub values: Vec<VCardValue>,
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
pub enum VCardProperty {
    Other(String),
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
}

#[derive(Debug, Clone, PartialEq)]
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
pub enum VCardValue {
    Text(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    PartialDateTime(PartialDateTime),
    Binary(Data),
    Sex(VCardSex),
    GramGender(VCardGramGender),
    Kind(VCardKind),
    Component(Vec<String>),
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
pub struct VCardParameter {
    pub name: VCardParameterName,
    pub value: VCardParameterValue,
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
pub enum VCardParameterValue {
    Text(String),
    Integer(u32),
    Timestamp(i64),
    Bool(bool),
    ValueType(VCardValueType),
    Type(VCardType),
    Calscale(CalendarScale),
    Level(VCardLevel),
    Phonetic(VCardPhonetic),
    Jscomps(Vec<Jscomp>),
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
#[cfg_attr(feature = "rkyv", rkyv(compare(PartialEq), derive(Debug)))]
pub enum Jscomp {
    Entry { position: u32, value: u32 },
    Separator(String),
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
pub enum VCardParameterName {
    Other(String),
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
    Jscomps,     // [RFC9555, Section 3.3.2]
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
pub enum VCardLevel {
    Beginner, // [RFC6715, Section 3.2]
    Average,  // [RFC6715, Section 3.2]
    Expert,   // [RFC6715, Section 3.2]
    High,     // [RFC6715, Section 3.2]
    Medium,   // [RFC6715, Section 3.2]
    Low,      // [RFC6715, Section 3.2]
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
pub enum VCardPhonetic {
    Ipa,    // [RFC9554, Section 4.6]
    Jyut,   // [RFC9554, Section 4.6]
    Piny,   // [RFC9554, Section 4.6]
    Script, // [RFC9554, Section 4.6]
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
pub enum VCardGramGender {
    Animate,   // [RFC9554, Section 3.2]
    Common,    // [RFC9554, Section 3.2]
    Feminine,  // [RFC9554, Section 3.2]
    Inanimate, // [RFC9554, Section 3.2]
    Masculine, // [RFC9554, Section 3.2]
    Neuter,    // [RFC9554, Section 3.2]
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
pub enum VCardSex {
    Male,
    Female,
    Other,
    NoneOrNotApplicable,
    Unknown,
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
pub enum VCardKind {
    Individual,  // [RFC6350, Section 6.1.4]
    Group,       // [RFC6350, Section 6.1.4]
    Org,         // [RFC6350, Section 6.1.4]
    Location,    // [RFC6350, Section 6.1.4]
    Application, // [RFC6473, Section 3]
    Device,      // [RFC6869, Section 3]
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ValueSeparator {
    None,
    Comma,
    Semicolon,
    SemicolonAndComma,
    Skip,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ValueType {
    Vcard(VCardValueType),
    Kind,
    Sex,
    GramGender,
}

impl ValueType {
    pub fn unwrap_vcard(self) -> VCardValueType {
        match self {
            ValueType::Vcard(v) => v,
            _ => VCardValueType::Text,
        }
    }
}

impl VCard {
    pub fn parse(value: impl AsRef<str>) -> Result<Self, Entry> {
        let mut parser = Parser::new(value.as_ref());
        match parser.entry() {
            Entry::VCard(vcard) => Ok(vcard),
            other => Err(other),
        }
    }
}
