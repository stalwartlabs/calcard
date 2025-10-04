/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

use chrono::{FixedOffset, NaiveDate, NaiveDateTime};
use mail_parser::DateTime;

pub mod iana;
pub mod parser;
pub mod timezone;
pub mod tokenizer;
pub mod types;
pub mod writer;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum IanaType<I, O> {
    Iana(I),
    Other(O),
}

pub trait IanaString {
    fn as_str(&self) -> &'static str;
}

pub trait IanaParse: Sized {
    fn parse(value: &[u8]) -> Option<Self>;
}

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize)
)]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Serialize, rkyv::Deserialize, rkyv::Archive)
)]
#[cfg_attr(feature = "rkyv", rkyv(compare(PartialEq), derive(Debug)))]
pub struct PartialDateTime {
    pub year: Option<u16>,
    pub month: Option<u8>,
    pub day: Option<u8>,
    pub hour: Option<u8>,
    pub minute: Option<u8>,
    pub second: Option<u8>,
    pub tz_hour: Option<u8>,
    pub tz_minute: Option<u8>,
    pub tz_minus: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
pub enum CalendarScale {
    #[default]
    Gregorian,
    Chinese,
    IslamicCivil,
    Hebrew,
    Ethiopic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Encoding {
    QuotedPrintable,
    Base64,
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
pub struct Data {
    pub content_type: Option<String>,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DateTimeResult {
    pub date_time: NaiveDateTime,
    pub offset: Option<FixedOffset>,
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
pub enum LinkRelation {
    About,
    Acl,
    Alternate,
    Amphtml,
    ApiCatalog,
    Appendix,
    AppleTouchIcon,
    AppleTouchStartupImage,
    Archives,
    Author,
    BlockedBy,
    Bookmark,
    C2paManifest,
    Canonical,
    Chapter,
    CiteAs,
    Collection,
    CompressionDictionary,
    Contents,
    Convertedfrom,
    Copyright,
    CreateForm,
    Current,
    Deprecation,
    Describedby,
    Describes,
    Disclosure,
    DnsPrefetch,
    Duplicate,
    Edit,
    EditForm,
    EditMedia,
    Enclosure,
    External,
    First,
    Geofeed,
    Glossary,
    Help,
    Hosts,
    Hub,
    IceServer,
    Icon,
    Index,
    Intervalafter,
    Intervalbefore,
    Intervalcontains,
    Intervaldisjoint,
    Intervalduring,
    Intervalequals,
    Intervalfinishedby,
    Intervalfinishes,
    Intervalin,
    Intervalmeets,
    Intervalmetby,
    Intervaloverlappedby,
    Intervaloverlaps,
    Intervalstartedby,
    Intervalstarts,
    Item,
    Last,
    LatestVersion,
    License,
    Linkset,
    Lrdd,
    Manifest,
    MaskIcon,
    Me,
    MediaFeed,
    Memento,
    Micropub,
    Modulepreload,
    Monitor,
    MonitorGroup,
    Next,
    NextArchive,
    Nofollow,
    Noopener,
    Noreferrer,
    Opener,
    Openid2LocalId,
    Openid2Provider,
    Original,
    P3pv1,
    Payment,
    Pingback,
    Preconnect,
    PredecessorVersion,
    Prefetch,
    Preload,
    Prerender,
    Prev,
    Preview,
    Previous,
    PrevArchive,
    PrivacyPolicy,
    Profile,
    Publication,
    RdapActive,
    RdapBottom,
    RdapDown,
    RdapTop,
    RdapUp,
    Related,
    Restconf,
    Replies,
    Ruleinput,
    Search,
    Section,
    Self_,
    Service,
    ServiceDesc,
    ServiceDoc,
    ServiceMeta,
    SipTrunkingCapability,
    Sponsored,
    Start,
    Status,
    Stylesheet,
    Subsection,
    SuccessorVersion,
    Sunset,
    Tag,
    TermsOfService,
    Timegate,
    Timemap,
    Type,
    Ugc,
    Up,
    VersionHistory,
    Via,
    Webmention,
    WorkingCopy,
    WorkingCopyOf,
}

pub(crate) enum IdReference<T> {
    Value(T),
    Reference(String),
    Error,
}

impl<T: std::str::FromStr> IdReference<T> {
    pub fn parse(s: &str) -> Self {
        if let Some(reference) = s.strip_prefix('#') {
            IdReference::Reference(reference.to_string())
        } else {
            T::from_str(s)
                .map(IdReference::Value)
                .unwrap_or(IdReference::Error)
        }
    }
}
