/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

use crate::jscontact::*;
use std::str::FromStr;

impl FromStr for JSContactProperty {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        hashify::map!(s.as_bytes(), JSContactProperty,
            "@type" => JSContactProperty::Type,
            "address" => JSContactProperty::Address,
            "addressBookIds" => JSContactProperty::AddressBookIds,
            "addresses" => JSContactProperty::Addresses,
            "anniversaries" => JSContactProperty::Anniversaries,
            "author" => JSContactProperty::Author,
            "blobId" => JSContactProperty::BlobId,
            "calendars" => JSContactProperty::Calendars,
            "calendarScale" => JSContactProperty::CalendarScale,
            "components" => JSContactProperty::Components,
            "contexts" => JSContactProperty::Contexts,
            "convertedProperties" => JSContactProperty::ConvertedProperties,
            "coordinates" => JSContactProperty::Coordinates,
            "countryCode" => JSContactProperty::CountryCode,
            "created" => JSContactProperty::Created,
            "cryptoKeys" => JSContactProperty::CryptoKeys,
            "date" => JSContactProperty::Date,
            "day" => JSContactProperty::Day,
            "defaultSeparator" => JSContactProperty::DefaultSeparator,
            "directories" => JSContactProperty::Directories,
            "emails" => JSContactProperty::Emails,
            "extra" => JSContactProperty::Extra,
            "features" => JSContactProperty::Features,
            "full" => JSContactProperty::Full,
            "grammaticalGender" => JSContactProperty::GrammaticalGender,
            "id" => JSContactProperty::Id,
            "isOrdered" => JSContactProperty::IsOrdered,
            "keywords" => JSContactProperty::Keywords,
            "kind" => JSContactProperty::Kind,
            "label" => JSContactProperty::Label,
            "language" => JSContactProperty::Language,
            "level" => JSContactProperty::Level,
            "links" => JSContactProperty::Links,
            "listAs" => JSContactProperty::ListAs,
            "localizations" => JSContactProperty::Localizations,
            "media" => JSContactProperty::Media,
            "mediaType" => JSContactProperty::MediaType,
            "members" => JSContactProperty::Members,
            "month" => JSContactProperty::Month,
            "name" => JSContactProperty::Name,
            "nicknames" => JSContactProperty::Nicknames,
            "note" => JSContactProperty::Note,
            "notes" => JSContactProperty::Notes,
            "number" => JSContactProperty::Number,
            "onlineServices" => JSContactProperty::OnlineServices,
            "organizationId" => JSContactProperty::OrganizationId,
            "organizations" => JSContactProperty::Organizations,
            "parameters" => JSContactProperty::Parameters,
            "personalInfo" => JSContactProperty::PersonalInfo,
            "phones" => JSContactProperty::Phones,
            "phonetic" => JSContactProperty::Phonetic,
            "phoneticScript" => JSContactProperty::PhoneticScript,
            "phoneticSystem" => JSContactProperty::PhoneticSystem,
            "place" => JSContactProperty::Place,
            "pref" => JSContactProperty::Pref,
            "preferredLanguages" => JSContactProperty::PreferredLanguages,
            "prodId" => JSContactProperty::ProdId,
            "pronouns" => JSContactProperty::Pronouns,
            "properties" => JSContactProperty::Properties,
            "relatedTo" => JSContactProperty::RelatedTo,
            "relation" => JSContactProperty::Relation,
            "schedulingAddresses" => JSContactProperty::SchedulingAddresses,
            "service" => JSContactProperty::Service,
            "sortAs" => JSContactProperty::SortAs,
            "speakToAs" => JSContactProperty::SpeakToAs,
            "timeZone" => JSContactProperty::TimeZone,
            "titles" => JSContactProperty::Titles,
            "uid" => JSContactProperty::Uid,
            "units" => JSContactProperty::Units,
            "updated" => JSContactProperty::Updated,
            "uri" => JSContactProperty::Uri,
            "user" => JSContactProperty::User,
            "utc" => JSContactProperty::Utc,
            "value" => JSContactProperty::Value,
            "vCard" => JSContactProperty::VCard,
            "version" => JSContactProperty::Version,
            "year" => JSContactProperty::Year,
        )
        .cloned()
        .ok_or(())
    }
}

impl JSContactProperty {
    pub fn to_string(&self) -> Cow<'static, str> {
        match self {
            JSContactProperty::Type => "@type",
            JSContactProperty::Address => "address",
            JSContactProperty::AddressBookIds => "addressBookIds",
            JSContactProperty::Addresses => "addresses",
            JSContactProperty::Anniversaries => "anniversaries",
            JSContactProperty::Author => "author",
            JSContactProperty::BlobId => "blobId",
            JSContactProperty::Calendars => "calendars",
            JSContactProperty::CalendarScale => "calendarScale",
            JSContactProperty::Components => "components",
            JSContactProperty::Contexts => "contexts",
            JSContactProperty::ConvertedProperties => "convertedProperties",
            JSContactProperty::Coordinates => "coordinates",
            JSContactProperty::CountryCode => "countryCode",
            JSContactProperty::Created => "created",
            JSContactProperty::CryptoKeys => "cryptoKeys",
            JSContactProperty::Date => "date",
            JSContactProperty::Day => "day",
            JSContactProperty::DefaultSeparator => "defaultSeparator",
            JSContactProperty::Directories => "directories",
            JSContactProperty::Emails => "emails",
            JSContactProperty::Extra => "extra",
            JSContactProperty::Features => "features",
            JSContactProperty::Full => "full",
            JSContactProperty::GrammaticalGender => "grammaticalGender",
            JSContactProperty::Id => "id",
            JSContactProperty::IsOrdered => "isOrdered",
            JSContactProperty::Keywords => "keywords",
            JSContactProperty::Kind => "kind",
            JSContactProperty::Label => "label",
            JSContactProperty::Language => "language",
            JSContactProperty::Level => "level",
            JSContactProperty::Links => "links",
            JSContactProperty::ListAs => "listAs",
            JSContactProperty::Localizations => "localizations",
            JSContactProperty::Media => "media",
            JSContactProperty::MediaType => "mediaType",
            JSContactProperty::Members => "members",
            JSContactProperty::Month => "month",
            JSContactProperty::Name => "name",
            JSContactProperty::Nicknames => "nicknames",
            JSContactProperty::Note => "note",
            JSContactProperty::Notes => "notes",
            JSContactProperty::Number => "number",
            JSContactProperty::OnlineServices => "onlineServices",
            JSContactProperty::OrganizationId => "organizationId",
            JSContactProperty::Organizations => "organizations",
            JSContactProperty::Parameters => "parameters",
            JSContactProperty::PersonalInfo => "personalInfo",
            JSContactProperty::Phones => "phones",
            JSContactProperty::Phonetic => "phonetic",
            JSContactProperty::PhoneticScript => "phoneticScript",
            JSContactProperty::PhoneticSystem => "phoneticSystem",
            JSContactProperty::Place => "place",
            JSContactProperty::Pref => "pref",
            JSContactProperty::PreferredLanguages => "preferredLanguages",
            JSContactProperty::ProdId => "prodId",
            JSContactProperty::Pronouns => "pronouns",
            JSContactProperty::Properties => "properties",
            JSContactProperty::RelatedTo => "relatedTo",
            JSContactProperty::Relation => "relation",
            JSContactProperty::SchedulingAddresses => "schedulingAddresses",
            JSContactProperty::Service => "service",
            JSContactProperty::SortAs => "sortAs",
            JSContactProperty::SpeakToAs => "speakToAs",
            JSContactProperty::TimeZone => "timeZone",
            JSContactProperty::Titles => "titles",
            JSContactProperty::Uid => "uid",
            JSContactProperty::Units => "units",
            JSContactProperty::Updated => "updated",
            JSContactProperty::Uri => "uri",
            JSContactProperty::User => "user",
            JSContactProperty::Utc => "utc",
            JSContactProperty::Value => "value",
            JSContactProperty::VCard => "vCard",
            JSContactProperty::Version => "version",
            JSContactProperty::Year => "year",
            JSContactProperty::Context(context) => context.as_str(),
            JSContactProperty::Feature(feature) => feature.as_str(),
            JSContactProperty::SortAsKind(kind) => kind.as_str(),
            JSContactProperty::Pointer(pointer) => return Cow::Owned(pointer.to_string()),
        }
        .into()
    }
}

impl FromStr for Context {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        hashify::map!(s.as_bytes(), Context,
            "billing" => Context::Billing,
            "delivery" => Context::Delivery,
            "private" => Context::Private,
            "work" => Context::Work,
        )
        .copied()
        .ok_or(())
    }
}

impl Context {
    pub fn as_str(&self) -> &'static str {
        match self {
            Context::Billing => "billing",
            Context::Delivery => "delivery",
            Context::Private => "private",
            Context::Work => "work",
        }
    }
}

impl FromStr for Feature {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        hashify::map!(s.as_bytes(), Feature,
            "fax" => Feature::Fax,
            "main-number" => Feature::MainNumber,
            "mobile" => Feature::Mobile,
            "pager" => Feature::Pager,
            "text" => Feature::Text,
            "textphone" => Feature::TextPhone,
            "video" => Feature::Video,
            "voice" => Feature::Voice,
        )
        .copied()
        .ok_or(())
    }
}

impl Feature {
    pub fn as_str(&self) -> &'static str {
        match self {
            Feature::Fax => "fax",
            Feature::MainNumber => "main-number",
            Feature::Mobile => "mobile",
            Feature::Pager => "pager",
            Feature::Text => "text",
            Feature::TextPhone => "textphone",
            Feature::Video => "video",
            Feature::Voice => "voice",
        }
    }
}

impl FromStr for JSContactGrammaticalGender {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        hashify::map!(s.as_bytes(), JSContactGrammaticalGender,
            "animate" => JSContactGrammaticalGender::Animate,
            "common" => JSContactGrammaticalGender::Common,
            "feminine" => JSContactGrammaticalGender::Feminine,
            "inanimate" => JSContactGrammaticalGender::Inanimate,
            "masculine" => JSContactGrammaticalGender::Masculine,
            "neuter" => JSContactGrammaticalGender::Neuter,
        )
        .copied()
        .ok_or(())
    }
}

impl JSContactGrammaticalGender {
    pub fn as_str(&self) -> &'static str {
        match self {
            JSContactGrammaticalGender::Animate => "animate",
            JSContactGrammaticalGender::Common => "common",
            JSContactGrammaticalGender::Feminine => "feminine",
            JSContactGrammaticalGender::Inanimate => "inanimate",
            JSContactGrammaticalGender::Masculine => "masculine",
            JSContactGrammaticalGender::Neuter => "neuter",
        }
    }
}

impl FromStr for JSContactKind {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        hashify::map!(s.as_bytes(), JSContactKind,
            "apartment" => JSContactKind::Apartment,
            "block" => JSContactKind::Block,
            "building" => JSContactKind::Building,
            "country" => JSContactKind::Country,
            "direction" => JSContactKind::Direction,
            "district" => JSContactKind::District,
            "floor" => JSContactKind::Floor,
            "landmark" => JSContactKind::Landmark,
            "locality" => JSContactKind::Locality,
            "name" => JSContactKind::Name,
            "number" => JSContactKind::Number,
            "postcode" => JSContactKind::Postcode,
            "postOfficeBox" => JSContactKind::PostOfficeBox,
            "region" => JSContactKind::Region,
            "room" => JSContactKind::Room,
            "separator" => JSContactKind::Separator,
            "subdistrict" => JSContactKind::Subdistrict,
            "birth" => JSContactKind::Birth,
            "death" => JSContactKind::Death,
            "wedding" => JSContactKind::Wedding,
            "calendar" => JSContactKind::Calendar,
            "freeBusy" => JSContactKind::FreeBusy,
            "application" => JSContactKind::Application,
            "device" => JSContactKind::Device,
            "group" => JSContactKind::Group,
            "individual" => JSContactKind::Individual,
            "location" => JSContactKind::Location,
            "org" => JSContactKind::Org,
            "directory" => JSContactKind::Directory,
            "entry" => JSContactKind::Entry,
            "contact" => JSContactKind::Contact,
            "logo" => JSContactKind::Logo,
            "photo" => JSContactKind::Photo,
            "sound" => JSContactKind::Sound,
            "credential" => JSContactKind::Credential,
            "generation" => JSContactKind::Generation,
            "given" => JSContactKind::Given,
            "given2" => JSContactKind::Given2,
            "surname" => JSContactKind::Surname,
            "surname2" => JSContactKind::Surname2,
            "title" => JSContactKind::Title,
            "expertise" => JSContactKind::Expertise,
            "hobby" => JSContactKind::Hobby,
            "interest" => JSContactKind::Interest,
            "role" => JSContactKind::Role,
        )
        .copied()
        .ok_or(())
    }
}

#[allow(clippy::wrong_self_convention)]
impl JSContactKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            JSContactKind::Apartment => "apartment",
            JSContactKind::Block => "block",
            JSContactKind::Building => "building",
            JSContactKind::Country => "country",
            JSContactKind::Direction => "direction",
            JSContactKind::District => "district",
            JSContactKind::Floor => "floor",
            JSContactKind::Landmark => "landmark",
            JSContactKind::Locality => "locality",
            JSContactKind::Name => "name",
            JSContactKind::Number => "number",
            JSContactKind::Postcode => "postcode",
            JSContactKind::PostOfficeBox => "postOfficeBox",
            JSContactKind::Region => "region",
            JSContactKind::Room => "room",
            JSContactKind::Separator => "separator",
            JSContactKind::Subdistrict => "subdistrict",
            JSContactKind::Birth => "birth",
            JSContactKind::Death => "death",
            JSContactKind::Wedding => "wedding",
            JSContactKind::Calendar => "calendar",
            JSContactKind::FreeBusy => "freeBusy",
            JSContactKind::Application => "application",
            JSContactKind::Device => "device",
            JSContactKind::Group => "group",
            JSContactKind::Individual => "individual",
            JSContactKind::Location => "location",
            JSContactKind::Org => "org",
            JSContactKind::Directory => "directory",
            JSContactKind::Entry => "entry",
            JSContactKind::Contact => "contact",
            JSContactKind::Logo => "logo",
            JSContactKind::Photo => "photo",
            JSContactKind::Sound => "sound",
            JSContactKind::Credential => "credential",
            JSContactKind::Generation => "generation",
            JSContactKind::Given => "given",
            JSContactKind::Given2 => "given2",
            JSContactKind::Surname => "surname",
            JSContactKind::Surname2 => "surname2",
            JSContactKind::Title => "title",
            JSContactKind::Expertise => "expertise",
            JSContactKind::Hobby => "hobby",
            JSContactKind::Interest => "interest",
            JSContactKind::Role => "role",
        }
    }

    pub(crate) fn from_vcard_adr_pos(pos: usize) -> Option<Self> {
        match pos {
            0 => JSContactKind::PostOfficeBox,
            1 => JSContactKind::Apartment,
            2 => JSContactKind::Name,
            3 => JSContactKind::Locality,
            4 => JSContactKind::Region,
            5 => JSContactKind::Postcode,
            6 => JSContactKind::Country,
            7 => JSContactKind::Room,
            8 => JSContactKind::Apartment,
            9 => JSContactKind::Floor,
            10 => JSContactKind::Number,
            11 => JSContactKind::Name,
            12 => JSContactKind::Building,
            13 => JSContactKind::Block,
            14 => JSContactKind::Subdistrict,
            15 => JSContactKind::District,
            16 => JSContactKind::Landmark,
            17 => JSContactKind::Direction,
            _ => return None,
        }
        .into()
    }

    pub(crate) fn to_vcard_adr_pos(&self) -> Option<usize> {
        match self {
            JSContactKind::PostOfficeBox => 0,
            JSContactKind::Apartment => 8,
            JSContactKind::Name => 11,
            JSContactKind::Locality => 3,
            JSContactKind::Region => 4,
            JSContactKind::Postcode => 5,
            JSContactKind::Country => 6,
            JSContactKind::Room => 7,
            JSContactKind::Floor => 9,
            JSContactKind::Number => 10,
            JSContactKind::Building => 12,
            JSContactKind::Block => 13,
            JSContactKind::Subdistrict => 14,
            JSContactKind::District => 15,
            JSContactKind::Landmark => 16,
            JSContactKind::Direction => 17,
            _ => return None,
        }
        .into()
    }

    pub(crate) fn from_vcard_n_pos(pos: usize) -> Option<Self> {
        match pos {
            0 => JSContactKind::Surname,
            1 => JSContactKind::Given,
            2 => JSContactKind::Given2,
            3 => JSContactKind::Title,
            4 => JSContactKind::Credential,
            5 => JSContactKind::Surname2,
            6 => JSContactKind::Generation,
            _ => return None,
        }
        .into()
    }

    pub(crate) fn to_vcard_n_pos(&self) -> Option<usize> {
        match self {
            JSContactKind::Surname => 0,
            JSContactKind::Given => 1,
            JSContactKind::Given2 => 2,
            JSContactKind::Title => 3,
            JSContactKind::Credential => 4,
            JSContactKind::Surname2 => 5,
            JSContactKind::Generation => 6,
            _ => return None,
        }
        .into()
    }
}

impl FromStr for JSContactLevel {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        hashify::map!(s.as_bytes(), JSContactLevel,
            "high" => JSContactLevel::High,
            "low" => JSContactLevel::Low,
            "medium" => JSContactLevel::Medium,
        )
        .copied()
        .ok_or(())
    }
}

impl JSContactLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            JSContactLevel::High => "high",
            JSContactLevel::Low => "low",
            JSContactLevel::Medium => "medium",
        }
    }
}

impl FromStr for JSContactRelation {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        hashify::map!(s.as_bytes(), JSContactRelation,
            "acquaintance" => JSContactRelation::Acquaintance,
            "agent" => JSContactRelation::Agent,
            "child" => JSContactRelation::Child,
            "colleague" => JSContactRelation::Colleague,
            "contact" => JSContactRelation::Contact,
            "co-resident" => JSContactRelation::CoResident,
            "co-worker" => JSContactRelation::CoWorker,
            "crush" => JSContactRelation::Crush,
            "date" => JSContactRelation::Date,
            "emergency" => JSContactRelation::Emergency,
            "friend" => JSContactRelation::Friend,
            "kin" => JSContactRelation::Kin,
            "me" => JSContactRelation::Me,
            "met" => JSContactRelation::Met,
            "muse" => JSContactRelation::Muse,
            "neighbor" => JSContactRelation::Neighbor,
            "parent" => JSContactRelation::Parent,
            "sibling" => JSContactRelation::Sibling,
            "spouse" => JSContactRelation::Spouse,
            "sweetheart" => JSContactRelation::Sweetheart,
        )
        .copied()
        .ok_or(())
    }
}

impl JSContactRelation {
    pub fn as_str(&self) -> &'static str {
        match self {
            JSContactRelation::Acquaintance => "acquaintance",
            JSContactRelation::Agent => "agent",
            JSContactRelation::Child => "child",
            JSContactRelation::Colleague => "colleague",
            JSContactRelation::Contact => "contact",
            JSContactRelation::CoResident => "co-resident",
            JSContactRelation::CoWorker => "co-worker",
            JSContactRelation::Crush => "crush",
            JSContactRelation::Date => "date",
            JSContactRelation::Emergency => "emergency",
            JSContactRelation::Friend => "friend",
            JSContactRelation::Kin => "kin",
            JSContactRelation::Me => "me",
            JSContactRelation::Met => "met",
            JSContactRelation::Muse => "muse",
            JSContactRelation::Neighbor => "neighbor",
            JSContactRelation::Parent => "parent",
            JSContactRelation::Sibling => "sibling",
            JSContactRelation::Spouse => "spouse",
            JSContactRelation::Sweetheart => "sweetheart",
        }
    }
}

impl FromStr for JSContactPhoneticSystem {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        hashify::map!(s.as_bytes(), JSContactPhoneticSystem,
            "ipa" => JSContactPhoneticSystem::Ipa,
            "jyut" => JSContactPhoneticSystem::Jyut,
            "piny" => JSContactPhoneticSystem::Piny,
            "script" => JSContactPhoneticSystem::Script,
        )
        .copied()
        .ok_or(())
    }
}

impl JSContactPhoneticSystem {
    pub fn as_str(&self) -> &'static str {
        match self {
            JSContactPhoneticSystem::Ipa => "ipa",
            JSContactPhoneticSystem::Jyut => "jyut",
            JSContactPhoneticSystem::Piny => "piny",
            JSContactPhoneticSystem::Script => "script",
        }
    }
}
