/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

use super::*;

impl IanaString for CalendarScale {
    fn as_str(&self) -> &'static str {
        match self {
            CalendarScale::Gregorian => "GREGORIAN",
            CalendarScale::Chinese => "CHINESE",
            CalendarScale::IslamicCivil => "ISLAMIC-CIVIL",
            CalendarScale::Hebrew => "HEBREW",
            CalendarScale::Ethiopic => "ETHIOPIC",
        }
    }
}

#[cfg(feature = "rkyv")]
impl IanaString for ArchivedCalendarScale {
    fn as_str(&self) -> &'static str {
        match self {
            ArchivedCalendarScale::Gregorian => "GREGORIAN",
            ArchivedCalendarScale::Chinese => "CHINESE",
            ArchivedCalendarScale::IslamicCivil => "ISLAMIC-CIVIL",
            ArchivedCalendarScale::Hebrew => "HEBREW",
            ArchivedCalendarScale::Ethiopic => "ETHIOPIC",
        }
    }
}

impl IanaParse for CalendarScale {
    fn parse(value: &[u8]) -> Option<Self> {
        hashify::tiny_map_ignore_case!(value,
            "gregorian" => CalendarScale::Gregorian,
            "chinese" => CalendarScale::Chinese,
            "islamic-civil" => CalendarScale::IslamicCivil,
            "hebrew" => CalendarScale::Hebrew,
            "ethiopic" => CalendarScale::Ethiopic,
        )
    }
}

impl Encoding {
    pub fn parse(value: &[u8]) -> Option<Self> {
        hashify::tiny_map_ignore_case!(value,
            b"QUOTED-PRINTABLE" => Encoding::QuotedPrintable,
            b"BASE64" => Encoding::Base64,
            b"Q" => Encoding::QuotedPrintable,
            b"B" => Encoding::Base64,
        )
    }
}

impl PartialDateTime {
    pub fn now() -> Self {
        Self::from_utc_timestamp(chrono::Utc::now().timestamp())
    }

    pub fn from_utc_timestamp(value: i64) -> Self {
        let dt = DateTime::from_timestamp(value);

        PartialDateTime {
            year: dt.year.into(),
            month: dt.month.into(),
            day: dt.day.into(),
            hour: dt.hour.into(),
            minute: dt.minute.into(),
            second: dt.second.into(),
            tz_hour: dt.tz_hour.into(),
            tz_minute: dt.tz_minute.into(),
            tz_minus: false,
        }
    }

    pub fn to_date_time(&self) -> Option<DateTimeResult> {
        let mut dt = DateTimeResult {
            date_time: NaiveDate::from_ymd_opt(
                self.year? as i32,
                self.month? as u32,
                self.day? as u32,
            )?
            .and_hms_opt(
                self.hour.unwrap_or(0) as u32,
                self.minute.unwrap_or(0) as u32,
                self.second.unwrap_or(0) as u32,
            )?,
            offset: None,
        };
        if let Some(tz_hour) = self.tz_hour {
            let secs = (tz_hour as i32 * 3600) + (self.tz_minute.unwrap_or(0) as i32 * 60);
            dt.offset = if self.tz_minus {
                FixedOffset::west_opt(secs)?
            } else {
                FixedOffset::east_opt(secs)?
            }
            .into();
        }
        Some(dt)
    }
}

#[cfg(feature = "rkyv")]
impl ArchivedPartialDateTime {
    pub fn to_date_time(&self) -> Option<DateTimeResult> {
        let mut dt = DateTimeResult {
            date_time: NaiveDate::from_ymd_opt(
                self.year.as_ref()?.to_native() as i32,
                *self.month.as_ref()? as u32,
                *self.day.as_ref()? as u32,
            )?
            .and_hms_opt(
                self.hour.unwrap_or(0) as u32,
                self.minute.unwrap_or(0) as u32,
                self.second.unwrap_or(0) as u32,
            )?,
            offset: None,
        };
        if let Some(tz_hour) = self.tz_hour.as_ref() {
            let secs = (*tz_hour as i32 * 3600) + (self.tz_minute.unwrap_or(0) as i32 * 60);
            dt.offset = if self.tz_minus {
                FixedOffset::west_opt(secs)?
            } else {
                FixedOffset::east_opt(secs)?
            }
            .into();
        }
        Some(dt)
    }
}

impl IanaParse for LinkRelation {
    fn parse(s: &[u8]) -> Option<Self> {
        hashify::map_ignore_case!(s, LinkRelation,
            "about" => LinkRelation::About,
            "acl" => LinkRelation::Acl,
            "alternate" => LinkRelation::Alternate,
            "amphtml" => LinkRelation::Amphtml,
            "api-catalog" => LinkRelation::ApiCatalog,
            "appendix" => LinkRelation::Appendix,
            "apple-touch-icon" => LinkRelation::AppleTouchIcon,
            "apple-touch-startup-image" => LinkRelation::AppleTouchStartupImage,
            "archives" => LinkRelation::Archives,
            "author" => LinkRelation::Author,
            "blocked-by" => LinkRelation::BlockedBy,
            "bookmark" => LinkRelation::Bookmark,
            "c2pa-manifest" => LinkRelation::C2paManifest,
            "canonical" => LinkRelation::Canonical,
            "chapter" => LinkRelation::Chapter,
            "cite-as" => LinkRelation::CiteAs,
            "collection" => LinkRelation::Collection,
            "compression-dictionary" => LinkRelation::CompressionDictionary,
            "contents" => LinkRelation::Contents,
            "convertedfrom" => LinkRelation::Convertedfrom,
            "copyright" => LinkRelation::Copyright,
            "create-form" => LinkRelation::CreateForm,
            "current" => LinkRelation::Current,
            "deprecation" => LinkRelation::Deprecation,
            "describedby" => LinkRelation::Describedby,
            "describes" => LinkRelation::Describes,
            "disclosure" => LinkRelation::Disclosure,
            "dns-prefetch" => LinkRelation::DnsPrefetch,
            "duplicate" => LinkRelation::Duplicate,
            "edit" => LinkRelation::Edit,
            "edit-form" => LinkRelation::EditForm,
            "edit-media" => LinkRelation::EditMedia,
            "enclosure" => LinkRelation::Enclosure,
            "external" => LinkRelation::External,
            "first" => LinkRelation::First,
            "geofeed" => LinkRelation::Geofeed,
            "glossary" => LinkRelation::Glossary,
            "help" => LinkRelation::Help,
            "hosts" => LinkRelation::Hosts,
            "hub" => LinkRelation::Hub,
            "ice-server" => LinkRelation::IceServer,
            "icon" => LinkRelation::Icon,
            "index" => LinkRelation::Index,
            "intervalafter" => LinkRelation::Intervalafter,
            "intervalbefore" => LinkRelation::Intervalbefore,
            "intervalcontains" => LinkRelation::Intervalcontains,
            "intervaldisjoint" => LinkRelation::Intervaldisjoint,
            "intervalduring" => LinkRelation::Intervalduring,
            "intervalequals" => LinkRelation::Intervalequals,
            "intervalfinishedby" => LinkRelation::Intervalfinishedby,
            "intervalfinishes" => LinkRelation::Intervalfinishes,
            "intervalin" => LinkRelation::Intervalin,
            "intervalmeets" => LinkRelation::Intervalmeets,
            "intervalmetby" => LinkRelation::Intervalmetby,
            "intervaloverlappedby" => LinkRelation::Intervaloverlappedby,
            "intervaloverlaps" => LinkRelation::Intervaloverlaps,
            "intervalstartedby" => LinkRelation::Intervalstartedby,
            "intervalstarts" => LinkRelation::Intervalstarts,
            "item" => LinkRelation::Item,
            "last" => LinkRelation::Last,
            "latest-version" => LinkRelation::LatestVersion,
            "license" => LinkRelation::License,
            "linkset" => LinkRelation::Linkset,
            "lrdd" => LinkRelation::Lrdd,
            "manifest" => LinkRelation::Manifest,
            "mask-icon" => LinkRelation::MaskIcon,
            "me" => LinkRelation::Me,
            "media-feed" => LinkRelation::MediaFeed,
            "memento" => LinkRelation::Memento,
            "micropub" => LinkRelation::Micropub,
            "modulepreload" => LinkRelation::Modulepreload,
            "monitor" => LinkRelation::Monitor,
            "monitor-group" => LinkRelation::MonitorGroup,
            "next" => LinkRelation::Next,
            "next-archive" => LinkRelation::NextArchive,
            "nofollow" => LinkRelation::Nofollow,
            "noopener" => LinkRelation::Noopener,
            "noreferrer" => LinkRelation::Noreferrer,
            "opener" => LinkRelation::Opener,
            "openid2.local_id" => LinkRelation::Openid2LocalId,
            "openid2.provider" => LinkRelation::Openid2Provider,
            "original" => LinkRelation::Original,
            "p3pv1" => LinkRelation::P3pv1,
            "payment" => LinkRelation::Payment,
            "pingback" => LinkRelation::Pingback,
            "preconnect" => LinkRelation::Preconnect,
            "predecessor-version" => LinkRelation::PredecessorVersion,
            "prefetch" => LinkRelation::Prefetch,
            "preload" => LinkRelation::Preload,
            "prerender" => LinkRelation::Prerender,
            "prev" => LinkRelation::Prev,
            "preview" => LinkRelation::Preview,
            "previous" => LinkRelation::Previous,
            "prev-archive" => LinkRelation::PrevArchive,
            "privacy-policy" => LinkRelation::PrivacyPolicy,
            "profile" => LinkRelation::Profile,
            "publication" => LinkRelation::Publication,
            "rdap-active" => LinkRelation::RdapActive,
            "rdap-bottom" => LinkRelation::RdapBottom,
            "rdap-down" => LinkRelation::RdapDown,
            "rdap-top" => LinkRelation::RdapTop,
            "rdap-up" => LinkRelation::RdapUp,
            "related" => LinkRelation::Related,
            "restconf" => LinkRelation::Restconf,
            "replies" => LinkRelation::Replies,
            "ruleinput" => LinkRelation::Ruleinput,
            "search" => LinkRelation::Search,
            "section" => LinkRelation::Section,
            "self" => LinkRelation::Self_,
            "service" => LinkRelation::Service,
            "service-desc" => LinkRelation::ServiceDesc,
            "service-doc" => LinkRelation::ServiceDoc,
            "service-meta" => LinkRelation::ServiceMeta,
            "sip-trunking-capability" => LinkRelation::SipTrunkingCapability,
            "sponsored" => LinkRelation::Sponsored,
            "start" => LinkRelation::Start,
            "status" => LinkRelation::Status,
            "stylesheet" => LinkRelation::Stylesheet,
            "subsection" => LinkRelation::Subsection,
            "successor-version" => LinkRelation::SuccessorVersion,
            "sunset" => LinkRelation::Sunset,
            "tag" => LinkRelation::Tag,
            "terms-of-service" => LinkRelation::TermsOfService,
            "timegate" => LinkRelation::Timegate,
            "timemap" => LinkRelation::Timemap,
            "type" => LinkRelation::Type,
            "ugc" => LinkRelation::Ugc,
            "up" => LinkRelation::Up,
            "version-history" => LinkRelation::VersionHistory,
            "via" => LinkRelation::Via,
            "webmention" => LinkRelation::Webmention,
            "working-copy" => LinkRelation::WorkingCopy,
            "working-copy-of" => LinkRelation::WorkingCopyOf,
        )
        .copied()
    }
}

impl IanaString for LinkRelation {
    fn as_str(&self) -> &'static str {
        match self {
            LinkRelation::About => "about",
            LinkRelation::Acl => "acl",
            LinkRelation::Alternate => "alternate",
            LinkRelation::Amphtml => "amphtml",
            LinkRelation::ApiCatalog => "api-catalog",
            LinkRelation::Appendix => "appendix",
            LinkRelation::AppleTouchIcon => "apple-touch-icon",
            LinkRelation::AppleTouchStartupImage => "apple-touch-startup-image",
            LinkRelation::Archives => "archives",
            LinkRelation::Author => "author",
            LinkRelation::BlockedBy => "blocked-by",
            LinkRelation::Bookmark => "bookmark",
            LinkRelation::C2paManifest => "c2pa-manifest",
            LinkRelation::Canonical => "canonical",
            LinkRelation::Chapter => "chapter",
            LinkRelation::CiteAs => "cite-as",
            LinkRelation::Collection => "collection",
            LinkRelation::CompressionDictionary => "compression-dictionary",
            LinkRelation::Contents => "contents",
            LinkRelation::Convertedfrom => "convertedfrom",
            LinkRelation::Copyright => "copyright",
            LinkRelation::CreateForm => "create-form",
            LinkRelation::Current => "current",
            LinkRelation::Deprecation => "deprecation",
            LinkRelation::Describedby => "describedby",
            LinkRelation::Describes => "describes",
            LinkRelation::Disclosure => "disclosure",
            LinkRelation::DnsPrefetch => "dns-prefetch",
            LinkRelation::Duplicate => "duplicate",
            LinkRelation::Edit => "edit",
            LinkRelation::EditForm => "edit-form",
            LinkRelation::EditMedia => "edit-media",
            LinkRelation::Enclosure => "enclosure",
            LinkRelation::External => "external",
            LinkRelation::First => "first",
            LinkRelation::Geofeed => "geofeed",
            LinkRelation::Glossary => "glossary",
            LinkRelation::Help => "help",
            LinkRelation::Hosts => "hosts",
            LinkRelation::Hub => "hub",
            LinkRelation::IceServer => "ice-server",
            LinkRelation::Icon => "icon",
            LinkRelation::Index => "index",
            LinkRelation::Intervalafter => "intervalafter",
            LinkRelation::Intervalbefore => "intervalbefore",
            LinkRelation::Intervalcontains => "intervalcontains",
            LinkRelation::Intervaldisjoint => "intervaldisjoint",
            LinkRelation::Intervalduring => "intervalduring",
            LinkRelation::Intervalequals => "intervalequals",
            LinkRelation::Intervalfinishedby => "intervalfinishedby",
            LinkRelation::Intervalfinishes => "intervalfinishes",
            LinkRelation::Intervalin => "intervalin",
            LinkRelation::Intervalmeets => "intervalmeets",
            LinkRelation::Intervalmetby => "intervalmetby",
            LinkRelation::Intervaloverlappedby => "intervaloverlappedby",
            LinkRelation::Intervaloverlaps => "intervaloverlaps",
            LinkRelation::Intervalstartedby => "intervalstartedby",
            LinkRelation::Intervalstarts => "intervalstarts",
            LinkRelation::Item => "item",
            LinkRelation::Last => "last",
            LinkRelation::LatestVersion => "latest-version",
            LinkRelation::License => "license",
            LinkRelation::Linkset => "linkset",
            LinkRelation::Lrdd => "lrdd",
            LinkRelation::Manifest => "manifest",
            LinkRelation::MaskIcon => "mask-icon",
            LinkRelation::Me => "me",
            LinkRelation::MediaFeed => "media-feed",
            LinkRelation::Memento => "memento",
            LinkRelation::Micropub => "micropub",
            LinkRelation::Modulepreload => "modulepreload",
            LinkRelation::Monitor => "monitor",
            LinkRelation::MonitorGroup => "monitor-group",
            LinkRelation::Next => "next",
            LinkRelation::NextArchive => "next-archive",
            LinkRelation::Nofollow => "nofollow",
            LinkRelation::Noopener => "noopener",
            LinkRelation::Noreferrer => "noreferrer",
            LinkRelation::Opener => "opener",
            LinkRelation::Openid2LocalId => "openid2.local_id",
            LinkRelation::Openid2Provider => "openid2.provider",
            LinkRelation::Original => "original",
            LinkRelation::P3pv1 => "p3pv1",
            LinkRelation::Payment => "payment",
            LinkRelation::Pingback => "pingback",
            LinkRelation::Preconnect => "preconnect",
            LinkRelation::PredecessorVersion => "predecessor-version",
            LinkRelation::Prefetch => "prefetch",
            LinkRelation::Preload => "preload",
            LinkRelation::Prerender => "prerender",
            LinkRelation::Prev => "prev",
            LinkRelation::Preview => "preview",
            LinkRelation::Previous => "previous",
            LinkRelation::PrevArchive => "prev-archive",
            LinkRelation::PrivacyPolicy => "privacy-policy",
            LinkRelation::Profile => "profile",
            LinkRelation::Publication => "publication",
            LinkRelation::RdapActive => "rdap-active",
            LinkRelation::RdapBottom => "rdap-bottom",
            LinkRelation::RdapDown => "rdap-down",
            LinkRelation::RdapTop => "rdap-top",
            LinkRelation::RdapUp => "rdap-up",
            LinkRelation::Related => "related",
            LinkRelation::Restconf => "restconf",
            LinkRelation::Replies => "replies",
            LinkRelation::Ruleinput => "ruleinput",
            LinkRelation::Search => "search",
            LinkRelation::Section => "section",
            LinkRelation::Self_ => "self",
            LinkRelation::Service => "service",
            LinkRelation::ServiceDesc => "service-desc",
            LinkRelation::ServiceDoc => "service-doc",
            LinkRelation::ServiceMeta => "service-meta",
            LinkRelation::SipTrunkingCapability => "sip-trunking-capability",
            LinkRelation::Sponsored => "sponsored",
            LinkRelation::Start => "start",
            LinkRelation::Status => "status",
            LinkRelation::Stylesheet => "stylesheet",
            LinkRelation::Subsection => "subsection",
            LinkRelation::SuccessorVersion => "successor-version",
            LinkRelation::Sunset => "sunset",
            LinkRelation::Tag => "tag",
            LinkRelation::TermsOfService => "terms-of-service",
            LinkRelation::Timegate => "timegate",
            LinkRelation::Timemap => "timemap",
            LinkRelation::Type => "type",
            LinkRelation::Ugc => "ugc",
            LinkRelation::Up => "up",
            LinkRelation::VersionHistory => "version-history",
            LinkRelation::Via => "via",
            LinkRelation::Webmention => "webmention",
            LinkRelation::WorkingCopy => "working-copy",
            LinkRelation::WorkingCopyOf => "working-copy-of",
        }
    }
}
