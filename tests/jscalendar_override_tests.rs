/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

use calcard::{
    common::timezone::Tz,
    icalendar::{ICalendar, ICalendarComponent, ICalendarParameterName, ICalendarProperty},
    jscalendar::{
        JSCALENDAR_VERSION, JSCalendar, JSCalendarDateTime, JSCalendarParticipantRole,
        JSCalendarProperty, JSCalendarValue,
        export::ExportOptions,
        ext::JSCalendarPatch,
        import::ImportOptions,
        overrides::{Inherited, OverrideDiff},
    },
};
use jmap_tools::{JsonPointer, JsonPointerHandler, Key, Value};
use serde_json::Value as JsonValue;
use std::{
    str::FromStr,
    time::{Duration, Instant},
};

fn normalize(jscal: &JSCalendar<'_, String, String>) -> JsonValue {
    let mut value: JsonValue = serde_json::from_str(&jscal.to_string_pretty()).unwrap();
    if let Some(obj) = value.as_object_mut() {
        obj.remove("iCalendar");
    }
    if let Some(entries) = value["entries"].as_array_mut() {
        for entry in entries.iter_mut().filter_map(JsonValue::as_object_mut) {
            entry.remove("iCalendar");
        }
    }
    value
}

fn import(ical: &str) -> JSCalendar<'static, String, String> {
    ICalendar::parse(ical).unwrap().into_jscalendar()
}

fn export(json: &str) -> ICalendar {
    let exported = JSCalendar::<String, String>::parse(json)
        .unwrap()
        .into_icalendar()
        .unwrap();
    ICalendar::parse(exported.to_string()).unwrap()
}

fn with_implied_version(mut value: JsonValue) -> JsonValue {
    if let Some(group) = value.as_object_mut() {
        group
            .entry("version")
            .or_insert_with(|| JSCALENDAR_VERSION.into());
    }
    value
}

fn assert_json_roundtrip(json: &str) -> ICalendar {
    let ical = export(json);
    let reimported = ical.clone().into_jscalendar::<String, String>();
    assert_eq!(
        normalize(&reimported),
        with_implied_version(normalize(&JSCalendar::parse(json).unwrap())),
        "{ical}"
    );
    ical
}

fn assert_ical_roundtrip(ical: &str) -> JSCalendar<'static, String, String> {
    let jscal = import(ical);
    let exported = jscal.clone().into_icalendar().unwrap();
    let reimported = ICalendar::parse(exported.to_string())
        .unwrap()
        .into_jscalendar::<String, String>();
    assert_eq!(normalize(&reimported), normalize(&jscal), "{exported}");
    jscal
}

fn overrides(jscal: &JSCalendar<'_, String, String>) -> JsonValue {
    normalize(jscal)["entries"][0]["recurrenceOverrides"].clone()
}

fn override_components(ical: &ICalendar) -> Vec<&ICalendarComponent> {
    ical.components
        .iter()
        .filter(|component| {
            component
                .property(&ICalendarProperty::RecurrenceId)
                .is_some()
        })
        .collect()
}

fn main_component(ical: &ICalendar) -> &ICalendarComponent {
    ical.components
        .iter()
        .find(|component| component.property(&ICalendarProperty::Rrule).is_some())
        .unwrap()
}

const PARTICIPANTS: &str = r#"
    "organizerCalendarAddress": "mailto:zoe@example.com",
    "participants": {
      "zoe": {"@type": "Participant", "name": "Zoe", "calendarAddress": "mailto:zoe@example.com", "roles": {"owner": true, "attendee": true}},
      "12": {"@type": "Participant", "name": "Tom", "calendarAddress": "mailto:tom@example.com", "roles": {"attendee": true, "optional": true}},
      "a/b": {"@type": "Participant", "name": "Ann", "calendarAddress": "mailto:ann@example.com"},
      "c~d": {"@type": "Participant", "name": "Carl", "calendarAddress": "mailto:carl@example.com"}
    },"#;

fn event(extra: &str, overrides: &str) -> String {
    format!(
        r#"{{"@type": "Group", "entries": [{{
            "@type": "Event", "uid": "override-test", "title": "Sync",
            "start": "2025-01-06T09:00:00", "timeZone": "Europe/Berlin", "duration": "PT1H",
            "recurrenceRule": {{"frequency": "daily"}},
            {extra}
            "recurrenceOverrides": {overrides}
        }}]}}"#
    )
}

#[test]
fn time_zone_mismatched_override() {
    let jscal = assert_ical_roundtrip(concat!(
        "BEGIN:VCALENDAR\r\n",
        "BEGIN:VEVENT\r\nUID:tz-1\r\nDTSTART;TZID=Europe/Berlin:20250106T090000\r\n",
        "DURATION:PT1H\r\nRRULE:FREQ=DAILY\r\nSUMMARY:Daily\r\nEND:VEVENT\r\n",
        "BEGIN:VEVENT\r\nUID:tz-1\r\nRECURRENCE-ID:20250108T080000Z\r\n",
        "DTSTART:20250108T090000Z\r\nDURATION:PT1H\r\nSUMMARY:Daily\r\nEND:VEVENT\r\n",
        "BEGIN:VEVENT\r\nUID:tz-1\r\nRECURRENCE-ID;TZID=America/New_York:20250109T030000\r\n",
        "DTSTART;TZID=America/New_York:20250109T040000\r\nDURATION:PT1H\r\n",
        "SUMMARY:Daily\r\nEND:VEVENT\r\n",
        "END:VCALENDAR\r\n"
    ));
    assert_eq!(
        overrides(&jscal),
        serde_json::json!({
            "2025-01-08T09:00:00": {"timeZone": "Etc/UTC"},
            "2025-01-09T09:00:00": {"timeZone": "America/New_York", "start": "2025-01-09T04:00:00"}
        })
    );

    let ical = jscal.into_icalendar().unwrap().to_string();
    assert!(ical.contains("DTSTART:20250108T090000Z"), "{ical}");
    assert!(
        ical.contains("RECURRENCE-ID;TZID=Europe/Berlin:20250108T090000"),
        "{ical}"
    );
    assert!(
        ical.contains("DTSTART;TZID=America/New_York:20250109T040000"),
        "{ical}"
    );
}

#[test]
fn date_recurrence_id_roundtrip() {
    let ical = concat!(
        "BEGIN:VCALENDAR\r\n",
        "BEGIN:VEVENT\r\nUID:date-1\r\nDTSTART;VALUE=DATE:20250106\r\n",
        "RRULE:FREQ=DAILY;COUNT=5\r\nSUMMARY:All day\r\nEND:VEVENT\r\n",
        "BEGIN:VEVENT\r\nUID:date-1\r\nRECURRENCE-ID;VALUE=DATE:20250108\r\n",
        "DTSTART;VALUE=DATE:20250108\r\nSUMMARY:All day (changed)\r\nEND:VEVENT\r\n",
        "END:VCALENDAR\r\n"
    );
    let jscal = assert_ical_roundtrip(ical);
    let normalized = normalize(&jscal);
    assert_eq!(normalized["entries"].as_array().map(Vec::len), Some(1));
    assert_eq!(
        overrides(&jscal),
        serde_json::json!({"2025-01-08T00:00:00": {"title": "All day (changed)"}})
    );

    let exported = jscal.into_icalendar().unwrap().to_string();
    assert!(
        exported.contains("RECURRENCE-ID;VALUE=DATE:20250108"),
        "{exported}"
    );
    assert!(
        exported.contains("DTSTART;VALUE=DATE:20250108"),
        "{exported}"
    );
}

#[test]
fn due_only_task_override() {
    let jscal = assert_ical_roundtrip(concat!(
        "BEGIN:VCALENDAR\r\n",
        "BEGIN:VTODO\r\nUID:todo-1\r\nDUE;TZID=Europe/Berlin:20250106T170000\r\n",
        "RRULE:FREQ=WEEKLY\r\nSUMMARY:Report\r\nEND:VTODO\r\n",
        "BEGIN:VTODO\r\nUID:todo-1\r\nRECURRENCE-ID;TZID=Europe/Berlin:20250113T170000\r\n",
        "DUE;TZID=Europe/Berlin:20250114T170000\r\nSUMMARY:Report\r\nEND:VTODO\r\n",
        "END:VCALENDAR\r\n"
    ));
    assert_eq!(
        overrides(&jscal),
        serde_json::json!({"2025-01-13T17:00:00": {"due": "2025-01-14T17:00:00"}})
    );
}

#[test]
fn this_and_future_is_preserved() {
    let jscal = assert_ical_roundtrip(concat!(
        "BEGIN:VCALENDAR\r\n",
        "BEGIN:VEVENT\r\nUID:range-1\r\nDTSTART;TZID=Europe/Berlin:20250106T090000\r\n",
        "DURATION:PT1H\r\nRRULE:FREQ=DAILY\r\nSUMMARY:Daily\r\nEND:VEVENT\r\n",
        "BEGIN:VEVENT\r\nUID:range-1\r\n",
        "RECURRENCE-ID;RANGE=THISANDFUTURE;TZID=Europe/Berlin:20250110T090000\r\n",
        "DTSTART;TZID=Europe/Berlin:20250110T110000\r\nDURATION:PT1H\r\n",
        "SUMMARY:Daily\r\nEND:VEVENT\r\n",
        "END:VCALENDAR\r\n"
    ));
    assert_eq!(
        overrides(&jscal)["2025-01-10T09:00:00"]["iCalendar"]["convertedProperties"]["recurrenceId"]
            ["parameters"]["range"],
        "THISANDFUTURE"
    );

    let exported = jscal.into_icalendar().unwrap().to_string();
    assert!(
        exported.contains("RECURRENCE-ID;RANGE=THISANDFUTURE;TZID=Europe/Berlin:20250110T090000"),
        "{exported}"
    );
    assert!(!exported.contains("RANGE=TRUE"), "{exported}");
}

#[test]
fn non_instance_override_key_adds_rdate() {
    let ical = assert_json_roundtrip(&event(
        "",
        r#"{"2025-01-08T09:00:00": {"title": "Rule instance"},
            "2025-01-08T20:00:00": {"title": "Extra instance"}}"#,
    ));
    let rdates = main_component(&ical)
        .properties(&ICalendarProperty::Rdate)
        .collect::<Vec<_>>();
    assert_eq!(rdates.len(), 1, "{ical}");
    assert_eq!(override_components(&ical).len(), 2, "{ical}");
    assert!(
        ical.to_string()
            .contains("RDATE;TZID=Europe/Berlin:20250108T200000")
    );
}

#[test]
fn duplicate_recurrence_ids_keep_the_last_component() {
    let jscal = import(concat!(
        "BEGIN:VCALENDAR\r\n",
        "BEGIN:VEVENT\r\nUID:dup-1\r\nDTSTART;TZID=Europe/Berlin:20250106T090000\r\n",
        "DURATION:PT1H\r\nRRULE:FREQ=DAILY\r\nSUMMARY:Daily\r\nEND:VEVENT\r\n",
        "BEGIN:VEVENT\r\nUID:dup-1\r\nRECURRENCE-ID;TZID=Europe/Berlin:20250108T090000\r\n",
        "DTSTART;TZID=Europe/Berlin:20250108T100000\r\nDURATION:PT1H\r\n",
        "SUMMARY:First\r\nEND:VEVENT\r\n",
        "BEGIN:VEVENT\r\nUID:dup-1\r\nRECURRENCE-ID;TZID=Europe/Berlin:20250108T090000\r\n",
        "DTSTART;TZID=Europe/Berlin:20250108T120000\r\nDURATION:PT1H\r\n",
        "SUMMARY:Second\r\nEND:VEVENT\r\n",
        "END:VCALENDAR\r\n"
    ));
    assert_eq!(
        overrides(&jscal),
        serde_json::json!({"2025-01-08T09:00:00": {"title": "Second", "start": "2025-01-08T12:00:00"}})
    );
    assert_eq!(
        override_components(&jscal.into_icalendar().unwrap()).len(),
        1
    );

    let jscal = import(concat!(
        "BEGIN:VCALENDAR\r\n",
        "BEGIN:VEVENT\r\nUID:dup-2\r\nDTSTART;TZID=Europe/Berlin:20250106T090000\r\n",
        "DURATION:PT1H\r\nRRULE:FREQ=DAILY\r\nSUMMARY:Daily\r\nEND:VEVENT\r\n",
        "BEGIN:VEVENT\r\nUID:dup-2\r\nRECURRENCE-ID;TZID=Europe/Berlin:20250108T090000\r\n",
        "SEQUENCE:3\r\nDTSTART;TZID=Europe/Berlin:20250108T140000\r\nDURATION:PT1H\r\n",
        "SUMMARY:Newer\r\nEND:VEVENT\r\n",
        "BEGIN:VEVENT\r\nUID:dup-2\r\nRECURRENCE-ID;TZID=Europe/Berlin:20250108T090000\r\n",
        "SEQUENCE:1\r\nDTSTART;TZID=Europe/Berlin:20250108T110000\r\nDURATION:PT1H\r\n",
        "SUMMARY:Older\r\nEND:VEVENT\r\n",
        "END:VCALENDAR\r\n"
    ));
    assert_eq!(
        overrides(&jscal)["2025-01-08T09:00:00"]["title"],
        "Newer",
        "RFC 5545 Section 3.8.7.4: the highest SEQUENCE is the latest revision"
    );

    let mut ical = String::from("BEGIN:VCALENDAR\r\n");
    for (pass, hour) in [("First", 10), ("Second", 11)] {
        for day in 2..=28 {
            for uid in ["dup-b", "dup-a"] {
                ical.push_str(&format!(
                    "BEGIN:VEVENT\r\nUID:{uid}\r\nRECURRENCE-ID;TZID=Europe/Berlin:202501{day:02}T090000\r\nDTSTART;TZID=Europe/Berlin:202501{day:02}T{hour}0000\r\nDURATION:PT1H\r\nSUMMARY:{pass} {day}\r\nEND:VEVENT\r\n"
                ));
            }
        }
    }
    for uid in ["dup-b", "dup-a"] {
        ical.push_str(&format!(
            "BEGIN:VEVENT\r\nUID:{uid}\r\nDTSTART;TZID=Europe/Berlin:20250101T090000\r\nDURATION:PT1H\r\nRRULE:FREQ=DAILY\r\nSUMMARY:Daily\r\nEND:VEVENT\r\n"
        ));
    }
    ical.push_str("END:VCALENDAR\r\n");
    let jscal = import(&ical);
    let normalized = normalize(&jscal);
    let entries = normalized["entries"].as_array().unwrap();
    assert_eq!(entries.len(), 2, "{normalized}");
    for entry in entries {
        let overrides = entry["recurrenceOverrides"].as_object().unwrap();
        assert_eq!(overrides.len(), 27, "{entry}");
        assert!(
            overrides.values().all(|patch| patch["title"]
                .as_str()
                .is_some_and(|title| title.starts_with("Second "))),
            "{entry}"
        );
    }
}

#[test]
fn organizer_participant_removed_in_one_occurrence() {
    let json = event(
        PARTICIPANTS,
        r#"{"2025-01-07T09:00:00": {"participants/zoe": null}}"#,
    );
    let ical = export(&json);
    let components = override_components(&ical);
    let [component] = components.as_slice() else {
        panic!("{ical}");
    };
    assert!(
        component.property(&ICalendarProperty::Organizer).is_some(),
        "{ical}"
    );
    assert_eq!(
        component.properties(&ICalendarProperty::Attendee).count(),
        3,
        "{ical}"
    );

    let reimported = ical.into_jscalendar::<String, String>();
    let patch = &overrides(&reimported)["2025-01-07T09:00:00"];
    assert_eq!(patch["participants/zoe"], JsonValue::Null);
    assert_eq!(
        normalize(&reimported)["entries"][0]["organizerCalendarAddress"],
        "mailto:zoe@example.com"
    );
    let reinstated = patch
        .as_object()
        .unwrap()
        .iter()
        .filter(|(key, _)| key.as_str() != "participants/zoe")
        .collect::<Vec<_>>();
    let [(_, organizer)] = reinstated.as_slice() else {
        panic!("{patch}");
    };
    assert_eq!(organizer["calendarAddress"], "mailto:zoe@example.com");
    assert_eq!(organizer["roles"]["owner"], true);
}

#[test]
fn organizer_dropped_when_no_participants_remain() {
    let ical = assert_json_roundtrip(&event(
        r#""organizerCalendarAddress": "mailto:zoe@example.com",
           "participants": {"zoe": {"@type": "Participant", "name": "Zoe", "calendarAddress": "mailto:zoe@example.com", "roles": {"owner": true}}},"#,
        r#"{"2025-01-07T09:00:00": {"participants": null}}"#,
    ));
    let components = override_components(&ical);
    assert!(
        components
            .iter()
            .all(|component| component.property(&ICalendarProperty::Organizer).is_none()),
        "{ical}"
    );
}

#[test]
fn nested_role_removal_and_escaped_member_ids() {
    let ical = assert_json_roundtrip(&event(
        PARTICIPANTS,
        r#"{"2025-01-08T09:00:00": {
              "participants/12/roles/optional": null,
              "participants/a~1b": null,
              "participants/c~0d/participationStatus": "declined"
            },
            "2025-01-09T09:00:00": {"participants/zoe/roles/attendee": null}}"#,
    ));
    let rendered = ical.to_string();
    assert!(rendered.contains("JSID=a/b"), "{rendered}");
    assert!(rendered.contains("JSID=c~d"), "{rendered}");
}

#[test]
fn excluded_false_is_not_exported() {
    let ical = export(&event(
        "",
        r#"{"2025-01-08T09:00:00": {"excluded": false, "title": "Kept"}}"#,
    ));
    let rendered = ical.to_string();
    assert!(!rendered.contains("excluded"), "{rendered}");
    assert!(rendered.contains("SUMMARY:Kept"), "{rendered}");
    assert!(!rendered.contains("EXDATE"), "{rendered}");
}

#[test]
fn null_patches_remove_members_and_properties() {
    let ical = assert_json_roundtrip(&event(
        r#""locations": {"room": {"@type": "Location", "name": "Room 1", "iCalendar": {"name": "vlocation"}}, "hall": {"@type": "Location", "name": "Hall", "iCalendar": {"name": "vlocation"}}},
           "alerts": {"reminder": {"@type": "Alert", "action": "display", "trigger": {"@type": "OffsetTrigger", "offset": "-PT15M"}}},"#,
        r#"{"2025-01-08T09:00:00": {"locations/room": null},
            "2025-01-09T09:00:00": {"alerts": null, "locations": null}}"#,
    ));
    let rendered = ical.to_string();
    assert_eq!(rendered.matches("JSID:room").count(), 1, "{rendered}");
    assert_eq!(rendered.matches("JSID:hall").count(), 2, "{rendered}");
    assert_eq!(rendered.matches("BEGIN:VALARM").count(), 2, "{rendered}");
}

#[test]
fn rejected_override_patches_are_reported() {
    let json = event(
        PARTICIPANTS,
        r#"{"2025-01-08T09:00:00": {"participants/bob/participationStatus": "declined"},
            "2025-01-09T09:00:00": {"title": "Kept"}}"#,
    );
    let (ical, rejected) = JSCalendar::<String, String>::parse(&json)
        .unwrap()
        .into_icalendar_with_report(ExportOptions::new())
        .expect("the calendar survives an unapplicable patch");

    assert_eq!(
        rejected
            .iter()
            .map(|patch| (patch.recurrence_id.as_str(), patch.pointer.as_str()))
            .collect::<Vec<_>>(),
        [(
            "2025-01-08T09:00:00",
            "participants/bob/participationStatus"
        )],
        "draft-ietf-calext-jscalendarbis-20 Section 1.5.9: the rejected PatchObject is reported"
    );
    assert!(ical.to_string().contains("SUMMARY:Kept"), "{ical}");

    let (_, rejected) = JSCalendar::<String, String>::parse(&event(
        PARTICIPANTS,
        r#"{"2025-01-08T09:00:00": {"title": "Kept"}}"#,
    ))
    .unwrap()
    .into_icalendar_with_report(ExportOptions::new())
    .expect("converts");
    assert!(rejected.is_empty(), "{rejected:?}");
}

#[test]
fn patch_with_missing_parent_is_rejected() {
    let json = event(
        PARTICIPANTS,
        r#"{"2025-01-08T09:00:00": {"participants/bob/participationStatus": "declined"},
            "2025-01-08T20:00:00": {"participants/bob/participationStatus": "declined"}}"#,
    );
    let ical = JSCalendar::<String, String>::parse(&json)
        .unwrap()
        .into_icalendar()
        .expect("the calendar survives an unapplicable patch");
    assert!(
        override_components(&ical).is_empty(),
        "draft-ietf-calext-jscalendarbis-20 Section 1.5.9: the PatchObject is rejected in its entirety\n{ical}"
    );
    let rendered = ical.to_string();
    assert!(rendered.contains("SUMMARY:Sync"), "{rendered}");
    assert!(
        rendered.contains("RDATE;TZID=Europe/Berlin:") && rendered.contains("20250108T200000"),
        "draft-ietf-calext-jscalendar-icalendar-28: the occurrence degrades to an RDATE\n{rendered}"
    );
}

#[test]
fn excluded_false_alone_is_an_empty_patch() {
    let ical = export(&event(
        "",
        r#"{"2025-01-08T09:00:00": {"excluded": false},
            "2025-01-08T20:00:00": {"excluded": false}}"#,
    ));
    assert!(override_components(&ical).is_empty(), "{ical}");
    let rendered = ical.to_string();
    assert!(
        rendered.contains("RDATE;TZID=Europe/Berlin:20250108T200000\r\n"),
        "draft-ietf-calext-jscalendarbis-20 Section 3.3.4, RFC 5545 Section 3.8.5.2: {rendered}"
    );
    assert!(!rendered.contains("20250108T090000"), "{rendered}");
    assert!(!rendered.contains("excluded"), "{rendered}");
}

const HIDDEN_ATTENDEES: &str = r#"{
  "@type": "Event",
  "uid": "hidden-attendees",
  "title": "Board meeting",
  "start": "2026-05-04T09:00:00",
  "timeZone": "Europe/Madrid",
  "duration": "PT1H",
  "organizerCalendarAddress": "mailto:jdoe@example.com",
  "recurrenceRule": {"frequency": "daily", "count": 3},
  "recurrenceOverrides": {
    "2026-05-05T09:00:00": {"title": "Board meeting (moved)"}
  },
  "participants": {
    "owner": {"@type": "Participant", "calendarAddress": "mailto:jdoe@example.com", "roles": {"owner": true, "attendee": true}, "participationStatus": "accepted"},
    "jane":  {"@type": "Participant", "calendarAddress": "mailto:jane.smith@example.com", "roles": {"attendee": true}},
    "bill":  {"@type": "Participant", "calendarAddress": "mailto:bill@example.com", "roles": {"attendee": true}}
  }
}"#;

fn hidden_attendees_group() -> String {
    format!(r#"{{"@type": "Group", "entries": [{HIDDEN_ATTENDEES}]}}"#)
}

fn import_first(ical: &str) -> JsonValue {
    let jscal = ICalendar::parse(ical)
        .unwrap()
        .into_jscalendar_with::<String, String, _>(
            ImportOptions::new()
                .include_ical_components(false)
                .return_first(true),
        )
        .expect("converts");
    serde_json::from_str(&jscal.to_string_pretty()).unwrap()
}

#[test]
fn owner_attendee_participants_roundtrip() {
    let expected = with_implied_version(serde_json::from_str(HIDDEN_ATTENDEES).unwrap());
    let ical = assert_json_roundtrip(&hidden_attendees_group()).to_string();
    assert_eq!(import_first(&ical), expected, "{ical}");

    let reexported = ICalendar::parse(&ical)
        .unwrap()
        .into_jscalendar::<String, String>()
        .into_icalendar()
        .unwrap()
        .to_string();
    assert_eq!(import_first(&reexported), expected, "{reexported}");
}

#[test]
fn stripped_attendees_do_not_create_partial_participants() {
    let ical = export(&hidden_attendees_group()).to_string();
    let stripped = ical
        .split("\r\n")
        .filter(|line| {
            !line.starts_with("ATTENDEE;PARTSTAT=ACCEPTED;JSID=owner")
                && !line.starts_with("ATTENDEE;JSID=bill")
        })
        .collect::<Vec<_>>()
        .join("\r\n");
    assert_ne!(stripped, ical);
    let imported = import_first(&stripped);
    let participants = imported["participants"].as_object().unwrap();
    assert!(
        participants.values().all(|participant| {
            participant["@type"] == "Participant" && participant["calendarAddress"].is_string()
        }),
        "{imported}"
    );
    assert_eq!(
        imported["participants"]["jane"],
        serde_json::json!({"@type": "Participant", "calendarAddress": "mailto:jane.smith@example.com", "roles": {"attendee": true}})
    );
    assert!(participants.get("owner").is_none(), "{imported}");
    assert!(participants.get("bill").is_none(), "{imported}");
    assert_eq!(
        imported["recurrenceOverrides"],
        serde_json::json!({"2026-05-05T09:00:00": {"title": "Board meeting (moved)"}}),
        "{imported}"
    );
}

#[test]
fn member_maps_roundtrip_with_override() {
    let json = event(
        r#""locations": {
             "room": {"@type": "Location", "name": "Room 1", "description": "First floor", "iCalendar": {"name": "vlocation"}},
             "hall": {"@type": "Location", "name": "Hall", "locationTypes": {"hall": true}, "iCalendar": {"name": "vlocation"}}
           },
           "virtualLocations": {
             "call": {"@type": "VirtualLocation", "name": "Call", "uri": "https://call.example.com/1", "features": {"audio": true, "video": true}},
             "chat": {"@type": "VirtualLocation", "uri": "https://chat.example.com/1"}
           },
           "alerts": {
             "before": {"@type": "Alert", "action": "display", "trigger": {"@type": "OffsetTrigger", "offset": "-PT15M"}},
             "after": {"@type": "Alert", "action": "email", "trigger": {"@type": "OffsetTrigger", "offset": "PT5M", "relativeTo": "end"}}
           },
           "links": {
             "agenda": {"@type": "Link", "href": "https://example.com/agenda.pdf", "contentType": "application/pdf", "rel": "enclosure", "title": "agenda.pdf"},
             "site": {"@type": "Link", "href": "https://example.com/", "rel": "about"}
           },"#,
        r#"{"2025-01-08T09:00:00": {"title": "Sync (moved)"}}"#,
    );
    let ical = assert_json_roundtrip(&json);
    let rendered = ical.to_string();
    let expected =
        with_implied_version(normalize(&JSCalendar::parse(&json).unwrap())["entries"][0].clone());
    assert_eq!(import_first(&rendered), expected, "{rendered}");
}

const ATTENDEE_ROLE_REFERENCE: &str = "draft-ietf-calext-jscalendarbis-22 Section 3.4.6 (unknown roles MUST be preserved), Section 7.6.15 and Appendix A.2.3 (obsolete \"attendee\" role, no iCalendar ROLE equivalent)";

fn attendee_role_event(roles: &str) -> String {
    format!(
        r#"{{"@type": "Group", "entries": [{{
            "@type": "Event", "uid": "attendee-role", "title": "Review",
            "start": "2026-05-04T09:00:00", "timeZone": "Europe/Madrid", "duration": "PT1H",
            "organizerCalendarAddress": "mailto:owner@example.com",
            "participants": {{"p1": {{"@type": "Participant", "calendarAddress": "mailto:p1@example.com", "roles": {roles}}}}}
        }}]}}"#
    )
}

fn sorted_lines(ical: &str) -> Vec<&str> {
    let mut lines = ical.split("\r\n").collect::<Vec<_>>();
    lines.sort_unstable();
    lines
}

fn participant_roles(jscal: &JSCalendar<'_, String, String>) -> Vec<String> {
    jscal
        .0
        .as_object_and_get(&Key::Property(JSCalendarProperty::Entries))
        .and_then(|entries| entries.as_array())
        .and_then(|entries| entries.first())
        .and_then(|entry| entry.as_object_and_get(&Key::Property(JSCalendarProperty::Participants)))
        .and_then(|participants| participants.as_object_and_get(&Key::Borrowed("p1")))
        .and_then(|participant| {
            participant.as_object_and_get(&Key::Property(JSCalendarProperty::Roles))
        })
        .and_then(|roles| roles.as_object())
        .unwrap()
        .keys()
        .map(|key| match key {
            Key::Property(JSCalendarProperty::ParticipantRole(role)) => role.as_str().to_string(),
            Key::Borrowed(role) => role.to_string(),
            Key::Owned(role) => role.clone(),
            key => panic!("{ATTENDEE_ROLE_REFERENCE}: unexpected role key {key:?}"),
        })
        .collect()
}

fn assert_attendee_role_conversion(
    roles: &str,
    expected_ical: &[&str],
    expected_roles: &[&str],
    expected_json: JsonValue,
) {
    let exported = JSCalendar::<String, String>::parse(&attendee_role_event(roles))
        .unwrap()
        .into_icalendar()
        .unwrap()
        .to_string();
    assert_eq!(
        sorted_lines(&exported),
        expected_ical,
        "{ATTENDEE_ROLE_REFERENCE}\n{exported}"
    );

    let reimported = ICalendar::parse(&exported)
        .unwrap()
        .into_jscalendar::<String, String>();
    assert_eq!(
        normalize(&reimported)["entries"][0],
        expected_json,
        "{ATTENDEE_ROLE_REFERENCE}\n{exported}"
    );

    assert_eq!(
        participant_roles(&reimported),
        expected_roles,
        "{ATTENDEE_ROLE_REFERENCE}\n{exported}"
    );

    let reexported = reimported.into_icalendar().unwrap().to_string();
    assert_eq!(
        sorted_lines(&reexported),
        expected_ical,
        "{ATTENDEE_ROLE_REFERENCE}\n{reexported}"
    );
}

#[test]
fn attendee_only_role_is_exported_as_jsprop_object() {
    assert_attendee_role_conversion(
        r#"{"attendee": true}"#,
        &[
            "",
            "ATTENDEE;JSID=p1:mailto:p1@example.com",
            "BEGIN:VCALENDAR",
            "BEGIN:VEVENT",
            "DTSTART;TZID=Europe/Madrid:20260504T090000",
            "DURATION:PT1H",
            "END:VCALENDAR",
            "END:VEVENT",
            r#"JSPROP;JSPTR="participants/p1/roles":{"attendee":true}"#,
            "ORGANIZER:mailto:owner@example.com",
            "SUMMARY:Review",
            "UID:attendee-role",
            "VERSION:2.0",
        ],
        &["attendee"],
        serde_json::json!({
            "@type": "Event", "uid": "attendee-role", "title": "Review",
            "start": "2026-05-04T09:00:00", "timeZone": "Europe/Madrid", "duration": "PT1H",
            "organizerCalendarAddress": "mailto:owner@example.com",
            "participants": {
                "p1": {"@type": "Participant", "calendarAddress": "mailto:p1@example.com", "roles": {"attendee": true}},
                "4b61c991-ca32-576b-8ff2-cd32275be85f": {"@type": "Participant", "calendarAddress": "mailto:owner@example.com", "roles": {"owner": true}}
            }
        }),
    );
}

#[test]
fn owner_and_attendee_roles_export_attendee_as_jsprop_member() {
    assert_attendee_role_conversion(
        r#"{"owner": true, "attendee": true}"#,
        &[
            "",
            "ATTENDEE;ROLE=OWNER;JSID=p1:mailto:p1@example.com",
            "BEGIN:VCALENDAR",
            "BEGIN:VEVENT",
            "DTSTART;TZID=Europe/Madrid:20260504T090000",
            "DURATION:PT1H",
            "END:VCALENDAR",
            "END:VEVENT",
            r#"JSPROP;JSPTR="participants/p1/roles/attendee":true"#,
            "ORGANIZER:mailto:owner@example.com",
            "SUMMARY:Review",
            "UID:attendee-role",
            "VERSION:2.0",
        ],
        &["owner", "attendee"],
        serde_json::json!({
            "@type": "Event", "uid": "attendee-role", "title": "Review",
            "start": "2026-05-04T09:00:00", "timeZone": "Europe/Madrid", "duration": "PT1H",
            "organizerCalendarAddress": "mailto:owner@example.com",
            "participants": {
                "p1": {"@type": "Participant", "calendarAddress": "mailto:p1@example.com", "roles": {"owner": true, "attendee": true}}
            }
        }),
    );
}

#[test]
fn obsolete_attendee_role_parses_as_unknown_role() {
    let json = attendee_role_event(r#"{"owner": true, "attendee": true}"#);
    let jscal = JSCalendar::<String, String>::parse(&json).unwrap();
    assert_eq!(
        participant_roles(&jscal),
        ["owner", "attendee"],
        "{ATTENDEE_ROLE_REFERENCE}"
    );
    assert!(
        JSCalendarParticipantRole::from_str("attendee").is_err(),
        "{ATTENDEE_ROLE_REFERENCE}"
    );
    assert_eq!(
        normalize(&jscal)["entries"][0]["participants"]["p1"]["roles"],
        serde_json::json!({"owner": true, "attendee": true}),
        "{ATTENDEE_ROLE_REFERENCE}"
    );
}

type JSCalendarEntry = Value<'static, JSCalendarProperty<String>, JSCalendarValue<String, String>>;

const RFC5545_DUPLICATES: &str = "RFC 5545 Section 3.8.5.2: duplicate instances are ignored";

fn patch_first_entry(
    jscal: &mut JSCalendar<'static, String, String>,
    pointer: &str,
    value: JSCalendarEntry,
) {
    let pointer = JsonPointer::<JSCalendarProperty<String>>::parse(pointer);
    let entry = jscal
        .0
        .as_object_mut()
        .and_then(|group| group.get_mut(&Key::Property(JSCalendarProperty::Entries)))
        .and_then(Value::as_array_mut)
        .and_then(|entries| entries.first_mut())
        .unwrap();
    assert!(entry.patch_jptr(pointer.iter(), value), "{pointer}");
}

fn recurrence_ids(ical: &ICalendar) -> Vec<String> {
    ical.components
        .iter()
        .filter_map(|component| component.property(&ICalendarProperty::RecurrenceId))
        .map(|entry| {
            let mut line = String::new();
            let _ = entry.write_to(&mut line);
            line
        })
        .collect()
}

fn instance_count(ical: &ICalendar) -> usize {
    ical.expand_dates(Tz::UTC, 100).events.len()
}

#[test]
fn override_membership_expansion_is_bounded() {
    let started = Instant::now();
    let ical = export(
        r#"{"@type": "Group", "entries": [{
            "@type": "Event", "uid": "secondly", "title": "Tick",
            "start": "2025-01-06T09:00:00", "timeZone": "Europe/Berlin", "duration": "PT1S",
            "recurrenceRule": {"frequency": "secondly"},
            "recurrenceOverrides": {
                "2025-01-06T09:00:30": {"title": "Near"},
                "2026-01-06T09:00:00": {"title": "Far"},
                "9999-01-06T09:00:00": {"title": "Farthest"}
            }
        }]}"#,
    );
    assert!(
        started.elapsed() < Duration::from_secs(1),
        "{:?}",
        started.elapsed()
    );
    assert_eq!(override_components(&ical).len(), 3, "{ical}");
    let rendered = ical.to_string();
    assert!(
        rendered.contains("RDATE;TZID=Europe/Berlin:20260106T090000,99990106T090000\r\n"),
        "keys not verified within the expansion limit are exported as RDATE ({RFC5545_DUPLICATES}): {rendered}"
    );

    let json = event("", r#"{"2025-01-10T09:00:00": {"title": "Fifth"}}"#);
    let has_rdate = |options: ExportOptions| {
        JSCalendar::<String, String>::parse(&json)
            .unwrap()
            .into_icalendar_with(options)
            .unwrap()
            .to_string()
            .contains("RDATE")
    };
    assert!(!has_rdate(ExportOptions::new()));
    assert!(has_rdate(ExportOptions::new().max_expansions(3)));
}

#[test]
fn until_bounded_last_instance_is_not_an_rdate() {
    let ical = export(
        r#"{"@type": "Group", "entries": [{
            "@type": "Event", "uid": "until-1", "title": "Sync",
            "start": "2025-01-06T09:00:00", "timeZone": "Europe/Berlin", "duration": "PT1H",
            "recurrenceRule": {"frequency": "daily", "until": "2025-01-10T09:00:00"},
            "recurrenceOverrides": {"2025-01-10T09:00:00": {"title": "Last"}}
        }]}"#,
    );
    let rendered = ical.to_string();
    assert!(rendered.contains("UNTIL=20250110T080000Z"), "{rendered}");
    assert!(
        !rendered.contains("RDATE"),
        "RFC 5545 Section 3.3.10: the synchronized UNTIL is the last instance: {rendered}"
    );
    assert_eq!(override_components(&ical).len(), 1, "{rendered}");
}

#[test]
fn floating_until_is_local_time() {
    let json = r#"{"@type": "Group", "entries": [{
        "@type": "Event", "uid": "until-2", "title": "Local",
        "start": "2025-01-06T09:00:00", "duration": "PT1H",
        "recurrenceRule": {"frequency": "daily", "until": "2025-01-10T09:00:00"},
        "recurrenceOverrides": {"2025-01-10T09:00:00": {"title": "Last"}}
    }]}"#;
    let ical = export(json);
    let rendered = ical.to_string();
    assert!(
        rendered.contains("RRULE:FREQ=DAILY;UNTIL=20250110T090000\r\n")
            && !rendered.contains("RDATE"),
        "RFC 5545 Section 3.3.10: a date with local time DTSTART has a local time UNTIL: {rendered}"
    );
    assert_eq!(instance_count(&ical), 5, "{rendered}");
    assert_eq!(
        normalize(&ical.into_jscalendar())["entries"][0]["recurrenceRule"]["until"],
        "2025-01-10T09:00:00"
    );
}

#[test]
fn date_recurrence_id_with_timed_override_start() {
    let ical = concat!(
        "BEGIN:VCALENDAR\r\n",
        "BEGIN:VEVENT\r\nUID:ad-1\r\nDTSTART;VALUE=DATE:20250106\r\n",
        "RRULE:FREQ=DAILY;COUNT=5\r\nSUMMARY:All day\r\nEND:VEVENT\r\n",
        "BEGIN:VEVENT\r\nUID:ad-1\r\nRECURRENCE-ID;VALUE=DATE:20250108\r\n",
        "DTSTART;TZID=Europe/Berlin:20250108T100000\r\nDURATION:PT1H\r\n",
        "SUMMARY:Timed\r\nEND:VEVENT\r\n",
        "END:VCALENDAR\r\n"
    );
    let jscal = import(ical);
    let keys = overrides(&jscal)
        .as_object()
        .map(|overrides| overrides.keys().cloned().collect::<Vec<_>>())
        .unwrap_or_default();
    assert_eq!(
        keys,
        ["2025-01-08T00:00:00"],
        "RFC 5545 Section 3.8.4.4: a DATE RECURRENCE-ID is the calendar date of the instance"
    );
    let exported = jscal.into_icalendar().unwrap();
    assert_eq!(
        recurrence_ids(&exported),
        ["RECURRENCE-ID;VALUE=DATE:20250108\r\n"],
        "{exported}"
    );
    assert!(!exported.to_string().contains("RDATE"), "{exported}");
}

#[test]
fn floating_override_key_is_stable_across_round_trips() {
    let mut ical = export(
        r#"{"@type": "Group", "entries": [{
            "@type": "Event", "uid": "flj-2", "title": "Local",
            "start": "2025-01-06T09:00:00", "duration": "PT1H",
            "recurrenceRule": {"frequency": "daily"},
            "recurrenceOverrides": {"2025-01-08T09:00:00": {"timeZone": "Asia/Tokyo"}}
        }]}"#,
    );
    for round in 0..3 {
        assert_eq!(
            recurrence_ids(&ical),
            ["RECURRENCE-ID:20250108T090000\r\n"],
            "round {round}: {ical}"
        );
        assert!(!ical.to_string().contains("RDATE"), "round {round}: {ical}");
        let jscal = ical.into_jscalendar::<String, String>();
        assert_eq!(
            overrides(&jscal),
            serde_json::json!({"2025-01-08T09:00:00": {"timeZone": "Asia/Tokyo"}}),
            "round {round}"
        );
        ical = ICalendar::parse(jscal.into_icalendar().unwrap().to_string()).unwrap();
    }
}

#[test]
fn standalone_recurrence_id_time_zone_follows_the_recurrence_id() {
    let normalized = normalize(&import(concat!(
        "BEGIN:VCALENDAR\r\n",
        "BEGIN:VEVENT\r\nUID:sa-1\r\nRECURRENCE-ID;VALUE=DATE:20250108\r\n",
        "DTSTART;TZID=Asia/Tokyo:20250108T100000\r\nDURATION:PT1H\r\nSUMMARY:Date\r\nEND:VEVENT\r\n",
        "BEGIN:VEVENT\r\nUID:sa-2\r\nRECURRENCE-ID:20050426T130000Z\r\n",
        "DTSTART;TZID=America/New_York:20050426T100000\r\nDURATION:PT1H\r\nSUMMARY:Utc\r\nEND:VEVENT\r\n",
        "END:VCALENDAR\r\n"
    )));
    let date = &normalized["entries"][0];
    assert_eq!(date["recurrenceId"], "2025-01-08T00:00:00", "{normalized}");
    assert!(
        date.get("recurrenceIdTimeZone").is_none(),
        "draft-ietf-calext-jscalendar-icalendar-28 Section 2.1.4: {normalized}"
    );
    let utc = &normalized["entries"][1];
    assert_eq!(utc["recurrenceId"], "2005-04-26T13:00:00", "{normalized}");
    assert_eq!(utc["recurrenceIdTimeZone"], "Etc/UTC", "{normalized}");
}

#[test]
fn utc_and_date_recurrence_ids_do_not_drift() {
    for (file, expected, start) in [
        (
            "resources/ical/205.ics",
            "RECURRENCE-ID:20050426T130000Z\r\n",
            "DTSTART;TZID=Eastern:20050426T100000\r\n",
        ),
        (
            "resources/ical/497.ics",
            "RECURRENCE-ID;VALUE=DATE:20180501\r\n",
            "DTSTART;TZID=Europe/Vienna:20180502T170000\r\n",
        ),
    ] {
        let mut ical = ICalendar::parse(std::fs::read_to_string(file).unwrap()).unwrap();
        let rdates = ical.to_string().matches("RDATE").count();
        for round in 0..3 {
            ical = ICalendar::parse(
                ical.into_jscalendar::<String, String>()
                    .into_icalendar()
                    .unwrap()
                    .to_string(),
            )
            .unwrap();
            assert_eq!(
                recurrence_ids(&ical),
                [expected],
                "{file} round {round}\n{ical}"
            );
            let override_start = ical
                .components
                .iter()
                .filter(|component| {
                    component
                        .property(&ICalendarProperty::RecurrenceId)
                        .is_some()
                })
                .filter_map(|component| component.property(&ICalendarProperty::Dtstart))
                .map(|entry| {
                    let mut line = String::new();
                    let _ = entry.write_to(&mut line);
                    line
                })
                .collect::<Vec<_>>();
            assert_eq!(override_start, [start], "{file} round {round}\n{ical}");
            assert_eq!(
                ical.to_string().matches("RDATE").count(),
                rdates,
                "{file} round {round}\n{ical}"
            );
        }
    }
}

#[test]
fn multi_value_recurrence_dates_keep_their_grouping() {
    let series = |ical: &ICalendar| {
        ical.components
            .iter()
            .flat_map(|component| component.entries.iter())
            .filter(|entry| {
                matches!(
                    entry.name,
                    ICalendarProperty::Rdate | ICalendarProperty::Exdate
                )
            })
            .map(|entry| {
                let mut line = String::new();
                let _ = entry.write_to(&mut line);
                line
            })
            .collect::<Vec<_>>()
    };
    for (ical, expected) in [
        (
            std::fs::read_to_string("resources/ical/653.ics").unwrap(),
            vec!["RDATE;VALUE=DATE:20150813,20150814,20150815\r\n"],
        ),
        (
            concat!(
                "BEGIN:VCALENDAR\r\n",
                "BEGIN:VEVENT\r\nUID:ex-2\r\nDTSTART;VALUE=DATE:20250106\r\n",
                "RRULE:FREQ=DAILY;COUNT=10\r\nEXDATE;VALUE=DATE:20250107,20250108,20250109\r\n",
                "SUMMARY:All day\r\nEND:VEVENT\r\n",
                "END:VCALENDAR\r\n"
            )
            .to_string(),
            vec!["EXDATE;VALUE=DATE:20250107,20250108,20250109\r\n"],
        ),
        (
            concat!(
                "BEGIN:VCALENDAR\r\n",
                "BEGIN:VEVENT\r\nUID:rd-2\r\nDTSTART;TZID=Europe/Berlin:20250106T090000\r\n",
                "DURATION:PT1H\r\nRRULE:FREQ=DAILY;COUNT=3\r\n",
                "RDATE;TZID=America/Noronha:20250110T050000,20250111T050000\r\n",
                "SUMMARY:Daily\r\nEND:VEVENT\r\n",
                "END:VCALENDAR\r\n"
            )
            .to_string(),
            vec!["RDATE;TZID=America/Noronha:20250110T050000,20250111T050000\r\n"],
        ),
    ] {
        let mut ical = ICalendar::parse(&ical).unwrap();
        let instances = instance_count(&ical);
        for round in 0..3 {
            ical = ICalendar::parse(
                ical.into_jscalendar::<String, String>()
                    .into_icalendar()
                    .unwrap()
                    .to_string(),
            )
            .unwrap();
            assert_eq!(series(&ical), expected, "round {round}\n{ical}");
            assert_eq!(instance_count(&ical), instances, "round {round}\n{ical}");
        }
    }
}

#[test]
fn unchanged_override_does_not_duplicate_an_instance() {
    let ical = ICalendar::parse(concat!(
        "BEGIN:VCALENDAR\r\n",
        "BEGIN:VEVENT\r\nUID:same-1\r\nDTSTART;TZID=Europe/Berlin:20250106T090000\r\n",
        "DURATION:PT1H\r\nRRULE:FREQ=DAILY;COUNT=4\r\nSUMMARY:Daily\r\nEND:VEVENT\r\n",
        "BEGIN:VEVENT\r\nUID:same-1\r\nRECURRENCE-ID;TZID=Europe/Berlin:20250108T090000\r\n",
        "DTSTART;TZID=Europe/Berlin:20250108T090000\r\nDURATION:PT1H\r\nSUMMARY:Daily\r\nEND:VEVENT\r\n",
        "END:VCALENDAR\r\n"
    ))
    .unwrap();
    let expected = instance_count(&ical);
    let exported = ical
        .into_jscalendar::<String, String>()
        .into_icalendar()
        .unwrap();
    assert_eq!(
        instance_count(&exported),
        expected,
        "{RFC5545_DUPLICATES}: {exported}"
    );
    assert!(!exported.to_string().contains("RDATE"), "{exported}");
}

#[test]
fn removing_the_last_patch_of_an_override_keeps_the_instance_count() {
    let stored = export(
        r#"{"@type": "Group", "entries": [{
            "@type": "Event", "uid": "fig4", "title": "FooBar team meeting",
            "start": "2025-01-08T09:00:00", "timeZone": "Europe/Berlin", "duration": "PT1H",
            "recurrenceRule": {"frequency": "weekly", "count": 10},
            "participants": {"tom": {"@type": "Participant", "calendarAddress": "mailto:tom@example.com", "participationStatus": "accepted"}},
            "recurrenceOverrides": {"2025-03-05T09:00:00": {"participants/tom/participationStatus": "declined"}}
        }]}"#,
    );
    let expected = instance_count(&stored);
    let mut jscal = stored.into_jscalendar::<String, String>();
    patch_first_entry(
        &mut jscal,
        "recurrenceOverrides/2025-03-05T09:00:00/participants~1tom~1participationStatus",
        Value::Null,
    );
    let updated = jscal.into_icalendar().unwrap();
    assert_eq!(
        instance_count(&updated),
        expected,
        "draft-ietf-jmap-calendars-29 Section 5.9.1 Figure 4; {RFC5545_DUPLICATES}: {updated}"
    );
    assert!(!updated.to_string().contains("RDATE"), "{updated}");
}

#[test]
fn numeric_member_ids_are_patchable() {
    let mut jscal = import(concat!(
        "BEGIN:VCALENDAR\r\n",
        "BEGIN:VEVENT\r\nUID:n-1\r\nDTSTART;TZID=Europe/Berlin:20250106T090000\r\n",
        "DURATION:PT1H\r\nRRULE:FREQ=WEEKLY\r\nSUMMARY:Weekly\r\n",
        "ATTENDEE;JSID=12;PARTSTAT=ACCEPTED:mailto:tom@example.com\r\n",
        "END:VEVENT\r\n",
        "BEGIN:VEVENT\r\nUID:n-1\r\nRECURRENCE-ID;TZID=Europe/Berlin:20250113T090000\r\n",
        "DTSTART;TZID=Europe/Berlin:20250113T100000\r\nDURATION:PT1H\r\nSUMMARY:Weekly\r\n",
        "ATTENDEE;JSID=12;PARTSTAT=DECLINED:mailto:tom@example.com\r\n",
        "END:VEVENT\r\n",
        "END:VCALENDAR\r\n"
    ));
    let pointer = "recurrenceOverrides/2025-01-13T09:00:00/participants~112~1participationStatus";
    assert_eq!(
        overrides(&jscal)["2025-01-13T09:00:00"]["participants/12/participationStatus"],
        "declined"
    );

    patch_first_entry(&mut jscal, pointer, Value::Str("tentative".into()));
    assert_eq!(
        jscal
            .to_string_pretty()
            .matches("\"participants/12/participationStatus\"")
            .count(),
        1,
        "draft-ietf-calext-jscalendarbis-20 Section 1.5.9: {}",
        jscal.to_string_pretty()
    );
    assert_eq!(
        overrides(&jscal)["2025-01-13T09:00:00"]["participants/12/participationStatus"],
        "tentative"
    );

    patch_first_entry(&mut jscal, pointer, Value::Null);
    assert_eq!(
        overrides(&jscal)["2025-01-13T09:00:00"].get("participants/12/participationStatus"),
        None,
        "draft-ietf-jmap-calendars-29 Section 5.9.1 Figure 4"
    );
}

#[test]
fn participant_calendar_address_change_replaces_the_member() {
    let jscal = import(concat!(
        "BEGIN:VCALENDAR\r\n",
        "BEGIN:VEVENT\r\nUID:p-1\r\nDTSTART;TZID=Europe/Berlin:20250106T090000\r\n",
        "DURATION:PT1H\r\nRRULE:FREQ=DAILY\r\nSUMMARY:Daily\r\n",
        "ORGANIZER;JSID=org:mailto:org@example.com\r\n",
        "ATTENDEE;JSID=p1:mailto:a@example.com\r\n",
        "END:VEVENT\r\n",
        "BEGIN:VEVENT\r\nUID:p-1\r\nRECURRENCE-ID;TZID=Europe/Berlin:20250108T090000\r\n",
        "DTSTART;TZID=Europe/Berlin:20250108T090000\r\nDURATION:PT1H\r\nSUMMARY:Daily\r\n",
        "ORGANIZER;JSID=org:mailto:org@example.com\r\n",
        "ATTENDEE;JSID=p1:mailto:b@example.com\r\n",
        "END:VEVENT\r\n",
        "END:VCALENDAR\r\n"
    ));
    let patch = &overrides(&jscal)["2025-01-08T09:00:00"];
    assert_eq!(
        patch,
        &serde_json::json!({"participants/p1": {"@type": "Participant", "calendarAddress": "mailto:b@example.com"}}),
        "draft-ietf-calext-jscalendarbis-20 Section 3.3.4 forbids participants/*/calendarAddress"
    );
    let exported = jscal.into_icalendar().unwrap().to_string();
    assert!(
        exported.contains("RECURRENCE-ID") && exported.contains("mailto:b@example.com"),
        "{exported}"
    );

    let object = |json: &str| {
        JSCalendar::<String, String>::parse(json)
            .unwrap()
            .0
            .into_owned()
            .into_object()
            .unwrap()
    };
    let base = object(
        r#"{"@type": "Event", "start": "2025-01-06T09:00:00",
            "participants": {"p1": {"@type": "Participant", "calendarAddress": "mailto:a@example.com", "name": "A"}}}"#,
    );
    let instance = object(
        r#"{"@type": "Event", "start": "2025-01-08T09:00:00",
            "participants": {"p1": {"@type": "Participant", "name": "B"}}}"#,
    );
    let patch = JSCalendar::<String, String>(Value::Object(
        OverrideDiff::new(&base).diff(
            JSCalendarDateTime::new(
                jiff::civil::date(2025, 1, 8)
                    .at(9, 0, 0, 0)
                    .to_zoned(jiff::tz::TimeZone::UTC)
                    .unwrap()
                    .timestamp()
                    .as_second(),
                true,
            ),
            instance,
            Inherited::Nothing,
        ),
    ));
    assert_eq!(
        serde_json::from_str::<JsonValue>(&patch.to_string_pretty()).unwrap(),
        serde_json::json!({"participants/p1": {"@type": "Participant", "name": "B"}})
    );
}

#[test]
fn recurrence_id_keeps_the_series_value_type() {
    let ical = export(
        r#"{"@type": "Group", "entries": [{
            "@type": "Event", "uid": "t2a-1", "title": "Sync",
            "start": "2025-01-06T09:00:00", "timeZone": "Europe/Berlin", "duration": "PT1H",
            "recurrenceRule": {"frequency": "daily"},
            "recurrenceOverrides": {"2025-01-08T09:00:00": {"start": "2025-01-08T00:00:00", "timeZone": null, "showWithoutTime": true, "duration": "P1D"}}
        }]}"#,
    );
    assert_eq!(
        recurrence_ids(&ical),
        ["RECURRENCE-ID;TZID=Europe/Berlin:20250108T090000\r\n"],
        "RFC 5545 Section 3.8.4.4: RECURRENCE-ID has the value type of the series DTSTART"
    );
    let reimported = ical.into_jscalendar::<String, String>();
    assert!(
        overrides(&reimported).get("2025-01-08T09:00:00").is_some(),
        "{}",
        overrides(&reimported)
    );

    let ical = export(
        r#"{"@type": "Group", "entries": [{
            "@type": "Event", "uid": "a2t-1", "title": "Holiday",
            "start": "2025-01-06T00:00:00", "showWithoutTime": true, "duration": "P1D",
            "recurrenceRule": {"frequency": "daily"},
            "recurrenceOverrides": {"2025-01-08T00:00:00": {"title": "Timed", "start": "2025-01-08T10:00:00", "timeZone": "Europe/Berlin", "showWithoutTime": null, "duration": "PT1H"}}
        }]}"#,
    );
    assert_eq!(
        recurrence_ids(&ical),
        ["RECURRENCE-ID;VALUE=DATE:20250108\r\n"],
        "RFC 5545 Section 3.8.4.4: {ical}"
    );
}

#[test]
fn override_start_keeps_its_own_value_type() {
    for (series, patch, lines) in [
        (
            r#""start": "2025-01-06T00:00:00", "showWithoutTime": true, "duration": "P1D""#,
            r#"{"2025-01-08T00:00:00": {"start": "2025-01-08T10:00:00", "timeZone": "Europe/Berlin", "showWithoutTime": null, "duration": "PT1H"}}"#,
            [
                "RECURRENCE-ID;VALUE=DATE:20250108\r\n",
                "DTSTART;TZID=Europe/Berlin:20250108T100000\r\n",
            ],
        ),
        (
            r#""start": "2025-01-06T09:00:00", "timeZone": "Europe/Berlin", "duration": "PT1H""#,
            r#"{"2025-01-08T09:00:00": {"start": "2025-01-08T00:00:00", "timeZone": null, "showWithoutTime": true, "duration": "P1D"}}"#,
            [
                "RECURRENCE-ID;TZID=Europe/Berlin:20250108T090000\r\n",
                "DTSTART;VALUE=DATE:20250108\r\n",
            ],
        ),
        (
            r#""start": "2025-01-06T09:00:00", "timeZone": "Europe/Berlin", "duration": "PT1H""#,
            r#"{"2025-01-08T09:00:00": {"timeZone": null}}"#,
            [
                "RECURRENCE-ID;TZID=Europe/Berlin:20250108T090000\r\n",
                "DTSTART:20250108T090000\r\n",
            ],
        ),
        (
            r#""start": "2025-01-06T09:00:00", "duration": "PT1H""#,
            r#"{"2025-01-08T09:00:00": {"timeZone": "Asia/Tokyo"}}"#,
            [
                "RECURRENCE-ID:20250108T090000\r\n",
                "DTSTART;TZID=Asia/Tokyo:20250108T090000\r\n",
            ],
        ),
    ] {
        let expected: JsonValue = serde_json::from_str(patch).unwrap();
        let mut ical = export(&format!(
            r#"{{"@type": "Group", "entries": [{{"@type": "Event", "uid": "form-1", "title": "Form",
                {series}, "recurrenceRule": {{"frequency": "daily"}}, "recurrenceOverrides": {patch}}}]}}"#
        ));
        for round in 0..3 {
            let rendered = ical.to_string();
            for line in lines {
                assert!(
                    rendered.contains(line),
                    "RFC 5545 Section 3.8.4.4, round {round}: missing {line} in {rendered}"
                );
            }
            let jscal = ical.into_jscalendar::<String, String>();
            assert_eq!(
                patches_without_ical(&jscal),
                expected,
                "round {round}: {rendered}"
            );
            ical = ICalendar::parse(jscal.into_icalendar().unwrap().to_string()).unwrap();
        }
    }

    let mut jscal = import(concat!(
        "BEGIN:VCALENDAR\r\n",
        "BEGIN:VEVENT\r\nUID:ad-3\r\nDTSTART;VALUE=DATE:20250106\r\nDTEND;VALUE=DATE:20250107\r\n",
        "RRULE:FREQ=DAILY;COUNT=5\r\nSUMMARY:All day\r\nEND:VEVENT\r\n",
        "END:VCALENDAR\r\n"
    ));
    let patch = r#"{"2025-01-08T00:00:00": {"start": "2025-01-08T10:00:00", "timeZone": "Europe/Berlin", "showWithoutTime": null, "duration": "PT1H"}}"#;
    let json = format!(r#"{{"recurrenceOverrides": {patch}}}"#);
    let overrides = JSCalendar::<String, String>::parse(&json)
        .unwrap()
        .0
        .into_owned()
        .into_object()
        .and_then(|object| object.into_vec().into_iter().next())
        .map(|(_, overrides)| overrides)
        .unwrap();
    patch_first_entry(&mut jscal, "recurrenceOverrides", overrides);
    let exported = jscal.into_icalendar().unwrap();
    let components = override_components(&exported);
    let [component] = components.as_slice() else {
        panic!("{exported}");
    };
    let mut lines = String::new();
    for entry in &component.entries {
        let _ = entry.write_to(&mut lines);
    }
    assert!(
        lines.contains("DTSTART;TZID=Europe/Berlin:20250108T100000\r\n")
            && lines.contains("DURATION:PT1H\r\n")
            && lines.contains("RECURRENCE-ID;VALUE=DATE:20250108\r\n"),
        "the preserved VALUE=DATE parameters of the series do not apply to a timed occurrence: {exported}"
    );
    assert_eq!(
        patches_without_ical(
            &ICalendar::parse(exported.to_string())
                .unwrap()
                .into_jscalendar()
        ),
        serde_json::from_str::<JsonValue>(patch).unwrap()
    );
}

fn patches_without_ical(jscal: &JSCalendar<'_, String, String>) -> JsonValue {
    let mut overrides = overrides(jscal);
    for patch in overrides
        .as_object_mut()
        .into_iter()
        .flat_map(|overrides| overrides.values_mut())
        .filter_map(JsonValue::as_object_mut)
    {
        patch.remove("iCalendar");
    }
    overrides
}

#[test]
fn ambiguous_and_nonexistent_local_times_follow_rfc5545() {
    let ical = export(
        r#"{"@type": "Group", "entries": [{
            "@type": "Event", "uid": "dst-1", "title": "Night",
            "start": "2025-10-20T02:30:00", "timeZone": "Europe/Berlin", "duration": "PT1H",
            "recurrenceRule": {"frequency": "daily"},
            "recurrenceOverrides": {
                "2025-10-26T02:30:00": {"title": "Ambiguous"},
                "2026-03-29T02:30:00": {"title": "Gap"},
                "2026-10-25T02:30:00": {"excluded": true}
            }
        }]}"#,
    );
    let rendered = ical.to_string();
    for line in [
        "RECURRENCE-ID;TZID=Europe/Berlin:20251026T023000\r\n",
        "DTSTART;TZID=Europe/Berlin:20251026T023000\r\n",
        "RECURRENCE-ID;TZID=Europe/Berlin:20260329T023000\r\n",
        "DTSTART;TZID=Europe/Berlin:20260329T023000\r\n",
        "EXDATE;TZID=Europe/Berlin:20261025T023000\r\n",
    ] {
        assert!(
            rendered.contains(line),
            "RFC 5545 Section 3.3.5: missing {line} in {rendered}"
        );
    }
    assert!(!rendered.contains("RDATE"), "{rendered}");
    assert_eq!(
        overrides(&ical.into_jscalendar()),
        serde_json::json!({
            "2025-10-26T02:30:00": {"title": "Ambiguous"},
            "2026-03-29T02:30:00": {"title": "Gap"},
            "2026-10-25T02:30:00": {"excluded": true}
        })
    );

    let jscal = import(concat!(
        "BEGIN:VCALENDAR\r\n",
        "BEGIN:VEVENT\r\nUID:gap-1\r\nDTSTART;TZID=Europe/Berlin:20251020T023000\r\n",
        "DURATION:PT1H\r\nRRULE:FREQ=DAILY\r\nSUMMARY:Night\r\nEND:VEVENT\r\n",
        "BEGIN:VEVENT\r\nUID:gap-1\r\nRECURRENCE-ID;TZID=Europe/Berlin:20251026T023000\r\n",
        "DTSTART;TZID=Europe/Berlin:20251026T043000\r\nDURATION:PT1H\r\nSUMMARY:Night\r\n",
        "END:VEVENT\r\n",
        "BEGIN:VEVENT\r\nUID:gap-1\r\nRECURRENCE-ID;TZID=Europe/Berlin:20260329T023000\r\n",
        "DTSTART;TZID=Europe/Berlin:20260329T043000\r\nDURATION:PT1H\r\nSUMMARY:Night\r\n",
        "END:VEVENT\r\n",
        "END:VCALENDAR\r\n"
    ));
    let normalized = normalize(&jscal);
    assert_eq!(
        normalized["entries"].as_array().map(Vec::len),
        Some(1),
        "{normalized}"
    );
    assert_eq!(
        overrides(&jscal),
        serde_json::json!({
            "2025-10-26T02:30:00": {"start": "2025-10-26T04:30:00"},
            "2026-03-29T02:30:00": {"start": "2026-03-29T04:30:00"}
        })
    );
}

#[test]
fn raw_series_properties_are_not_copied_into_overrides() {
    let mut jscal = import(concat!(
        "BEGIN:VCALENDAR\r\n",
        "BEGIN:VEVENT\r\nUID:raw-2\r\nDTSTART;TZID=Europe/Berlin:20250106T090000\r\n",
        "DURATION:PT1H\r\nRRULE:FREQ=DAILY;COUNT=3\r\nEXRULE:FREQ=WEEKLY;BYDAY=SU\r\n",
        "RDATE;VALUE=PERIOD:20250120T090000Z/PT1H,20250121T090000Z/PTXH\r\n",
        "SUMMARY:Daily\r\nEND:VEVENT\r\n",
        "END:VCALENDAR\r\n"
    ));
    let expected = instance_count(&jscal.clone().into_icalendar().unwrap());
    let overrides = JSCalendar::<String, String>::parse(
        r#"{"recurrenceOverrides": {"2025-01-07T09:00:00": {"title": "Moved"}}}"#,
    )
    .unwrap()
    .0
    .into_owned()
    .into_object()
    .and_then(|object| object.into_vec().into_iter().next())
    .map(|(_, overrides)| overrides)
    .unwrap();
    patch_first_entry(&mut jscal, "recurrenceOverrides", overrides);
    let exported = jscal.into_icalendar().unwrap();
    let components = override_components(&exported);
    let [component] = components.as_slice() else {
        panic!("{exported}");
    };
    assert!(
        component.property(&ICalendarProperty::Rdate).is_none()
            && component.property(&ICalendarProperty::Exrule).is_none(),
        "{exported}"
    );
    assert_eq!(instance_count(&exported), expected, "{exported}");
}

#[test]
fn exdate_does_not_hide_an_override() {
    let ical = concat!(
        "BEGIN:VCALENDAR\r\n",
        "BEGIN:VEVENT\r\nUID:ex-1\r\nDTSTART;TZID=Europe/Berlin:20250106T090000\r\n",
        "DURATION:PT1H\r\nRRULE:FREQ=DAILY\r\nEXDATE;TZID=Europe/Berlin:20250108T090000\r\n",
        "SUMMARY:Daily\r\nEND:VEVENT\r\n",
        "BEGIN:VEVENT\r\nUID:ex-1\r\nRECURRENCE-ID;TZID=Europe/Berlin:20250108T090000\r\n",
        "DTSTART;TZID=Europe/Berlin:20250108T100000\r\nDURATION:PT1H\r\nSUMMARY:Daily\r\n",
        "END:VEVENT\r\n",
        "END:VCALENDAR\r\n"
    );
    let jscal = import(ical);
    assert_eq!(
        overrides(&jscal)["2025-01-08T09:00:00"]["start"],
        "2025-01-08T10:00:00"
    );
    assert_eq!(
        normalize(&jscal)["entries"].as_array().map(Vec::len),
        Some(1)
    );

    let mut ical =
        ICalendar::parse(std::fs::read_to_string("resources/ical/197.ics").unwrap()).unwrap();
    for round in 0..3 {
        ical = ICalendar::parse(
            ical.into_jscalendar::<String, String>()
                .into_icalendar()
                .unwrap()
                .to_string(),
        )
        .unwrap();
        let mut instances = override_components(&ical)
            .into_iter()
            .map(|component| {
                let mut lines = String::new();
                for entry in component
                    .property(&ICalendarProperty::RecurrenceId)
                    .into_iter()
                    .chain(
                        component
                            .component_ids
                            .iter()
                            .filter_map(|id| ical.components.get(*id as usize))
                            .filter_map(|alarm| alarm.property(&ICalendarProperty::Trigger)),
                    )
                {
                    let _ = entry.write_to(&mut lines);
                }
                lines
            })
            .collect::<Vec<_>>();
        instances.sort();
        assert_eq!(
            instances,
            [
                "RECURRENCE-ID;VALUE=DATE:20040224\r\nTRIGGER:-PT16H2M\r\n",
                "RECURRENCE-ID;VALUE=DATE:20040324\r\nTRIGGER:-PT16H46M\r\n",
            ],
            "round {round}\n{ical}"
        );
    }
}

#[test]
fn member_map_order_does_not_change_the_patch() {
    let ical = |attendees: [&str; 3]| {
        format!(
            concat!(
                "BEGIN:VCALENDAR\r\n",
                "BEGIN:VEVENT\r\nUID:order-1\r\nDTSTART;TZID=Europe/Berlin:20250106T090000\r\n",
                "DURATION:PT1H\r\nRRULE:FREQ=DAILY\r\nSUMMARY:Daily\r\n",
                "ATTENDEE;JSID=a:mailto:a@example.com\r\n",
                "ATTENDEE;JSID=b:mailto:b@example.com\r\n",
                "ATTENDEE;JSID=c:mailto:c@example.com\r\n",
                "END:VEVENT\r\n",
                "BEGIN:VEVENT\r\nUID:order-1\r\nRECURRENCE-ID;TZID=Europe/Berlin:20250108T090000\r\n",
                "DTSTART;TZID=Europe/Berlin:20250108T090000\r\nDURATION:PT1H\r\nSUMMARY:Daily\r\n",
                "{}{}{}",
                "END:VEVENT\r\n",
                "END:VCALENDAR\r\n"
            ),
            attendees[0], attendees[1], attendees[2]
        )
    };
    let a = "ATTENDEE;JSID=a:mailto:a@example.com\r\n";
    let b = "ATTENDEE;JSID=b;PARTSTAT=DECLINED:mailto:b@example.com\r\n";
    let c = "ATTENDEE;JSID=c:mailto:c@example.com\r\n";
    let expected = serde_json::json!({"2025-01-08T09:00:00": {"participants/b/participationStatus": "declined"}});
    assert_eq!(overrides(&import(&ical([a, b, c]))), expected);
    assert_eq!(overrides(&import(&ical([c, b, a]))), expected);

    let object = |json: &str| {
        JSCalendar::<String, String>::parse(json)
            .unwrap()
            .0
            .into_owned()
            .into_object()
            .unwrap()
    };
    let base = object(
        r#"{"@type": "Event", "keywords": {"x": true, "y": true, "z": true},
            "participants": {"p1": {"@type": "Participant", "name": "A", "calendarAddress": "mailto:a@example.com"}}}"#,
    );
    let instance = object(
        r#"{"@type": "Event", "keywords": {"x": true, "z": true, "y": true},
            "participants": {"p1": {"calendarAddress": "mailto:a@example.com", "name": "A", "@type": "Participant"}}}"#,
    );
    assert!(
        OverrideDiff::new(&base)
            .diff(
                JSCalendarDateTime::new(0, true),
                instance,
                Inherited::Nothing
            )
            .is_empty()
    );
}

#[test]
fn forbidden_patches_are_not_instance_patches() {
    for (patch, expected) in [
        (r#"{"title": "Moved"}"#, true),
        (r#"{"title": "Moved", "uid": "other"}"#, true),
        (r#"{"excluded": false, "title": "Moved"}"#, true),
        (r#"{}"#, false),
        (r#"{"excluded": false}"#, false),
        (r#"{"excluded": true}"#, false),
        (r#"{"privacy": "secret"}"#, false),
        (
            r#"{"privacy": "secret", "recurrenceRule/frequency": "weekly"}"#,
            false,
        ),
        (
            r#"{"participants/p1/calendarAddress": "mailto:b@example.com"}"#,
            false,
        ),
    ] {
        let json = format!(r#"{{"recurrenceOverrides": {{"2025-01-08T09:00:00": {patch}}}}}"#);
        let overrides = JSCalendar::<String, String>::parse(&json).unwrap();
        let patch_value = overrides
            .0
            .as_object_and_get(&Key::Property(JSCalendarProperty::RecurrenceOverrides))
            .and_then(Value::as_object)
            .and_then(|overrides| overrides.values().next())
            .unwrap();
        assert_eq!(patch_value.is_instance_patch(), expected, "{patch}");
    }
}

#[test]
fn recurrence_rule_numbers_are_not_folded() {
    for (rule, jsprop, rrule_part) in [
        (
            r#"{"frequency": "daily", "count": -5}"#,
            "recurrenceRule/count",
            "COUNT",
        ),
        (
            r#"{"frequency": "daily", "count": 4294967297}"#,
            "recurrenceRule/count",
            "COUNT",
        ),
        (
            r#"{"frequency": "daily", "interval": 70000}"#,
            "recurrenceRule/interval",
            "INTERVAL",
        ),
        (
            r#"{"frequency": "monthly", "byDay": [{"day": "mo", "nthOfPeriod": 18446744073709551615}]}"#,
            "recurrenceRule/byDay",
            "BYDAY",
        ),
    ] {
        let json = format!(
            r#"{{"@type": "Group", "entries": [{{"@type": "Event", "uid": "r13-count", "start": "2025-01-08T09:00:00", "timeZone": "Etc/UTC", "recurrenceRule": {rule}}}]}}"#
        );
        let ical = export(&json);
        let rendered = ical.to_string();
        assert!(
            !rendered.contains(rrule_part) && rendered.contains(jsprop),
            "draft-ietf-calext-jscalendarbis-20 Section 3.3.3: {rendered}"
        );
        assert_eq!(
            normalize(&ical.into_jscalendar())["entries"][0]["recurrenceRule"],
            serde_json::from_str::<JsonValue>(rule).unwrap(),
            "{rendered}"
        );
    }

    let rendered = export(
        r#"{"@type": "Group", "entries": [{"@type": "Event", "uid": "r13-count", "start": "2025-01-08T09:00:00", "timeZone": "Etc/UTC",
            "recurrenceRule": {"frequency": "monthly", "count": 3, "interval": 2, "byDay": [{"day": "mo", "nthOfPeriod": -1}]}}]}"#,
    )
    .to_string();
    for part in ["COUNT=3", "INTERVAL=2", "BYDAY=-1MO"] {
        assert!(rendered.contains(part), "{rendered}");
    }
    assert!(!rendered.contains("JSPROP"), "{rendered}");
}

#[test]
fn jscalendarbis_1_5_9_override_patch_parent_must_exist() {
    let event = |base: &str, patch: &str| {
        format!(
            r#"{{"@type": "Group", "entries": [{{"@type": "Event", "uid": "parents",
                "start": "2025-01-06T09:00:00", "timeZone": "Europe/Berlin", "duration": "PT1H",
                {base}
                "recurrenceRule": {{"frequency": "daily"}},
                "recurrenceOverrides": {{"2025-01-07T09:00:00": {patch}}}}}]}}"#
        )
    };
    let png = r#"{"@type": "Link", "href": "https://example.com/b.png", "rel": "enclosure"}"#;
    let links = r#""links": {"doc": {"@type": "Link", "href": "https://example.com/a.pdf", "rel": "enclosure"}},"#;
    let participants = concat!(
        r#""participants": {"tom": {"@type": "Participant", "#,
        r#""calendarAddress": "mailto:tom@example.com", "roles": {"attendee": true}}},"#
    );

    for (base, patch, applies) in [
        (links, format!(r#"{{"links/png": {png}}}"#), true),
        ("", format!(r#"{{"links": {{"png": {png}}}}}"#), true),
        ("", format!(r#"{{"links/png": {png}}}"#), false),
        (
            participants,
            r#"{"participants/tom/participationStatus": "declined"}"#.to_string(),
            true,
        ),
        (
            participants,
            r#"{"participants/zoe/participationStatus": "declined"}"#.to_string(),
            false,
        ),
    ] {
        let exported = JSCalendar::<String, String>::parse(&event(base, &patch))
            .expect("valid JSCalendar")
            .into_icalendar()
            .expect("the calendar survives an unapplicable patch")
            .to_string();

        assert_eq!(
            exported.contains("RECURRENCE-ID"),
            applies,
            "draft-ietf-calext-jscalendarbis-20 Section 1.5.9 rule 2: every reference token before the last must already exist\n{patch}\n{exported}"
        );
    }
}

const TITLE_ONLY_OVERRIDE: &str = r#"{"@type": "Group", "entries": [{
    "@type": "Event", "uid": "title-only", "title": "Base",
    "start": "2006-01-02T12:00:00", "timeZone": "America/New_York", "duration": "PT1H",
    "recurrenceRule": {"frequency": "daily", "count": 5},
    "recurrenceOverrides": {"2006-01-04T12:00:00": {"title": "Only title"}}
}]}"#;

fn instance_lengths(ical: &ICalendar) -> Vec<i64> {
    let mut events = ical.expand_dates(Tz::UTC, 100).events;
    events.sort_by_key(|event| event.start);
    events
        .iter()
        .map(|event| {
            let (start, end) = event.timestamps();
            end - start
        })
        .collect()
}

fn sparse_override(series: &str, occurrence: &str) -> String {
    format!(
        "BEGIN:VCALENDAR\r\n\
         BEGIN:VEVENT\r\nUID:sparse\r\n{series}RRULE:FREQ=DAILY;COUNT=3\r\nSUMMARY:Base\r\nEND:VEVENT\r\n\
         BEGIN:VEVENT\r\nUID:sparse\r\n{occurrence}SUMMARY:Renamed\r\nEND:VEVENT\r\n\
         END:VCALENDAR\r\n"
    )
}

#[test]
fn overrides_are_exported_as_complete_components() {
    let ical = assert_json_roundtrip(TITLE_ONLY_OVERRIDE);
    let components = override_components(&ical);
    let [component] = components.as_slice() else {
        panic!("{ical}");
    };
    assert!(
        component
            .property(&ICalendarProperty::Dtstart)
            .is_some_and(|entry| entry.parameter(&ICalendarParameterName::Tzid).is_some()),
        "RFC 5545 Section 3.6.1: DTSTART is REQUIRED when METHOD is absent\n{ical}"
    );
    assert!(
        component.property(&ICalendarProperty::Duration).is_some(),
        "RFC 5545 has no inheritance, so the occurrence states the series duration\n{ical}"
    );
    assert_eq!(instance_lengths(&ical), [3600; 5], "{ical}");
}

#[test]
fn a_round_tripped_override_follows_later_edits_of_the_series() {
    let mut json = serde_json::from_str::<JsonValue>(
        &export(TITLE_ONLY_OVERRIDE)
            .into_jscalendar::<String, String>()
            .to_string_pretty(),
    )
    .unwrap();
    json["entries"][0]["duration"] = JsonValue::from("PT2H");
    let ical = export(&json.to_string());
    assert_eq!(
        instance_lengths(&ical),
        [7200; 5],
        "draft-ietf-calext-jscalendarbis-20 Section 3.3.4: an occurrence inherits what its patch does not set\n{ical}"
    );
}

#[test]
fn an_override_without_dtstart_imports_as_a_minimal_patch() {
    for (series, occurrence, recurrence_id) in [
        (
            "DTSTART;TZID=America/New_York:20060102T120000\r\nDURATION:PT1H\r\n",
            "RECURRENCE-ID;TZID=America/New_York:20060103T120000\r\n",
            "2006-01-03T12:00:00",
        ),
        (
            "DTSTART:20060102T120000Z\r\nDTEND:20060102T133000Z\r\n",
            "RECURRENCE-ID:20060103T120000Z\r\n",
            "2006-01-03T12:00:00",
        ),
        (
            "DTSTART;VALUE=DATE:20060102\r\nDTEND;VALUE=DATE:20060104\r\n",
            "RECURRENCE-ID;VALUE=DATE:20060103\r\n",
            "2006-01-03T00:00:00",
        ),
    ] {
        let ical = sparse_override(series, occurrence);
        assert_eq!(
            overrides(&assert_ical_roundtrip(&ical)),
            serde_json::json!({recurrence_id: {"title": "Renamed"}}),
            "the occurrence starts at its RECURRENCE-ID and lasts as long as the instance it replaces\n{ical}"
        );
    }

    let task = concat!(
        "BEGIN:VCALENDAR\r\n",
        "BEGIN:VTODO\r\nUID:task\r\nDTSTART:20060102T090000Z\r\nDUE:20060102T170000Z\r\n",
        "RRULE:FREQ=DAILY;COUNT=3\r\nSUMMARY:Base\r\nEND:VTODO\r\n",
        "BEGIN:VTODO\r\nUID:task\r\nRECURRENCE-ID:20060103T090000Z\r\nSUMMARY:Renamed\r\nEND:VTODO\r\n",
        "END:VCALENDAR\r\n"
    );
    assert_eq!(
        overrides(&assert_ical_roundtrip(task)),
        serde_json::json!({"2006-01-03T09:00:00": {"title": "Renamed"}}),
        "{task}"
    );
}

#[test]
fn an_override_with_dtstart_but_no_end_is_zero_length() {
    let ical = sparse_override(
        "DTSTART;TZID=America/New_York:20060102T120000\r\nDURATION:PT1H\r\n",
        "RECURRENCE-ID;TZID=America/New_York:20060103T120000\r\nDTSTART;TZID=America/New_York:20060103T120000\r\n",
    );
    let jscal = assert_ical_roundtrip(&ical);
    assert_eq!(
        patches_without_ical(&jscal),
        serde_json::json!({"2006-01-03T12:00:00": {"duration": null, "title": "Renamed"}}),
        "RFC 5545 Section 3.6.1: a VEVENT with a DATE-TIME DTSTART and no DTEND ends when it starts\n{ical}"
    );
    assert_eq!(
        instance_lengths(&export(&jscal.to_string_pretty())),
        [3600, 0, 3600]
    );
}

fn instants(ical: &ICalendar) -> Vec<(i64, i64)> {
    let mut instants = ical
        .expand_dates(Tz::UTC, 100)
        .events
        .iter()
        .map(|event| event.timestamps())
        .collect::<Vec<_>>();
    instants.sort();
    instants
}

#[test]
fn a_superseded_override_does_not_lend_its_duration() {
    let ical = concat!(
        "BEGIN:VCALENDAR\r\n",
        "BEGIN:VEVENT\r\nUID:dup\r\nDTSTART:20060102T120000Z\r\nDURATION:PT1H\r\n",
        "RRULE:FREQ=DAILY;COUNT=3\r\nSUMMARY:Base\r\nEND:VEVENT\r\n",
        "BEGIN:VEVENT\r\nUID:dup\r\nRECURRENCE-ID:20060103T120000Z\r\nSEQUENCE:1\r\n",
        "DTSTART:20060103T120000Z\r\nDURATION:PT2H\r\nSUMMARY:Older\r\nEND:VEVENT\r\n",
        "BEGIN:VEVENT\r\nUID:dup\r\nRECURRENCE-ID:20060103T120000Z\r\nSEQUENCE:2\r\n",
        "SUMMARY:Newer\r\nEND:VEVENT\r\n",
        "END:VCALENDAR\r\n"
    );
    let jscal = import(ical);
    let patches = patches_without_ical(&jscal);
    assert_eq!(
        patches["2006-01-03T12:00:00"]["title"], "Newer",
        "RFC 5545 Section 3.8.7.4: the highest SEQUENCE is the latest revision\n{patches}"
    );
    assert!(
        patches["2006-01-03T12:00:00"].get("duration").is_none(),
        "the occurrence inherits the length of the series, not that of a superseded revision\n{patches}"
    );
    assert_eq!(
        instants(&export(&jscal.to_string_pretty())),
        instants(&ICalendar::parse(ical).unwrap())
    );
}

#[test]
fn an_override_without_dtstart_moves_its_due_into_the_series_time_zone() {
    let ical = concat!(
        "BEGIN:VCALENDAR\r\n",
        "BEGIN:VTODO\r\nUID:task\r\nDTSTART;TZID=America/New_York:20060102T120000\r\n",
        "DUE;TZID=America/New_York:20060102T170000\r\n",
        "RRULE:FREQ=DAILY;COUNT=3\r\nSUMMARY:Base\r\nEND:VTODO\r\n",
        "BEGIN:VTODO\r\nUID:task\r\nRECURRENCE-ID;TZID=America/New_York:20060103T120000\r\n",
        "DUE:20060103T230000Z\r\nSUMMARY:Renamed\r\nEND:VTODO\r\n",
        "END:VCALENDAR\r\n"
    );
    let jscal = import(ical);
    assert_eq!(
        patches_without_ical(&jscal),
        serde_json::json!({"2006-01-03T12:00:00": {"due": "2006-01-03T18:00:00", "title": "Renamed"}}),
        "draft-ietf-calext-jscalendarbis-20 Section 4.2.1: due is a LocalDateTime in the inherited timeZone\n{ical}"
    );
    assert_eq!(
        instants(&export(&jscal.to_string_pretty())),
        instants(&ICalendar::parse(ical).unwrap())
    );
}

#[test]
fn an_override_with_dtend_but_no_dtstart_keeps_its_length() {
    let ical = sparse_override(
        "DTSTART:20060102T120000Z\r\nDURATION:PT1H\r\n",
        "RECURRENCE-ID:20060103T120000Z\r\nDTEND:20060103T150000Z\r\n",
    );
    let jscal = assert_ical_roundtrip(&ical);
    assert_eq!(
        patches_without_ical(&jscal),
        serde_json::json!({"2006-01-03T12:00:00": {"duration": "PT3H", "title": "Renamed"}}),
        "RFC 5545 Section 3.8.2.2: DTEND ends the occurrence that starts at its recurrence id\n{ical}"
    );
    assert_eq!(
        instants(&export(&jscal.to_string_pretty())),
        instants(&ICalendar::parse(&ical).unwrap())
    );
}

#[test]
fn a_standalone_date_instance_without_an_end_lasts_one_day() {
    let ical = concat!(
        "BEGIN:VCALENDAR\r\nMETHOD:REQUEST\r\n",
        "BEGIN:VEVENT\r\nUID:standalone\r\nRECURRENCE-ID;VALUE=DATE:20060103\r\n",
        "SUMMARY:Holiday\r\nEND:VEVENT\r\n",
        "END:VCALENDAR\r\n"
    );
    let jscal = assert_ical_roundtrip(ical);
    let entry = normalize(&jscal)["entries"][0].clone();
    assert_eq!(
        (
            &entry["start"],
            &entry["showWithoutTime"],
            &entry["duration"]
        ),
        (
            &JsonValue::from("2006-01-03T00:00:00"),
            &JsonValue::from(true),
            &JsonValue::from("P1D")
        ),
        "RFC 5545 Section 3.6.1: a VEVENT on a DATE without DTEND or DURATION lasts one day\n{entry}"
    );
    assert_eq!(
        instants(&export(&jscal.to_string_pretty())),
        instants(&ICalendar::parse(ical).unwrap())
    );
}

fn instances(ical: &ICalendar) -> Vec<String> {
    let mut events = ical
        .expand_dates(Tz::UTC, 100)
        .events
        .iter()
        .map(|event| format!("{}/{}", event.start, event.end))
        .collect::<Vec<_>>();
    events.sort();
    events
}

#[test]
fn an_override_without_dtstart_keeps_the_time_zone_of_the_series() {
    for recurrence_id in [
        "RECURRENCE-ID:20060103T170000Z\r\n",
        "RECURRENCE-ID:20060103T120000\r\n",
    ] {
        let ical = sparse_override(
            "DTSTART;TZID=America/New_York:20060102T120000\r\nDURATION:PT1H\r\n",
            recurrence_id,
        );
        let jscal = assert_ical_roundtrip(&ical);
        assert_eq!(
            overrides(&jscal),
            serde_json::json!({"2006-01-03T12:00:00": {"title": "Renamed"}}),
            "draft-ietf-calext-jscalendarbis-20 Section 3.3.4: the occurrence inherits everything but its start, which is shifted to the recurrence id\n{ical}"
        );
        assert_eq!(
            instances(&export(&jscal.to_string_pretty())),
            [
                "2006-01-02T12:00:00-05:00/2006-01-02T13:00:00-05:00",
                "2006-01-03T12:00:00-05:00/2006-01-03T13:00:00-05:00",
                "2006-01-04T12:00:00-05:00/2006-01-04T13:00:00-05:00",
            ],
            "{ical}"
        );
    }
}

#[test]
fn an_override_without_dtstart_keeps_the_period_of_its_rdate() {
    let ical = sparse_override(
        "DTSTART:20060102T120000Z\r\nDURATION:PT1H\r\nRDATE;VALUE=PERIOD:20060105T090000Z/PT3H\r\n",
        "RECURRENCE-ID:20060105T090000Z\r\n",
    );
    assert_eq!(
        instances(&ICalendar::parse(&ical).unwrap()),
        [
            "2006-01-02T12:00:00+00:00/2006-01-02T13:00:00+00:00",
            "2006-01-03T12:00:00+00:00/2006-01-03T13:00:00+00:00",
            "2006-01-04T12:00:00+00:00/2006-01-04T13:00:00+00:00",
            "2006-01-05T09:00:00+00:00/2006-01-05T12:00:00+00:00",
        ]
    );
    let jscal = import(&ical);
    assert_eq!(
        patches_without_ical(&jscal),
        serde_json::json!({"2006-01-05T09:00:00": {"duration": "PT3H", "title": "Renamed"}}),
        "draft-ietf-calext-jscalendar-icalendar-28 Section 2.3.33: the duration of a PERIOD converts to the PatchObject's duration\n{ical}"
    );
    assert_eq!(
        instances(&export(&jscal.to_string_pretty())),
        instances(&ICalendar::parse(&ical).unwrap()),
        "RFC 5545 Section 5: the instance lasts as long as its RDATE PERIOD"
    );
}

#[test]
fn an_override_without_dtstart_inherits_how_the_series_is_shown() {
    for (series, occurrence, recurrence_id) in [
        (
            "DTSTART;TZID=Europe/Berlin:20241017T130000\r\nDTEND;TZID=Asia/Bangkok:20241018T040000\r\n",
            "RECURRENCE-ID;TZID=Europe/Berlin:20241018T130000\r\n",
            "2024-10-18T13:00:00",
        ),
        (
            "DTSTART;TZID=Europe/Berlin:20241017T000000\r\nDURATION:PT24H\r\nSHOW-WITHOUT-TIME;VALUE=BOOLEAN:TRUE\r\n",
            "RECURRENCE-ID;TZID=Europe/Berlin:20241018T000000\r\n",
            "2024-10-18T00:00:00",
        ),
    ] {
        let ical = sparse_override(series, occurrence);
        assert_eq!(
            overrides(&assert_ical_roundtrip(&ical)),
            serde_json::json!({recurrence_id: {"title": "Renamed"}}),
            "draft-ietf-calext-jscalendarbis-20 Section 4.1.3: endTimeZone, and the showWithoutTime of the start, are inherited\n{ical}"
        );
    }
}

#[test]
fn a_due_in_an_event_is_not_the_end_of_the_occurrence() {
    let ical = sparse_override(
        "DTSTART:20060102T120000Z\r\nDURATION:PT1H\r\n",
        "RECURRENCE-ID:20060103T120000Z\r\nDUE:20060103T150000Z\r\n",
    );
    let jscal = import(&ical);
    assert_eq!(
        patches_without_ical(&jscal),
        serde_json::json!({"2006-01-03T12:00:00": {"title": "Renamed"}}),
        "RFC 5545 Section 3.6.1: DUE is not a VEVENT property, so the occurrence has no end of its own\n{ical}"
    );
    assert_eq!(
        instances(&export(&jscal.to_string_pretty())),
        [
            "2006-01-02T12:00:00+00:00/2006-01-02T13:00:00+00:00",
            "2006-01-03T12:00:00+00:00/2006-01-03T13:00:00+00:00",
            "2006-01-04T12:00:00+00:00/2006-01-04T13:00:00+00:00",
        ]
    );
}

#[test]
fn a_standalone_instance_without_dtstart_starts_at_its_recurrence_id() {
    let jscal = import(concat!(
        "BEGIN:VCALENDAR\r\nMETHOD:CANCEL\r\n",
        "BEGIN:VEVENT\r\nUID:standalone\r\nRECURRENCE-ID;TZID=America/New_York:20060103T120000\r\n",
        "SEQUENCE:2\r\nEND:VEVENT\r\n",
        "END:VCALENDAR\r\n"
    ));
    let entry = normalize(&jscal)["entries"][0].clone();
    assert_eq!(
        (&entry["start"], &entry["timeZone"], &entry["recurrenceId"]),
        (
            &JsonValue::from("2006-01-03T12:00:00"),
            &JsonValue::from("America/New_York"),
            &JsonValue::from("2006-01-03T12:00:00")
        ),
        "draft-ietf-calext-jscalendarbis-20 Section 4.1.1: start is mandatory\n{entry}"
    );
    let exported = jscal.into_icalendar().unwrap().to_string();
    assert!(
        exported.contains("DTSTART;TZID=America/New_York:20060103T120000"),
        "draft-ietf-calext-jscalendarbis-20 Section 4.1.1: the mandatory start converts to DTSTART\n{exported}"
    );
}

#[test]
fn inherited_conversions_match_parsed_converted_property_keys() {
    let object = |json: &str| {
        JSCalendar::<String, String>::parse(json)
            .unwrap()
            .0
            .into_owned()
            .into_object()
            .unwrap()
    };
    let base = object(
        r#"{"@type": "Event", "title": "Base", "start": "2006-01-02T00:00:00", "showWithoutTime": true, "duration": "P2D",
            "iCalendar": {"name": "vevent", "convertedProperties": {
                "start": {"parameters": {"value": "DATE"}},
                "duration": {"name": "dtend", "parameters": {"value": "DATE"}}}}}"#,
    );
    let instance = object(r#"{"@type": "Event", "title": "Renamed", "showWithoutTime": true}"#);
    let recurrence_id = JSCalendarDateTime::new(
        jiff::civil::date(2006, 1, 9)
            .at(0, 0, 0, 0)
            .to_zoned(jiff::tz::TimeZone::UTC)
            .unwrap()
            .timestamp()
            .as_second(),
        true,
    );
    for (inherited, expected) in [
        (
            Inherited::StartAndEnd,
            serde_json::json!({"title": "Renamed"}),
        ),
        (
            Inherited::Start,
            serde_json::json!({
                "duration": null,
                "iCalendar": {"name": "vevent", "convertedProperties": {"start": {"parameters": {"value": "DATE"}}}},
                "title": "Renamed"
            }),
        ),
    ] {
        let patch = JSCalendar::<String, String>(Value::Object(OverrideDiff::new(&base).diff(
            recurrence_id,
            instance.clone(),
            inherited,
        )));
        assert_eq!(
            serde_json::from_str::<JsonValue>(&patch.to_string_pretty()).unwrap(),
            expected,
            "{inherited:?}"
        );
    }
}

fn all_day_override(series: &str, occurrence: &str) -> String {
    format!(
        "BEGIN:VCALENDAR\r\n\
         BEGIN:VEVENT\r\nUID:all-day\r\n{series}RRULE:FREQ=DAILY;COUNT=3\r\nSUMMARY:Base\r\nEND:VEVENT\r\n\
         BEGIN:VEVENT\r\nUID:all-day\r\n{occurrence}SUMMARY:Renamed\r\nEND:VEVENT\r\n\
         END:VCALENDAR\r\n"
    )
}

fn exported_override_ends(jscal: &JSCalendar<'_, String, String>) -> Vec<String> {
    let exported = jscal.clone().into_icalendar().expect("exports");
    override_components(&exported)
        .into_iter()
        .flat_map(|component| component.entries.iter())
        .filter(|entry| {
            matches!(
                entry.name,
                ICalendarProperty::Dtend | ICalendarProperty::Duration
            )
        })
        .map(|entry| {
            let mut line = String::new();
            let _ = entry.write_to(&mut line);
            line
        })
        .collect()
}

#[test]
fn rfc5545_3_6_1_a_date_override_without_an_end_lasts_one_day() {
    const OCCURRENCE: &str = "RECURRENCE-ID;VALUE=DATE:20250107\r\nDTSTART;VALUE=DATE:20250107\r\n";
    let ical = all_day_override("DTSTART;VALUE=DATE:20250106\r\n", OCCURRENCE);
    let jscal = assert_ical_roundtrip(&ical);
    assert_eq!(
        overrides(&jscal),
        serde_json::json!({"2025-01-07T00:00:00": {"title": "Renamed"}}),
        "RFC 5545 Section 3.6.1: the occurrence and the series both last one day\n{ical}"
    );
    assert_eq!(normalize(&jscal)["entries"][0]["duration"], "P1D");
    assert_eq!(
        exported_override_ends(&jscal),
        Vec::<String>::new(),
        "{ical}"
    );
    assert_eq!(
        instance_lengths(&export(&jscal.to_string_pretty())),
        [86400; 3]
    );

    let ical = all_day_override(
        "DTSTART;VALUE=DATE:20250106\r\nDTEND;VALUE=DATE:20250108\r\n",
        OCCURRENCE,
    );
    let jscal = assert_ical_roundtrip(&ical);
    assert_eq!(
        patches_without_ical(&jscal),
        serde_json::json!({"2025-01-07T00:00:00": {"duration": "P1D", "title": "Renamed"}}),
        "RFC 5545 Section 3.6.1: RFC 5545 has no inheritance, so the occurrence lasts one day where the series lasts two\n{ical}"
    );
    assert_eq!(
        exported_override_ends(&jscal),
        Vec::<String>::new(),
        "the occurrence exports without an end, as it was imported\n{ical}"
    );
    let expected = [2 * 86400, 86400, 2 * 86400];
    assert_eq!(
        instance_lengths(&ICalendar::parse(&ical).expect("valid iCalendar")),
        expected
    );
    assert_eq!(
        instance_lengths(&export(&jscal.to_string_pretty())),
        expected
    );

    let ical = all_day_override(
        "DTSTART:20250106T090000Z\r\n",
        "RECURRENCE-ID:20250107T090000Z\r\nDTSTART;VALUE=DATE:20250107\r\n",
    );
    let jscal = import(&ical);
    assert_eq!(
        patches_without_ical(&jscal)["2025-01-07T09:00:00"]["duration"],
        "P1D",
        "RFC 5545 Section 3.6.1: a DATE occurrence of a series of DATE-TIME points lasts one day\n{ical}"
    );
    assert_eq!(
        instance_lengths(&export(&jscal.to_string_pretty())),
        instance_lengths(&ICalendar::parse(&ical).expect("valid iCalendar"))
    );
}

#[test]
fn an_override_without_dtstart_inherits_the_day_of_a_date_series() {
    let ical = all_day_override(
        "DTSTART;VALUE=DATE:20250106\r\n",
        "RECURRENCE-ID;VALUE=DATE:20250107\r\n",
    );
    let jscal = assert_ical_roundtrip(&ical);
    assert_eq!(
        overrides(&jscal),
        serde_json::json!({"2025-01-07T00:00:00": {"title": "Renamed"}}),
        "draft-ietf-calext-jscalendarbis-20 Section 3.3.4: the occurrence inherits the duration of the series\n{ical}"
    );
    assert_eq!(
        instance_lengths(&export(&jscal.to_string_pretty())),
        [86400; 3]
    );
}

#[test]
fn task_override_without_start_keeps_the_series_due() {
    let ical = export(
        r#"{"@type": "Group", "version": "2.0", "entries": [{"@type": "Task", "uid": "task",
            "due": "2025-01-10T10:00:00", "timeZone": "Europe/Berlin",
            "recurrenceOverrides": {"2025-01-17T10:00:00": {"title": "Moved"}}}]}"#,
    )
    .to_string();
    let lines = sorted_lines(&ical);
    assert_eq!(
        lines
            .iter()
            .copied()
            .filter(|line| line.starts_with("DUE"))
            .collect::<Vec<_>>(),
        [
            "DUE;TZID=Europe/Berlin:20250110T100000",
            "DUE;TZID=Europe/Berlin:20250110T100000"
        ],
        "draft-ietf-calext-jscalendarbis-22 Section 3.3.4: only the start is shifted\n{ical}"
    );
    assert!(
        lines.contains(&"RECURRENCE-ID;TZID=Europe/Berlin:20250117T100000"),
        "{ical}"
    );
}

#[test]
fn task_override_without_start_patches_its_own_due() {
    let jscal = import(
        "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nBEGIN:VTODO\r\nUID:task\r\nDUE;TZID=Europe/Berlin:20250110T100000\r\nEND:VTODO\r\nBEGIN:VTODO\r\nUID:task\r\nRECURRENCE-ID;TZID=Europe/Berlin:20250117T100000\r\nDUE;TZID=Europe/Berlin:20250117T100000\r\nSUMMARY:Moved\r\nEND:VTODO\r\nEND:VCALENDAR\r\n",
    );
    assert_eq!(
        normalize(&jscal)["entries"][0]["recurrenceOverrides"],
        serde_json::json!({"2025-01-17T10:00:00": {"title": "Moved", "due": "2025-01-17T10:00:00"}}),
        "draft-ietf-calext-jscalendarbis-22 Section 3.3.4: only the start is shifted"
    );
}

#[test]
fn task_override_without_start_inherits_the_series_due() {
    let jscal = import(
        "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nBEGIN:VTODO\r\nUID:task\r\nDUE;TZID=Europe/Berlin:20250110T100000\r\nEND:VTODO\r\nBEGIN:VTODO\r\nUID:task\r\nRECURRENCE-ID;TZID=Europe/Berlin:20250117T100000\r\nDUE;TZID=Europe/Berlin:20250110T100000\r\nSUMMARY:Moved\r\nEND:VTODO\r\nEND:VCALENDAR\r\n",
    );
    assert_eq!(
        normalize(&jscal)["entries"][0]["recurrenceOverrides"],
        serde_json::json!({"2025-01-17T10:00:00": {"title": "Moved"}}),
        "draft-ietf-calext-jscalendarbis-22 Section 3.3.4: an occurrence inherits everything but the start"
    );
}
