/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

/*
    DISCLAIMER:
    This code has been written by an LLM and is inefficient and unidiomatic.
    It was created to demonstrate calcard in the browser and is not intended for production use.
    It may contain errors, security vulnerabilities, or other issues that could cause harm if used
    in a real-world application. Use at your own risk.
*/

use std::borrow::Cow;

use calcard::{
    Entry, Parser, common::timezone::Tz, icalendar::dates::CalendarExpand, jscalendar::JSCalendar,
    jscontact::JSContact,
};
use serde::Serialize;
use wasm_bindgen::prelude::*;

const BUG: &str = "Looks like you've found a bug in the conversion. Please report it.";

#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub fn version() -> String {
    env!("CALCARD_VERSION").to_string()
}

#[derive(Serialize)]
struct Occurrence {
    from: String,
    to: String,
}

#[derive(Serialize)]
struct ConvertResult {
    source_type: String,
    counterpart: String,
    conversion: String,
    roundtrip: String,
    occurrences: Vec<Occurrence>,
    error: Option<String>,
}

impl ConvertResult {
    fn ok(
        source_type: &str,
        counterpart: &str,
        conversion: String,
        roundtrip: String,
        occurrences: Vec<Occurrence>,
    ) -> Self {
        ConvertResult {
            source_type: source_type.to_string(),
            counterpart: counterpart.to_string(),
            conversion,
            roundtrip,
            occurrences,
            error: None,
        }
    }

    fn error(message: impl Into<String>) -> Self {
        ConvertResult {
            source_type: String::new(),
            counterpart: String::new(),
            conversion: String::new(),
            roundtrip: String::new(),
            occurrences: Vec::new(),
            error: Some(message.into()),
        }
    }
}

#[wasm_bindgen]
pub fn convert(input: &str) -> JsValue {
    serde_wasm_bindgen::to_value(&do_convert(input)).unwrap_or(JsValue::NULL)
}

fn do_convert(input: &str) -> ConvertResult {
    let source = input.trim_start();

    if source.is_empty() {
        return ConvertResult::error("Paste an iCalendar, JSCalendar, vCard or JSContact document.");
    }

    if source.starts_with("BEGIN:") {
        match Parser::new(source).entry() {
            Entry::ICalendar(icalendar) => {
                let occurrences = expand(icalendar.expand_dates(Tz::Floating, 25));
                let jscalendar = icalendar.into_jscalendar::<String, String>();
                let conversion = jscalendar.to_string_pretty();
                match jscalendar.into_icalendar() {
                    Some(roundtrip) => ConvertResult::ok(
                        "iCalendar",
                        "JSCalendar",
                        conversion,
                        roundtrip.to_string(),
                        occurrences,
                    ),
                    None => ConvertResult::error(BUG),
                }
            }
            Entry::VCard(vcard) => {
                let jscontact = vcard.into_jscontact::<String, String>();
                let conversion = jscontact.to_string_pretty();
                match jscontact.into_vcard() {
                    Some(roundtrip) => ConvertResult::ok(
                        "vCard",
                        "JSContact",
                        conversion,
                        roundtrip.to_string(),
                        Vec::new(),
                    ),
                    None => ConvertResult::error(BUG),
                }
            }
            Entry::InvalidLine(text) => ConvertResult::error(format!("Invalid line found: {text}")),
            Entry::UnexpectedComponentEnd { expected, found } => ConvertResult::error(format!(
                "Unexpected component end: expected {}, found {}",
                expected.as_str(),
                found.as_str()
            )),
            Entry::UnterminatedComponent(component) => {
                ConvertResult::error(format!("Unterminated component: {component}"))
            }
            Entry::TooManyComponents => ConvertResult::error("Too many components."),
            Entry::Eof => ConvertResult::error("Unexpected end of file."),
            _ => ConvertResult::error(
                "Unrecognized format. Please provide a valid iCalendar, JSCalendar, vCard or JSContact document.",
            ),
        }
    } else if source.starts_with('{') {
        if source.contains("\"Group\"") {
            match JSCalendar::<String, String>::parse(source.trim_end()) {
                Ok(jscalendar) => match jscalendar.into_icalendar() {
                    Some(icalendar) => {
                        let conversion = icalendar.to_string();
                        let occurrences = expand(icalendar.expand_dates(Tz::Floating, 25));
                        let roundtrip = icalendar.into_jscalendar::<String, String>().to_string_pretty();
                        ConvertResult::ok(
                            "JSCalendar",
                            "iCalendar",
                            conversion,
                            roundtrip,
                            occurrences,
                        )
                    }
                    None => ConvertResult::error(BUG),
                },
                Err(err) => ConvertResult::error(format!("Failed to parse JSCalendar: {err}")),
            }
        } else if source.contains("\"Card\"") {
            match JSContact::<String, String>::parse(source.trim_end()) {
                Ok(jscontact) => match jscontact.into_vcard() {
                    Some(vcard) => {
                        let conversion = vcard.to_string();
                        let roundtrip = vcard.into_jscontact::<String, String>().to_string_pretty();
                        ConvertResult::ok("JSContact", "vCard", conversion, roundtrip, Vec::new())
                    }
                    None => ConvertResult::error(BUG),
                },
                Err(err) => ConvertResult::error(format!("Failed to parse JSContact: {err}")),
            }
        } else {
            ConvertResult::error("This does not look like a valid JSCalendar or JSContact document.")
        }
    } else {
        ConvertResult::error(
            "Unrecognized format. Please provide a valid iCalendar, JSCalendar, vCard or JSContact document.",
        )
    }
}

fn expand(expanded: CalendarExpand) -> Vec<Occurrence> {
    let mut events = expanded
        .events
        .into_iter()
        .filter_map(|event| event.try_into_date_time())
        .collect::<Vec<_>>();
    events.sort_unstable_by(|a, b| a.start.cmp(&b.start));
    events
        .into_iter()
        .map(|event| Occurrence {
            from: format!(
                "{} ({})",
                event.start.format("%a %b %-d, %Y %-I:%M%P"),
                event
                    .start
                    .timezone()
                    .name()
                    .unwrap_or(Cow::Borrowed("Floating"))
            ),
            to: format!(
                "{} ({})",
                event.end.format("%a %b %-d, %Y %-I:%M%P"),
                event
                    .end
                    .timezone()
                    .name()
                    .unwrap_or(Cow::Borrowed("Floating"))
            ),
        })
        .collect()
}
