/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

use calcard::{
    common::{IanaParse, LinkRelation},
    jscalendar::{
        JSCalendarDateTime, JSCalendarId, JSCalendarLinkDisplay, JSCalendarParticipantRole,
        JSCalendarProperty, JSCalendarRelation, JSCalendarVirtualLocationFeature,
    },
    jscontact::{Context, Feature, JSContactId, JSContactKind, JSContactProperty},
};
use jmap_tools::{JsonPointer, JsonPointerItem, Key, Property};
use std::str::FromStr;

const ROUNDS: usize = 20_000;
const FOUR_DIGIT_MIN: i64 = -62_167_219_200;
const FOUR_DIGIT_MAX: i64 = 253_402_300_799;
const GREGORIAN_CYCLE: i64 = 12_622_780_800;
const YEAR_WRAP: i64 = 4096 * GREGORIAN_CYCLE;

const LINK_DISPLAYS: &[&str] = &["badge", "graphic", "fullsize", "thumbnail"];
const FEATURES: &[&str] = &[
    "audio",
    "chat",
    "feed",
    "moderator",
    "phone",
    "screen",
    "video",
];
const ROLES: &[&str] = &["owner", "optional", "informational", "chair", "required"];
const RELATIONS: &[&str] = &["first", "next", "child", "parent", "snooze"];
const CONTEXTS: &[&str] = &["billing", "delivery", "private", "work"];
const CONTACT_FEATURES: &[&str] = &[
    "fax",
    "main-number",
    "mobile",
    "pager",
    "text",
    "textphone",
    "video",
    "voice",
];
const CONTACT_KINDS: &[&str] = &[
    "apartment",
    "block",
    "building",
    "country",
    "direction",
    "district",
    "floor",
    "landmark",
    "locality",
    "name",
    "number",
    "postcode",
    "postOfficeBox",
    "region",
    "room",
    "separator",
    "subdistrict",
    "birth",
    "death",
    "wedding",
    "calendar",
    "freeBusy",
    "application",
    "device",
    "group",
    "individual",
    "location",
    "org",
    "directory",
    "entry",
    "contact",
    "logo",
    "photo",
    "sound",
    "credential",
    "generation",
    "given",
    "given2",
    "surname",
    "surname2",
    "title",
    "expertise",
    "hobby",
    "interest",
    "role",
];
const POINTER_PIECES: &[&str] = &[
    "/",
    "/",
    "/",
    "~0",
    "~1",
    "~2",
    "~",
    "*",
    "0",
    "12",
    "007",
    "title",
    "participants",
    "calendarAddress",
    "2025-03-05T09:00:00",
    "a",
    "é",
    "x/y",
    "",
];
const FIXED_TIMESTAMPS: &[i64] = &[0, 1_744_018_200, 1_744_018_201, -1, 86_400, 951_782_400];

const CALENDAR_NAMES: &[&str] = &[
    "@type",
    "acknowledged",
    "action",
    "alerts",
    "baseEventId",
    "byDay",
    "byHour",
    "byMinute",
    "byMonth",
    "byMonthDay",
    "bySecond",
    "bySetPosition",
    "byWeekNo",
    "byYearDay",
    "calendarAddress",
    "calendarIds",
    "categories",
    "color",
    "contentType",
    "coordinates",
    "count",
    "created",
    "day",
    "delegatedFrom",
    "delegatedTo",
    "description",
    "descriptionContentType",
    "display",
    "due",
    "duration",
    "email",
    "entries",
    "estimatedDuration",
    "excluded",
    "expectReply",
    "features",
    "firstDayOfWeek",
    "freeBusyStatus",
    "frequency",
    "hideAttendees",
    "href",
    "id",
    "interval",
    "invitedBy",
    "isDraft",
    "isOrigin",
    "keywords",
    "kind",
    "links",
    "locale",
    "locations",
    "locationTypes",
    "mayInviteOthers",
    "mayInviteSelf",
    "memberOf",
    "method",
    "name",
    "nthOfPeriod",
    "offset",
    "participants",
    "participationComment",
    "participationStatus",
    "percentComplete",
    "priority",
    "privacy",
    "prodId",
    "progress",
    "recurrenceId",
    "recurrenceIdTimeZone",
    "recurrenceOverrides",
    "rel",
    "relatedTo",
    "relation",
    "relativeTo",
    "replyTo",
    "requestStatus",
    "roles",
    "rscale",
    "sentBy",
    "scheduleAgent",
    "scheduleForceSend",
    "scheduleSequence",
    "scheduleStatus",
    "scheduleUpdated",
    "sendTo",
    "sequence",
    "showWithoutTime",
    "size",
    "skip",
    "source",
    "start",
    "status",
    "timeZone",
    "title",
    "trigger",
    "uid",
    "until",
    "updated",
    "uri",
    "useDefaultAlerts",
    "utcEnd",
    "utcStart",
    "version",
    "virtualLocations",
    "when",
    "endTimeZone",
    "mainLocationId",
    "organizerCalendarAddress",
    "recurrenceRule",
    "properties",
    "components",
    "valueType",
    "convertedProperties",
    "parameters",
    "iCalendar",
    "blobId",
    "metadata",
    "privateMetadata",
];

const CONTACT_NAMES: &[&str] = &[
    "@type",
    "address",
    "addressBookIds",
    "addresses",
    "anniversaries",
    "author",
    "blobId",
    "calendars",
    "calendarScale",
    "components",
    "contexts",
    "convertedProperties",
    "coordinates",
    "countryCode",
    "created",
    "cryptoKeys",
    "date",
    "day",
    "defaultSeparator",
    "directories",
    "emails",
    "extra",
    "features",
    "full",
    "grammaticalGender",
    "id",
    "isOrdered",
    "keywords",
    "kind",
    "label",
    "language",
    "level",
    "links",
    "listAs",
    "localizations",
    "media",
    "mediaType",
    "members",
    "month",
    "name",
    "nicknames",
    "note",
    "notes",
    "number",
    "onlineServices",
    "organizationId",
    "organizations",
    "parameters",
    "personalInfo",
    "phones",
    "phonetic",
    "phoneticScript",
    "phoneticSystem",
    "place",
    "pref",
    "preferredLanguages",
    "prodId",
    "pronouns",
    "properties",
    "relatedTo",
    "relation",
    "schedulingAddresses",
    "service",
    "sortAs",
    "speakToAs",
    "timeZone",
    "titles",
    "uid",
    "units",
    "updated",
    "uri",
    "user",
    "utc",
    "value",
    "vCard",
    "version",
    "year",
    "metadata",
    "privateMetadata",
];

const LINK_RELATIONS: &[&str] = &[
    "about",
    "acl",
    "alternate",
    "amphtml",
    "api-catalog",
    "appendix",
    "apple-touch-icon",
    "apple-touch-startup-image",
    "archives",
    "author",
    "blocked-by",
    "bookmark",
    "c2pa-manifest",
    "canonical",
    "chapter",
    "cite-as",
    "collection",
    "compression-dictionary",
    "contents",
    "convertedfrom",
    "copyright",
    "create-form",
    "current",
    "deprecation",
    "describedby",
    "describes",
    "disclosure",
    "dns-prefetch",
    "duplicate",
    "edit",
    "edit-form",
    "edit-media",
    "enclosure",
    "external",
    "first",
    "geofeed",
    "glossary",
    "help",
    "hosts",
    "hub",
    "ice-server",
    "icon",
    "index",
    "intervalafter",
    "intervalbefore",
    "intervalcontains",
    "intervaldisjoint",
    "intervalduring",
    "intervalequals",
    "intervalfinishedby",
    "intervalfinishes",
    "intervalin",
    "intervalmeets",
    "intervalmetby",
    "intervaloverlappedby",
    "intervaloverlaps",
    "intervalstartedby",
    "intervalstarts",
    "item",
    "last",
    "latest-version",
    "license",
    "linkset",
    "lrdd",
    "manifest",
    "mask-icon",
    "me",
    "media-feed",
    "memento",
    "micropub",
    "modulepreload",
    "monitor",
    "monitor-group",
    "next",
    "next-archive",
    "nofollow",
    "noopener",
    "noreferrer",
    "opener",
    "openid2.local_id",
    "openid2.provider",
    "original",
    "p3pv1",
    "payment",
    "pingback",
    "preconnect",
    "predecessor-version",
    "prefetch",
    "preload",
    "prerender",
    "prev",
    "preview",
    "previous",
    "prev-archive",
    "privacy-policy",
    "profile",
    "publication",
    "rdap-active",
    "rdap-bottom",
    "rdap-down",
    "rdap-top",
    "rdap-up",
    "related",
    "restconf",
    "replies",
    "ruleinput",
    "search",
    "section",
    "self",
    "service",
    "service-desc",
    "service-doc",
    "service-meta",
    "sip-trunking-capability",
    "sponsored",
    "start",
    "status",
    "stylesheet",
    "subsection",
    "successor-version",
    "sunset",
    "tag",
    "terms-of-service",
    "timegate",
    "timemap",
    "type",
    "ugc",
    "up",
    "version-history",
    "via",
    "webmention",
    "working-copy",
    "working-copy-of",
];

struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_f491_4f6c_dd1d)
    }

    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }

    fn chance(&mut self, one_in: u64) -> bool {
        self.next().is_multiple_of(one_in)
    }

    fn pick<'x>(&mut self, items: &[&'x str]) -> &'x str {
        items[self.below(items.len())]
    }
}

fn random_timestamp(rng: &mut Rng) -> i64 {
    let offset = rng.below(3) as i64 - 1;
    match rng.below(12) {
        0 => FOUR_DIGIT_MIN + offset,
        1 => FOUR_DIGIT_MAX + offset,
        2 => {
            FIXED_TIMESTAMPS[rng.below(FIXED_TIMESTAMPS.len())]
                + YEAR_WRAP * (rng.below(5) as i64 - 2)
        }
        3 => rng.next() as i64,
        4 => (rng.next() as i64) % (200 * GREGORIAN_CYCLE),
        5 | 6 => FIXED_TIMESTAMPS[rng.below(FIXED_TIMESTAMPS.len())],
        _ => FOUR_DIGIT_MIN + (rng.next() % (FOUR_DIGIT_MAX - FOUR_DIGIT_MIN) as u64) as i64,
    }
}

fn random_text(rng: &mut Rng) -> String {
    match rng.below(8) {
        0 => rng.pick(CALENDAR_NAMES).to_string(),
        1 => rng.pick(CONTACT_NAMES).to_string(),
        2 => rng.pick(LINK_RELATIONS).to_string(),
        3 => JSCalendarDateTime::new(random_timestamp(rng), rng.chance(2)).to_rfc3339(),
        4 => format!("#{}", rng.pick(CALENDAR_NAMES)),
        5 => rng.next().to_string(),
        6 => random_pointer_text(rng),
        _ => ["", "é", "~", "/", "#", "Z", "1f2e3d4c"][rng.below(7)].to_string(),
    }
}

fn random_pointer_text(rng: &mut Rng) -> String {
    (0..rng.below(8))
        .map(|_| rng.pick(POINTER_PIECES))
        .collect()
}

fn random_items<P: Property>(
    rng: &mut Rng,
    mut property: impl FnMut(&mut Rng) -> P,
) -> JsonPointer<P> {
    let len = if rng.chance(2) { 1 } else { rng.below(4) };
    JsonPointer::new(
        (0..len)
            .map(|_| match rng.below(8) {
                0 => JsonPointerItem::Root,
                1 => JsonPointerItem::Wildcard,
                2 => JsonPointerItem::Invalid(
                    ["a~2b", "x", "", "*", "7", "title", "é"][rng.below(7)].to_string(),
                ),
                3 => JsonPointerItem::Number(rng.next() % 20),
                4 | 5 => JsonPointerItem::Key(Key::Property(property(rng))),
                6 => JsonPointerItem::Key(Key::Borrowed(
                    rng.pick(&["", "*", "7", "007", "title", "a/b", "x~y", "start", "é"]),
                )),
                _ => JsonPointerItem::Key(Key::Owned(random_text(rng))),
            })
            .collect(),
    )
}

fn equivalent_items<P: Property>(
    rng: &mut Rng,
    pointer: &JsonPointer<P>,
    parse: impl Fn(&str) -> Option<P>,
) -> JsonPointer<P> {
    JsonPointer::new(
        pointer
            .as_slice()
            .iter()
            .map(|item| {
                let text = match item {
                    JsonPointerItem::Root => String::new(),
                    JsonPointerItem::Wildcard => "*".to_string(),
                    JsonPointerItem::Invalid(text) => text.clone(),
                    JsonPointerItem::Number(number) => number.to_string(),
                    JsonPointerItem::Key(key) => key.to_string().into_owned(),
                };
                match rng.below(6) {
                    0 => text
                        .parse()
                        .ok()
                        .filter(|number: &u64| number.to_string() == text)
                        .map_or(JsonPointerItem::Key(Key::Owned(text.clone())), |number| {
                            JsonPointerItem::Number(number)
                        }),
                    1 => JsonPointerItem::Invalid(text),
                    2 => parse(&text).map_or(JsonPointerItem::Key(Key::Owned(text)), |property| {
                        JsonPointerItem::Key(Key::Property(property))
                    }),
                    3 if text.is_empty() => JsonPointerItem::Root,
                    4 if text == "*" => JsonPointerItem::Wildcard,
                    _ => JsonPointerItem::Key(Key::Owned(text)),
                }
            })
            .collect(),
    )
}

fn random_id<I: FromStr>(rng: &mut Rng) -> I {
    let text = if rng.chance(2) {
        random_text(rng)
    } else {
        (rng.next() % 1000).to_string()
    };
    text.parse()
        .or_else(|_| "7".parse())
        .unwrap_or_else(|_| panic!("id parses"))
}

fn calendar_property<I: JSCalendarId>(rng: &mut Rng) -> JSCalendarProperty<I> {
    match rng.below(16) {
        0..=5 => JSCalendarProperty::from_str(rng.pick(CALENDAR_NAMES)).unwrap(),
        6 => JSCalendarProperty::LinkDisplay(
            JSCalendarLinkDisplay::from_str(rng.pick(LINK_DISPLAYS)).unwrap(),
        ),
        7 => JSCalendarProperty::VirtualLocationFeature(
            JSCalendarVirtualLocationFeature::from_str(rng.pick(FEATURES)).unwrap(),
        ),
        8 => JSCalendarProperty::ParticipantRole(
            JSCalendarParticipantRole::from_str(rng.pick(ROLES)).unwrap(),
        ),
        9 => JSCalendarProperty::RelationValue(
            JSCalendarRelation::from_str(rng.pick(RELATIONS)).unwrap(),
        ),
        10 => JSCalendarProperty::LinkRelation(
            LinkRelation::parse(rng.pick(LINK_RELATIONS).as_bytes()).unwrap(),
        ),
        11 | 12 => JSCalendarProperty::DateTime(JSCalendarDateTime::new(
            random_timestamp(rng),
            rng.chance(2),
        )),
        13 if rng.chance(2) => JSCalendarProperty::Pointer(random_items(rng, |rng| {
            if rng.chance(8) {
                JSCalendarProperty::Pointer(JsonPointer::parse(&random_pointer_text(rng)))
            } else {
                JSCalendarProperty::from_str(rng.pick(CALENDAR_NAMES)).unwrap()
            }
        })),
        13 => JSCalendarProperty::Pointer(JsonPointer::parse(&random_pointer_text(rng))),
        14 => JSCalendarProperty::IdValue(random_id(rng)),
        _ => JSCalendarProperty::IdReference(random_text(rng)),
    }
}

fn related_calendar_property<I: JSCalendarId>(
    rng: &mut Rng,
    property: &JSCalendarProperty<I>,
) -> JSCalendarProperty<I> {
    let text = property.to_cow();
    match (rng.below(6), property) {
        (0, JSCalendarProperty::DateTime(dt)) => {
            JSCalendarProperty::DateTime(JSCalendarDateTime::new(
                dt.timestamp + YEAR_WRAP * (rng.below(3) as i64 - 1),
                dt.is_local,
            ))
        }
        (1, JSCalendarProperty::DateTime(dt)) => {
            JSCalendarProperty::DateTime(JSCalendarDateTime::new(
                dt.timestamp + rng.below(3) as i64 - 1,
                !dt.is_local || rng.chance(2),
            ))
        }
        (2 | 5, JSCalendarProperty::Pointer(pointer)) => {
            JSCalendarProperty::Pointer(equivalent_items(rng, pointer, |text| {
                JSCalendarProperty::from_str(text).ok()
            }))
        }
        (2, _) => JSCalendarProperty::from_str(&text)
            .unwrap_or_else(|_| JSCalendarProperty::IdReference(text.into_owned())),
        (3, _) => text
            .parse()
            .map(JSCalendarProperty::IdValue)
            .unwrap_or_else(|_| JSCalendarProperty::Pointer(JsonPointer::parse(&text))),
        (4, _) => LinkRelation::parse(text.as_bytes())
            .map(JSCalendarProperty::LinkRelation)
            .unwrap_or_else(|| JSCalendarProperty::Pointer(JsonPointer::parse(&text))),
        _ => calendar_property(rng),
    }
}

fn contact_property<I: JSContactId>(rng: &mut Rng) -> JSContactProperty<I> {
    match rng.below(12) {
        0..=5 => JSContactProperty::from_str(rng.pick(CONTACT_NAMES)).unwrap(),
        6 => JSContactProperty::Context(Context::from_str(rng.pick(CONTEXTS)).unwrap()),
        7 => JSContactProperty::Feature(Feature::from_str(rng.pick(CONTACT_FEATURES)).unwrap()),
        8 => {
            JSContactProperty::SortAsKind(JSContactKind::from_str(rng.pick(CONTACT_KINDS)).unwrap())
        }
        9 if rng.chance(2) => JSContactProperty::Pointer(random_items(rng, |rng| {
            if rng.chance(8) {
                JSContactProperty::Pointer(JsonPointer::parse(&random_pointer_text(rng)))
            } else {
                JSContactProperty::from_str(rng.pick(CONTACT_NAMES)).unwrap()
            }
        })),
        9 => JSContactProperty::Pointer(JsonPointer::parse(&random_pointer_text(rng))),
        10 => JSContactProperty::IdValue(random_id(rng)),
        _ => JSContactProperty::IdReference(random_text(rng)),
    }
}

fn related_contact_property<I: JSContactId>(
    rng: &mut Rng,
    property: &JSContactProperty<I>,
) -> JSContactProperty<I> {
    let text = property.to_cow();
    if let JSContactProperty::Pointer(pointer) = property
        && rng.chance(2)
    {
        return JSContactProperty::Pointer(equivalent_items(rng, pointer, |text| {
            JSContactProperty::from_str(text).ok()
        }));
    }
    match rng.below(5) {
        0 => JSContactProperty::from_str(&text)
            .unwrap_or_else(|_| JSContactProperty::IdReference(text.into_owned())),
        1 => text
            .parse()
            .map(JSContactProperty::IdValue)
            .unwrap_or_else(|_| JSContactProperty::Pointer(JsonPointer::parse(&text))),
        2 => JSContactKind::from_str(&text)
            .map(JSContactProperty::SortAsKind)
            .unwrap_or_else(|_| JSContactProperty::Pointer(JsonPointer::parse(&text))),
        _ => contact_property(rng),
    }
}

fn related_text<P: Property>(rng: &mut Rng, property: &P) -> String {
    let mut text = property.to_cow().into_owned();
    match rng.below(6) {
        0 | 1 => text,
        2 => {
            text.pop();
            text
        }
        3 => {
            text.push('Z');
            text
        }
        4 => format!("#{text}"),
        _ => random_text(rng),
    }
}

fn check_pair<P: Property>(a: &P, b: &P, text: &str, stats: &mut [usize; 2]) {
    let expected = a.to_cow() == b.to_cow();
    assert_eq!(a.key_eq(b), expected, "{a:?} {b:?}");
    assert_eq!(b.key_eq(a), expected, "{b:?} {a:?}");
    assert_eq!(
        Key::Property(a.clone()) == Key::Property(b.clone()),
        expected,
        "{a:?} {b:?}"
    );
    let expected_str = a.to_cow() == text;
    assert_eq!(a.key_eq_str(text), expected_str, "{a:?} {text:?}");
    assert_eq!(
        Key::Property(a.clone()) == Key::Borrowed(text),
        expected_str,
        "{a:?} {text:?}"
    );
    stats[0] += usize::from(expected);
    stats[1] += usize::from(expected_str);
}

fn check_calendar<I: JSCalendarId>(seed: u64) {
    let mut rng = Rng(seed);
    let mut stats = [0; 2];
    for _ in 0..ROUNDS {
        let a = calendar_property::<I>(&mut rng);
        let b = if rng.chance(2) {
            related_calendar_property(&mut rng, &a)
        } else {
            calendar_property(&mut rng)
        };
        let text = related_text(&mut rng, &b);
        check_pair(&a, &b, &text, &mut stats);
    }
    assert!(stats.iter().all(|&hits| hits > ROUNDS / 20), "{stats:?}");
}

fn check_contact<I: JSContactId>(seed: u64) {
    let mut rng = Rng(seed);
    let mut stats = [0; 2];
    for _ in 0..ROUNDS {
        let a = contact_property::<I>(&mut rng);
        let b = if rng.chance(2) {
            related_contact_property(&mut rng, &a)
        } else {
            contact_property(&mut rng)
        };
        let text = related_text(&mut rng, &b);
        check_pair(&a, &b, &text, &mut stats);
    }
    assert!(stats.iter().all(|&hits| hits > ROUNDS / 20), "{stats:?}");
}

#[test]
fn calendar_property_keys_compare_by_text() {
    check_calendar::<String>(0x6361_6c5f_7374_7269);
    check_calendar::<u64>(0x6361_6c5f_7536_3421);
}

#[test]
fn contact_property_keys_compare_by_text() {
    check_contact::<String>(0x636f_6e5f_7374_7269);
    check_contact::<u64>(0x636f_6e5f_7536_3421);
}

#[test]
fn unit_property_names_are_distinct() {
    for name in CALENDAR_NAMES {
        let property = JSCalendarProperty::<String>::from_str(name).unwrap();
        assert_eq!(property.to_cow(), *name);
    }
    for name in CONTACT_NAMES {
        let property = JSContactProperty::<String>::from_str(name).unwrap();
        assert_eq!(property.to_cow(), *name);
    }
}

#[test]
fn wrapped_years_compare_by_text() {
    for timestamp in FIXED_TIMESTAMPS {
        for is_local in [true, false] {
            let a = JSCalendarProperty::<String>::DateTime(JSCalendarDateTime::new(
                *timestamp, is_local,
            ));
            let b = JSCalendarProperty::<String>::DateTime(JSCalendarDateTime::new(
                timestamp + YEAR_WRAP,
                is_local,
            ));
            assert_eq!(a.to_cow(), b.to_cow());
            assert!(a.key_eq(&b));
            assert!(b.key_eq_str(&a.to_cow()));
        }
    }
}

fn check_pointer_pairs<P: Property>(
    seed: u64,
    wrap: impl Fn(JsonPointer<P>) -> P,
    property: impl Fn(&mut Rng) -> P,
    parse: impl Fn(&str) -> Option<P>,
) {
    let mut rng = Rng(seed);
    let mut stats = [0usize; 4];
    for _ in 0..ROUNDS {
        let a = random_items(&mut rng, &property);
        let b = match rng.below(3) {
            0 => a.clone(),
            1 => equivalent_items(&mut rng, &a, &parse),
            _ => random_items(&mut rng, &property),
        };
        let text = if rng.chance(2) {
            a.to_string()
        } else {
            b.to_string()
        };
        let expected = a.to_string() == b.to_string();
        stats[usize::from(a == b) * 2 + usize::from(expected)] += 1;
        let (a, b) = (wrap(a), wrap(b));
        assert_eq!(a.key_eq(&b), expected, "{a:?} {b:?}");
        assert_eq!(b.key_eq(&a), expected, "{b:?} {a:?}");
        assert_eq!(a.key_eq_str(&text), a.to_cow() == text, "{a:?} {text:?}");
        assert_eq!(b.key_eq_str(&text), b.to_cow() == text, "{b:?} {text:?}");
    }
    assert_eq!(
        stats[2], 0,
        "structurally equal pointers with different text"
    );
    assert!(
        [stats[0], stats[1], stats[3]]
            .iter()
            .all(|&hits| hits > ROUNDS / 50),
        "{stats:?}"
    );
}

#[test]
fn pointer_keys_compare_by_text() {
    check_pointer_pairs::<JSCalendarProperty<String>>(
        0x7074_725f_6361_6c21,
        JSCalendarProperty::Pointer,
        |rng| JSCalendarProperty::from_str(rng.pick(CALENDAR_NAMES)).unwrap(),
        |text| JSCalendarProperty::from_str(text).ok(),
    );
    check_pointer_pairs::<JSContactProperty<String>>(
        0x7074_725f_636f_6e21,
        JSContactProperty::Pointer,
        |rng| JSContactProperty::from_str(rng.pick(CONTACT_NAMES)).unwrap(),
        |text| JSContactProperty::from_str(text).ok(),
    );
}
