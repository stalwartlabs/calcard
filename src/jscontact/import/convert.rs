/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

use crate::{
    jscontact::{
        import::{EntryState, GetObjectOrCreate, State},
        JSContact, JSContactKind, JSContactProperty, JSContactValue,
    },
    vcard::{VCard, VCardParameter, VCardParameterName, VCardProperty, VCardValue, VCardValueType},
};
use jmap_tools::{JsonPointer, JsonPointerHandler, Key, Map, Property, Value};

impl VCard {
    pub fn into_jscontact(mut self) -> JSContact<'static> {
        let mut state = State::new(&mut self);
        let mut patch_objects = Vec::new();

        for entry in self.entries {
            let mut entry = EntryState::new(entry);

            match &entry.entry.name {
                VCardProperty::Kind => {
                    if !state.has_property(JSContactProperty::Kind) {
                        if let Some(kind) = entry.to_kind() {
                            state
                                .entries
                                .insert(Key::Property(JSContactProperty::Kind), kind);
                            entry.set_converted_to(&[JSContactProperty::Kind.to_string().as_ref()]);
                        }
                    }
                }
                VCardProperty::Source => {
                    state.map_named_entry(
                        &mut entry,
                        &[
                            VCardParameterName::PropId,
                            VCardParameterName::Pref,
                            VCardParameterName::Mediatype,
                        ],
                        JSContactProperty::Directories,
                        JSContactProperty::Uri,
                        [(
                            Key::Property(JSContactProperty::Kind),
                            Value::Element(JSContactValue::Kind(JSContactKind::Entry)),
                        )],
                    );
                }
                VCardProperty::OrgDirectory => {
                    state.map_named_entry(
                        &mut entry,
                        &[
                            VCardParameterName::PropId,
                            VCardParameterName::Pref,
                            VCardParameterName::Index,
                            VCardParameterName::Label,
                        ],
                        JSContactProperty::Directories,
                        JSContactProperty::Uri,
                        [(
                            Key::Property(JSContactProperty::Kind),
                            Value::Element(JSContactValue::Kind(JSContactKind::Directory)),
                        )],
                    );
                }
                VCardProperty::Anniversary | VCardProperty::Bday | VCardProperty::Deathdate => {
                    let kind = match &entry.entry.name {
                        VCardProperty::Anniversary => JSContactKind::Wedding,
                        VCardProperty::Bday => JSContactKind::Birth,
                        VCardProperty::Deathdate => JSContactKind::Death,
                        _ => unreachable!(),
                    };

                    state.map_named_entry(
                        &mut entry,
                        &[VCardParameterName::PropId, VCardParameterName::Calscale],
                        JSContactProperty::Anniversaries,
                        JSContactProperty::Date,
                        [(
                            Key::Property(JSContactProperty::Kind),
                            Value::Element(JSContactValue::Kind(kind)),
                        )],
                    );
                }
                VCardProperty::Birthplace | VCardProperty::Deathplace => {
                    // Only text and GEO can be mapped
                    let is_geo = matches!(entry.entry.values.first(), Some(VCardValue::Text(uri)) if uri.starts_with("geo:"));
                    if is_geo || !entry.entry.is_type(&VCardValueType::Uri) {
                        if let Some(text) = entry.to_text() {
                            // Extract language and value type
                            let mut params = state.extract_params(
                                &mut entry.entry.params,
                                &[VCardParameterName::Language, VCardParameterName::PropId],
                            );

                            let prop_id = params.prop_id();
                            let alt_id = params.alt_id();

                            let kind = Value::Element(JSContactValue::Kind(
                                if entry.entry.name == VCardProperty::Birthplace {
                                    JSContactKind::Birth
                                } else {
                                    JSContactKind::Death
                                },
                            ));

                            let patch_prop = if params.language.is_some() {
                                &entry.entry.name
                            } else if entry.entry.name == VCardProperty::Birthplace {
                                &VCardProperty::Bday
                            } else {
                                &VCardProperty::Deathdate
                            };
                            let mut patch_id = prop_id
                                .as_deref()
                                .filter(|prop_id| state.has_prop_id(patch_prop, prop_id))
                                .or_else(|| {
                                    state.find_prop_id(
                                        patch_prop,
                                        entry.entry.group.as_deref(),
                                        alt_id.as_deref(),
                                    )
                                })
                                .map(|prop_id| prop_id.to_string());

                            if patch_id.is_none() && params.language.is_some() {
                                entry.entry.params.push(VCardParameter::Language(
                                    params.language.take().unwrap(),
                                ));
                            }

                            let entries =
                                state.get_mut_object_or_insert(JSContactProperty::Anniversaries);

                            if let Some(lang) = params.language() {
                                let path = format!(
                                    "{}/{}/{}/{}",
                                    JSContactProperty::Anniversaries.to_cow().as_ref(),
                                    patch_id.unwrap(),
                                    JSContactProperty::Place.to_cow().as_ref(),
                                    JSContactProperty::Full.to_cow().as_ref()
                                );

                                entry.set_converted_to(&[
                                    JSContactProperty::Localizations.to_cow().as_ref(),
                                    lang.as_str(),
                                    path.as_str(),
                                ]);

                                state
                                    .localizations
                                    .entry(lang)
                                    .or_default()
                                    .push((path, text));
                            } else {
                                // Place needs to be wrapped in an option to dance around the borrow checker
                                let prop_name = if !is_geo {
                                    JSContactProperty::Full
                                } else {
                                    JSContactProperty::Coordinates
                                };
                                let mut place =
                                    Some(Map::from(vec![(Key::Property(prop_name.clone()), text)]));

                                if let Some(anniversary) = patch_id.clone().and_then(|patch_id| {
                                    entries
                                        .get_mut(&Key::Owned(patch_id))
                                        .and_then(|v| v.as_object_mut())
                                }) {
                                    anniversary.insert(
                                        Key::Property(JSContactProperty::Place),
                                        Value::Object(place.take().unwrap()),
                                    );
                                }

                                if place.is_some() {
                                    if let Some((key, anniversary)) =
                                        entries.iter_mut().find_map(|(k, v)| {
                                            let obj = v.as_object_mut()?;
                                            if obj.iter().any(|(k, v)| {
                                                k == &Key::Property(JSContactProperty::Kind)
                                                    && v == &kind
                                            }) {
                                                Some((k, obj))
                                            } else {
                                                None
                                            }
                                        })
                                    {
                                        anniversary.insert(
                                            Key::Property(JSContactProperty::Place),
                                            Value::Object(place.take().unwrap()),
                                        );
                                        patch_id = Some(key.to_string().into_owned());
                                    }
                                }

                                if let Some(place) = place {
                                    patch_id =
                                        entries.insert_named(prop_id, Value::Object(place)).into();
                                }

                                let patch_id = patch_id.unwrap();

                                entry.set_converted_to(&[
                                    JSContactProperty::Anniversaries.to_cow().as_ref(),
                                    patch_id.as_ref(),
                                    JSContactProperty::Place.to_cow().as_ref(),
                                    prop_name.to_cow().as_ref(),
                                ]);

                                state.track_prop(
                                    &entry.entry,
                                    JSContactProperty::Anniversaries,
                                    alt_id,
                                    patch_id,
                                );
                            }
                        }
                    }
                }
                VCardProperty::Fn => {
                    if !state.has_fn
                        || (entry
                            .non_default_language(state.default_language.as_deref())
                            .is_some()
                            && !state.has_fn_localization)
                    {
                        if let Some(text) = entry.to_text() {
                            if let Some(lang) = state
                                .extract_params(
                                    &mut entry.entry.params,
                                    &[VCardParameterName::Language],
                                )
                                .language
                            {
                                let path = format!(
                                    "{}/{}",
                                    JSContactProperty::Name.to_cow().as_ref(),
                                    JSContactProperty::Full.to_cow().as_ref()
                                );

                                entry.set_converted_to(&[
                                    JSContactProperty::Localizations.to_cow().as_ref(),
                                    lang.as_str(),
                                    path.as_str(),
                                ]);
                                state.has_fn_localization = true;
                                state
                                    .localizations
                                    .entry(lang)
                                    .or_default()
                                    .push((path, text));
                            } else {
                                state.has_fn = true;
                                state
                                    .get_mut_object_or_insert(JSContactProperty::Name)
                                    .insert(Key::Property(JSContactProperty::Name), text);
                                entry.set_converted_to(&[
                                    JSContactProperty::Name.to_cow().as_ref(),
                                    JSContactProperty::Full.to_cow().as_ref(),
                                ]);
                            }
                        }
                    }
                }
                VCardProperty::N => {
                    /*
                      Only process this N component if it matches the main ALTID and:
                      - It is the first N component
                      - Or it has a language that has not been processed yet
                    */
                    if state.name_alt_id.as_deref() == entry.entry.alt_id()
                        && (!state.has_n
                            || (entry
                                .non_default_language(state.default_language.as_deref())
                                .is_some()
                                && !state.has_n_localization))
                    {
                        let mut params = state.extract_params(
                            &mut entry.entry.params,
                            &[
                                VCardParameterName::Language,
                                VCardParameterName::Phonetic,
                                VCardParameterName::Script,
                                VCardParameterName::SortAs,
                            ],
                        );

                        if params.language.is_some() && !state.has_n {
                            entry
                                .entry
                                .params
                                .push(VCardParameter::Language(params.language.take().unwrap()));
                        }

                        /*

                           0 = family names (also known as surnames);
                           1 = given names;
                           2 = additional names;
                           3 = honorific prefixes;
                           4 = honorific suffixes;
                           5 = secondary surname;
                           6 = generation

                        */

                        let mut components = Vec::with_capacity(7);
                        for (comp_id, value) in entry.text_parts_borrowed().enumerate() {
                            let kind = match comp_id {
                                0 => JSContactKind::Surname,
                                1 => JSContactKind::Given,
                                2 => JSContactKind::Given2,
                                3 => JSContactKind::Title,
                                4 => JSContactKind::Credential,
                                5 => JSContactKind::Surname2,
                                6 => JSContactKind::Generation,
                                _ => continue,
                            };

                            for value in value.split(',') {
                                let value = value.trim();
                                if !value.is_empty() {
                                    components.push((kind, value));
                                }
                            }
                        }

                        // 'credential' From vCard: ignore any value that also occurs in the Generation component.
                        let components = components
                            .iter()
                            .filter(|(kind, value)| {
                                !matches!(kind, JSContactKind::Credential)
                                    || !components.contains(&(JSContactKind::Generation, value))
                            })
                            .map(|(kind, value)| {
                                Value::Object(Map::from(vec![
                                    (
                                        Key::Property(JSContactProperty::Kind),
                                        Value::Element(JSContactValue::Kind(*kind)),
                                    ),
                                    (
                                        Key::Property(JSContactProperty::Value),
                                        Value::Str(value.to_string().into()),
                                    ),
                                ]))
                            })
                            .collect::<Vec<_>>();

                        let name = state
                            .entries
                            .get_mut_object_or_insert(JSContactProperty::Name);

                        if let Some(lang) = params.language() {
                            let path = format!(
                                "{}/{}",
                                JSContactProperty::Name.to_cow().as_ref(),
                                JSContactProperty::Components.to_cow().as_ref(),
                            );

                            entry.set_converted_to(&[
                                JSContactProperty::Localizations.to_cow().as_ref(),
                                lang.as_str(),
                                path.as_str(),
                            ]);

                            state.has_n_localization = true;
                            let locale = state.localizations.entry(lang).or_default();
                            locale.push((path, Value::Array(components)));

                            for (prop, value) in params.into_iter(&entry.entry.name) {
                                locale.push((
                                    format!(
                                        "{}/{}",
                                        JSContactProperty::Name.to_cow().as_ref(),
                                        prop.to_string()
                                    ),
                                    value,
                                ));
                            }
                        } else {
                            state.has_n = true;
                            name.extend(params.into_iter(&entry.entry.name));
                            name.insert(
                                Key::Property(JSContactProperty::Components),
                                Value::Array(components),
                            );
                            entry.set_converted_to(&[
                                JSContactProperty::Name.to_cow().as_ref(),
                                JSContactProperty::Components.to_cow().as_ref(),
                            ]);
                        }
                    }
                }
                VCardProperty::Gramgender => {
                    if !state.has_gram_gender {
                        if let Some(gram_gender) = entry.to_gram_gender() {
                            state.extract_params(&mut entry.entry.params, &[]);
                            state
                                .get_mut_object_or_insert(JSContactProperty::SpeakToAs)
                                .insert(JSContactProperty::GrammaticalGender, gram_gender);
                            state.has_gram_gender = true;
                            entry.set_converted_to(&[
                                JSContactProperty::SpeakToAs.to_cow().as_ref(),
                                JSContactProperty::GrammaticalGender.to_cow().as_ref(),
                            ]);
                        }
                    }
                }
                VCardProperty::Pronouns => {
                    state.map_named_entry(
                        &mut entry,
                        &[
                            VCardParameterName::Language,
                            VCardParameterName::PropId,
                            VCardParameterName::Altid,
                            VCardParameterName::Pref,
                        ],
                        JSContactProperty::SpeakToAs,
                        JSContactProperty::Pronouns,
                        [],
                    );
                }
                VCardProperty::Nickname => {
                    state.map_named_entry(
                        &mut entry,
                        &[
                            VCardParameterName::Type,
                            VCardParameterName::Pref,
                            VCardParameterName::PropId,
                            VCardParameterName::Altid,
                            VCardParameterName::Language,
                        ],
                        JSContactProperty::Nicknames,
                        JSContactProperty::Name,
                        [],
                    );
                }
                VCardProperty::Photo | VCardProperty::Logo | VCardProperty::Sound => {
                    let kind = match &entry.entry.name {
                        VCardProperty::Photo => JSContactKind::Photo,
                        VCardProperty::Logo => JSContactKind::Logo,
                        VCardProperty::Sound => JSContactKind::Sound,
                        _ => unreachable!(),
                    };
                    state.map_named_entry(
                        &mut entry,
                        &[
                            VCardParameterName::Mediatype,
                            VCardParameterName::Pref,
                            VCardParameterName::PropId,
                            VCardParameterName::Label,
                        ],
                        JSContactProperty::Media,
                        JSContactProperty::Uri,
                        [(
                            Key::Property(JSContactProperty::Kind),
                            Value::Element(JSContactValue::Kind(kind)),
                        )],
                    );
                }
                VCardProperty::Adr => {
                    let mut params = state.extract_params(
                        &mut entry.entry.params,
                        &[
                            VCardParameterName::Language,
                            VCardParameterName::Phonetic,
                            VCardParameterName::Script,
                            VCardParameterName::Label,
                            VCardParameterName::Geo,
                            VCardParameterName::Tz,
                            VCardParameterName::Cc,
                            VCardParameterName::PropId,
                            VCardParameterName::Altid,
                            VCardParameterName::Pref,
                            VCardParameterName::Type,
                        ],
                    );

                    let prop_id = params.prop_id();
                    let alt_id = params.alt_id();

                    // Locate the address to patch or reset language
                    let addr_patch = if params.language.is_some() {
                        let addr_patch = prop_id
                            .as_deref()
                            .filter(|prop_id| state.has_prop_id(&entry.entry.name, prop_id))
                            .or_else(|| {
                                state.find_prop_id(
                                    &entry.entry.name,
                                    entry.entry.group.as_deref(),
                                    alt_id.as_deref(),
                                )
                            });

                        if addr_patch.is_none() {
                            entry
                                .entry
                                .params
                                .push(VCardParameter::Language(params.language().unwrap()));
                        }
                        addr_patch
                    } else {
                        None
                    };

                    /*

                    0 = ADR-component-pobox ";"
                    1 = ADR-component-ext ";"
                    2 = ADR-component-street ";"
                    3 = ADR-component-locality ";"
                    4 = ADR-component-region ";"
                    5 = ADR-component-code ";"
                    6 = ADR-component-country ";"
                    7 = ADR-component-room ";"
                    8 = ADR-component-apartment ";"
                    9 = ADR-component-floor ";"
                    10 = ADR-component-streetnumber ";"
                    11 = ADR-component-streetname ";"
                    12 = ADR-component-building ";"
                    13 = ADR-component-block ";"
                    14 = ADR-component-subdistrict ";"
                    15 = ADR-component-district ";"
                    16 = ADR-component-landmark ";"
                    17 = ADR-component-direction ";"

                    */

                    let components = entry
                        .text_parts()
                        .enumerate()
                        .filter_map(|(i, value)| {
                            let kind = match i {
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
                                _ => {
                                    return None;
                                }
                            };
                            Value::Object(Map::from(vec![
                                (
                                    Key::Property(JSContactProperty::Kind),
                                    Value::Element(JSContactValue::Kind(kind)),
                                ),
                                (
                                    Key::Property(JSContactProperty::Value),
                                    Value::Str(value.to_string().into()),
                                ),
                            ]))
                            .into()
                        })
                        .collect::<Vec<_>>();

                    if let Some(lang) = params.language() {
                        let path = format!(
                            "{}/{}/{}",
                            JSContactProperty::Addresses.to_cow().as_ref(),
                            addr_patch.unwrap(),
                            JSContactProperty::Components.to_cow().as_ref(),
                        );
                        entry.set_converted_to(&[
                            JSContactProperty::Localizations.to_cow().as_ref(),
                            lang.as_str(),
                            path.as_str(),
                        ]);

                        let locale = state.localizations.entry(lang).or_default();

                        let base_path = path.rsplit_once('/').unwrap().0;
                        for (prop_name, value) in params.into_iter(&entry.entry.name) {
                            locale
                                .push((format!("{}/{}", base_path, prop_name.to_string()), value));
                        }

                        locale.push((path, Value::Array(components)));
                    } else {
                        let entries = state.get_mut_object_or_insert(JSContactProperty::Addresses);
                        let mut addr = Map::from(Vec::with_capacity(4));

                        addr.extend(params.into_iter(&entry.entry.name));
                        addr.insert_unchecked(
                            Key::Property(JSContactProperty::Components),
                            Value::Array(components),
                        );

                        let prop_id = entries.insert_named(prop_id, Value::Object(addr));

                        entry.set_converted_to(&[
                            JSContactProperty::Addresses.to_cow().as_ref(),
                            prop_id.as_str(),
                            JSContactProperty::Components.to_cow().as_ref(),
                        ]);

                        state.track_prop(
                            &entry.entry,
                            JSContactProperty::Addresses,
                            alt_id,
                            prop_id,
                        );
                    }
                }
                VCardProperty::Email => {
                    state.map_named_entry(
                        &mut entry,
                        &[
                            VCardParameterName::Type,
                            VCardParameterName::Pref,
                            VCardParameterName::PropId,
                            VCardParameterName::Label,
                        ],
                        JSContactProperty::Emails,
                        JSContactProperty::Address,
                        [],
                    );
                }
                VCardProperty::Impp => {
                    state.map_named_entry(
                        &mut entry,
                        &[
                            VCardParameterName::Type,
                            VCardParameterName::Pref,
                            VCardParameterName::PropId,
                            VCardParameterName::Label,
                            VCardParameterName::ServiceType,
                            VCardParameterName::Username,
                        ],
                        JSContactProperty::OnlineServices,
                        JSContactProperty::Uri,
                        [],
                    );
                    entry.set_map_name();
                }
                VCardProperty::Lang => {
                    state.map_named_entry(
                        &mut entry,
                        &[
                            VCardParameterName::Type,
                            VCardParameterName::Pref,
                            VCardParameterName::PropId,
                            VCardParameterName::Label,
                        ],
                        JSContactProperty::PreferredLanguages,
                        JSContactProperty::Language,
                        [],
                    );
                }
                VCardProperty::Language | VCardProperty::Prodid | VCardProperty::Uid => {
                    let key = Key::Property(match &entry.entry.name {
                        VCardProperty::Language => JSContactProperty::Language,
                        VCardProperty::Prodid => JSContactProperty::ProdId,
                        VCardProperty::Uid => JSContactProperty::Uid,
                        _ => unreachable!(),
                    });

                    if !state.entries.contains_key(&key) {
                        if let Some(text) = entry.to_text() {
                            entry.set_converted_to(&[key.to_string().as_ref()]);
                            state.entries.insert(key, text);
                        }
                    }
                }
                VCardProperty::Socialprofile => {
                    state.map_named_entry(
                        &mut entry,
                        &[
                            VCardParameterName::Type,
                            VCardParameterName::Pref,
                            VCardParameterName::PropId,
                            VCardParameterName::Label,
                            VCardParameterName::ServiceType,
                            VCardParameterName::Username,
                        ],
                        JSContactProperty::OnlineServices,
                        JSContactProperty::Uri,
                        [],
                    );
                }
                VCardProperty::Tel => {
                    state.map_named_entry(
                        &mut entry,
                        &[
                            VCardParameterName::Type,
                            VCardParameterName::Pref,
                            VCardParameterName::PropId,
                            VCardParameterName::Label,
                        ],
                        JSContactProperty::Phones,
                        JSContactProperty::Number,
                        [],
                    );
                }
                VCardProperty::ContactUri => {
                    state.map_named_entry(
                        &mut entry,
                        &[
                            VCardParameterName::Type,
                            VCardParameterName::Pref,
                            VCardParameterName::PropId,
                            VCardParameterName::Label,
                            VCardParameterName::Mediatype,
                        ],
                        JSContactProperty::Links,
                        JSContactProperty::Uri,
                        [(
                            Key::Property(JSContactProperty::Kind),
                            Value::Element(JSContactValue::Kind(JSContactKind::Contact)),
                        )],
                    );
                }
                VCardProperty::Title => {
                    state.map_named_entry(
                        &mut entry,
                        &[
                            VCardParameterName::PropId,
                            VCardParameterName::Altid,
                            VCardParameterName::Language,
                        ],
                        JSContactProperty::Titles,
                        JSContactProperty::Name,
                        [(
                            Key::Property(JSContactProperty::Kind),
                            Value::Element(JSContactValue::Kind(JSContactKind::Title)),
                        )],
                    );
                }
                VCardProperty::Role => {
                    let prop_id = state
                        .find_prop_id(
                            &VCardProperty::Org,
                            entry.entry.group.as_deref(),
                            entry.entry.alt_id(),
                        )
                        .map(|s| s.to_string());
                    state.map_named_entry(
                        &mut entry,
                        &[
                            VCardParameterName::PropId,
                            VCardParameterName::Altid,
                            VCardParameterName::Language,
                        ],
                        JSContactProperty::Titles,
                        JSContactProperty::Name,
                        [
                            Some((
                                Key::Property(JSContactProperty::Kind),
                                Value::Element(JSContactValue::Kind(JSContactKind::Role)),
                            )),
                            prop_id.map(|prop_id| {
                                (
                                    Key::Property(JSContactProperty::OrganizationId),
                                    Value::Str(prop_id.into()),
                                )
                            }),
                        ]
                        .into_iter()
                        .flatten(),
                    );
                }
                VCardProperty::Hobby | VCardProperty::Interest | VCardProperty::Expertise => {
                    let kind = match &entry.entry.name {
                        VCardProperty::Expertise => JSContactKind::Expertise,
                        VCardProperty::Hobby => JSContactKind::Hobby,
                        VCardProperty::Interest => JSContactKind::Interest,
                        _ => unreachable!(),
                    };
                    state.map_named_entry(
                        &mut entry,
                        &[
                            VCardParameterName::Level,
                            VCardParameterName::Index,
                            VCardParameterName::Label,
                            VCardParameterName::PropId,
                            VCardParameterName::Altid,
                            VCardParameterName::Language,
                        ],
                        JSContactProperty::PersonalInfo,
                        JSContactProperty::Value,
                        [(
                            Key::Property(JSContactProperty::Kind),
                            Value::Element(JSContactValue::Kind(kind)),
                        )],
                    );
                }
                VCardProperty::Org => {
                    let units = entry
                        .text_parts_borrowed()
                        .skip(1)
                        .map(|unit| {
                            Value::Object(Map::from(vec![(
                                Key::Property(JSContactProperty::Name),
                                Value::Str(unit.to_string().into()),
                            )]))
                        })
                        .collect::<Vec<_>>();

                    state.map_named_entry(
                        &mut entry,
                        &[VCardParameterName::Type, VCardParameterName::SortAs],
                        JSContactProperty::Organizations,
                        JSContactProperty::Name,
                        [(!units.is_empty()).then_some({
                            (Key::Property(JSContactProperty::Units), Value::Array(units))
                        })]
                        .into_iter()
                        .flatten(),
                    );
                }
                VCardProperty::Member | VCardProperty::Categories => {
                    let key = if entry.entry.name == VCardProperty::Member {
                        JSContactProperty::Members
                    } else {
                        JSContactProperty::Keywords
                    };

                    entry.set_converted_to(&[key.to_cow().as_ref()]);

                    let obj = state.get_mut_object_or_insert(key);

                    for key in entry.text_parts() {
                        obj.insert(Key::Owned(key), Value::Bool(true));
                    }
                }
                VCardProperty::Created | VCardProperty::Rev => {
                    let key = Key::Property(match &entry.entry.name {
                        VCardProperty::Created => JSContactProperty::Created,
                        VCardProperty::Rev => JSContactProperty::Updated,
                        _ => unreachable!(),
                    });

                    if !state.entries.contains_key(&key) {
                        if let Some(text) = entry.to_timestamp() {
                            entry.set_converted_to(&[key.to_string().as_ref()]);
                            state.entries.insert(key, text);
                        }
                    }
                }
                VCardProperty::Note => {
                    state.map_named_entry(
                        &mut entry,
                        &[
                            VCardParameterName::Author,
                            VCardParameterName::AuthorName,
                            VCardParameterName::Created,
                            VCardParameterName::Language,
                            VCardParameterName::PropId,
                        ],
                        JSContactProperty::Notes,
                        JSContactProperty::Note,
                        [],
                    );
                }
                VCardProperty::Url | VCardProperty::Key | VCardProperty::Caladruri => {
                    let prop = match &entry.entry.name {
                        VCardProperty::Url => JSContactProperty::Links,
                        VCardProperty::Key => JSContactProperty::CryptoKeys,
                        VCardProperty::Caladruri => JSContactProperty::SchedulingAddresses,
                        _ => unreachable!(),
                    };
                    state.map_named_entry(
                        &mut entry,
                        &[
                            VCardParameterName::Type,
                            VCardParameterName::Pref,
                            VCardParameterName::PropId,
                            VCardParameterName::Label,
                            VCardParameterName::Mediatype,
                        ],
                        prop,
                        JSContactProperty::Uri,
                        [],
                    );
                }
                VCardProperty::Fburl | VCardProperty::Caluri => {
                    let kind = match &entry.entry.name {
                        VCardProperty::Fburl => JSContactKind::FreeBusy,
                        VCardProperty::Caluri => JSContactKind::Calendar,
                        _ => unreachable!(),
                    };
                    state.map_named_entry(
                        &mut entry,
                        &[
                            VCardParameterName::Type,
                            VCardParameterName::Pref,
                            VCardParameterName::PropId,
                            VCardParameterName::Label,
                            VCardParameterName::Mediatype,
                        ],
                        JSContactProperty::Calendars,
                        JSContactProperty::Uri,
                        [(
                            Key::Property(JSContactProperty::Kind),
                            Value::Element(JSContactValue::Kind(kind)),
                        )],
                    );
                }
                VCardProperty::Related => {
                    if let Some(text) = entry.to_string() {
                        let mut params = state
                            .extract_params(&mut entry.entry.params, &[VCardParameterName::Type]);
                        entry.set_converted_to(&[
                            JSContactProperty::RelatedTo.to_cow().as_ref(),
                            text.as_ref(),
                        ]);
                        state
                            .get_mut_object_or_insert(JSContactProperty::RelatedTo)
                            .insert(
                                Key::from(text),
                                Value::Object(Map::from(vec![(
                                    Key::Property(JSContactProperty::Relation),
                                    Value::Object(Map::from_iter(
                                        params.types().into_iter().map(|t| {
                                            (Key::from(t.into_string()), Value::Bool(true))
                                        }),
                                    )),
                                )])),
                            );
                    }
                }
                VCardProperty::Tz | VCardProperty::Geo => {
                    let (key, value) = match &entry.entry.name {
                        VCardProperty::Tz => {
                            (Key::Property(JSContactProperty::TimeZone), entry.to_tz())
                        }
                        VCardProperty::Geo => (
                            Key::Property(JSContactProperty::Coordinates),
                            entry.to_text(),
                        ),
                        _ => unreachable!(),
                    };

                    if let Some(value) = value {
                        let prop_id = state
                            .find_prop_id(
                                &VCardProperty::Adr,
                                entry.entry.group.as_deref(),
                                entry.entry.alt_id(),
                            )
                            .map(|s| s.to_string());
                        let addresses =
                            state.get_mut_object_or_insert(JSContactProperty::Addresses);
                        if let Some(addr) = prop_id
                            .clone()
                            .and_then(|prop_id| addresses.get_mut(&Key::Owned(prop_id)))
                            .and_then(|v| v.as_object_mut())
                            .filter(|v| !v.contains_key(&key))
                        {
                            entry.set_converted_to(&[
                                key.to_string().as_ref(),
                                prop_id.clone().unwrap().as_str(),
                            ]);
                            addr.insert(key, value);
                        } else {
                            let prop_id = addresses.insert_named(
                                None,
                                Value::Object(Map::from(vec![(key.clone(), value)])),
                            );
                            entry.set_converted_to(&[key.to_string().as_ref(), prop_id.as_str()]);
                        }
                    }
                }
                VCardProperty::Other(name)
                    if name.eq_ignore_ascii_case("X-ABLabel") && entry.entry.group.is_some() =>
                {
                    if let Some(prop) = state.find_entry_by_group(entry.entry.group.as_deref()) {
                        let prop_id = prop.prop_id.to_string();
                        let prop_js = Key::Property(prop.prop_js.clone());

                        if let Some(obj) = state
                            .entries
                            .get_mut(&prop_js)
                            .and_then(|v| v.as_object_mut())
                            .and_then(|v| v.get_mut(&Key::Owned(prop_id.clone())))
                            .and_then(|v| v.as_object_mut())
                            .filter(|v| !v.contains_key(&Key::Property(JSContactProperty::Label)))
                        {
                            if let Some(value) = entry.to_text() {
                                obj.insert_unchecked(
                                    Key::Property(JSContactProperty::Label),
                                    value,
                                );
                                entry.set_converted_to(&[
                                    prop_js.to_string().as_ref(),
                                    prop_id.as_str(),
                                    JSContactProperty::Label.to_cow().as_ref(),
                                ]);
                            }
                        }
                    }
                }
                VCardProperty::Jsprop => {
                    if let Some(VCardParameter::Jsptr(ptr)) = entry.entry.params.first() {
                        let ptr = JsonPointer::<JSContactProperty>::parse(ptr);

                        if let Some(VCardValue::Text(text)) = entry.entry.values.first() {
                            if let Ok(jscontact) = JSContact::parse(text) {
                                patch_objects.push((ptr, jscontact.0.into_owned()));
                                continue;
                            }
                        }
                    }
                }
                VCardProperty::Version
                | VCardProperty::Xml
                | VCardProperty::Gender
                | VCardProperty::Clientpidmap
                | VCardProperty::Other(_) => (),
                VCardProperty::Begin | VCardProperty::End => {
                    continue;
                }
            }

            state.add_conversion_props(entry);
        }

        let mut jscontact = state.into_jscontact();

        for (ptr, patch) in patch_objects {
            jscontact.0.patch_jptr(ptr.iter(), patch);
        }

        jscontact
    }
}
