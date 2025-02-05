use ahash::AHashMap;
use serde::{Deserialize, Serialize};

use crate::vcard::{VCardParameter, VCardProperty};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct JSContact {
    pub items: AHashMap<Property, Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Property {
    Created,
    Kind,
    Language,
    Members,
    ProdId,
    RelatedTo,
    Uid,
    Updated,
    Name,
    Nicknames,
    Organizations,
    SpeakToAs,
    Titles,
    Emails,
    OnlineServices,
    PreferredLanguages,
    Calendars,
    SchedulingAddresses,
    Addresses,
    CryptoKeys,
    Directories,
    Links,
    Media,
    Localizations,
    Anniversaries,
    Keywords,
    Notes,
    PersonalInfo,
    VCardProps,
    Components,
    IsOrdered,
    DefaultSeparator,
    SortAs,
    Phonetic,
    PhoneticScript,
    PhoneticSystem,
    Value,
    Contexts,
    Pref,
    Units,
    GrammaticalGender,
    Pronouns,
    Address,
    Label,
    Service,
    Uri,
    Number,
    CountryCode,
    Coordinates,
    Timezone,
    Full,
    MediaType,
    ListAs,
    Date,
    Place,
    Year,
    Month,
    Day,
    CalendarScale,
    Hour,
    Minute,
    Second,
    Note,
    Author,
    Level,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VCardProp {
    name: String,
    params: VCardParams,
    value: Value,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct VCardParams(pub Vec<(VCardParameter, Vec<Value>)>);

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Text(String),
    Uri(String),
    Date(PartialDate),
    Time(PartialTime),
    DateTime {
        is_and_or: bool,
        date: PartialDate,
        time: PartialTime,
    },
    Timestamp(UTCDateTime),
    Boolean(bool),
    Float(f64),
    UTCOffset(i16),
    UInt(u32),
    Kind(Kind),
    LanguageTag(String),
    Phonetic(PhoneticSystem),
    CalendarScale(CalendarScale),
    Addresses(Addresses),
    Anniversaries(Anniversaries),
    Calendars(Calendars),
    Directories(Directories),
    Emails(Emails),
    Keywords(Keywords),
    Links(Links),
    Media(Medias),
    Members(Members),
    Name(Name),
    Nicknames(Nicknames),
    Notes(Notes),
    OnlineServices(OnlineServices),
    Organizations(Organizations),
    PersonalInfo(PersonalInfos),
    Phones(Phones),
    PreferredLanguages(PreferredLanguages),
    RelatedTo(RelatedTo),
    SchedulingAddresses(SchedulingAddresses),
    SpeakToAs(SpeakToAs),
    Titles(Titles),
    CryptoKeys(CryptoKeys),
    Localizations(Localizations),
    VCardProps(Vec<VCardProp>),
}

impl Eq for Value {}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Kind {
    #[default]
    Individual,
    Group,
    Org,
    Location,
    Device,
    Application,
    Other(String),
}

impl TryFrom<&[u8]> for Kind {
    type Error = ();
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        hashify::tiny_map_ignore_case!(value,
            "individual" => Kind::Individual,
            "group" => Kind::Group,
            "org" => Kind::Org,
            "location" => Kind::Location,
            "device" => Kind::Device,
            "application" => Kind::Application,
        )
        .ok_or(())
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct UTCDateTime(pub i64);

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Members(pub Vec<String>);

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RelatedTo(pub Vec<(String, Vec<Relation>)>);

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Relation {
    Acquaintance,
    Agent,
    Child,
    Colleague,
    #[default]
    Contact,
    CoResident,
    CoWorker,
    Crush,
    Date,
    Emergency,
    Friend,
    Kin,
    Me,
    Met,
    Muse,
    Neighbor,
    Parent,
    Sibling,
    Spouse,
    Sweetheart,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Name {
    components: Vec<NameComponent>,
    is_ordered: bool,
    default_separator: Option<String>,
    full: Option<String>,
    sort_as: Vec<(NameKind, String)>,
    phonetic_script: Option<String>,
    phonetic_system: Option<PhoneticSystem>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PhoneticSystem {
    #[default]
    Ipa,
    Jyut,
    Piny,
    Script,
    Other(String),
}

impl TryFrom<&[u8]> for PhoneticSystem {
    type Error = ();
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        hashify::tiny_map_ignore_case!(value,
            "ipa" => PhoneticSystem::Ipa,
            "jyut" => PhoneticSystem::Jyut,
            "piny" => PhoneticSystem::Piny,
            "script" => PhoneticSystem::Script,
        )
        .ok_or(())
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NameComponent {
    value: String,
    kind: NameKind,
    phonetic: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum NameKind {
    Credential,
    Generation,
    Given,
    Given2,
    Separator,
    #[default]
    Surname,
    Surname2,
    Title,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Nickname {
    name: String,
    contexts: Vec<DefaultContext>,
    pref: Option<u32>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Nicknames(pub Vec<(String, Nickname)>);

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DefaultContext {
    Private,
    #[default]
    Work,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Organizations(pub Vec<(String, Organization)>);

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Organization {
    name: String,
    units: Vec<OrgUnit>,
    sort_as: Option<String>,
    contexts: Vec<DefaultContext>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct OrgUnit {
    name: String,
    sort_as: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SpeakToAs {
    pub grammatical_gender: Option<GrammaticalGender>,
    pub pronouns: Vec<(String, Pronouns)>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Pronouns {
    pronouns: String,
    contexts: Vec<DefaultContext>,
    pref: Option<u32>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum GrammaticalGender {
    Animate,
    #[default]
    Common,
    Feminine,
    Inanimate,
    Masculine,
    Neuter,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Titles(pub Vec<(String, Title)>);

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Title {
    name: String,
    kind: TitleKind,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TitleKind {
    #[default]
    Title,
    Role,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Emails(pub Vec<(String, EmailAddress)>);

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EmailAddress {
    address: String,
    contexts: Vec<DefaultContext>,
    pref: Option<u32>,
    label: Option<String>,
    vcard_params: VCardParams,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct OnlineServices(pub Vec<(String, OnlineService)>);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OnlineService {
    service: Option<String>,
    uri: Option<String>,
    user: Option<String>,
    contexts: Vec<DefaultContext>,
    pref: Option<u32>,
    label: Option<String>,
    vcard_name: Option<VCardProperty>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Phones(pub Vec<(String, Phone)>);

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Phone {
    number: String,
    features: Vec<PhoneFeature>,
    contexts: Vec<DefaultContext>,
    pref: Option<u32>,
    label: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PhoneFeature {
    Fax,
    #[default]
    MainNumber,
    Mobile,
    Pager,
    Text,
    Textphone,
    Video,
    Voice,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PreferredLanguages(pub Vec<(String, LanguagePref)>);

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LanguagePref {
    language: String,
    contexts: Vec<DefaultContext>,
    pref: Option<u32>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Calendars(pub Vec<(String, Calendar)>);

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Calendar {
    kind: CalendarKind,
    uri: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CalendarKind {
    #[default]
    Calendar,
    FreeBusy,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SchedulingAddresses(pub Vec<(String, SchedulingAddress)>);

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SchedulingAddress {
    uri: String,
    contexts: Vec<DefaultContext>,
    pref: Option<u32>,
    label: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Addresses(pub Vec<(String, Address)>);

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Address {
    components: Vec<AddressComponent>,
    is_ordered: bool,
    country_code: Option<String>,
    coordinates: Option<String>,
    timezone: Option<String>,
    contexts: Vec<AddressContext>,
    full: Option<String>,
    default_separator: Option<String>,
    pref: Option<u32>,
    phonetic_script: Option<String>,
    phonetic_system: Option<PhoneticSystem>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AddressComponent {
    value: String,
    kind: AddressComponentKind,
    phonetic: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AddressComponentKind {
    Apartment,
    Block,
    Building,
    Country,
    Direction,
    District,
    Floor,
    Landmark,
    Locality,
    #[default]
    Name,
    Number,
    Postcode,
    PostOfficeBox,
    Region,
    Room,
    Separator,
    Subdistrict,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AddressContext {
    Billing,
    Delivery,
    Private,
    #[default]
    Work,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Resource<K, C> {
    kind: K,
    uri: ResourceData,
    media_type: Option<String>,
    contexts: Vec<C>,
    pref: Option<u32>,
    label: Option<String>,
    vcard_params: VCardParams,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceData {
    Uri(String),
    Data(Vec<u8>),
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CryptoKeys(pub Vec<(String, CryptoKey)>);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CryptoKey(pub Resource<Option<String>, DefaultContext>);

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Directories(pub Vec<(String, Directory)>);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Directory {
    res: Resource<DirectoryKind, DefaultContext>,
    list_as: Option<u32>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DirectoryKind {
    #[default]
    Directory,
    Entry,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Links(pub Vec<(String, Link)>);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Link(pub Resource<Option<LinkKind>, DefaultContext>);

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LinkKind {
    #[default]
    Contact,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Medias(pub Vec<(String, Media)>);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Media(pub Resource<MediaKind, DefaultContext>);

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MediaKind {
    Logo,
    #[default]
    Photo,
    Sound,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Localizations(pub Vec<(String, Vec<PatchObject>)>);

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Anniversaries(pub Vec<(String, Anniversary)>);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Anniversary {
    kind: AnniversaryKind,
    date: Date,
    place: Option<Address>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AnniversaryKind {
    #[default]
    Birth,
    Death,
    Wedding,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Date {
    PartialDate(PartialDate),
    Timestamp(UTCDateTime),
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PartialDate {
    year: Option<u32>,
    month: Option<u32>,
    day: Option<u32>,
    calendar_scale: Option<CalendarScale>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PartialTime {
    hour: Option<u16>,
    minute: Option<u16>,
    second: Option<u16>,
    timezone: Option<u16>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CalendarScale {
    #[default]
    Gregorian,
    Chinese,
    IslamicCivil,
    Hebrew,
    Ethiopic,
    Other(String),
}

impl TryFrom<&[u8]> for CalendarScale {
    type Error = ();
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        hashify::tiny_map_ignore_case!(value,
            "gregorian" => CalendarScale::Gregorian,
            "chinese" => CalendarScale::Chinese,
            "islamic-civil" => CalendarScale::IslamicCivil,
            "hebrew" => CalendarScale::Hebrew,
            "ethiopic" => CalendarScale::Ethiopic,
        )
        .ok_or(())
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Keywords(pub Vec<String>);

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Notes(pub Vec<(String, Note)>);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Note {
    note: String,
    created: Option<UTCDateTime>,
    author: Option<Author>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Author {
    name: Option<String>,
    uri: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PersonalInfos(pub Vec<(String, PersonalInfo)>);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PersonalInfo {
    kind: PersonalInfoKind,
    value: String,
    level: Option<PersonalInfoLevel>,
    list_as: Option<u32>,
    label: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PersonalInfoKind {
    Expertise,
    Hobby,
    #[default]
    Interest,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PersonalInfoLevel {
    High,
    Low,
    #[default]
    Medium,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PatchPath(pub Vec<PatchItem>);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PatchItem {
    Index(usize),
    Key(String),
    Property(Property),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PatchObject {
    path: PatchPath,
    op: PatchOperation,
    value: Value,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PatchOperation {
    Set,
    Update,
}
