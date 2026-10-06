/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

use crate::{
    common::{
        IanaParse, LinkRelation,
        jsprop::text::{ConvertedKeys, PointerString},
        xorshift::XorShift,
    },
    jscalendar::{
        JSCAL_NAMESPACE, JSCalendarDateTime, JSCalendarLinkDisplay, JSCalendarParticipantRole,
        JSCalendarProperty, JSCalendarRelation, JSCalendarVirtualLocationFeature,
        ext::JSCalendarKeyExt, uuid5,
    },
};
use jmap_tools::{JsonPointer, JsonPointerItem, Key};
use std::{borrow::Cow, str::FromStr};

const PROPERTY_NAMES: &[&str] = &[
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

type Prop = JSCalendarProperty<String>;
type TestKey = Key<'static, Prop>;

const TEXTS: &[&str] = &[
    "",
    "*",
    "**",
    "*a",
    "0",
    "00",
    "07",
    "7",
    "12",
    "12a",
    "18446744073709551615",
    "18446744073709551616",
    "~",
    "~0",
    "~1",
    "~2",
    "a~",
    "~~0",
    "a~1b",
    "a~0b",
    "/",
    "//",
    "a/b",
    "/title",
    "title/",
    "é",
    "日本",
    "x-foo",
    "@type",
    "#ref",
    "#",
    "2025-03-05T09:00:00",
    "2025-03-05T09:00:00Z",
    "1970-01-01T00:00:00",
    "1970-01-01T00:00:00Z",
    "participants",
    "recurrenceOverrides",
    "calendarIds",
    "convertedProperties",
    "links",
    "title",
    "roles",
    "owner",
    "status",
    "icon",
    "badge",
    "audio",
    "first",
    "3fa85f64-5717-4562-b3fc-2c963f66afa6",
    "excluded",
    "duration",
    "calendarAddress",
    "participants/p1/roles",
    "recurrenceOverrides/2025-03-05T09:00:00",
    "recurrenceOverrides/2025-03-05T09:00:00/title",
    "calendarIds/abc",
    "a b",
    "\\",
];

const REL_NAMES: &[&str] = &[
    "about",
    "icon",
    "status",
    "describedby",
    "enclosure",
    "alternate",
    "next",
    "license",
    "api-catalog",
];

const PUNCTUATION: &[char] = &['/', '~', '0', '1', '9', '*', 'a', 'é', '-', ':', 'T', '#'];

fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let year = if month <= 2 { year - 1 } else { year };
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let yoe = year - era * 400;
    let doy = (153 * (if month > 2 { month - 3 } else { month + 9 }) + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146097 + doe - 719468
}

fn random_timestamp(rng: &mut XorShift) -> i64 {
    let colliding = |years: i64| days_from_civil(1970 + years, 3, 5) * 86400 + 32400;
    match rng.below(8) {
        0 => *rng.pick(&[
            0,
            -1,
            1,
            86399,
            86400,
            1_741_165_200,
            -62_135_596_800,
            -62_135_596_801,
            253_402_300_799,
            253_402_300_800,
            i64::MIN,
            i64::MIN + 1,
            i64::MAX,
        ]),
        1 => colliding(0),
        2 => colliding(65536),
        3 => colliding(-65536),
        4 => rng.next() as i64,
        5 => (rng.next() % 400_000_000_000) as i64 - 100_000_000_000,
        _ => 1_741_165_200 + (rng.below(4) as i64) * 86400,
    }
}

fn random_text(rng: &mut XorShift) -> String {
    let mut text = String::new();
    for _ in 0..rng.below(5) {
        match rng.below(4) {
            0 => text.push_str(rng.pick(PROPERTY_NAMES)),
            1 => text.push_str(rng.pick(TEXTS)),
            2 => text.push(*rng.pick(PUNCTUATION)),
            _ => text.push('/'),
        }
    }
    text
}

fn random_property(rng: &mut XorShift, depth: usize) -> Prop {
    match rng.below(14) {
        0..=4 => Prop::from_str(rng.pick(PROPERTY_NAMES)).expect("known property"),
        5 => Prop::DateTime(JSCalendarDateTime::new(
            random_timestamp(rng),
            rng.one_in(2),
        )),
        6 | 7 => Prop::Pointer(random_pointer(rng, depth)),
        8 => Prop::IdValue(random_text(rng)),
        9 => Prop::IdReference(random_text(rng)),
        10 => LinkRelation::parse(rng.pick(REL_NAMES).as_bytes())
            .map(Prop::LinkRelation)
            .unwrap_or(Prop::Title),
        11 => JSCalendarParticipantRole::from_str(rng.pick(&[
            "owner",
            "chair",
            "optional",
            "informational",
            "required",
        ]))
        .map(Prop::ParticipantRole)
        .unwrap_or(Prop::Roles),
        12 => JSCalendarLinkDisplay::from_str(rng.pick(&[
            "badge",
            "graphic",
            "fullsize",
            "thumbnail",
        ]))
        .map(Prop::LinkDisplay)
        .unwrap_or(Prop::Display),
        _ => match rng.below(2) {
            0 => JSCalendarVirtualLocationFeature::from_str(rng.pick(&[
                "audio",
                "chat",
                "feed",
                "moderator",
                "phone",
                "screen",
                "video",
            ]))
            .map(Prop::VirtualLocationFeature)
            .unwrap_or(Prop::Features),
            _ => JSCalendarRelation::from_str(
                rng.pick(&["first", "next", "child", "parent", "snooze"]),
            )
            .map(Prop::RelationValue)
            .unwrap_or(Prop::Relation),
        },
    }
}

fn random_pointer(rng: &mut XorShift, depth: usize) -> JsonPointer<Prop> {
    if depth > 2 || rng.one_in(2) {
        return JsonPointer::parse(&random_text(rng));
    }
    JsonPointer::new(
        (0..rng.below(4))
            .map(|_| match rng.below(7) {
                0 => JsonPointerItem::Root,
                1 => JsonPointerItem::Wildcard,
                2 => JsonPointerItem::Invalid(random_text(rng)),
                3 => {
                    let random = rng.next();
                    JsonPointerItem::Number(*rng.pick(&[0, 7, 12, u64::MAX, random]))
                }
                _ => JsonPointerItem::Key(random_key(rng, depth + 1)),
            })
            .collect(),
    )
}

fn random_key(rng: &mut XorShift, depth: usize) -> TestKey {
    match rng.below(6) {
        0 => Key::Borrowed(rng.pick(TEXTS)),
        1 => Key::Borrowed(rng.pick(PROPERTY_NAMES)),
        2 => Key::Owned(random_text(rng)),
        _ => Key::Property(random_property(rng, depth)),
    }
}

fn equivalent_key(rng: &mut XorShift, key: &TestKey) -> TestKey {
    match rng.below(5) {
        0 => Key::Owned(key.to_string().into_owned()),
        1 => Key::Property(Prop::Pointer(JsonPointer::new(vec![JsonPointerItem::Key(
            key.clone(),
        )]))),
        2 => Key::Property(Prop::Pointer(JsonPointer::parse(key.to_string().as_ref()))),
        3 => match Prop::from_str(key.to_string().as_ref()) {
            Ok(property) => Key::Property(property),
            Err(_) => key.clone(),
        },
        _ => key.clone(),
    }
}

#[test]
fn unit_property_names_are_distinct_static_names() {
    for name in PROPERTY_NAMES {
        let property = Prop::from_str(name).expect("known property");
        assert_eq!(property.to_string(), *name);
        assert_eq!(property.static_name(), Some(*name));
        assert!(!property.has_data());
        for other_name in PROPERTY_NAMES {
            let other = Prop::from_str(other_name).expect("known property");
            assert_eq!(property == other, name == other_name, "{name} {other_name}");
            assert_eq!(
                Key::Property(property.clone()).same_key(&Key::Property(other)),
                name == other_name
            );
        }
    }
    let mut rng = XorShift::new(0x7e57_0001);
    for _ in 0..20_000 {
        let property = random_property(&mut rng, 0);
        if let Some(name) = property.static_name() {
            assert_eq!(name, property.to_string(), "{property:?}");
        }
    }
}

#[test]
fn same_key_matches_key_eq() {
    let mut rng = XorShift::new(0x7e57_0002);
    let mut equal = 0usize;
    for _ in 0..60_000 {
        let a = random_key(&mut rng, 0);
        let b = if rng.one_in(2) {
            equivalent_key(&mut rng, &a)
        } else {
            random_key(&mut rng, 0)
        };
        let expected = a == b;
        equal += usize::from(expected);
        assert_eq!(a.same_key(&b), expected, "{a:?} {b:?}");
        assert_eq!(b.same_key(&a), expected, "{b:?} {a:?}");
    }
    assert!(equal > 10_000, "{equal}");

    let colliding = |years: i64| days_from_civil(1970 + years, 3, 5) * 86400 + 32400;
    for is_local in [false, true] {
        let a: TestKey = Key::Property(Prop::DateTime(JSCalendarDateTime::new(
            colliding(0),
            is_local,
        )));
        let b: TestKey = Key::Property(Prop::DateTime(JSCalendarDateTime::new(
            colliding(65536),
            is_local,
        )));
        assert_eq!(a.to_string(), b.to_string());
        assert!(a == b);
        assert!(a.same_key(&b));
    }
}

#[test]
fn encode_pointer_matches_json_pointer_encode() {
    let mut rng = XorShift::new(0x7e57_0003);
    for _ in 0..20_000 {
        let parts = (0..rng.below(5))
            .map(|_| random_text(&mut rng))
            .collect::<Vec<_>>();
        assert_eq!(
            String::from_pointer(parts.iter().map(String::as_str)),
            JsonPointer::<Prop>::encode(&parts),
            "{parts:?}"
        );
    }
}

#[test]
fn uuid5_matches_uuid_new_v5() {
    let reference = |data: &[u8]| {
        uuid::Uuid::new_v5(&JSCAL_NAMESPACE, data)
            .hyphenated()
            .to_string()
    };
    let mut rng = XorShift::new(0x7e57_0005);
    for len in (0..=300).chain([1023, 1024, 1025, 4096, 49152]) {
        let data = (0..len).map(|_| rng.next() as u8).collect::<Vec<_>>();
        assert_eq!(uuid5(&data), reference(&data), "{len}");
    }
    for _ in 0..2_000 {
        let text = random_text(&mut rng);
        let expected = reference(text.as_bytes());
        assert_eq!(uuid5(&text), expected, "{text:?}");
        let key: TestKey = Key::Owned(expected.clone());
        assert!(key.is_uuid5_of(text.as_bytes()));
        let other: TestKey = Key::Owned(reference(format!("{text}x").as_bytes()));
        assert_eq!(
            other.is_uuid5_of(text.as_bytes()),
            other.to_string() == expected
        );
    }
}

#[test]
fn converted_keys_split_like_json_pointer_parse() {
    for (text, expected) in [
        (
            "participants/p1/roles",
            r#"[Property(Participants), Owned("p1"), Property(Roles)]"#,
        ),
        (
            "/participants/p1",
            r#"[Property(Participants), Owned("p1")]"#,
        ),
        (
            "participants//p1",
            r#"[Property(Participants), Borrowed(""), Owned("p1")]"#,
        ),
        (
            "participants/p1/",
            r#"[Property(Participants), Owned("p1"), Borrowed("")]"#,
        ),
        (
            "recurrenceOverrides/2025-03-05T09:00:00/title",
            "[Property(RecurrenceOverrides), Property(DateTime(JSCalendarDateTime { timestamp: 1741165200, is_local: true })), Property(Title)]",
        ),
        (
            "links/k1~1x/href",
            r#"[Property(Links), Owned("k1"), Owned("x"), Property(Href)]"#,
        ),
        ("alerts/*/trigger", "[Property(Alerts), Property(Trigger)]"),
        (
            "007/12/18446744073709551616",
            r#"[Owned("007"), Owned("12"), Owned("18446744073709551616")]"#,
        ),
        ("", "[]"),
        ("/", r#"[Borrowed("")]"#),
        (
            "\u{e9}/\u{65e5}\u{672c}",
            "[Owned(\"\u{e9}\"), Owned(\"\u{65e5}\u{672c}\")]",
        ),
        (
            "calendarIds/abc",
            r#"[Property(CalendarIds), Property(IdValue("abc"))]"#,
        ),
        ("x~0y/1", r#"[Owned("x~y"), Owned("1")]"#),
        (
            "metadata/title/start",
            r#"[Property(Metadata), Owned("title"), Owned("start")]"#,
        ),
        (
            "privateMetadata/x.example/0/*/",
            r#"[Property(PrivateMetadata), Owned("x.example"), Owned("0"), Owned("*"), Owned("")]"#,
        ),
        (
            "*/metadata/title/start",
            r#"[Property(Metadata), Owned("title"), Property(Start)]"#,
        ),
    ] {
        for key in [
            TestKey::Owned(text.to_string()),
            TestKey::Borrowed(text),
            TestKey::Property(Prop::Pointer(JsonPointer::parse(text))),
        ] {
            assert_eq!(
                format!("{:?}", key.clone().into_converted_keys()),
                expected,
                "{key:?}"
            );
        }
    }
    assert_eq!(
        format!("{:?}", TestKey::Property(Prop::Title).into_converted_keys()),
        "[Property(Title)]"
    );
}

#[test]
fn link_relation_names_are_borrowed() {
    let source = include_str!("../common/types.rs");
    let names = source
        .lines()
        .filter_map(|line| {
            line.trim()
                .strip_prefix('"')?
                .split_once("\" => LinkRelation::")
                .map(|(name, _)| name)
        })
        .collect::<Vec<_>>();
    assert!(names.len() > 100, "{}", names.len());
    for name in names {
        let relation = LinkRelation::parse(name.as_bytes()).expect("known relation");
        let property = Prop::LinkRelation(relation);
        assert!(
            matches!(property.to_string(), Cow::Borrowed(text) if text == name),
            "{name}"
        );
        assert_eq!(property.static_name(), Some(name));
        let key: TestKey = Key::Property(property);
        assert!(key.same_key(&Key::Borrowed(name)));
        assert_eq!(key.to_string(), name);
    }
}
