use serde::{Deserialize, Serialize};

use crate::vcard::VCardProperty;

pub struct JSContact {
    // version
    created: Option<UTCDateTime>,
    kind: Option<Kind>,
    language: Option<String>,
    members: Members,
    prod_id: Option<String>,
    related_to: RelatedTo,
    uid: String,
    updated: Option<UTCDateTime>,

    // Name and Organization Properties
    name: Option<Name>,
    nicknames: Nicknames,
    organizations: Organizations,
    speak_to_as: Option<SpeakToAs>,
    titles: Titles,

    // Contact Properties
    emails: Emails,
    online_services: OnlineServices,
    phones: Phones,
    preferred_languages: PreferredLanguages,

    // Calendaring and Scheduling Properties
    calendars: Calendars,
    scheduling_addresses: SchedulingAddresses,

    // Address and Location Properties
    addresses: Addresses,

    // Resource Properties
    crypto_keys: CryptoKeys,
    directories: Directories,
    links: Links,
    media: Medias,

    // Multilingual Properties
    localizations: Localizations,

    // Additional Properties
    anniversaries: Anniversaries,
    keywords: Keywords,
    notes: Notes,
    personal_info: PersonalInfos,

    // VCard Properties
    vcard_props: Vec<VCardProp>,
}

pub struct VCardProp {
    name: String,
    params: VCardParams,
    value: AnyValue,
}

pub struct VCardParams(Vec<(String, Vec<String>)>);

pub enum AnyValue {
    Text(String),
    Uri(String),
    Date(PartialDate),
    Time(PartialTime),
    DateTime {
        date: PartialDate,
        time: PartialTime,
    },
    DateAndOrTime {
        date: PartialDate,
        time: PartialTime,
    },
    Timestamp(UTCDateTime),
    Boolean(bool),
    Float(f64),
    UTCOffset(i16),
    LanguageTag(String),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Kind {
    Individual,
    Group,
    Org,
    Location,
    Device,
    Application,
}

pub struct UTCDateTime(pub i64);

pub struct Members(Vec<String>);

pub struct RelatedTo(Vec<(String, Vec<Relation>)>);

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Relation {
    Acquaintance,
    Agent,
    Child,
    Colleague,
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

pub struct Name {
    components: Vec<NameComponent>,
    is_ordered: bool,
    default_separator: Option<String>,
    full: Option<String>,
    sort_as: Vec<(NameKind, String)>,
    phonetic_script: Option<String>,
    phonetic_system: Option<PhoneticSystem>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PhoneticSystem {
    Ipa,
    Jyut,
    Piny,
}

pub struct NameComponent {
    value: String,
    kind: NameKind,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum NameKind {
    Credential,
    Generation,
    Given,
    Given2,
    Separator,
    Surname,
    Surname2,
    Title,
}

pub struct Nickname {
    name: String,
    contexts: Vec<DefaultContext>,
    pref: Option<u32>,
}

pub struct Nicknames(Vec<(String, Nickname)>);

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DefaultContext {
    Private,
    Work,
}

pub struct Organizations(Vec<(String, Organization)>);

pub struct Organization {
    name: String,
    units: Vec<OrgUnit>,
    sort_as: Option<String>,
    contexts: Vec<DefaultContext>,
}

pub struct OrgUnit {
    name: String,
    sort_as: Option<String>,
}

pub struct SpeakToAs {
    pub grammatical_gender: Option<GrammaticalGender>,
    pub pronouns: Vec<(String, Pronouns)>,
}

pub struct Pronouns {
    pronouns: String,
    contexts: Vec<DefaultContext>,
    pref: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum GrammaticalGender {
    Animate,
    Common,
    Feminine,
    Inanimate,
    Masculine,
    Neuter,
}

pub struct Titles(Vec<(String, Title)>);

pub struct Title {
    name: String,
    kind: TitleKind,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TitleKind {
    Title,
    Role,
}

pub struct Emails(Vec<(String, EmailAddress)>);

pub struct EmailAddress {
    address: String,
    contexts: Vec<DefaultContext>,
    pref: Option<u32>,
    label: Option<String>,
    vcard_params: VCardParams,
}

pub struct OnlineServices(Vec<(String, OnlineService)>);

pub struct OnlineService {
    service: Option<String>,
    uri: Option<String>,
    user: Option<String>,
    contexts: Vec<DefaultContext>,
    pref: Option<u32>,
    label: Option<String>,
    vcard_name: Option<VCardProperty>,
}

pub struct Phones(Vec<(String, Phone)>);

pub struct Phone {
    number: String,
    features: Vec<PhoneFeature>,
    contexts: Vec<DefaultContext>,
    pref: Option<u32>,
    label: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PhoneFeature {
    Fax,
    MainNumber,
    Mobile,
    Pager,
    Text,
    Textphone,
    Video,
    Voice,
}

pub struct PreferredLanguages(Vec<(String, LanguagePref)>);

pub struct LanguagePref {
    language: String,
    contexts: Vec<DefaultContext>,
    pref: Option<u32>,
}

pub struct Calendars(Vec<(String, Calendar)>);

pub struct Calendar {
    kind: CalendarKind,
    uri: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CalendarKind {
    Calendar,
    FreeBusy,
}

pub struct SchedulingAddresses(Vec<(String, SchedulingAddress)>);

pub struct SchedulingAddress {
    uri: String,
    contexts: Vec<DefaultContext>,
    pref: Option<u32>,
    label: Option<String>,
}

pub struct Addresses(Vec<(String, Address)>);

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

pub struct AddressComponent {
    value: String,
    kind: AddressComponentKind,
    phonetic: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
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
    Name,
    Number,
    Postcode,
    PostOfficeBox,
    Region,
    Room,
    Separator,
    Subdistrict,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AddressContext {
    Billing,
    Delivery,
    Private,
    Work,
}

pub struct Resource<K, C> {
    kind: K,
    uri: ResourceData,
    media_type: Option<String>,
    contexts: Vec<C>,
    pref: Option<u32>,
    label: Option<String>,
}

pub enum ResourceData {
    Uri(String),
    Data(Vec<u8>),
}

pub struct CryptoKeys(Vec<(String, CryptoKey)>);

pub struct CryptoKey(pub Resource<Option<String>, DefaultContext>);

pub struct Directories(Vec<(String, Directory)>);

pub struct Directory {
    res: Resource<DirectoryKind, DefaultContext>,
    list_as: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DirectoryKind {
    Directory,
    Entry,
}

pub struct Links(Vec<(String, Link)>);

pub struct Link(pub Resource<Option<LinkKind>, DefaultContext>);

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LinkKind {
    Contact,
}

pub struct Medias(Vec<(String, Media)>);

pub struct Media(pub Resource<MediaKind, DefaultContext>);

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MediaKind {
    Logo,
    Photo,
    Sound,
}

pub struct Localizations(Vec<(String, PatchObject)>);

pub struct Anniversaries(Vec<(String, Anniversary)>);

pub struct Anniversary {
    kind: AnniversaryKind,
    date: Date,
    place: Option<Address>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AnniversaryKind {
    Birth,
    Death,
    Wedding,
}

pub enum Date {
    PartialDate(PartialDate),
    Timestamp(UTCDateTime),
}

pub struct PartialDate {
    year: Option<u32>,
    month: Option<u32>,
    day: Option<u32>,
    calendar_scale: Option<CalendarScale>,
}

pub struct PartialTime {
    hour: Option<u16>,
    minute: Option<u16>,
    second: Option<u16>,
    timezone: Option<u16>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CalendarScale {
    Gregorian,
    Chinese,
    IslamicCivil,
    Hebrew,
    Ethiopic,
}

pub struct Keywords(Vec<String>);

pub struct Notes(Vec<(String, Note)>);

pub struct Note {
    note: String,
    created: Option<UTCDateTime>,
    author: Option<Author>,
}

pub struct Author {
    name: Option<String>,
    uri: Option<String>,
}

pub struct PersonalInfos(Vec<(String, PersonalInfo)>);

pub struct PersonalInfo {
    kind: PersonalInfoKind,
    value: String,
    level: Option<PersonalInfoLevel>,
    list_as: Option<u32>,
    label: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PersonalInfoKind {
    Expertise,
    Hobby,
    Interest,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PersonalInfoLevel {
    High,
    Low,
    Medium,
}

pub enum PatchObject {
    Set(Vec<SetProperty>),
    Update(Vec<(String, AnyValue)>),
}

pub enum SetProperty {
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
}
