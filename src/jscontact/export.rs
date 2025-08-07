/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

use std::{borrow::Cow, collections::HashMap, hash::Hash, iter::Peekable, vec::IntoIter};

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
use jmap_tools::{Element, JsonPointer, JsonPointerItem, Key, Map, Property, Value};

#[derive(Default)]
#[allow(clippy::type_complexity)]
struct State<'x> {
    vcard: VCard,
    js_props: Vec<(String, Value<'x, JSContactProperty, JSContactValue>)>,
    converted_props: Vec<(
        Vec<Key<'static, JSContactProperty>>,
        Value<'x, JSContactProperty, JSContactValue>,
    )>,
    converted_props_count: usize,
    language: Option<String>,
}

impl JSContact<'_> {
    pub fn into_vcard(self) -> Option<VCard> {
        let mut state = State::default();
        let js_properties = self.0.into_object()?.into_vec();
        let mut properties = Vec::with_capacity(js_properties.len());
        let mut localized_properties: HashMap<String, Vec<_>> = HashMap::new();

        for (property, value) in js_properties {
            let Key::Property(property) = property else {
                state.insert_jsprop(&[property.to_string().as_ref()], value);
                continue;
            };
            match property {
                JSContactProperty::Localizations => {
                    for (lang, value) in value.into_expanded_object() {
                        let mut localizations: Value<'_, JSContactProperty, JSContactValue> =
                            Value::Object(Map::from(Vec::with_capacity(3)));
                        for (ptr, value) in value.into_expanded_object() {
                            if let Key::Property(JSContactProperty::Pointer(ptr)) = ptr {
                                build_path(
                                    &mut localizations,
                                    ptr.into_inner().into_iter().peekable(),
                                    value,
                                );
                            } else {
                                state.insert_jsprop(
                                    &[
                                        JSContactProperty::Localizations.to_string().as_ref(),
                                        ptr.to_string().as_ref(),
                                    ],
                                    value,
                                );
                            }
                        }
                        if let Some(localizations) =
                            localizations.into_object().filter(|x| !x.is_empty())
                        {
                            localized_properties.insert(
                                lang.into_string(),
                                localizations
                                    .into_vec()
                                    .into_iter()
                                    .filter_map(|(key, value)| {
                                        if let Key::Property(property) = key {
                                            Some((property, value))
                                        } else {
                                            None
                                        }
                                    })
                                    .collect(),
                            );
                        }
                    }
                }
                JSContactProperty::VCard => {
                    for (sub_property, value) in value.into_expanded_object() {
                        match sub_property {
                            Key::Property(JSContactProperty::ConvertedProperties) => {
                                for (ptr, value) in value.into_expanded_object() {
                                    if let Key::Property(JSContactProperty::Pointer(ptr)) = ptr {
                                        let mut keys = Vec::with_capacity(2);
                                        for item in ptr.into_iter() {
                                            match item {
                                                JsonPointerItem::Key(key) => {
                                                    let key = match &key {
                                                        Key::Borrowed(v) if v.contains('/') => v,
                                                        Key::Owned(v) if v.contains('/') => {
                                                            v.as_str()
                                                        }
                                                        _ => {
                                                            keys.push(key);
                                                            continue;
                                                        }
                                                    };
                                                    for item in JsonPointer::parse(key).into_iter()
                                                    {
                                                        keys.push(match item {
                                                            JsonPointerItem::Key(k) => k,
                                                            JsonPointerItem::Number(n) => {
                                                                Key::Owned(n.to_string())
                                                            }
                                                            JsonPointerItem::Root
                                                            | JsonPointerItem::Wildcard => continue,
                                                        });
                                                    }
                                                }
                                                JsonPointerItem::Number(v) => {
                                                    keys.push(Key::Owned(v.to_string()));
                                                }
                                                JsonPointerItem::Root
                                                | JsonPointerItem::Wildcard => (),
                                            }
                                        }

                                        state.converted_props.push((keys, value));
                                    } else {
                                        state.insert_jsprop(
                                            &[
                                                JSContactProperty::VCard.to_string().as_ref(),
                                                JSContactProperty::ConvertedProperties
                                                    .to_string()
                                                    .as_ref(),
                                                ptr.to_string().as_ref(),
                                            ],
                                            value,
                                        );
                                    }
                                }
                            }
                            Key::Property(JSContactProperty::Properties) => {
                                if let Some(value) = value.into_array() {
                                    state.import_properties(value);
                                }
                            }
                            _ => {
                                state.insert_jsprop(
                                    &[
                                        JSContactProperty::VCard.to_string().as_ref(),
                                        sub_property.to_string().as_ref(),
                                    ],
                                    value,
                                );
                            }
                        }
                    }

                    if !state.converted_props.is_empty() {
                        state.converted_props.sort_unstable_by(|a, b| a.0.cmp(&b.0));
                    }
                }

                _ => {
                    properties.push((property, value));
                }
            }
        }

        for (properties, language) in [(properties, None)].into_iter().chain(
            localized_properties
                .into_iter()
                .map(|(lang, properties)| (properties, Some(lang))),
        ) {
            state.language = language;

            for (property, value) in properties {
                match property {
                    JSContactProperty::Uid
                    | JSContactProperty::Kind
                    | JSContactProperty::Language
                    | JSContactProperty::ProdId
                    | JSContactProperty::Created
                    | JSContactProperty::Updated => match convert_value(value) {
                        Ok(value) => {
                            let vcard_property = match property {
                                JSContactProperty::Uid => VCardProperty::Uid,
                                JSContactProperty::Kind => VCardProperty::Kind,
                                JSContactProperty::Language => VCardProperty::Language,
                                JSContactProperty::ProdId => VCardProperty::Prodid,
                                JSContactProperty::Created => VCardProperty::Created,
                                JSContactProperty::Updated => VCardProperty::Rev,
                                _ => unreachable!(),
                            };
                            state.insert_vcard(
                                &[property],
                                VCardEntry::new(vcard_property).with_value(value),
                            );
                        }
                        Err(value) => {
                            state.insert_jsprop(&[property.to_string().as_ref()], value);
                        }
                    },
                    JSContactProperty::Directories => {
                        for (name, value) in value.into_expanded_object() {
                            let mut entry = VCardEntry::new(VCardProperty::OrgDirectory);
                            let mut value_type = None;

                            for (sub_property, value) in value.into_expanded_object() {
                                match sub_property {
                                    Key::Property(JSContactProperty::Kind) => {
                                        if matches!(
                                            value,
                                            Value::Element(JSContactValue::Kind(
                                                JSContactKind::Entry
                                            ))
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
                                        state.insert_jsprop(
                                            &[
                                                property.to_string().as_ref(),
                                                name.to_string().as_ref(),
                                                sub_property.to_string().as_ref(),
                                            ],
                                            value,
                                        );
                                    }
                                }
                            }

                            if let Some(value_type) = value_type {
                                entry.params.push(VCardParameter::Value(vec![value_type]));
                            }
                            state.insert_vcard(
                                &[JSContactProperty::Directories],
                                entry.with_param(VCardParameter::PropId(name.into_string())),
                            );
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

                                for (sub_property, value) in value.into_expanded_object() {
                                    match sub_property {
                                        Key::Property(JSContactProperty::Kind) => {}
                                        Key::Property(JSContactProperty::Date) => {
                                            if let Ok((date_, calscale_)) =
                                                convert_anniversary(value)
                                            {
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
                                            for (place_property, value) in
                                                value.into_expanded_object()
                                            {
                                                match place_property {
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
                                                    Key::Property(
                                                        JSContactProperty::Coordinates,
                                                    ) if vcard_place_property.is_some() => {
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
                                                        state.insert_jsprop(
                                                            &[
                                                                property.to_string().as_ref(),
                                                                name.to_string().as_ref(),
                                                                sub_property.to_string().as_ref(),
                                                                place_property.to_string().as_ref(),
                                                            ],
                                                            value,
                                                        );
                                                    }
                                                }
                                            }
                                        }
                                        _ => {
                                            state.insert_jsprop(
                                                &[
                                                    JSContactProperty::Anniversaries
                                                        .to_string()
                                                        .as_ref(),
                                                    name.to_string().as_ref(),
                                                    sub_property.to_string().as_ref(),
                                                ],
                                                value,
                                            );
                                        }
                                    }
                                }

                                // Add place if it exists
                                if let Some(place) = place {
                                    state.insert_vcard(
                                        &[
                                            JSContactProperty::Anniversaries,
                                            JSContactProperty::Place,
                                        ],
                                        place.with_param(VCardParameter::PropId(
                                            name.to_string().into_owned(),
                                        )),
                                    );
                                }

                                // Add date
                                if let Some(date) = date {
                                    let mut entry = VCardEntry::new(vcard_property);
                                    if let Some(calscale) = calscale {
                                        entry.params.push(VCardParameter::Calscale(calscale));
                                    }
                                    state.insert_vcard(
                                        &[
                                            JSContactProperty::Anniversaries,
                                            JSContactProperty::Date,
                                        ],
                                        entry
                                            .with_param(VCardParameter::PropId(name.into_string()))
                                            .with_value(VCardValue::PartialDateTime(date)),
                                    );
                                }
                            } else {
                                state.insert_jsprop(
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

                        for (sub_property, value) in value.into_expanded_object() {
                            match sub_property {
                                Key::Property(JSContactProperty::Components) => {
                                    for components in value.into_array().unwrap_or_default() {
                                        let mut comp_value = None;
                                        let mut comp_pos = None;

                                        for (component, value) in components.into_expanded_object()
                                        {
                                            match component {
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
                                                    state.insert_jsprop(
                                                        &[
                                                            JSContactProperty::Name
                                                                .to_string()
                                                                .as_ref(),
                                                            JSContactProperty::Components
                                                                .to_string()
                                                                .as_ref(),
                                                            component.to_string().as_ref(),
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
                                        state.insert_vcard(
                                            &[JSContactProperty::Name, JSContactProperty::Full],
                                            VCardEntry::new(VCardProperty::Fn).with_value(text),
                                        );
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
                                                state.insert_jsprop(
                                                    &[
                                                        JSContactProperty::Name
                                                            .to_string()
                                                            .as_ref(),
                                                        JSContactProperty::SortAs
                                                            .to_string()
                                                            .as_ref(),
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
                                    state.insert_jsprop(
                                        &[
                                            property.to_string().as_ref(),
                                            sub_property.to_string().as_ref(),
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

                        state.insert_vcard(
                            &[JSContactProperty::Name, JSContactProperty::Components],
                            VCardEntry::new(VCardProperty::N)
                                .with_params(params)
                                .with_value(n),
                        );
                    }
                    JSContactProperty::SpeakToAs => {
                        for (sub_property, value) in value.into_expanded_object() {
                            match sub_property {
                                Key::Property(JSContactProperty::GrammaticalGender) => {
                                    if let Ok(gender) = convert_value(value) {
                                        state.insert_vcard(
                                            &[
                                                JSContactProperty::SpeakToAs,
                                                JSContactProperty::GrammaticalGender,
                                            ],
                                            VCardEntry::new(VCardProperty::Gramgender)
                                                .with_value(gender),
                                        );
                                    }
                                }
                                Key::Property(JSContactProperty::Pronouns) => {
                                    for (name, value) in value.into_expanded_object() {
                                        let mut entry = VCardEntry::new(VCardProperty::Pronouns);

                                        for (pronoun_property, value) in
                                            value.into_expanded_object()
                                        {
                                            match pronoun_property {
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
                                                    state.insert_jsprop(
                                                        &[
                                                            property.to_string().as_ref(),
                                                            sub_property.to_string().as_ref(),
                                                            name.to_string().as_ref(),
                                                            pronoun_property.to_string().as_ref(),
                                                        ],
                                                        value,
                                                    );
                                                }
                                            }
                                        }

                                        state.insert_vcard(
                                            &[
                                                JSContactProperty::SpeakToAs,
                                                JSContactProperty::Pronouns,
                                            ],
                                            entry.with_param(VCardParameter::PropId(
                                                name.into_string(),
                                            )),
                                        );
                                    }
                                }
                                _ => {
                                    state.insert_jsprop(
                                        &[
                                            property.to_string().as_ref(),
                                            sub_property.to_string().as_ref(),
                                        ],
                                        value,
                                    );
                                }
                            }
                        }
                    }
                    JSContactProperty::Nicknames => {
                        for (name, value) in value.into_expanded_object() {
                            let mut entry = VCardEntry::new(VCardProperty::Nickname);

                            for (sub_property, value) in value.into_expanded_object() {
                                match sub_property {
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
                                        state.insert_jsprop(
                                            &[
                                                property.to_string().as_ref(),
                                                name.to_string().as_ref(),
                                                sub_property.to_string().as_ref(),
                                            ],
                                            value,
                                        );
                                    }
                                }
                            }

                            state.insert_vcard(
                                &[JSContactProperty::Nicknames],
                                entry.with_param(VCardParameter::PropId(name.into_string())),
                            );
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

                                for (sub_property, value) in value.into_expanded_object() {
                                    match sub_property {
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
                                            state.insert_jsprop(
                                                &[
                                                    property.to_string().as_ref(),
                                                    name.to_string().as_ref(),
                                                    sub_property.to_string().as_ref(),
                                                ],
                                                value,
                                            );
                                        }
                                    }
                                }

                                state.insert_vcard(
                                    &[JSContactProperty::Media],
                                    entry.with_param(VCardParameter::PropId(name.into_string())),
                                );
                            } else {
                                state.insert_jsprop(
                                    &[property.to_string().as_ref(), name.to_string().as_ref()],
                                    value,
                                );
                            }
                        }
                    }
                    JSContactProperty::Addresses => {
                        for (name, value) in value.into_expanded_object() {
                            let mut params =
                                vec![VCardParameter::PropId(name.to_string().to_string())];
                            let mut parts: Vec<Option<String>> = vec![None; 18];
                            let mut num_parts = 0;

                            for (sub_property, value) in value.into_expanded_object() {
                                match sub_property {
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
                                                            Value::Element(
                                                                JSContactValue::Kind(
                                                                    JSContactKind::PostOfficeBox,
                                                                ),
                                                            ) => 0,
                                                            Value::Element(
                                                                JSContactValue::Kind(
                                                                    JSContactKind::Apartment,
                                                                ),
                                                            ) => 1,
                                                            Value::Element(
                                                                JSContactValue::Kind(
                                                                    JSContactKind::Name,
                                                                ),
                                                            ) => 2,
                                                            Value::Element(
                                                                JSContactValue::Kind(
                                                                    JSContactKind::Locality,
                                                                ),
                                                            ) => 3,
                                                            Value::Element(
                                                                JSContactValue::Kind(
                                                                    JSContactKind::Region,
                                                                ),
                                                            ) => 4,
                                                            Value::Element(
                                                                JSContactValue::Kind(
                                                                    JSContactKind::Postcode,
                                                                ),
                                                            ) => 5,
                                                            Value::Element(
                                                                JSContactValue::Kind(
                                                                    JSContactKind::Country,
                                                                ),
                                                            ) => 6,
                                                            Value::Element(
                                                                JSContactValue::Kind(
                                                                    JSContactKind::Room,
                                                                ),
                                                            ) => 7,
                                                            Value::Element(
                                                                JSContactValue::Kind(
                                                                    JSContactKind::Floor,
                                                                ),
                                                            ) => 9,
                                                            Value::Element(
                                                                JSContactValue::Kind(
                                                                    JSContactKind::Number,
                                                                ),
                                                            ) => 10,
                                                            Value::Element(
                                                                JSContactValue::Kind(
                                                                    JSContactKind::Building,
                                                                ),
                                                            ) => 12,
                                                            Value::Element(
                                                                JSContactValue::Kind(
                                                                    JSContactKind::Block,
                                                                ),
                                                            ) => 13,
                                                            Value::Element(
                                                                JSContactValue::Kind(
                                                                    JSContactKind::Subdistrict,
                                                                ),
                                                            ) => 14,
                                                            Value::Element(
                                                                JSContactValue::Kind(
                                                                    JSContactKind::District,
                                                                ),
                                                            ) => 15,
                                                            Value::Element(
                                                                JSContactValue::Kind(
                                                                    JSContactKind::Landmark,
                                                                ),
                                                            ) => 16,
                                                            Value::Element(
                                                                JSContactValue::Kind(
                                                                    JSContactKind::Direction,
                                                                ),
                                                            ) => 17,
                                                            _ => continue,
                                                        }
                                                        .into();
                                                    }
                                                    Key::Property(JSContactProperty::Value) => {
                                                        comp_value = value.into_string();
                                                    }
                                                    _ => {
                                                        state.insert_jsprop(
                                                            &[
                                                                property.to_string().as_ref(),
                                                                name.to_string().as_ref(),
                                                                sub_property.to_string().as_ref(),
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
                                        if let Ok(phonetic_system) = VCardPhonetic::try_from(value)
                                        {
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
                                        state.insert_jsprop(
                                            &[
                                                property.to_string().as_ref(),
                                                name.to_string().as_ref(),
                                                sub_property.to_string().as_ref(),
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

                            state.insert_vcard(
                                &[JSContactProperty::Addresses, JSContactProperty::Components],
                                VCardEntry::new(VCardProperty::Adr)
                                    .with_params(params)
                                    .with_value(n),
                            );
                        }
                    }
                    JSContactProperty::Organizations => {
                        for (name, value) in value.into_expanded_object() {
                            let mut entry = VCardEntry::new(VCardProperty::Org);

                            for (sub_property, value) in value.into_expanded_object() {
                                match sub_property {
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
                                                        state.insert_jsprop(
                                                            &[
                                                                property.to_string().as_ref(),
                                                                name.to_string().as_ref(),
                                                                sub_property.to_string().as_ref(),
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
                                        state.insert_jsprop(
                                            &[
                                                property.to_string().as_ref(),
                                                name.to_string().as_ref(),
                                                sub_property.to_string().as_ref(),
                                            ],
                                            value,
                                        );
                                    }
                                }
                            }

                            state.insert_vcard(
                                &[JSContactProperty::Organizations],
                                entry.with_param(VCardParameter::PropId(name.into_string())),
                            );
                        }
                    }
                    JSContactProperty::Members => {
                        for (name, set) in value.into_expanded_boolean_set() {
                            if set {
                                state.insert_vcard(
                                    &[JSContactProperty::Members],
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

                        state.insert_vcard(
                            &[JSContactProperty::Keywords],
                            VCardEntry::new(VCardProperty::Categories).with_values(keywords),
                        );
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

                                for (sub_property, value) in value.into_expanded_object() {
                                    match sub_property {
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
                                            state.insert_jsprop(
                                                &[
                                                    property.to_string().as_ref(),
                                                    name.to_string().as_ref(),
                                                    sub_property.to_string().as_ref(),
                                                ],
                                                value,
                                            );
                                        }
                                    }
                                }

                                state.insert_vcard(
                                    &[JSContactProperty::Titles],
                                    entry.with_param(VCardParameter::PropId(name.into_string())),
                                );
                            } else {
                                state.insert_jsprop(
                                    &[property.to_string().as_ref(), name.to_string().as_ref()],
                                    value,
                                );
                            }
                        }
                    }
                    JSContactProperty::Emails => {
                        for (name, value) in value.into_expanded_object() {
                            let mut entry = VCardEntry::new(VCardProperty::Email);

                            for (sub_property, value) in value.into_expanded_object() {
                                match sub_property {
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
                                        state.insert_jsprop(
                                            &[
                                                property.to_string().as_ref(),
                                                name.to_string().as_ref(),
                                                sub_property.to_string().as_ref(),
                                            ],
                                            value,
                                        );
                                    }
                                }
                            }

                            state.insert_vcard(
                                &[JSContactProperty::Titles],
                                entry.with_param(VCardParameter::PropId(name.into_string())),
                            );
                        }
                    }
                    JSContactProperty::OnlineServices => {
                        for (name, value) in value.into_expanded_object() {
                            let mut entry = VCardEntry::new(VCardProperty::Socialprofile);

                            for (sub_property, value) in value.into_expanded_object() {
                                match sub_property {
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
                                        state.insert_jsprop(
                                            &[
                                                property.to_string().as_ref(),
                                                name.to_string().as_ref(),
                                                sub_property.to_string().as_ref(),
                                            ],
                                            value,
                                        );
                                    }
                                }
                            }

                            state.insert_vcard(
                                &[JSContactProperty::OnlineServices],
                                entry.with_param(VCardParameter::PropId(name.into_string())),
                            );
                        }
                    }
                    JSContactProperty::Phones => {
                        for (name, value) in value.into_expanded_object() {
                            let mut entry = VCardEntry::new(VCardProperty::Tel);
                            let mut types = vec![];

                            for (sub_property, value) in value.into_expanded_object() {
                                match sub_property {
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
                                        state.insert_jsprop(
                                            &[
                                                property.to_string().as_ref(),
                                                name.to_string().as_ref(),
                                                sub_property.to_string().as_ref(),
                                            ],
                                            value,
                                        );
                                    }
                                }
                            }

                            if !types.is_empty() {
                                entry.params.push(VCardParameter::Type(types));
                            }
                            state.insert_vcard(
                                &[JSContactProperty::Phones],
                                entry.with_param(VCardParameter::PropId(name.into_string())),
                            );
                        }
                    }
                    JSContactProperty::PreferredLanguages => {
                        for (name, value) in value.into_expanded_object() {
                            let mut entry = VCardEntry::new(VCardProperty::Lang);

                            for (sub_property, value) in value.into_expanded_object() {
                                match sub_property {
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
                                        state.insert_jsprop(
                                            &[
                                                property.to_string().as_ref(),
                                                name.to_string().as_ref(),
                                                sub_property.to_string().as_ref(),
                                            ],
                                            value,
                                        );
                                    }
                                }
                            }

                            state.insert_vcard(
                                &[JSContactProperty::PreferredLanguages],
                                entry.with_param(VCardParameter::PropId(name.into_string())),
                            );
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

                                for (sub_property, value) in value.into_expanded_object() {
                                    match sub_property {
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
                                            state.insert_jsprop(
                                                &[
                                                    property.to_string().as_ref(),
                                                    name.to_string().as_ref(),
                                                    sub_property.to_string().as_ref(),
                                                ],
                                                value,
                                            );
                                        }
                                    }
                                }

                                state.insert_vcard(
                                    &[JSContactProperty::Calendars],
                                    entry.with_param(VCardParameter::PropId(name.into_string())),
                                );
                            } else {
                                state.insert_jsprop(
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

                            for (sub_property, value) in value.into_expanded_object() {
                                match sub_property {
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
                                        state.insert_jsprop(
                                            &[
                                                property.to_string().as_ref(),
                                                name.to_string().as_ref(),
                                                sub_property.to_string().as_ref(),
                                            ],
                                            value,
                                        );
                                    }
                                }
                            }

                            state.insert_vcard(
                                &[property.clone()],
                                entry.with_param(VCardParameter::PropId(name.into_string())),
                            );
                        }
                    }
                    JSContactProperty::Notes => {
                        for (name, value) in value.into_expanded_object() {
                            let mut entry = VCardEntry::new(VCardProperty::Note);

                            for (sub_property, value) in value.into_expanded_object() {
                                match sub_property {
                                    Key::Property(JSContactProperty::Note) => {
                                        if let Some(text) = value.into_string() {
                                            entry.values = vec![VCardValue::Text(text)];
                                        }
                                    }
                                    Key::Property(JSContactProperty::Created) => {
                                        if let Value::Element(JSContactValue::Timestamp(t)) = value
                                        {
                                            entry.params.push(VCardParameter::Created(t));
                                        }
                                    }
                                    Key::Property(JSContactProperty::Author) => {
                                        for (name, value) in value.into_expanded_object() {
                                            match name {
                                                Key::Property(JSContactProperty::Name) => {
                                                    if let Some(name) = value.into_string() {
                                                        entry.add_param(
                                                            VCardParameter::AuthorName(name),
                                                        );
                                                    }
                                                }
                                                Key::Property(JSContactProperty::Uri) => {
                                                    if let Some(uri) = value.into_string() {
                                                        entry
                                                            .add_param(VCardParameter::Author(uri));
                                                    }
                                                }
                                                _ => {
                                                    state.insert_jsprop(
                                                        &[
                                                            property.to_string().as_ref(),
                                                            name.to_string().as_ref(),
                                                            sub_property.to_string().as_ref(),
                                                            name.to_string().as_ref(),
                                                        ],
                                                        value,
                                                    );
                                                }
                                            }
                                        }
                                    }

                                    _ => {
                                        state.insert_jsprop(
                                            &[
                                                property.to_string().as_ref(),
                                                name.to_string().as_ref(),
                                                sub_property.to_string().as_ref(),
                                            ],
                                            value,
                                        );
                                    }
                                }
                            }

                            state.insert_vcard(
                                &[JSContactProperty::Notes],
                                entry.with_param(VCardParameter::PropId(name.into_string())),
                            );
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

                                for (sub_property, value) in value.into_expanded_object() {
                                    match sub_property {
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
                                            state.insert_jsprop(
                                                &[
                                                    property.to_string().as_ref(),
                                                    name.to_string().as_ref(),
                                                    sub_property.to_string().as_ref(),
                                                ],
                                                value,
                                            );
                                        }
                                    }
                                }

                                state.insert_vcard(
                                    &[JSContactProperty::PersonalInfo],
                                    entry.with_param(VCardParameter::PropId(name.into_string())),
                                );
                            } else {
                                state.insert_jsprop(
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

                            for (sub_property, value) in value.into_expanded_object() {
                                if let Key::Property(JSContactProperty::Relation) = sub_property {
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
                                    state.insert_jsprop(
                                        &[
                                            property.to_string().as_ref(),
                                            name.to_string().as_ref(),
                                            sub_property.to_string().as_ref(),
                                        ],
                                        value,
                                    );
                                }
                            }

                            if !types.is_empty() {
                                entry.params.push(VCardParameter::Type(types));
                            }

                            state.insert_vcard(&[JSContactProperty::RelatedTo], entry);
                        }
                    }

                    _ => {
                        state.insert_jsprop(&[property.to_string().as_ref()], value);
                    }
                }
            }
        }

        Some(state.into_vcard())
    }
}

impl<'x> State<'x> {
    pub fn insert_vcard(&mut self, path: &[JSContactProperty], mut entry: VCardEntry) {
        if self.converted_props_count < self.converted_props.len() {
            let prop_id = if matches!(entry.name, VCardProperty::Member | VCardProperty::Related) {
                entry.values.first().and_then(|v| v.as_text())
            } else {
                entry.prop_id()
            };
            let skip_tz_geo = matches!(entry.name, VCardProperty::Adr);
            let mut matched_once = false;

            'outer: for (keys, value) in self.converted_props.iter_mut() {
                let is_localized_key = keys
                    .first()
                    .is_some_and(|k| matches!(k, Key::Property(JSContactProperty::Localizations)));

                if let Some(lang) = &self.language {
                    if !is_localized_key || keys.get(1).is_none_or(|k| &k.to_string() != lang) {
                        continue;
                    }
                } else if is_localized_key {
                    continue;
                }
                if matches!(value, Value::Null) {
                    continue;
                }

                for (pos, item) in path.iter().enumerate() {
                    if keys
                        .iter()
                        .any(|k| matches!(k, Key::Property(p) if p == item))
                    {
                        if pos == 0 && matched_once {
                            // Array is sorted, so if we didn't match the first item,
                            // we won't match any further.
                            break 'outer;
                        } else {
                            continue 'outer;
                        }
                    } else {
                        matched_once = true;
                    }
                }

                if prop_id.is_none_or(|prop_id| {
                    keys.iter().any(|k| match k {
                        Key::Borrowed(v) => *v == prop_id,
                        Key::Owned(v) => v == prop_id,
                        _ => false,
                    })
                }) && (!skip_tz_geo
                    || !keys.iter().any(|k| {
                        matches!(
                            k,
                            Key::Property(
                                JSContactProperty::TimeZone | JSContactProperty::Coordinates
                            )
                        )
                    }))
                {
                    entry.import_converted_properties(std::mem::take(value));
                    self.converted_props_count += 1;
                    break;
                }
            }
        }

        if let Some(lang) = &self.language {
            entry.params.push(VCardParameter::Language(lang.clone()));
        }

        self.vcard.entries.push(entry);
    }

    pub fn insert_jsprop(
        &mut self,
        path: &[&str],
        value: Value<'x, JSContactProperty, JSContactValue>,
    ) {
        self.js_props
            .push((JsonPointer::<JSContactProperty>::encode(path), value));
    }

    pub fn import_properties(&mut self, props: Vec<Value<'x, JSContactProperty, JSContactValue>>) {
        let todo = "implement";
    }

    pub fn into_vcard(mut self) -> VCard {
        for (ptr, value) in self.js_props {
            self.vcard.entries.push(
                VCardEntry::new(VCardProperty::Jsprop)
                    .with_param(VCardParameter::Jsptr(ptr))
                    .with_value(serde_json::to_string(&value).unwrap_or_default()),
            );
        }
        self.vcard
    }
}

fn build_path<'x>(
    obj: &mut Value<'x, JSContactProperty, JSContactValue>,
    mut ptr: Peekable<IntoIter<JsonPointerItem<JSContactProperty>>>,
    value: Value<'x, JSContactProperty, JSContactValue>,
) -> bool {
    if let Some(item) = ptr.next() {
        match item {
            JsonPointerItem::Root | JsonPointerItem::Wildcard => {}
            JsonPointerItem::Key(key) => {
                if let Some(obj_) = obj.as_object_mut().map(|obj_| {
                    obj_.insert_or_get_mut(
                        key,
                        if matches!(ptr.peek(), Some(JsonPointerItem::Key(_))) {
                            Value::Object(Map::from(Vec::new()))
                        } else {
                            Value::Array(Vec::new())
                        },
                    )
                }) {
                    return build_path(obj_, ptr, value);
                }
            }
            JsonPointerItem::Number(idx) => {
                if let Some(arr) = obj.as_array_mut() {
                    if (idx as usize) < arr.len() {
                        return build_path(&mut arr[idx as usize], ptr, value);
                    }

                    if idx < 20 {
                        arr.resize_with(idx as usize + 1, || Value::Null);
                        return build_path(&mut arr[idx as usize], ptr, value);
                    }
                }
            }
        }
        false
    } else {
        *obj = value;
        true
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
    fn import_converted_properties(&mut self, props: Value<'_, JSContactProperty, JSContactValue>) {
        for (key, value) in props.into_expanded_object() {
            match key {
                Key::Property(JSContactProperty::Name) => {
                    if let Some(name) = value.into_string() {
                        self.name = VCardProperty::try_from(name.as_ref())
                            .unwrap_or(VCardProperty::Other(name));
                    }
                }
                Key::Property(JSContactProperty::Parameters) => {
                    self.import_jcard_params(value);
                }
                _ => {}
            }
        }
    }

    fn import_jcard_params(&mut self, params: Value<'_, JSContactProperty, JSContactValue>) {
        for (key, value) in params.into_expanded_object() {
            let mut values = match value {
                Value::Array(values) => values.into_iter().filter_map(ParamValue::try_from_value),
                value => vec![value]
                    .into_iter()
                    .filter_map(ParamValue::try_from_value),
            }
            .peekable();

            if values.peek().is_none() {
                continue;
            }

            let key = key.to_string();
            let Some(param) = VCardParameterName::try_parse(key.as_ref()) else {
                self.params.push(VCardParameter::Other(
                    [key.into_owned()]
                        .into_iter()
                        .chain(values.map(|v| v.into_string().into_owned()))
                        .collect(),
                ));
                continue;
            };

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
