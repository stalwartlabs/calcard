/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

use crate::{
    common::{Data, IanaParse, IanaType, timezone::Tz},
    jscontact::{
        JSContact, JSContactId, JSContactKind, JSContactProperty, JSContactValue,
        export::{
            State,
            props::{
                build_path, convert_anniversary, convert_types, convert_value, find_text_param,
                map_kind,
            },
        },
    },
    vcard::{
        Jscomp, VCard, VCardEntry, VCardLevel, VCardParameter, VCardParameterValue, VCardPhonetic,
        VCardProperty, VCardType, VCardValue, VCardValueType, ValueType,
    },
};
use jmap_tools::{JsonPointer, JsonPointerItem, Key, Map, Value};
use std::{collections::HashMap, str::FromStr};

impl<'x, I, B> JSContact<'x, I, B>
where
    I: JSContactId,
    B: JSContactId,
{
    pub fn into_vcard(self) -> Option<VCard> {
        let mut state = State::<I, B>::default();
        let mut properties = self.0.into_object()?.into_vec();
        let mut localized_properties: HashMap<String, Vec<_>> = HashMap::new();

        for (property, value) in &mut properties {
            match (property, value) {
                (Key::Property(JSContactProperty::Localizations), Value::Object(obj)) => {
                    for (lang, value) in std::mem::take(obj.as_mut_vec()) {
                        let mut localizations: Value<
                            '_,
                            JSContactProperty<I>,
                            JSContactValue<I, B>,
                        > = Value::Object(Map::from(Vec::with_capacity(3)));
                        for (key, value) in value.into_expanded_object() {
                            let ptr = match key {
                                Key::Property(JSContactProperty::Pointer(ptr)) => ptr,
                                _ => JsonPointer::parse(key.to_string().as_ref()),
                            };

                            let ptr_path = ptr.to_string();
                            if let Some(value) = build_path(
                                &mut localizations,
                                ptr.into_inner().into_iter().peekable(),
                                value,
                            ) {
                                state.insert_jsprop(
                                    &[
                                        JSContactProperty::Localizations::<I>.to_string().as_ref(),
                                        lang.to_string().as_ref(),
                                        ptr_path.as_str(),
                                    ],
                                    value,
                                );
                            }
                        }
                        if let Some(localizations) =
                            localizations.into_object().filter(|x| !x.is_empty())
                        {
                            localized_properties
                                .insert(lang.into_string(), localizations.into_vec());
                        }
                    }
                }
                (Key::Property(JSContactProperty::VCard), Value::Object(obj)) => {
                    for (sub_property, value) in std::mem::take(obj.as_mut_vec()) {
                        match sub_property {
                            Key::Property(JSContactProperty::ConvertedProperties) => {
                                for (key, value) in value.into_expanded_object() {
                                    let ptr = match key {
                                        Key::Property(JSContactProperty::Pointer(ptr)) => ptr,
                                        _ => JsonPointer::parse(key.to_string().as_ref()),
                                    };

                                    let mut keys = Vec::with_capacity(2);
                                    for item in ptr.into_iter() {
                                        match item {
                                            JsonPointerItem::Key(key) => {
                                                let key = match &key {
                                                    Key::Borrowed(v) if v.contains('/') => v,
                                                    Key::Owned(v) if v.contains('/') => v.as_str(),
                                                    _ => {
                                                        keys.push(key);
                                                        continue;
                                                    }
                                                };
                                                for item in JsonPointer::parse(key).into_iter() {
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
                                            JsonPointerItem::Root | JsonPointerItem::Wildcard => (),
                                        }
                                    }

                                    state.converted_props.push((keys, value));
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
                                        JSContactProperty::VCard::<I>.to_string().as_ref(),
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
                _ => {}
            }
        }

        // Resolve organizational mappings
        let mut org_mappings: HashMap<String, (String, bool)> = HashMap::new();
        for (keys, value) in &state.converted_props {
            match (keys.first(), keys.get(2)) {
                (
                    Some(Key::Property(JSContactProperty::Titles)),
                    Some(Key::Property(JSContactProperty::Name)),
                ) => {
                    if let Some(group) = find_text_param(value, "group") {
                        org_mappings.entry(group.into_owned()).or_default().0 =
                            keys[1].to_string().into_owned();
                    }
                }
                (Some(Key::Property(JSContactProperty::Organizations)), _) => {
                    if let Some(group) = find_text_param(value, "group") {
                        org_mappings.entry(group.into_owned()).or_default().1 = true;
                    }
                }
                _ => {}
            }
        }

        // Localization maps
        let has_localizations = !localized_properties.is_empty();
        let mut name_pos_map: [Option<usize>; 20] = [None; 20];
        let mut adr_pos_map: HashMap<String, [Option<usize>; 20]> = HashMap::new();
        let mut places_map: HashMap<String, (VCardProperty, Option<VCardProperty>)> =
            HashMap::new();

        for (properties, language) in [(properties, None)].into_iter().chain(
            localized_properties
                .into_iter()
                .map(|(lang, properties)| (properties, Some(lang))),
        ) {
            let is_localized = language.is_some();
            state.language = language;

            for (property, value) in properties {
                let Key::Property(property) = property else {
                    state.insert_jsprop(&[property.to_string().as_ref()], value);
                    continue;
                };

                match property {
                    JSContactProperty::Uid
                    | JSContactProperty::Kind
                    | JSContactProperty::Language
                    | JSContactProperty::ProdId
                    | JSContactProperty::Created
                    | JSContactProperty::Updated => {
                        match convert_value(value, &ValueType::Vcard(VCardValueType::Text)) {
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
                        }
                    }
                    JSContactProperty::Directories => {
                        for (name, value) in value.into_expanded_object() {
                            let mut entry = VCardEntry::new(VCardProperty::OrgDirectory);

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
                                            entry.values.push(VCardValue::Text(text.into_owned()));
                                        }
                                    }
                                    Key::Property(JSContactProperty::Pref) => {
                                        if let Some(pref) = value.as_u64() {
                                            entry.params.push(VCardParameter::pref(pref as u32));
                                        }
                                    }
                                    Key::Property(JSContactProperty::ListAs) => {
                                        if let Some(index) = value.as_u64() {
                                            entry.params.push(VCardParameter::index(index as u32));
                                        }
                                    }
                                    Key::Property(JSContactProperty::MediaType) => {
                                        if let Some(text) = value.into_string() {
                                            entry
                                                .params
                                                .push(VCardParameter::mediatype(text.into_owned()));
                                        }
                                    }
                                    Key::Property(JSContactProperty::Label) => {
                                        if let Some(text) = value.into_string() {
                                            entry.values.push(VCardValue::Text(text.into_owned()));
                                        }
                                    }
                                    Key::Property(JSContactProperty::Contexts) => {
                                        if let Some(types) = convert_types(value, true) {
                                            for typ in types {
                                                entry.params.push(VCardParameter::typ(typ));
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
                                &[JSContactProperty::Directories],
                                entry.with_param(VCardParameter::prop_id(name.into_string())),
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
                            )
                            .or_else(|| {
                                if is_localized {
                                    places_map.get(name.to_string().as_ref()).cloned()
                                } else {
                                    None
                                }
                            }) {
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
                                            if has_localizations && !is_localized {
                                                places_map
                                                    .entry(name.to_string().into_owned())
                                                    .or_insert_with(|| {
                                                        (
                                                            vcard_property.clone(),
                                                            vcard_place_property.clone(),
                                                        )
                                                    });
                                            }

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
                                                                .with_value(text.into_owned()),
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
                                                                .with_value(text.into_owned())
                                                                .with_param(VCardParameter::value(
                                                                    VCardValueType::Uri,
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
                                                    JSContactProperty::Anniversaries::<I>
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
                                        place.with_param(VCardParameter::prop_id(
                                            name.to_string().into_owned(),
                                        )),
                                    );
                                }

                                // Add date
                                if let Some(date) = date {
                                    let mut entry = VCardEntry::new(vcard_property);
                                    if let Some(calscale) = calscale {
                                        entry.params.push(VCardParameter::calscale(calscale));
                                    }
                                    state.insert_vcard(
                                        &[
                                            JSContactProperty::Anniversaries,
                                            JSContactProperty::Date,
                                        ],
                                        entry
                                            .with_param(VCardParameter::prop_id(name.into_string()))
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
                        let mut parts: Vec<Option<Vec<String>>> = vec![None; 7];
                        let mut jscomps = vec![Jscomp::Separator(String::new())];
                        let mut num_parts = 0;

                        for (sub_property, value) in value.into_expanded_object() {
                            match sub_property {
                                Key::Property(JSContactProperty::Components) => {
                                    for (index, components) in value
                                        .into_array()
                                        .unwrap_or_default()
                                        .into_iter()
                                        .enumerate()
                                    {
                                        let mut comp_value = None;
                                        let mut comp_phonetic = None;
                                        let mut comp_pos = None;
                                        let mut is_separator = false;

                                        for (component, value) in components.into_expanded_object()
                                        {
                                            match (component, value) {
                                                (
                                                    Key::Property(JSContactProperty::Kind),
                                                    Value::Element(JSContactValue::Kind(
                                                        JSContactKind::Separator,
                                                    )),
                                                ) => {
                                                    is_separator = true;
                                                }
                                                (
                                                    Key::Property(JSContactProperty::Kind),
                                                    Value::Element(JSContactValue::Kind(kind)),
                                                ) => {
                                                    comp_pos = kind.to_vcard_n_pos();
                                                }
                                                (
                                                    Key::Property(JSContactProperty::Value),
                                                    value,
                                                ) => {
                                                    comp_value = value.into_string();
                                                }
                                                (
                                                    Key::Property(JSContactProperty::Phonetic),
                                                    value,
                                                ) if is_localized => {
                                                    comp_phonetic = value.into_string();
                                                }
                                                (component, value) => {
                                                    state.insert_jsprop(
                                                        &[
                                                            JSContactProperty::Name::<I>
                                                                .to_string()
                                                                .as_ref(),
                                                            JSContactProperty::Components::<I>
                                                                .to_string()
                                                                .as_ref(),
                                                            index.to_string().as_str(),
                                                            component.to_string().as_ref(),
                                                        ],
                                                        value,
                                                    );
                                                }
                                            }
                                        }

                                        if has_localizations
                                            && !is_localized
                                            && let Some(value) = name_pos_map.get_mut(index)
                                        {
                                            *value = comp_pos;
                                        }

                                        if let Some(comp_value) = comp_value.or(comp_phonetic) {
                                            if let Some(comp_pos) = comp_pos.or_else(|| {
                                                if is_localized && !is_separator {
                                                    name_pos_map
                                                        .get(index)
                                                        .copied()
                                                        .unwrap_or_default()
                                                } else {
                                                    None
                                                }
                                            }) {
                                                let part = &mut parts[comp_pos];
                                                if let Some(part) = part {
                                                    let value = part.len() as u32;
                                                    part.push(comp_value.into_owned());
                                                    jscomps.push(Jscomp::Entry {
                                                        position: comp_pos as u32,
                                                        value,
                                                    });
                                                } else {
                                                    *part = Some(vec![comp_value.into_owned()]);
                                                    jscomps.push(Jscomp::Entry {
                                                        position: comp_pos as u32,
                                                        value: 0,
                                                    });
                                                }
                                                num_parts = num_parts.max(comp_pos + 1);
                                            } else if is_separator {
                                                jscomps.push(Jscomp::Separator(
                                                    comp_value.into_owned(),
                                                ));
                                            }
                                        }
                                    }
                                }
                                Key::Property(JSContactProperty::Full) => {
                                    if let Some(text) = value.into_string() {
                                        state.insert_vcard(
                                            &[JSContactProperty::Name, JSContactProperty::Full],
                                            VCardEntry::new(VCardProperty::Fn)
                                                .with_value(text.into_owned()),
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
                                                        JSContactProperty::Name::<I>
                                                            .to_string()
                                                            .as_ref(),
                                                        JSContactProperty::SortAs::<I>
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
                                        params.push(VCardParameter::sort_as(format!(
                                            "{surname},{given}"
                                        )));
                                    } else if let Some(surname) = surname.or(given) {
                                        params.push(VCardParameter::sort_as(surname.into_owned()));
                                    }
                                }
                                Key::Property(JSContactProperty::PhoneticSystem) => {
                                    if let Ok(phonetic_system) =
                                        IanaType::<VCardPhonetic, String>::try_from(value)
                                    {
                                        params.push(VCardParameter::phonetic(phonetic_system));
                                    }
                                }
                                Key::Property(JSContactProperty::PhoneticScript) => {
                                    if let Some(phonetic_script) = value.into_string() {
                                        params.push(VCardParameter::script(
                                            phonetic_script.into_owned(),
                                        ));
                                    }
                                }
                                Key::Property(JSContactProperty::DefaultSeparator) => {
                                    if let Some(separator) = value.into_string() {
                                        jscomps[0] = Jscomp::Separator(separator.into_owned());
                                    }
                                }
                                Key::Property(JSContactProperty::IsOrdered) => {}
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

                        if num_parts > 0 || !params.is_empty() {
                            if jscomps.len() > 1 {
                                params.push(VCardParameter::jscomps(jscomps));
                            }

                            // To vCard: add any "surname2" NameComponent to the Family name component, after all "surname" values.
                            // To vCard: add any "generation" NameComponent to the Honorific suffix component.
                            for (from_pos, to_pos) in [(5, 0), (6, 4)] {
                                if let Some(v) = &parts[from_pos] {
                                    let v = v.clone();
                                    if let Some(to) = &mut parts[to_pos] {
                                        to.extend(v);
                                    } else {
                                        parts[to_pos] = Some(v);
                                    }
                                }
                            }

                            state.insert_vcard(
                                &[JSContactProperty::Name, JSContactProperty::Components],
                                VCardEntry::new(VCardProperty::N)
                                    .with_params(params)
                                    .with_values(
                                        parts
                                            .into_iter()
                                            .map(|v| {
                                                if let Some(v) = v {
                                                    if v.len() > 1 {
                                                        VCardValue::Component(v)
                                                    } else {
                                                        VCardValue::Text(
                                                            v.into_iter().next().unwrap(),
                                                        )
                                                    }
                                                } else {
                                                    VCardValue::Text(Default::default())
                                                }
                                            })
                                            .collect(),
                                    ),
                            );
                        }
                    }
                    JSContactProperty::SpeakToAs => {
                        for (sub_property, value) in value.into_expanded_object() {
                            match sub_property {
                                Key::Property(JSContactProperty::GrammaticalGender) => {
                                    if let Ok(gender) = convert_value(value, &ValueType::GramGender)
                                    {
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
                                                    if let Some(value) = value
                                                        .into_owned_string()
                                                        .map(VCardValue::Text)
                                                    {
                                                        entry.values = vec![value];
                                                    }
                                                }
                                                Key::Property(JSContactProperty::Pref) => {
                                                    if let Some(pref) = value.as_i64() {
                                                        entry.add_param(VCardParameter::pref(
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
                                            entry.with_param(VCardParameter::prop_id(
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
                                        if let Some(name) =
                                            value.into_owned_string().map(VCardValue::Text)
                                        {
                                            entry.values = vec![name];
                                        }
                                    }
                                    Key::Property(JSContactProperty::Pref) => {
                                        if let Some(pref) = value.as_i64() {
                                            entry.add_param(VCardParameter::pref(pref as u32));
                                        }
                                    }
                                    Key::Property(JSContactProperty::Contexts) => {
                                        if let Some(types) = convert_types(value, true) {
                                            for typ in types {
                                                entry.params.push(VCardParameter::typ(typ));
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
                                &[JSContactProperty::Nicknames],
                                entry.with_param(VCardParameter::prop_id(name.into_string())),
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
                                                    None => VCardValue::Text(uri.into_owned()),
                                                };

                                                entry.values = vec![uri];
                                            }
                                        }
                                        Key::Property(JSContactProperty::MediaType) => {
                                            if let Some(text) = value.into_string() {
                                                entry.params.push(VCardParameter::mediatype(
                                                    text.into_owned(),
                                                ));
                                            }
                                        }
                                        Key::Property(JSContactProperty::Label) => {
                                            if let Some(text) = value.into_string() {
                                                entry
                                                    .params
                                                    .push(VCardParameter::label(text.into_owned()));
                                            }
                                        }
                                        Key::Property(JSContactProperty::Pref) => {
                                            if let Some(pref) = value.as_i64() {
                                                entry.add_param(VCardParameter::pref(pref as u32));
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
                                    entry.with_param(VCardParameter::prop_id(name.into_string())),
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
                            let mut params = vec![];
                            let mut parts: Vec<Option<Vec<String>>> = vec![None; 18];
                            let mut jscomps = vec![Jscomp::Separator(String::new())];
                            let mut num_parts = 0;
                            let mut timezone = None;
                            let mut geo = None;

                            for (sub_property, value) in value.into_expanded_object() {
                                match sub_property {
                                    Key::Property(JSContactProperty::Components) => {
                                        for (index, components) in value
                                            .into_array()
                                            .unwrap_or_default()
                                            .into_iter()
                                            .enumerate()
                                        {
                                            let mut comp_value = None;
                                            let mut comp_phonetic = None;
                                            let mut comp_pos = None;
                                            let mut is_separator = false;

                                            for (key, value) in components.into_expanded_object() {
                                                match (key, value) {
                                                    (
                                                        Key::Property(JSContactProperty::Kind),
                                                        Value::Element(JSContactValue::Kind(
                                                            JSContactKind::Separator,
                                                        )),
                                                    ) => {
                                                        is_separator = true;
                                                    }
                                                    (
                                                        Key::Property(JSContactProperty::Kind),
                                                        Value::Element(JSContactValue::Kind(kind)),
                                                    ) => {
                                                        comp_pos = kind.to_vcard_adr_pos();
                                                    }
                                                    (
                                                        Key::Property(JSContactProperty::Value),
                                                        value,
                                                    ) => {
                                                        comp_value = value.into_string();
                                                    }
                                                    (
                                                        Key::Property(JSContactProperty::Phonetic),
                                                        value,
                                                    ) if is_localized => {
                                                        comp_phonetic = value.into_string();
                                                    }
                                                    (key, value) => {
                                                        state.insert_jsprop(
                                                            &[
                                                                property.to_string().as_ref(),
                                                                name.to_string().as_ref(),
                                                                sub_property.to_string().as_ref(),
                                                                index.to_string().as_str(),
                                                                key.to_string().as_ref(),
                                                            ],
                                                            value,
                                                        );
                                                    }
                                                }
                                            }

                                            if has_localizations
                                                && !is_localized
                                                && let Some(value) = adr_pos_map
                                                    .entry(name.to_string().into_owned())
                                                    .or_default()
                                                    .get_mut(index)
                                            {
                                                *value = comp_pos;
                                            }

                                            if let Some(comp_value) = comp_value.or(comp_phonetic) {
                                                if let Some(comp_pos) = comp_pos.or_else(|| {
                                                    if is_localized && !is_separator {
                                                        adr_pos_map
                                                            .get(name.to_string().as_ref())
                                                            .and_then(|map| map.get(index).copied())
                                                            .unwrap_or_default()
                                                    } else {
                                                        None
                                                    }
                                                }) {
                                                    let part = &mut parts[comp_pos];
                                                    if let Some(part) = part {
                                                        let value = part.len() as u32;
                                                        part.push(comp_value.into_owned());
                                                        jscomps.push(Jscomp::Entry {
                                                            position: comp_pos as u32,
                                                            value,
                                                        });
                                                    } else {
                                                        *part = Some(vec![comp_value.into_owned()]);
                                                        jscomps.push(Jscomp::Entry {
                                                            position: comp_pos as u32,
                                                            value: 0,
                                                        });
                                                    }
                                                    num_parts = num_parts.max(comp_pos + 1);
                                                } else if is_separator {
                                                    jscomps.push(Jscomp::Separator(
                                                        comp_value.into_owned(),
                                                    ));
                                                }
                                            }
                                        }
                                    }
                                    Key::Property(JSContactProperty::Full) => {
                                        if let Some(value) = value.into_string() {
                                            params.push(VCardParameter::label(value.into_owned()));
                                        }
                                    }
                                    Key::Property(JSContactProperty::Coordinates) => {
                                        geo = value.into_string();
                                    }
                                    Key::Property(JSContactProperty::TimeZone) => {
                                        timezone = value.into_string();
                                    }
                                    Key::Property(JSContactProperty::CountryCode) => {
                                        if let Some(value) = value.into_string() {
                                            params.push(VCardParameter::cc(value.into_owned()));
                                        }
                                    }
                                    Key::Property(JSContactProperty::Pref) => {
                                        if let Some(value) = value.as_i64() {
                                            params.push(VCardParameter::pref(value as u32));
                                        }
                                    }
                                    Key::Property(JSContactProperty::PhoneticSystem) => {
                                        if let Ok(phonetic_system) =
                                            IanaType::<VCardPhonetic, String>::try_from(value)
                                        {
                                            params.push(VCardParameter::phonetic(phonetic_system));
                                        }
                                    }
                                    Key::Property(JSContactProperty::PhoneticScript) => {
                                        if let Some(value) = value.into_string() {
                                            params.push(VCardParameter::script(value.into_owned()));
                                        }
                                    }
                                    Key::Property(JSContactProperty::Contexts) => {
                                        if let Some(types) = convert_types(value, true) {
                                            for typ in types {
                                                params.push(VCardParameter::typ(typ));
                                            }
                                        }
                                    }
                                    Key::Property(JSContactProperty::DefaultSeparator) => {
                                        if let Some(value) = value.into_string() {
                                            jscomps[0] = Jscomp::Separator(value.into_owned());
                                        }
                                    }
                                    Key::Property(JSContactProperty::IsOrdered) => {}
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

                            if let Some(timezone) = timezone {
                                if state.converted_props.iter().any(|(prop, _)| {
                                    prop.len() == 3
                                        && prop[0] == Key::Property(JSContactProperty::Addresses)
                                        && prop[1] == name
                                        && prop[2] == Key::Property(JSContactProperty::TimeZone)
                                }) {
                                    state.insert_vcard(
                                        &[
                                            JSContactProperty::Addresses,
                                            JSContactProperty::TimeZone,
                                        ],
                                        VCardEntry::new(VCardProperty::Tz).with_value(
                                            Tz::from_str(&timezone)
                                                .map(|tz| {
                                                    VCardValue::PartialDateTime(tz.offset_parts())
                                                })
                                                .unwrap_or_else(|_| {
                                                    VCardValue::Text(timezone.into_owned())
                                                }),
                                        ),
                                    );
                                } else {
                                    params.push(VCardParameter::tz(timezone.into_owned()));
                                }
                            }

                            if let Some(geo) = geo {
                                if state.converted_props.iter().any(|(prop, _)| {
                                    prop.len() == 3
                                        && prop[0] == Key::Property(JSContactProperty::Addresses)
                                        && prop[1] == name
                                        && prop[2] == Key::Property(JSContactProperty::Coordinates)
                                }) {
                                    state.insert_vcard(
                                        &[
                                            JSContactProperty::Addresses,
                                            JSContactProperty::Coordinates,
                                        ],
                                        VCardEntry::new(VCardProperty::Geo)
                                            .with_value(geo.into_owned()),
                                    );
                                } else {
                                    params.push(VCardParameter::geo(geo.into_owned()));
                                }
                            }

                            if num_parts > 0 || !params.is_empty() {
                                params.push(VCardParameter::prop_id(name.to_string().to_string()));

                                if jscomps.len() > 1 {
                                    params.push(VCardParameter::jscomps(jscomps));
                                }

                                /*

                                apartment To vCard: set the values of the following components:
                                 room, floor, apartment, building

                                name To vCard: set the values of the following components:
                                  number, name, block, direction, landmark, subdistrict, district

                                */

                                for (comp_target, source) in [
                                    (1, [7, 8, 9, 12].as_slice()),
                                    (2, [10, 11, 13, 17, 16, 14, 15].as_slice()),
                                ] {
                                    let mut target = vec![];

                                    for comp_pos in source {
                                        if let Some(v) = &parts[*comp_pos] {
                                            target.extend(v.iter().cloned());
                                        }
                                    }

                                    if !target.is_empty() {
                                        parts[comp_target] = Some(target);
                                    }
                                }

                                state.insert_vcard(
                                    &[JSContactProperty::Addresses, JSContactProperty::Components],
                                    VCardEntry::new(VCardProperty::Adr)
                                        .with_params(params)
                                        .with_values(
                                            parts
                                                .into_iter()
                                                .map(|v| {
                                                    if let Some(v) = v {
                                                        if v.len() > 1 {
                                                            VCardValue::Component(v)
                                                        } else {
                                                            VCardValue::Text(
                                                                v.into_iter().next().unwrap(),
                                                            )
                                                        }
                                                    } else {
                                                        VCardValue::Text(Default::default())
                                                    }
                                                })
                                                .collect(),
                                        ),
                                );
                            }
                        }
                    }
                    JSContactProperty::Organizations => {
                        for (name, value) in value.into_expanded_object() {
                            let mut entry = VCardEntry::new(VCardProperty::Org);

                            for (sub_property, value) in value.into_expanded_object() {
                                match sub_property {
                                    Key::Property(JSContactProperty::Name) => {
                                        if let Some(value) =
                                            value.into_owned_string().map(VCardValue::Text)
                                        {
                                            if entry.values.is_empty() {
                                                entry.values = vec![value];
                                            } else {
                                                entry.values.insert(0, value);
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
                                                        if let Some(value) = value
                                                            .into_owned_string()
                                                            .map(VCardValue::Text)
                                                        {
                                                            entry.values.push(value);
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
                                            entry.add_param(VCardParameter::sort_as(
                                                sort_as.into_owned(),
                                            ));
                                        }
                                    }
                                    Key::Property(JSContactProperty::Contexts) => {
                                        if let Some(types) = convert_types(value, true) {
                                            for typ in types {
                                                entry.params.push(VCardParameter::typ(typ));
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
                                &[JSContactProperty::Organizations],
                                entry.with_param(VCardParameter::prop_id(name.into_string())),
                            );
                        }
                    }
                    JSContactProperty::Members => {
                        let mut has_members = false;
                        for name in value.into_expanded_boolean_set() {
                            has_members = true;
                            state.insert_vcard(
                                &[JSContactProperty::Members],
                                VCardEntry::new(VCardProperty::Member)
                                    .with_value(name.into_string()),
                            );
                        }

                        if !has_members {
                            state.insert_jsprop(
                                &[property.to_string().as_ref()],
                                Value::Object(Map::from(Vec::new())),
                            );
                        }
                    }
                    JSContactProperty::Keywords => {
                        let mut keywords = Vec::new();
                        for name in value.into_expanded_boolean_set() {
                            keywords.push(VCardValue::Text(name.into_string()));
                        }

                        state.insert_vcard(
                            &[JSContactProperty::Keywords],
                            VCardEntry::new(VCardProperty::Categories).with_values(keywords),
                        );
                    }
                    JSContactProperty::Titles => {
                        for (name, value) in value.into_expanded_object() {
                            let mut entry = VCardEntry::new(
                                map_kind(
                                    &value,
                                    [
                                        (JSContactKind::Title, VCardProperty::Title),
                                        (JSContactKind::Role, VCardProperty::Role),
                                    ],
                                )
                                .unwrap_or(VCardProperty::Title),
                            );

                            for (sub_property, value) in value.into_expanded_object() {
                                match sub_property {
                                    Key::Property(JSContactProperty::Name) => {
                                        if let Some(value) =
                                            value.into_owned_string().map(VCardValue::Text)
                                        {
                                            entry.values = vec![value];
                                        }
                                    }
                                    Key::Property(JSContactProperty::Pref) => {
                                        if let Some(pref) = value.as_i64() {
                                            entry.add_param(VCardParameter::pref(pref as u32));
                                        }
                                    }
                                    Key::Property(JSContactProperty::Contexts) => {
                                        if let Some(types) = convert_types(value, true) {
                                            for typ in types {
                                                entry.params.push(VCardParameter::typ(typ));
                                            }
                                        }
                                    }
                                    Key::Property(JSContactProperty::OrganizationId)
                                        if org_mappings.values().any(|(id, has_mapping)| {
                                            *has_mapping && id == name.to_string().as_ref()
                                        }) => {}
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
                                &[JSContactProperty::Titles],
                                entry.with_param(VCardParameter::prop_id(name.into_string())),
                            );
                        }
                    }
                    JSContactProperty::Emails => {
                        for (name, value) in value.into_expanded_object() {
                            let mut entry = VCardEntry::new(VCardProperty::Email);

                            for (sub_property, value) in value.into_expanded_object() {
                                match sub_property {
                                    Key::Property(JSContactProperty::Address) => {
                                        if let Some(email) = value.into_string() {
                                            entry.values =
                                                vec![VCardValue::Text(email.into_owned())];
                                        }
                                    }
                                    Key::Property(JSContactProperty::Contexts) => {
                                        if let Some(types) = convert_types(value, true) {
                                            for typ in types {
                                                entry.params.push(VCardParameter::typ(typ));
                                            }
                                        }
                                    }
                                    Key::Property(JSContactProperty::Pref) => {
                                        if let Some(pref) = value.as_i64() {
                                            entry.add_param(VCardParameter::pref(pref as u32));
                                        }
                                    }
                                    Key::Property(JSContactProperty::Label) => {
                                        if let Some(label) = value.into_string() {
                                            entry
                                                .params
                                                .push(VCardParameter::label(label.into_owned()));
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
                                &[JSContactProperty::Emails],
                                entry.with_param(VCardParameter::prop_id(name.into_string())),
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
                                            entry.values =
                                                vec![VCardValue::Text(service.into_owned())];
                                        }
                                    }
                                    Key::Property(JSContactProperty::Service) => {
                                        if let Some(service) = value.into_string() {
                                            entry.add_param(VCardParameter::service_type(
                                                service.into_owned(),
                                            ));
                                        }
                                    }
                                    Key::Property(JSContactProperty::User) => {
                                        if let Some(username) = value.into_string() {
                                            entry.add_param(VCardParameter::username(
                                                username.into_owned(),
                                            ));
                                        }
                                    }
                                    Key::Property(JSContactProperty::Contexts) => {
                                        if let Some(types) = convert_types(value, true) {
                                            for typ in types {
                                                entry.params.push(VCardParameter::typ(typ));
                                            }
                                        }
                                    }
                                    Key::Property(JSContactProperty::Label) => {
                                        if let Some(label) = value.into_string() {
                                            entry.add_param(VCardParameter::label(
                                                label.into_owned(),
                                            ));
                                        }
                                    }
                                    Key::Property(JSContactProperty::Pref) => {
                                        if let Some(pref) = value.as_i64() {
                                            entry.add_param(VCardParameter::pref(pref as u32));
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
                                entry.with_param(VCardParameter::prop_id(name.into_string())),
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
                                            entry.values =
                                                vec![VCardValue::Text(email.into_owned())];
                                        }
                                    }
                                    Key::Property(
                                        JSContactProperty::Contexts | JSContactProperty::Features,
                                    ) => {
                                        if let Some(types_) = convert_types(
                                            value,
                                            matches!(
                                                sub_property,
                                                Key::Property(JSContactProperty::Contexts)
                                            ),
                                        ) {
                                            if types.is_empty() {
                                                types = types_;
                                            } else {
                                                types.extend(types_);
                                            }
                                        }
                                    }
                                    Key::Property(JSContactProperty::Pref) => {
                                        if let Some(pref) = value.as_i64() {
                                            entry.add_param(VCardParameter::pref(pref as u32));
                                        }
                                    }
                                    Key::Property(JSContactProperty::Label) => {
                                        if let Some(label) = value.into_string() {
                                            entry
                                                .params
                                                .push(VCardParameter::label(label.into_owned()));
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
                                for typ in types {
                                    entry.params.push(VCardParameter::typ(typ));
                                }
                            }
                            state.insert_vcard(
                                &[JSContactProperty::Phones],
                                entry.with_param(VCardParameter::prop_id(name.into_string())),
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
                                            entry.values =
                                                vec![VCardValue::Text(lang.into_owned())];
                                        }
                                    }
                                    Key::Property(JSContactProperty::Pref) => {
                                        if let Some(pref) = value.as_i64() {
                                            entry.add_param(VCardParameter::pref(pref as u32));
                                        }
                                    }
                                    Key::Property(JSContactProperty::Contexts) => {
                                        if let Some(types) = convert_types(value, true) {
                                            for typ in types {
                                                entry.params.push(VCardParameter::typ(typ));
                                            }
                                        }
                                    }
                                    Key::Property(JSContactProperty::Label) => {
                                        if let Some(label) = value.into_string() {
                                            entry
                                                .params
                                                .push(VCardParameter::label(label.into_owned()));
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
                                entry.with_param(VCardParameter::prop_id(name.into_string())),
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
                                            if let Some(value) =
                                                value.into_owned_string().map(VCardValue::Text)
                                            {
                                                entry.values = vec![value];
                                            }
                                        }
                                        Key::Property(JSContactProperty::MediaType) => {
                                            if let Some(text) = value.into_string() {
                                                entry.params.push(VCardParameter::mediatype(
                                                    text.into_owned(),
                                                ));
                                            }
                                        }
                                        Key::Property(JSContactProperty::Label) => {
                                            if let Some(text) = value.into_string() {
                                                entry
                                                    .params
                                                    .push(VCardParameter::label(text.into_owned()));
                                            }
                                        }
                                        Key::Property(JSContactProperty::Pref) => {
                                            if let Some(pref) = value.as_i64() {
                                                entry.add_param(VCardParameter::pref(pref as u32));
                                            }
                                        }
                                        Key::Property(JSContactProperty::Contexts) => {
                                            if let Some(types) = convert_types(value, true) {
                                                for typ in types {
                                                    entry.params.push(VCardParameter::typ(typ));
                                                }
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
                                    entry.with_param(VCardParameter::prop_id(name.into_string())),
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
                                        if let Some(value) =
                                            value.into_owned_string().map(VCardValue::Text)
                                        {
                                            entry.values = vec![value];
                                        }
                                    }
                                    Key::Property(JSContactProperty::MediaType) => {
                                        if let Some(text) = value.into_string() {
                                            entry
                                                .params
                                                .push(VCardParameter::mediatype(text.into_owned()));
                                        }
                                    }
                                    Key::Property(JSContactProperty::Label) => {
                                        if let Some(text) = value.into_string() {
                                            entry
                                                .params
                                                .push(VCardParameter::label(text.into_owned()));
                                        }
                                    }
                                    Key::Property(JSContactProperty::Pref) => {
                                        if let Some(pref) = value.as_i64() {
                                            entry.add_param(VCardParameter::pref(pref as u32));
                                        }
                                    }
                                    Key::Property(JSContactProperty::Contexts) => {
                                        if let Some(types) = convert_types(value, true) {
                                            for typ in types {
                                                entry.params.push(VCardParameter::typ(typ));
                                            }
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
                                std::slice::from_ref(&property),
                                entry.with_param(VCardParameter::prop_id(name.into_string())),
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
                                            entry.values =
                                                vec![VCardValue::Text(text.into_owned())];
                                        }
                                    }
                                    Key::Property(JSContactProperty::Created) => {
                                        if let Value::Element(JSContactValue::Timestamp(t)) = value
                                        {
                                            entry.params.push(VCardParameter::created(
                                                VCardParameterValue::Timestamp(t),
                                            ));
                                        }
                                    }
                                    Key::Property(JSContactProperty::Author) => {
                                        for (name, value) in value.into_expanded_object() {
                                            match name {
                                                Key::Property(JSContactProperty::Name) => {
                                                    if let Some(name) = value.into_string() {
                                                        entry.add_param(
                                                            VCardParameter::author_name(
                                                                name.into_owned(),
                                                            ),
                                                        );
                                                    }
                                                }
                                                Key::Property(JSContactProperty::Uri) => {
                                                    if let Some(uri) = value.into_string() {
                                                        entry.add_param(VCardParameter::author(
                                                            uri.into_owned(),
                                                        ));
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
                                entry.with_param(VCardParameter::prop_id(name.into_string())),
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
                                            if let Some(value) =
                                                value.into_owned_string().map(VCardValue::Text)
                                            {
                                                entry.values = vec![value];
                                            }
                                        }
                                        Key::Property(JSContactProperty::Label) => {
                                            if let Some(label) = value.into_string() {
                                                entry.add_param(VCardParameter::label(
                                                    label.into_owned(),
                                                ));
                                            }
                                        }
                                        Key::Property(JSContactProperty::ListAs) => {
                                            if let Some(pref) = value.as_i64() {
                                                entry.add_param(VCardParameter::index(pref as u32));
                                            }
                                        }
                                        Key::Property(JSContactProperty::Level) => {
                                            if let Ok(level) =
                                                IanaType::<VCardLevel, String>::try_from(value)
                                            {
                                                let level = if matches!(
                                                    entry.name,
                                                    VCardProperty::Expertise
                                                ) {
                                                    match level {
                                                        IanaType::Iana(VCardLevel::High) => {
                                                            IanaType::Iana(VCardLevel::Expert)
                                                        }
                                                        IanaType::Iana(VCardLevel::Medium) => {
                                                            IanaType::Iana(VCardLevel::Average)
                                                        }
                                                        IanaType::Iana(VCardLevel::Low) => {
                                                            IanaType::Iana(VCardLevel::Beginner)
                                                        }
                                                        _ => level,
                                                    }
                                                } else {
                                                    level
                                                };
                                                entry.params.push(VCardParameter::level(level));
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
                                    entry.with_param(VCardParameter::prop_id(name.into_string())),
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

                            for (sub_property, value) in value.into_expanded_object() {
                                if let Key::Property(JSContactProperty::Relation) = sub_property {
                                    for typ in value.into_expanded_boolean_set() {
                                        let typ = typ.to_string();
                                        match VCardType::parse(typ.as_ref().as_bytes()) {
                                            Some(typ) => {
                                                entry.params.push(VCardParameter::typ(typ))
                                            }
                                            None => {
                                                entry
                                                    .params
                                                    .push(VCardParameter::typ(typ.into_owned()));
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

                            state.insert_vcard(&[JSContactProperty::RelatedTo], entry);
                        }
                    }
                    JSContactProperty::Type
                    | JSContactProperty::Version
                    | JSContactProperty::Localizations
                    | JSContactProperty::VCard => (),
                    _ => {
                        if !matches!(value, Value::Null) {
                            state.insert_jsprop(&[property.to_string().as_ref()], value);
                        }
                    }
                }
            }
        }

        Some(state.into_vcard())
    }

    pub fn into_inner(self) -> Value<'x, JSContactProperty<I>, JSContactValue<I, B>> {
        self.0
    }
}
