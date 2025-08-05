/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

use std::borrow::Cow;

use crate::{
    common::{parser::Timestamp, CalendarScale, Data, PartialDateTime},
    jscontact::{
        JSContact, JSContactGrammaticalGender, JSContactKind, JSContactLevel,
        JSContactPhoneticSystem, JSContactProperty, JSContactValue,
    },
    vcard::{
        VCard, VCardEntry, VCardGramGender, VCardKind, VCardLevel, VCardParameter,
        VCardParameterName, VCardPhonetic, VCardProperty, VCardType, VCardValue, VCardValueType,
    },
};
use jmap_tools::{Element, JsonPointer, Key, Property, Value};

#[derive(Debug, Default)]
struct JSProps<'x>(Vec<(String, Value<'x, JSContactProperty, JSContactValue>)>);

impl<'x> JSProps<'x> {
    pub fn insert(&mut self, jsptr: &[&str], value: Value<'x, JSContactProperty, JSContactValue>) {
        self.0
            .push((JsonPointer::<JSContactProperty>::encode(jsptr), value));
    }
}

impl JSContact<'_> {
    pub fn into_vcard(self) -> Option<VCard> {
        let mut vcard = VCard::default();
        let mut js_props = JSProps::default();
        let mut localizations = None;
        let mut properties = None;

        for (property, value) in self.0.into_object()?.into_vec() {
            let Key::Property(property) = property else {
                js_props.insert(&[property.to_string().as_ref()], value);
                continue;
            };

            match property {
                JSContactProperty::Uid
                | JSContactProperty::Kind
                | JSContactProperty::Language
                | JSContactProperty::ProdId
                | JSContactProperty::Created
                | JSContactProperty::Updated => match convert_value(value) {
                    Ok(value) => {
                        vcard.entries.push(
                            VCardEntry::new(match property {
                                JSContactProperty::Uid => VCardProperty::Uid,
                                JSContactProperty::Kind => VCardProperty::Kind,
                                JSContactProperty::Language => VCardProperty::Language,
                                JSContactProperty::ProdId => VCardProperty::Prodid,
                                JSContactProperty::Created => VCardProperty::Created,
                                JSContactProperty::Updated => VCardProperty::Rev,
                                _ => unreachable!(),
                            })
                            .with_value(value),
                        );
                    }
                    Err(value) => {
                        js_props.insert(&[property.to_string().as_ref()], value);
                    }
                },
                JSContactProperty::Directories => {
                    for (name, value) in value.into_expanded_object() {
                        let mut entry = VCardEntry::new(VCardProperty::OrgDirectory);
                        let mut value_type = None;

                        for (prop, value) in value.into_expanded_object() {
                            match prop {
                                Key::Property(JSContactProperty::Kind) => {
                                    if matches!(
                                        value,
                                        Value::Element(JSContactValue::Kind(JSContactKind::Entry))
                                    ) {
                                        entry.name = VCardProperty::Source;
                                    }
                                }
                                Key::Property(JSContactProperty::Uri) => {
                                    if let Some(text) = value.into_string() {
                                        entry.values.push(VCardValue::Text(text));
                                        value_type = Some(VCardValueType::Uri);
                                    }
                                }
                                Key::Property(JSContactProperty::Pref) => {
                                    if let Some(pref) = value.as_u64() {
                                        entry.params.push(VCardParameter::Pref(pref as u32));
                                    }
                                }
                                Key::Property(JSContactProperty::ListAs) => {
                                    if let Some(index) = value.as_u64() {
                                        entry.params.push(VCardParameter::Index(index as u32));
                                    }
                                }
                                Key::Property(JSContactProperty::MediaType) => {
                                    if let Some(text) = value.into_string() {
                                        entry.params.push(VCardParameter::Mediatype(text));
                                    }
                                }
                                Key::Property(JSContactProperty::Label) => {
                                    if let Some(text) = value.into_string() {
                                        entry.values.push(VCardValue::Text(text));
                                    }
                                }
                                Key::Property(JSContactProperty::Contexts) => {
                                    if let Some(types) = convert_types(value) {
                                        entry.params.push(VCardParameter::Type(types));
                                    }
                                }
                                _ => {
                                    js_props.insert(
                                        &[
                                            property.to_string().as_ref(),
                                            name.to_string().as_ref(),
                                            prop.to_string().as_ref(),
                                        ],
                                        value,
                                    );
                                }
                            }
                        }

                        if !entry.values.is_empty() {
                            if let Some(value_type) = value_type {
                                entry.params.push(VCardParameter::Value(vec![value_type]));
                            }
                            vcard
                                .entries
                                .push(entry.with_param(VCardParameter::PropId(name.into_string())));
                        }
                    }
                }
                JSContactProperty::Anniversaries => {
                    for (name, value) in value.into_expanded_object() {
                        if let Some((vcard_property, mut vcard_place_property)) = map_kind(
                            &value,
                            [
                                (
                                    JSContactKind::Birth,
                                    (VCardProperty::Bday, Some(VCardProperty::Birthplace)),
                                ),
                                (
                                    JSContactKind::Death,
                                    (VCardProperty::Deathdate, Some(VCardProperty::Deathplace)),
                                ),
                                (JSContactKind::Wedding, (VCardProperty::Anniversary, None)),
                            ],
                        ) {
                            let mut date = None;
                            let mut place = None;
                            let mut calscale = None;

                            for (prop, value) in value.into_expanded_object() {
                                match prop {
                                    Key::Property(JSContactProperty::Kind) => {}
                                    Key::Property(JSContactProperty::Date) => {
                                        if let Ok((date_, calscale_)) = convert_anniversary(value) {
                                            date = Some(date_);
                                            calscale = calscale_;
                                        }
                                    }
                                    Key::Property(JSContactProperty::Place)
                                        if vcard_place_property.is_some()
                                            && value.is_object_and_contains_any_key(&[
                                                Key::Property(JSContactProperty::Full),
                                                Key::Property(JSContactProperty::Coordinates),
                                            ]) =>
                                    {
                                        for (prop, value) in value.into_expanded_object() {
                                            match prop {
                                                Key::Property(JSContactProperty::Full)
                                                    if vcard_place_property.is_some() =>
                                                {
                                                    if let Some(text) = value.into_string() {
                                                        place = Some(
                                                            VCardEntry::new(
                                                                vcard_place_property
                                                                    .take()
                                                                    .unwrap(),
                                                            )
                                                            .with_value(text),
                                                        );
                                                    }
                                                }
                                                Key::Property(JSContactProperty::Coordinates)
                                                    if vcard_place_property.is_some() =>
                                                {
                                                    if let Some(text) = value.into_string() {
                                                        place = Some(
                                                            VCardEntry::new(
                                                                vcard_place_property
                                                                    .take()
                                                                    .unwrap(),
                                                            )
                                                            .with_value(text)
                                                            .with_param(VCardParameter::Value(
                                                                vec![VCardValueType::Uri],
                                                            )),
                                                        );
                                                    }
                                                }
                                                _ => {
                                                    let todo = "map unsupported vCard properties";
                                                }
                                            }
                                        }
                                    }
                                    _ => {
                                        js_props.insert(
                                            &[
                                                JSContactProperty::Anniversaries
                                                    .to_string()
                                                    .as_ref(),
                                                name.to_string().as_ref(),
                                                prop.to_string().as_ref(),
                                            ],
                                            value,
                                        );
                                    }
                                }
                            }

                            // Add place if it exists
                            if let Some(place) = place {
                                vcard.entries.push(place.with_param(VCardParameter::PropId(
                                    name.to_string().into_owned(),
                                )));
                            }

                            // Add date
                            if let Some(date) = date {
                                let mut entry = VCardEntry::new(vcard_property);
                                if let Some(calscale) = calscale {
                                    entry.params.push(VCardParameter::Calscale(calscale));
                                }
                                vcard.entries.push(
                                    entry
                                        .with_param(VCardParameter::PropId(name.into_string()))
                                        .with_value(VCardValue::PartialDateTime(date)),
                                );
                            }
                        } else {
                            js_props.insert(
                                &[property.to_string().as_ref(), name.to_string().as_ref()],
                                value,
                            );
                        }
                    }
                }
                JSContactProperty::Name => {
                    let mut params = Vec::new();
                    let mut parts: Vec<Option<String>> = vec![None; 7];
                    let mut num_parts = 0;

                    for (key, value) in value.into_expanded_object() {
                        match key {
                            Key::Property(JSContactProperty::Components) => {
                                for components in value.into_array().unwrap_or_default() {
                                    let mut comp_value = None;
                                    let mut comp_pos = None;

                                    for (key, value) in components.into_expanded_object() {
                                        match key {
                                            Key::Property(JSContactProperty::Kind) => {
                                                comp_pos = match value {
                                                    Value::Element(JSContactValue::Kind(
                                                        JSContactKind::Surname,
                                                    )) => 0,
                                                    Value::Element(JSContactValue::Kind(
                                                        JSContactKind::Given,
                                                    )) => 1,
                                                    Value::Element(JSContactValue::Kind(
                                                        JSContactKind::Given2,
                                                    )) => 2,
                                                    Value::Element(JSContactValue::Kind(
                                                        JSContactKind::Title,
                                                    )) => 3,
                                                    Value::Element(JSContactValue::Kind(
                                                        JSContactKind::Credential,
                                                    )) => 4,
                                                    Value::Element(JSContactValue::Kind(
                                                        JSContactKind::Surname2,
                                                    )) => 5,
                                                    Value::Element(JSContactValue::Kind(
                                                        JSContactKind::Generation,
                                                    )) => 6,
                                                    _ => continue,
                                                }
                                                .into();
                                            }
                                            Key::Property(JSContactProperty::Value) => {
                                                comp_value = value.into_string();
                                            }
                                            _ => {
                                                js_props.insert(
                                                    &[
                                                        JSContactProperty::Name
                                                            .to_string()
                                                            .as_ref(),
                                                        JSContactProperty::Components
                                                            .to_string()
                                                            .as_ref(),
                                                        key.to_string().as_ref(),
                                                    ],
                                                    value,
                                                );
                                            }
                                        }
                                    }

                                    if let (Some(comp_value), Some(comp_pos)) =
                                        (comp_value, comp_pos)
                                    {
                                        let part = &mut parts[comp_pos];
                                        if let Some(part) = part {
                                            *part = format!("{part},{comp_value}");
                                        } else {
                                            *part = Some(comp_value);
                                        }
                                        num_parts = num_parts.max(comp_pos + 1);
                                    }
                                }
                            }
                            Key::Property(JSContactProperty::Full) => {
                                if let Some(text) = value.into_string() {
                                    vcard
                                        .entries
                                        .push(VCardEntry::new(VCardProperty::Fn).with_value(text));
                                }
                            }
                            Key::Property(JSContactProperty::SortAs) => {
                                let mut surname = None;
                                let mut given = None;

                                for (key, value) in value.into_expanded_object() {
                                    match key {
                                        Key::Property(JSContactProperty::SortAsKind(
                                            JSContactKind::Surname,
                                        )) => {
                                            surname = value.into_string();
                                        }
                                        Key::Property(JSContactProperty::SortAsKind(
                                            JSContactKind::Given,
                                        )) => {
                                            given = value.into_string();
                                        }
                                        _ => {
                                            js_props.insert(
                                                &[
                                                    JSContactProperty::Name.to_string().as_ref(),
                                                    JSContactProperty::SortAs.to_string().as_ref(),
                                                    key.to_string().as_ref(),
                                                ],
                                                value,
                                            );
                                        }
                                    }
                                }

                                if let (Some(surname), Some(given)) =
                                    (surname.as_ref(), given.as_ref())
                                {
                                    params.push(VCardParameter::SortAs(format!(
                                        "{surname}, {given}"
                                    )));
                                } else if let Some(surname) = surname.or(given) {
                                    params.push(VCardParameter::SortAs(surname));
                                }
                            }
                            Key::Property(JSContactProperty::PhoneticSystem) => {
                                if let Ok(phonetic_system) = VCardPhonetic::try_from(value) {
                                    params.push(VCardParameter::Phonetic(phonetic_system));
                                }
                            }
                            Key::Property(JSContactProperty::PhoneticScript) => {
                                if let Some(phonetic_script) = value.into_string() {
                                    params.push(VCardParameter::Script(phonetic_script));
                                }
                            }
                            _ => {
                                js_props.insert(
                                    &[property.to_string().as_ref(), key.to_string().as_ref()],
                                    value,
                                );
                            }
                        }
                    }

                    if num_parts > 0 {
                        let mut n = String::with_capacity(16);
                        for (pos, part) in parts.into_iter().take(num_parts).enumerate() {
                            if pos > 0 {
                                n.push(';');
                            }
                            if let Some(part) = part {
                                n.push_str(&part);
                            }
                        }

                        vcard.entries.push(
                            VCardEntry::new(VCardProperty::N)
                                .with_params(params)
                                .with_value(n),
                        );
                    }
                }
                JSContactProperty::SpeakToAs => {
                    for (key, value) in value.into_expanded_object() {
                        match key {
                            Key::Property(JSContactProperty::GrammaticalGender) => {
                                if let Ok(gender) = convert_value(value) {
                                    vcard.entries.push(
                                        VCardEntry::new(VCardProperty::Gramgender)
                                            .with_value(gender),
                                    );
                                }
                            }
                            Key::Property(JSContactProperty::Pronouns) => {
                                for (name, value) in value.into_expanded_object() {
                                    let mut entry = VCardEntry::new(VCardProperty::Pronouns);

                                    for (prop, value) in value.into_expanded_object() {
                                        match prop {
                                            Key::Property(JSContactProperty::Pronouns) => {
                                                if let Ok(pronoun_type) = convert_value(value) {
                                                    entry.values = vec![pronoun_type];
                                                }
                                            }
                                            Key::Property(JSContactProperty::Pref) => {
                                                if let Some(pref) = value.as_i64() {
                                                    entry.add_param(VCardParameter::Pref(
                                                        pref as u32,
                                                    ));
                                                }
                                            }
                                            _ => {
                                                js_props.insert(
                                                    &[
                                                        property.to_string().as_ref(),
                                                        key.to_string().as_ref(),
                                                        name.to_string().as_ref(),
                                                        prop.to_string().as_ref(),
                                                    ],
                                                    value,
                                                );
                                            }
                                        }
                                    }

                                    if !entry.values.is_empty() {
                                        vcard.entries.push(entry.with_param(
                                            VCardParameter::PropId(name.into_string()),
                                        ));
                                    }
                                }
                            }
                            _ => {
                                js_props.insert(
                                    &[property.to_string().as_ref(), key.to_string().as_ref()],
                                    value,
                                );
                            }
                        }
                    }
                }
                JSContactProperty::Nicknames => {
                    for (name, value) in value.into_expanded_object() {
                        let mut entry = VCardEntry::new(VCardProperty::Nickname);

                        for (prop, value) in value.into_expanded_object() {
                            match prop {
                                Key::Property(JSContactProperty::Name) => {
                                    if let Ok(name) = convert_value(value) {
                                        entry.values = vec![name];
                                    }
                                }
                                Key::Property(JSContactProperty::Pref) => {
                                    if let Some(pref) = value.as_i64() {
                                        entry.add_param(VCardParameter::Pref(pref as u32));
                                    }
                                }
                                Key::Property(JSContactProperty::Contexts) => {
                                    if let Some(types) = convert_types(value) {
                                        entry.params.push(VCardParameter::Type(types));
                                    }
                                }
                                _ => {
                                    js_props.insert(
                                        &[
                                            property.to_string().as_ref(),
                                            name.to_string().as_ref(),
                                            prop.to_string().as_ref(),
                                        ],
                                        value,
                                    );
                                }
                            }
                        }

                        if !entry.values.is_empty() {
                            vcard
                                .entries
                                .push(entry.with_param(VCardParameter::PropId(name.into_string())));
                        }
                    }
                }
                JSContactProperty::Media => {
                    for (name, value) in value.into_expanded_object() {
                        if let Some(entry_type) = map_kind(
                            &value,
                            [
                                (JSContactKind::Photo, VCardProperty::Photo),
                                (JSContactKind::Logo, VCardProperty::Logo),
                                (JSContactKind::Sound, VCardProperty::Sound),
                            ],
                        ) {
                            let mut entry = VCardEntry::new(entry_type);

                            for (prop, value) in value.into_expanded_object() {
                                match prop {
                                    Key::Property(JSContactProperty::Uri) => {
                                        if let Some(uri) = value.into_string() {
                                            let uri = match Data::try_parse(uri.as_bytes()) {
                                                Some(data) => VCardValue::Binary(data),
                                                None => VCardValue::Text(uri),
                                            };

                                            entry.values = vec![uri];
                                        }
                                    }
                                    Key::Property(JSContactProperty::MediaType) => {
                                        if let Some(text) = value.into_string() {
                                            entry.params.push(VCardParameter::Mediatype(text));
                                        }
                                    }
                                    Key::Property(JSContactProperty::Label) => {
                                        if let Some(text) = value.into_string() {
                                            entry.params.push(VCardParameter::Label(text));
                                        }
                                    }
                                    Key::Property(JSContactProperty::Pref) => {
                                        if let Some(pref) = value.as_i64() {
                                            entry.add_param(VCardParameter::Pref(pref as u32));
                                        }
                                    }
                                    Key::Property(JSContactProperty::Kind) => {}
                                    _ => {
                                        js_props.insert(
                                            &[
                                                property.to_string().as_ref(),
                                                name.to_string().as_ref(),
                                                prop.to_string().as_ref(),
                                            ],
                                            value,
                                        );
                                    }
                                }
                            }

                            if !entry.values.is_empty() {
                                vcard.entries.push(
                                    entry.with_param(VCardParameter::PropId(name.into_string())),
                                );
                            }
                        } else {
                            js_props.insert(
                                &[property.to_string().as_ref(), name.to_string().as_ref()],
                                value,
                            );
                        }
                    }
                }
                JSContactProperty::Addresses => {
                    for (name, value) in value.into_expanded_object() {
                        let mut params = vec![VCardParameter::PropId(name.to_string().to_string())];
                        let mut parts: Vec<Option<String>> = vec![None; 18];
                        let mut num_parts = 0;

                        for (prop, value) in value.into_expanded_object() {
                            match prop {
                                Key::Property(JSContactProperty::Components) => {
                                    for components in value.into_array().unwrap_or_default() {
                                        let mut comp_value = None;
                                        let mut comp_pos = None;

                                        let todo = "map GEO and TZ";
                                        let todo = "map to the next slot when filled (some props map to same pos)";
                                        for (key, value) in components.into_expanded_object() {
                                            match key {
                                                Key::Property(JSContactProperty::Kind) => {
                                                    comp_pos = match value {
                                                        Value::Element(JSContactValue::Kind(
                                                            JSContactKind::PostOfficeBox,
                                                        )) => 0,
                                                        Value::Element(JSContactValue::Kind(
                                                            JSContactKind::Apartment,
                                                        )) => 1,
                                                        Value::Element(JSContactValue::Kind(
                                                            JSContactKind::Name,
                                                        )) => 2,
                                                        Value::Element(JSContactValue::Kind(
                                                            JSContactKind::Locality,
                                                        )) => 3,
                                                        Value::Element(JSContactValue::Kind(
                                                            JSContactKind::Region,
                                                        )) => 4,
                                                        Value::Element(JSContactValue::Kind(
                                                            JSContactKind::Postcode,
                                                        )) => 5,
                                                        Value::Element(JSContactValue::Kind(
                                                            JSContactKind::Country,
                                                        )) => 6,
                                                        Value::Element(JSContactValue::Kind(
                                                            JSContactKind::Room,
                                                        )) => 7,
                                                        Value::Element(JSContactValue::Kind(
                                                            JSContactKind::Floor,
                                                        )) => 9,
                                                        Value::Element(JSContactValue::Kind(
                                                            JSContactKind::Number,
                                                        )) => 10,
                                                        Value::Element(JSContactValue::Kind(
                                                            JSContactKind::Building,
                                                        )) => 12,
                                                        Value::Element(JSContactValue::Kind(
                                                            JSContactKind::Block,
                                                        )) => 13,
                                                        Value::Element(JSContactValue::Kind(
                                                            JSContactKind::Subdistrict,
                                                        )) => 14,
                                                        Value::Element(JSContactValue::Kind(
                                                            JSContactKind::District,
                                                        )) => 15,
                                                        Value::Element(JSContactValue::Kind(
                                                            JSContactKind::Landmark,
                                                        )) => 16,
                                                        Value::Element(JSContactValue::Kind(
                                                            JSContactKind::Direction,
                                                        )) => 17,
                                                        _ => continue,
                                                    }
                                                    .into();
                                                }
                                                Key::Property(JSContactProperty::Value) => {
                                                    comp_value = value.into_string();
                                                }
                                                _ => {
                                                    js_props.insert(
                                                        &[
                                                            property.to_string().as_ref(),
                                                            name.to_string().as_ref(),
                                                            prop.to_string().as_ref(),
                                                            key.to_string().as_ref(),
                                                        ],
                                                        value,
                                                    );
                                                }
                                            }
                                        }

                                        if let (Some(comp_value), Some(comp_pos)) =
                                            (comp_value, comp_pos)
                                        {
                                            let part = &mut parts[comp_pos];
                                            if let Some(part) = part {
                                                *part = format!("{part},{comp_value}");
                                            } else {
                                                *part = Some(comp_value);
                                            }
                                            num_parts = num_parts.max(comp_pos + 1);
                                        }
                                    }
                                }
                                Key::Property(JSContactProperty::Label) => {
                                    if let Some(value) = value.into_string() {
                                        params.push(VCardParameter::Label(value));
                                    }
                                }
                                Key::Property(JSContactProperty::Coordinates) => {
                                    if let Some(value) = value.into_string() {
                                        params.push(VCardParameter::Geo(value));
                                    }
                                }
                                Key::Property(JSContactProperty::TimeZone) => {
                                    if let Some(value) = value.into_string() {
                                        params.push(VCardParameter::Tz(value));
                                    }
                                }
                                Key::Property(JSContactProperty::CountryCode) => {
                                    if let Some(value) = value.into_string() {
                                        params.push(VCardParameter::Cc(value));
                                    }
                                }
                                Key::Property(JSContactProperty::Pref) => {
                                    if let Some(value) = value.as_i64() {
                                        params.push(VCardParameter::Pref(value as u32));
                                    }
                                }
                                Key::Property(JSContactProperty::PhoneticSystem) => {
                                    if let Ok(phonetic_system) = VCardPhonetic::try_from(value) {
                                        params.push(VCardParameter::Phonetic(phonetic_system));
                                    }
                                }
                                Key::Property(JSContactProperty::PhoneticScript) => {
                                    if let Some(value) = value.into_string() {
                                        params.push(VCardParameter::Script(value));
                                    }
                                }
                                Key::Property(JSContactProperty::Contexts) => {
                                    if let Some(types) = convert_types(value) {
                                        params.push(VCardParameter::Type(types));
                                    }
                                }
                                _ => {
                                    js_props.insert(
                                        &[
                                            property.to_string().as_ref(),
                                            name.to_string().as_ref(),
                                            prop.to_string().as_ref(),
                                        ],
                                        value,
                                    );
                                }
                            }
                        }

                        let mut n = String::with_capacity(16);
                        for (pos, part) in parts.into_iter().take(num_parts).enumerate() {
                            if pos > 0 {
                                n.push(';');
                            }
                            if let Some(part) = part {
                                n.push_str(&part);
                            }
                        }

                        vcard.entries.push(
                            VCardEntry::new(VCardProperty::Adr)
                                .with_params(params)
                                .with_value(n),
                        );
                    }
                }
                JSContactProperty::Organizations => {
                    for (name, value) in value.into_expanded_object() {
                        let mut entry = VCardEntry::new(VCardProperty::Org);

                        for (prop, value) in value.into_expanded_object() {
                            match prop {
                                Key::Property(JSContactProperty::Name) => {
                                    if let Ok(name) = convert_value(value) {
                                        if entry.values.is_empty() {
                                            entry.values = vec![name];
                                        } else {
                                            entry.values.insert(0, name);
                                        }
                                    }
                                }
                                Key::Property(JSContactProperty::Units) => {
                                    for (index, item) in value
                                        .into_array()
                                        .unwrap_or_default()
                                        .into_iter()
                                        .enumerate()
                                    {
                                        for (key, value) in item.into_expanded_object() {
                                            match key {
                                                Key::Property(JSContactProperty::Name) => {
                                                    if let Ok(name) = convert_value(value) {
                                                        entry.values.push(name);
                                                    }
                                                }
                                                _ => {
                                                    js_props.insert(
                                                        &[
                                                            property.to_string().as_ref(),
                                                            name.to_string().as_ref(),
                                                            prop.to_string().as_ref(),
                                                            index.to_string().as_ref(),
                                                            key.to_string().as_ref(),
                                                        ],
                                                        value,
                                                    );
                                                }
                                            }
                                        }
                                    }
                                }
                                Key::Property(JSContactProperty::SortAs) => {
                                    if let Some(sort_as) = value.into_string() {
                                        entry.add_param(VCardParameter::SortAs(sort_as));
                                    }
                                }
                                Key::Property(JSContactProperty::Contexts) => {
                                    if let Some(types) = convert_types(value) {
                                        entry.params.push(VCardParameter::Type(types));
                                    }
                                }
                                _ => {
                                    js_props.insert(
                                        &[
                                            property.to_string().as_ref(),
                                            name.to_string().as_ref(),
                                            prop.to_string().as_ref(),
                                        ],
                                        value,
                                    );
                                }
                            }
                        }

                        if !entry.values.is_empty() {
                            vcard
                                .entries
                                .push(entry.with_param(VCardParameter::PropId(name.into_string())));
                        }
                    }
                }
                JSContactProperty::Members => {
                    for (name, set) in value.into_expanded_boolean_set() {
                        if set {
                            vcard.entries.push(
                                VCardEntry::new(VCardProperty::Member)
                                    .with_value(name.into_string()),
                            );
                        }
                    }
                }
                JSContactProperty::Keywords => {
                    let mut keywords = Vec::new();
                    for (name, set) in value.into_expanded_boolean_set() {
                        if set {
                            keywords.push(VCardValue::Text(name.into_string()));
                        }
                    }
                    if !keywords.is_empty() {
                        vcard
                            .entries
                            .push(VCardEntry::new(VCardProperty::Categories).with_values(keywords));
                    }
                }
                JSContactProperty::Titles => {
                    for (name, value) in value.into_expanded_object() {
                        if let Some(entry_type) = map_kind(
                            &value,
                            [
                                (JSContactKind::Title, VCardProperty::Title),
                                (JSContactKind::Role, VCardProperty::Role),
                            ],
                        ) {
                            let mut entry = VCardEntry::new(entry_type);

                            for (prop, value) in value.into_expanded_object() {
                                match prop {
                                    Key::Property(JSContactProperty::Name) => {
                                        if let Ok(name) = convert_value(value) {
                                            entry.values = vec![name];
                                        }
                                    }
                                    Key::Property(JSContactProperty::Pref) => {
                                        if let Some(pref) = value.as_i64() {
                                            entry.add_param(VCardParameter::Pref(pref as u32));
                                        }
                                    }
                                    Key::Property(JSContactProperty::Contexts) => {
                                        if let Some(types) = convert_types(value) {
                                            entry.params.push(VCardParameter::Type(types));
                                        }
                                    }
                                    Key::Property(JSContactProperty::Kind) => {}
                                    _ => {
                                        let todo = "organizationId";
                                        js_props.insert(
                                            &[
                                                property.to_string().as_ref(),
                                                name.to_string().as_ref(),
                                                prop.to_string().as_ref(),
                                            ],
                                            value,
                                        );
                                    }
                                }
                            }

                            if !entry.values.is_empty() {
                                vcard.entries.push(
                                    entry.with_param(VCardParameter::PropId(name.into_string())),
                                );
                            }
                        } else {
                            js_props.insert(
                                &[property.to_string().as_ref(), name.to_string().as_ref()],
                                value,
                            );
                        }
                    }
                }
                JSContactProperty::Emails => {
                    for (name, value) in value.into_expanded_object() {
                        let mut entry = VCardEntry::new(VCardProperty::Email);

                        for (prop, value) in value.into_expanded_object() {
                            match prop {
                                Key::Property(JSContactProperty::Address) => {
                                    if let Some(email) = value.into_string() {
                                        entry.values = vec![VCardValue::Text(email)];
                                    }
                                }
                                Key::Property(JSContactProperty::Contexts) => {
                                    if let Some(types) = convert_types(value) {
                                        entry.params.push(VCardParameter::Type(types));
                                    }
                                }
                                Key::Property(JSContactProperty::Pref) => {
                                    if let Some(pref) = value.as_i64() {
                                        entry.add_param(VCardParameter::Pref(pref as u32));
                                    }
                                }
                                Key::Property(JSContactProperty::Label) => {
                                    if let Some(label) = value.into_string() {
                                        entry.params.push(VCardParameter::Label(label));
                                    }
                                }
                                _ => {
                                    js_props.insert(
                                        &[
                                            property.to_string().as_ref(),
                                            name.to_string().as_ref(),
                                            prop.to_string().as_ref(),
                                        ],
                                        value,
                                    );
                                }
                            }
                        }

                        if !entry.values.is_empty() {
                            vcard
                                .entries
                                .push(entry.with_param(VCardParameter::PropId(name.into_string())));
                        }
                    }
                }
                JSContactProperty::OnlineServices => {
                    for (name, value) in value.into_expanded_object() {
                        let mut entry = VCardEntry::new(VCardProperty::Socialprofile);

                        for (prop, value) in value.into_expanded_object() {
                            match prop {
                                Key::Property(JSContactProperty::Uri) => {
                                    if let Some(service) = value.into_string() {
                                        entry.values = vec![VCardValue::Text(service)];
                                    }
                                }
                                Key::Property(JSContactProperty::Service) => {
                                    if let Some(service) = value.into_string() {
                                        entry.add_param(VCardParameter::ServiceType(service));
                                    }
                                }
                                Key::Property(JSContactProperty::User) => {
                                    if let Some(username) = value.into_string() {
                                        entry.add_param(VCardParameter::Username(username));
                                    }
                                }
                                Key::Property(JSContactProperty::Contexts) => {
                                    if let Some(types) = convert_types(value) {
                                        entry.params.push(VCardParameter::Type(types));
                                    }
                                }
                                Key::Property(JSContactProperty::Label) => {
                                    if let Some(label) = value.into_string() {
                                        entry.add_param(VCardParameter::Label(label));
                                    }
                                }
                                Key::Property(JSContactProperty::Pref) => {
                                    if let Some(pref) = value.as_i64() {
                                        entry.add_param(VCardParameter::Pref(pref as u32));
                                    }
                                }
                                _ => {
                                    js_props.insert(
                                        &[
                                            property.to_string().as_ref(),
                                            name.to_string().as_ref(),
                                            prop.to_string().as_ref(),
                                        ],
                                        value,
                                    );
                                }
                            }
                        }

                        if !entry.values.is_empty() {
                            vcard
                                .entries
                                .push(entry.with_param(VCardParameter::PropId(name.into_string())));
                        }
                    }
                }
                JSContactProperty::Phones => {
                    for (name, value) in value.into_expanded_object() {
                        let mut entry = VCardEntry::new(VCardProperty::Tel);
                        let mut types = vec![];

                        for (prop, value) in value.into_expanded_object() {
                            match prop {
                                Key::Property(JSContactProperty::Number) => {
                                    if let Some(email) = value.into_string() {
                                        entry.values = vec![VCardValue::Text(email)];
                                    }
                                }
                                Key::Property(
                                    JSContactProperty::Contexts | JSContactProperty::Features,
                                ) => {
                                    if let Some(types_) = convert_types(value) {
                                        if types.is_empty() {
                                            types = types_;
                                        } else {
                                            types.extend(types_);
                                        }
                                    }
                                }
                                Key::Property(JSContactProperty::Pref) => {
                                    if let Some(pref) = value.as_i64() {
                                        entry.add_param(VCardParameter::Pref(pref as u32));
                                    }
                                }
                                Key::Property(JSContactProperty::Label) => {
                                    if let Some(label) = value.into_string() {
                                        entry.params.push(VCardParameter::Label(label));
                                    }
                                }
                                _ => {
                                    js_props.insert(
                                        &[
                                            property.to_string().as_ref(),
                                            name.to_string().as_ref(),
                                            prop.to_string().as_ref(),
                                        ],
                                        value,
                                    );
                                }
                            }
                        }

                        if !entry.values.is_empty() {
                            if !types.is_empty() {
                                entry.params.push(VCardParameter::Type(types));
                            }
                            vcard
                                .entries
                                .push(entry.with_param(VCardParameter::PropId(name.into_string())));
                        }
                    }
                }
                JSContactProperty::PreferredLanguages => {
                    for (name, value) in value.into_expanded_object() {
                        let mut entry = VCardEntry::new(VCardProperty::Lang);

                        for (prop, value) in value.into_expanded_object() {
                            match prop {
                                Key::Property(JSContactProperty::Language) => {
                                    if let Some(lang) = value.into_string() {
                                        entry.values = vec![VCardValue::Text(lang)];
                                    }
                                }
                                Key::Property(JSContactProperty::Pref) => {
                                    if let Some(pref) = value.as_i64() {
                                        entry.add_param(VCardParameter::Pref(pref as u32));
                                    }
                                }
                                Key::Property(JSContactProperty::Contexts) => {
                                    if let Some(types) = convert_types(value) {
                                        entry.params.push(VCardParameter::Type(types));
                                    }
                                }
                                Key::Property(JSContactProperty::Label) => {
                                    if let Some(label) = value.into_string() {
                                        entry.params.push(VCardParameter::Label(label));
                                    }
                                }
                                _ => {
                                    js_props.insert(
                                        &[
                                            property.to_string().as_ref(),
                                            name.to_string().as_ref(),
                                            prop.to_string().as_ref(),
                                        ],
                                        value,
                                    );
                                }
                            }
                        }

                        if !entry.values.is_empty() {
                            vcard
                                .entries
                                .push(entry.with_param(VCardParameter::PropId(name.into_string())));
                        }
                    }
                }
                JSContactProperty::Calendars => {
                    for (name, value) in value.into_expanded_object() {
                        if let Some(entry_type) = map_kind(
                            &value,
                            [
                                (JSContactKind::FreeBusy, VCardProperty::Fburl),
                                (JSContactKind::Calendar, VCardProperty::Caluri),
                            ],
                        ) {
                            let mut entry = VCardEntry::new(entry_type);

                            for (prop, value) in value.into_expanded_object() {
                                match prop {
                                    Key::Property(JSContactProperty::Uri) => {
                                        if let Ok(name) = convert_value(value) {
                                            entry.values = vec![name];
                                        }
                                    }
                                    Key::Property(JSContactProperty::MediaType) => {
                                        if let Some(text) = value.into_string() {
                                            entry.params.push(VCardParameter::Mediatype(text));
                                        }
                                    }
                                    Key::Property(JSContactProperty::Label) => {
                                        if let Some(text) = value.into_string() {
                                            entry.params.push(VCardParameter::Label(text));
                                        }
                                    }
                                    Key::Property(JSContactProperty::Pref) => {
                                        if let Some(pref) = value.as_i64() {
                                            entry.add_param(VCardParameter::Pref(pref as u32));
                                        }
                                    }
                                    Key::Property(JSContactProperty::Contexts) => {
                                        if let Some(types) = convert_types(value) {
                                            entry.params.push(VCardParameter::Type(types));
                                        }
                                    }
                                    Key::Property(JSContactProperty::Kind) => {}
                                    _ => {
                                        js_props.insert(
                                            &[
                                                property.to_string().as_ref(),
                                                name.to_string().as_ref(),
                                                prop.to_string().as_ref(),
                                            ],
                                            value,
                                        );
                                    }
                                }
                            }

                            if !entry.values.is_empty() {
                                vcard.entries.push(
                                    entry.with_param(VCardParameter::PropId(name.into_string())),
                                );
                            }
                        } else {
                            js_props.insert(
                                &[property.to_string().as_ref(), name.to_string().as_ref()],
                                value,
                            );
                        }
                    }
                }
                JSContactProperty::SchedulingAddresses
                | JSContactProperty::CryptoKeys
                | JSContactProperty::Links => {
                    let vcard_property = match property {
                        JSContactProperty::SchedulingAddresses => VCardProperty::Caladruri,
                        JSContactProperty::CryptoKeys => VCardProperty::Key,
                        JSContactProperty::Links => VCardProperty::Url,
                        _ => unreachable!(),
                    };

                    for (name, value) in value.into_expanded_object() {
                        let mut entry = VCardEntry::new(vcard_property.clone());

                        for (prop, value) in value.into_expanded_object() {
                            match prop {
                                Key::Property(JSContactProperty::Uri) => {
                                    if let Ok(name) = convert_value(value) {
                                        entry.values = vec![name];
                                    }
                                }
                                Key::Property(JSContactProperty::MediaType) => {
                                    if let Some(text) = value.into_string() {
                                        entry.params.push(VCardParameter::Mediatype(text));
                                    }
                                }
                                Key::Property(JSContactProperty::Label) => {
                                    if let Some(text) = value.into_string() {
                                        entry.params.push(VCardParameter::Label(text));
                                    }
                                }
                                Key::Property(JSContactProperty::Pref) => {
                                    if let Some(pref) = value.as_i64() {
                                        entry.add_param(VCardParameter::Pref(pref as u32));
                                    }
                                }
                                Key::Property(JSContactProperty::Contexts) => {
                                    if let Some(types) = convert_types(value) {
                                        entry.params.push(VCardParameter::Type(types));
                                    }
                                }
                                Key::Property(JSContactProperty::Kind) => {
                                    if matches!(
                                        value,
                                        Value::Element(JSContactValue::Kind(
                                            JSContactKind::Contact
                                        ))
                                    ) {
                                        entry.name = VCardProperty::ContactUri;
                                    }
                                }
                                _ => {
                                    js_props.insert(
                                        &[
                                            property.to_string().as_ref(),
                                            name.to_string().as_ref(),
                                            prop.to_string().as_ref(),
                                        ],
                                        value,
                                    );
                                }
                            }
                        }

                        if !entry.values.is_empty() {
                            vcard
                                .entries
                                .push(entry.with_param(VCardParameter::PropId(name.into_string())));
                        }
                    }
                }
                JSContactProperty::Notes => {
                    for (name, value) in value.into_expanded_object() {
                        let mut entry = VCardEntry::new(VCardProperty::Note);

                        for (prop, value) in value.into_expanded_object() {
                            match prop {
                                Key::Property(JSContactProperty::Note) => {
                                    if let Some(text) = value.into_string() {
                                        entry.values = vec![VCardValue::Text(text)];
                                    }
                                }
                                Key::Property(JSContactProperty::Created) => {
                                    if let Value::Element(JSContactValue::Timestamp(t)) = value {
                                        entry.params.push(VCardParameter::Created(t));
                                    }
                                }
                                Key::Property(JSContactProperty::Author) => {
                                    for (name, value) in value.into_expanded_object() {
                                        match name {
                                            Key::Property(JSContactProperty::Name) => {
                                                if let Some(name) = value.into_string() {
                                                    entry.add_param(VCardParameter::AuthorName(
                                                        name,
                                                    ));
                                                }
                                            }
                                            Key::Property(JSContactProperty::Uri) => {
                                                if let Some(uri) = value.into_string() {
                                                    entry.add_param(VCardParameter::Author(uri));
                                                }
                                            }
                                            _ => {
                                                js_props.insert(
                                                    &[
                                                        property.to_string().as_ref(),
                                                        name.to_string().as_ref(),
                                                        prop.to_string().as_ref(),
                                                        name.to_string().as_ref(),
                                                    ],
                                                    value,
                                                );
                                            }
                                        }
                                    }
                                }

                                _ => {
                                    js_props.insert(
                                        &[
                                            property.to_string().as_ref(),
                                            name.to_string().as_ref(),
                                            prop.to_string().as_ref(),
                                        ],
                                        value,
                                    );
                                }
                            }
                        }

                        if !entry.values.is_empty() {
                            vcard
                                .entries
                                .push(entry.with_param(VCardParameter::PropId(name.into_string())));
                        }
                    }
                }
                JSContactProperty::PersonalInfo => {
                    for (name, value) in value.into_expanded_object() {
                        if let Some(entry_type) = map_kind(
                            &value,
                            [
                                (JSContactKind::Expertise, VCardProperty::Expertise),
                                (JSContactKind::Hobby, VCardProperty::Hobby),
                                (JSContactKind::Interest, VCardProperty::Interest),
                            ],
                        ) {
                            let mut entry = VCardEntry::new(entry_type);

                            for (prop, value) in value.into_expanded_object() {
                                match prop {
                                    Key::Property(JSContactProperty::Value) => {
                                        if let Ok(name) = convert_value(value) {
                                            entry.values = vec![name];
                                        }
                                    }
                                    Key::Property(JSContactProperty::Label) => {
                                        if let Some(label) = value.into_string() {
                                            entry.add_param(VCardParameter::Label(label));
                                        }
                                    }
                                    Key::Property(JSContactProperty::ListAs) => {
                                        if let Some(pref) = value.as_i64() {
                                            entry.add_param(VCardParameter::Index(pref as u32));
                                        }
                                    }
                                    Key::Property(JSContactProperty::Level) => {
                                        if let Ok(level) = VCardLevel::try_from(value) {
                                            entry.params.push(VCardParameter::Level(level));
                                        }
                                    }
                                    Key::Property(JSContactProperty::Kind) => {}
                                    _ => {
                                        js_props.insert(
                                            &[
                                                property.to_string().as_ref(),
                                                name.to_string().as_ref(),
                                                prop.to_string().as_ref(),
                                            ],
                                            value,
                                        );
                                    }
                                }
                            }

                            if !entry.values.is_empty() {
                                vcard.entries.push(
                                    entry.with_param(VCardParameter::PropId(name.into_string())),
                                );
                            }
                        } else {
                            js_props.insert(
                                &[property.to_string().as_ref(), name.to_string().as_ref()],
                                value,
                            );
                        }
                    }
                }
                JSContactProperty::RelatedTo => {
                    for (name, value) in value.into_expanded_object() {
                        let mut entry = VCardEntry::new(VCardProperty::Related)
                            .with_value(name.to_string().into_owned());
                        let mut types = vec![];

                        for (prop, value) in value.into_expanded_object() {
                            if let Key::Property(JSContactProperty::Relation) = prop {
                                for (typ, set) in value.into_expanded_boolean_set() {
                                    if set {
                                        let typ = typ.to_string();
                                        match VCardType::try_from(typ.as_ref().as_bytes()) {
                                            Ok(typ) => types.push(typ),
                                            Err(_) => {
                                                types.push(VCardType::Other(typ.into_owned()));
                                            }
                                        }
                                    }
                                }
                            } else {
                                js_props.insert(
                                    &[
                                        property.to_string().as_ref(),
                                        name.to_string().as_ref(),
                                        prop.to_string().as_ref(),
                                    ],
                                    value,
                                );
                            }
                        }

                        if !types.is_empty() {
                            entry.params.push(VCardParameter::Type(types));
                        }

                        vcard.entries.push(entry);
                    }
                }
                JSContactProperty::Localizations => {
                    localizations = value.into_object();
                }
                JSContactProperty::VCard => {
                    properties = value.into_object();
                }
                _ => {
                    js_props.insert(&[property.to_string().as_ref()], value);
                }
            }
        }

        for (ptr, value) in js_props.0 {
            vcard.entries.push(
                VCardEntry::new(VCardProperty::Jsprop)
                    .with_param(VCardParameter::Jsptr(ptr))
                    .with_value(serde_json::to_string(&value).unwrap_or_default()),
            );
        }

        Some(vcard)
    }
}

enum ParamValue<'x> {
    Text(Cow<'x, str>),
    Number(i64),
    Bool(bool),
}

impl<'x> ParamValue<'x> {
    fn try_from_value<P: Property, E: Element>(value: Value<'x, P, E>) -> Option<Self> {
        match value {
            Value::Str(s) => Some(Self::Text(s)),
            Value::Number(n) => Some(Self::Number(n.cast_to_i64())),
            Value::Bool(b) => Some(Self::Bool(b)),
            Value::Element(e) => Some(Self::Text(e.to_cow().to_string().into())),
            _ => None,
        }
    }

    fn into_string(self) -> Cow<'x, str> {
        match self {
            Self::Text(s) => s,
            Self::Number(n) => n.to_string().into(),
            Self::Bool(b) => b.to_string().into(),
        }
    }

    fn into_number(self) -> Result<i64, Self> {
        match self {
            Self::Number(n) => Ok(n),
            Self::Text(s) => s.parse().map_err(|_| Self::Text(s)),
            _ => Err(self),
        }
    }

    fn into_bool(self) -> Result<bool, Self> {
        match self {
            Self::Bool(b) => Ok(b),
            _ => Err(self),
        }
    }
}

impl VCardEntry {
    fn import_jcard_params(&mut self, params: Value<'_, JSContactProperty, JSContactValue>) {
        for (key, value) in params.into_expanded_object() {
            let values = match value {
                Value::Array(values) => values
                    .into_iter()
                    .filter_map(ParamValue::try_from_value)
                    .collect::<Vec<_>>(),
                value => {
                    if let Some(value) = ParamValue::try_from_value(value) {
                        vec![value]
                    } else {
                        continue;
                    }
                }
            };
            if values.is_empty() {
                continue;
            }

            let key = key.to_string();
            let Some(param) = VCardParameterName::try_parse(key.as_ref()) else {
                self.params.push(VCardParameter::Other(
                    [key.into_owned()]
                        .into_iter()
                        .chain(values.into_iter().map(|v| v.into_string().into_owned()))
                        .collect(),
                ));
                continue;
            };
            let mut values = values.into_iter();

            let param = match param {
                VCardParameterName::Language => VCardParameter::Language(
                    values.map(|v| v.into_string().into_owned()).next().unwrap(),
                ),
                VCardParameterName::Value => VCardParameter::Value(
                    values
                        .map(|v| {
                            let v = v.into_string();
                            VCardValueType::try_from(v.as_bytes())
                                .unwrap_or_else(|_| VCardValueType::Other(v.into_owned()))
                        })
                        .collect(),
                ),
                VCardParameterName::Pref => values
                    .next()
                    .map(|v| match v.into_number() {
                        Ok(n) => VCardParameter::Pref(n as u32),
                        Err(v) => VCardParameter::Other(vec![
                            VCardParameterName::Pref.as_str().to_string(),
                            v.into_string().into_owned(),
                        ]),
                    })
                    .unwrap(),
                VCardParameterName::Altid => VCardParameter::Altid(
                    values.map(|v| v.into_string().into_owned()).next().unwrap(),
                ),
                VCardParameterName::Pid => {
                    VCardParameter::Pid(values.map(|v| v.into_string().into_owned()).collect())
                }
                VCardParameterName::Type => VCardParameter::Type(
                    values
                        .map(|v| {
                            let v = v.into_string();
                            VCardType::try_from(v.as_bytes())
                                .unwrap_or_else(|_| VCardType::Other(v.into_owned()))
                        })
                        .collect(),
                ),
                VCardParameterName::Mediatype => VCardParameter::Mediatype(
                    values.map(|v| v.into_string().into_owned()).next().unwrap(),
                ),
                VCardParameterName::Calscale => {
                    let value = values.next().map(|v| v.into_string()).unwrap();
                    CalendarScale::try_from(value.as_bytes())
                        .map(VCardParameter::Calscale)
                        .unwrap_or_else(|_| {
                            VCardParameter::Other(vec![
                                VCardParameterName::Calscale.as_str().to_string(),
                                value.into_owned(),
                            ])
                        })
                }
                VCardParameterName::SortAs => VCardParameter::SortAs(
                    values.map(|v| v.into_string().into_owned()).next().unwrap(),
                ),
                VCardParameterName::Geo => VCardParameter::Geo(
                    values.map(|v| v.into_string().into_owned()).next().unwrap(),
                ),
                VCardParameterName::Tz => {
                    VCardParameter::Tz(values.map(|v| v.into_string().into_owned()).next().unwrap())
                }
                VCardParameterName::Index => values
                    .next()
                    .map(|v| match v.into_number() {
                        Ok(n) => VCardParameter::Index(n as u32),
                        Err(v) => VCardParameter::Other(vec![
                            VCardParameterName::Index.as_str().to_string(),
                            v.into_string().into_owned(),
                        ]),
                    })
                    .unwrap(),
                VCardParameterName::Level => {
                    let value = values.next().map(|v| v.into_string()).unwrap();
                    VCardLevel::try_from(value.as_bytes())
                        .map(VCardParameter::Level)
                        .unwrap_or_else(|_| {
                            VCardParameter::Other(vec![
                                VCardParameterName::Level.as_str().to_string(),
                                value.into_owned(),
                            ])
                        })
                }
                VCardParameterName::Group => {
                    self.group = values.map(|v| v.into_string().into_owned()).next();
                    continue;
                }
                VCardParameterName::Cc => {
                    VCardParameter::Cc(values.map(|v| v.into_string().into_owned()).next().unwrap())
                }
                VCardParameterName::Author => VCardParameter::Author(
                    values.map(|v| v.into_string().into_owned()).next().unwrap(),
                ),
                VCardParameterName::AuthorName => VCardParameter::AuthorName(
                    values.map(|v| v.into_string().into_owned()).next().unwrap(),
                ),
                VCardParameterName::Created => {
                    let value = values.next().map(|v| v.into_string()).unwrap();
                    Timestamp::try_from(value.as_bytes())
                        .map(|v| VCardParameter::Created(v.0))
                        .unwrap_or_else(|_| {
                            VCardParameter::Other(vec![
                                VCardParameterName::Created.as_str().to_string(),
                                value.into_owned(),
                            ])
                        })
                }
                VCardParameterName::Derived => values
                    .next()
                    .map(|v| match v.into_bool() {
                        Ok(n) => VCardParameter::Derived(n),
                        Err(v) => VCardParameter::Other(vec![
                            VCardParameterName::Derived.as_str().to_string(),
                            v.into_string().into_owned(),
                        ]),
                    })
                    .unwrap(),
                VCardParameterName::Label => VCardParameter::Label(
                    values.map(|v| v.into_string().into_owned()).next().unwrap(),
                ),
                VCardParameterName::Phonetic => {
                    let value = values.next().map(|v| v.into_string()).unwrap();
                    VCardPhonetic::try_from(value.as_bytes())
                        .map(VCardParameter::Phonetic)
                        .unwrap_or_else(|_| {
                            VCardParameter::Other(vec![
                                VCardParameterName::Phonetic.as_str().to_string(),
                                value.into_owned(),
                            ])
                        })
                }
                VCardParameterName::PropId => VCardParameter::PropId(
                    values.map(|v| v.into_string().into_owned()).next().unwrap(),
                ),
                VCardParameterName::Script => VCardParameter::Script(
                    values.map(|v| v.into_string().into_owned()).next().unwrap(),
                ),
                VCardParameterName::ServiceType => VCardParameter::ServiceType(
                    values.map(|v| v.into_string().into_owned()).next().unwrap(),
                ),
                VCardParameterName::Username => VCardParameter::Username(
                    values.map(|v| v.into_string().into_owned()).next().unwrap(),
                ),
                VCardParameterName::Jsptr => VCardParameter::Jsptr(
                    values.map(|v| v.into_string().into_owned()).next().unwrap(),
                ),
                _ => unreachable!(),
            };

            self.params.push(param);
        }
    }
}

pub fn convert_anniversary(
    value: Value<'_, JSContactProperty, JSContactValue>,
) -> Result<(PartialDateTime, Option<CalendarScale>), Value<'_, JSContactProperty, JSContactValue>>
{
    let mut date = PartialDateTime::default();
    let mut calendar_scale = None;
    let Some(object) = value.as_object() else {
        return Err(value);
    };

    for (key, value) in object.as_vec() {
        match key {
            Key::Property(JSContactProperty::Day) => {
                if let Value::Number(day) = value {
                    date.day = Some(day.cast_to_i64() as u8);
                }
            }
            Key::Property(JSContactProperty::Month) => {
                if let Value::Number(month) = value {
                    date.month = Some(month.cast_to_i64() as u8);
                }
            }
            Key::Property(JSContactProperty::Year) => {
                if let Value::Number(year) = value {
                    date.year = Some(year.cast_to_i64() as u16);
                }
            }
            Key::Property(JSContactProperty::CalendarScale) => {
                if let Value::Element(JSContactValue::CalendarScale(scale)) = value {
                    calendar_scale = Some(scale.clone());
                }
            }
            Key::Property(JSContactProperty::Utc) => {
                if let Value::Element(JSContactValue::Timestamp(timestamp)) = value {
                    return Ok((PartialDateTime::from_utc_timestamp(*timestamp), None));
                }
            }
            _ => {}
        }
    }

    if date.year.is_some() || date.month.is_some() || date.day.is_some() {
        Ok((date, calendar_scale))
    } else {
        Err(value)
    }
}

fn convert_value(
    value: Value<'_, JSContactProperty, JSContactValue>,
) -> Result<VCardValue, Value<'_, JSContactProperty, JSContactValue>> {
    match value {
        Value::Element(e) => match e {
            JSContactValue::Timestamp(t) => Ok(VCardValue::PartialDateTime(
                PartialDateTime::from_utc_timestamp(t),
            )),
            JSContactValue::GrammaticalGender(g) => Ok(VCardValue::GramGender(match g {
                JSContactGrammaticalGender::Animate => VCardGramGender::Animate,
                JSContactGrammaticalGender::Common => VCardGramGender::Common,
                JSContactGrammaticalGender::Feminine => VCardGramGender::Feminine,
                JSContactGrammaticalGender::Inanimate => VCardGramGender::Inanimate,
                JSContactGrammaticalGender::Masculine => VCardGramGender::Masculine,
                JSContactGrammaticalGender::Neuter => VCardGramGender::Neuter,
            })),
            JSContactValue::Kind(k) => match k {
                JSContactKind::Individual => Ok(VCardValue::Kind(VCardKind::Individual)),
                JSContactKind::Group => Ok(VCardValue::Kind(VCardKind::Group)),
                JSContactKind::Location => Ok(VCardValue::Kind(VCardKind::Location)),
                JSContactKind::Org => Ok(VCardValue::Kind(VCardKind::Org)),
                JSContactKind::Application => Ok(VCardValue::Kind(VCardKind::Application)),
                JSContactKind::Device => Ok(VCardValue::Kind(VCardKind::Device)),
                _ => Err(Value::Element(JSContactValue::Kind(k))),
            },
            JSContactValue::Level(_)
            | JSContactValue::Type(_)
            | JSContactValue::Relation(_)
            | JSContactValue::PhoneticSystem(_)
            | JSContactValue::CalendarScale(_) => Err(Value::Element(e)),
        },
        Value::Str(s) => Ok(VCardValue::Text(s.into_owned())),
        Value::Bool(b) => Ok(VCardValue::Boolean(b)),
        Value::Number(n) => match n.try_cast_to_i64() {
            Ok(i) => Ok(VCardValue::Integer(i)),
            Err(f) => Ok(VCardValue::Float(f)),
        },
        value => Err(value),
    }
}

fn convert_types(value: Value<'_, JSContactProperty, JSContactValue>) -> Option<Vec<VCardType>> {
    let mut types = Vec::new();
    for (typ, set) in value.into_expanded_boolean_set() {
        if set {
            let typ = typ.to_string();
            match VCardType::try_from(typ.as_ref().as_bytes()) {
                Ok(typ) => types.push(typ),
                Err(_) => {
                    types.push(VCardType::Other(typ.into_owned()));
                }
            }
        }
    }
    if !types.is_empty() {
        Some(types)
    } else {
        None
    }
}

fn map_kind<T>(
    value: &Value<'_, JSContactProperty, JSContactValue>,
    types: impl IntoIterator<Item = (JSContactKind, T)>,
) -> Option<T> {
    value
        .as_object()
        .and_then(|obj| obj.get(&Key::Property(JSContactProperty::Kind)))
        .and_then(|v| match v {
            Value::Element(JSContactValue::Kind(kind)) => {
                types.into_iter().find_map(|(js_kind, vcard_property)| {
                    if js_kind == *kind {
                        Some(vcard_property)
                    } else {
                        None
                    }
                })
            }

            _ => None,
        })
}

impl TryFrom<Value<'_, JSContactProperty, JSContactValue>> for VCardPhonetic {
    type Error = ();

    fn try_from(value: Value<'_, JSContactProperty, JSContactValue>) -> Result<Self, Self::Error> {
        match value {
            Value::Element(JSContactValue::PhoneticSystem(system)) => Ok(match system {
                JSContactPhoneticSystem::Ipa => VCardPhonetic::Ipa,
                JSContactPhoneticSystem::Jyut => VCardPhonetic::Jyut,
                JSContactPhoneticSystem::Piny => VCardPhonetic::Piny,
                JSContactPhoneticSystem::Script => VCardPhonetic::Script,
            }),
            Value::Str(text) => Ok(VCardPhonetic::Other(text.into_owned())),
            _ => Err(()),
        }
    }
}

impl TryFrom<Value<'_, JSContactProperty, JSContactValue>> for VCardLevel {
    type Error = ();

    fn try_from(value: Value<'_, JSContactProperty, JSContactValue>) -> Result<Self, Self::Error> {
        match value {
            Value::Element(JSContactValue::Level(level)) => Ok(match level {
                JSContactLevel::High => VCardLevel::High,
                JSContactLevel::Low => VCardLevel::Low,
                JSContactLevel::Medium => VCardLevel::Medium,
            }),
            Value::Str(text) => VCardLevel::try_from(text.as_ref().as_bytes()),
            _ => Err(()),
        }
    }
}
