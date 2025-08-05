/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

use crate::{
    common::CalendarScale,
    jscontact::{
        JSContact, JSContactGrammaticalGender, JSContactKind, JSContactLevel,
        JSContactPhoneticSystem, JSContactProperty, JSContactType, JSContactValue,
    },
    vcard::{
        VCard, VCardEntry, VCardGramGender, VCardKind, VCardLevel, VCardParameter,
        VCardParameterName, VCardPhonetic, VCardProperty, VCardType, VCardValue, VCardValueType,
        ValueType,
    },
};
use ahash::AHashMap;
use jmap_tools::{JsonPointer, JsonPointerHandler, Key, Map, Property, Value};
use std::{
    borrow::Cow,
    collections::{hash_map::Entry, HashMap},
};

struct PropIdKey {
    prop: VCardProperty,
    prop_js: JSContactProperty,
    group: Option<String>,
    alt_id: Option<String>,
    prop_id: String,
}

#[derive(Debug, Default)]
struct VCardConvertedProperty {
    name: Option<VCardProperty>,
    params: VCardParams,
}

#[derive(Debug, Default)]
struct VCardParams(
    AHashMap<VCardParameterName, Vec<Value<'static, JSContactProperty, JSContactValue>>>,
);

#[allow(clippy::type_complexity)]
struct State {
    entries: AHashMap<
        Key<'static, JSContactProperty>,
        Value<'static, JSContactProperty, JSContactValue>,
    >,
    vcard_converted_properties: AHashMap<String, VCardConvertedProperty>,
    vcard_properties: Vec<Value<'static, JSContactProperty, JSContactValue>>,
    localizations:
        HashMap<String, Vec<(String, Value<'static, JSContactProperty, JSContactValue>)>>,
    default_language: Option<String>,
    prop_ids: Vec<PropIdKey>,
    name_alt_id: Option<String>,
    has_fn: bool,
    has_fn_localization: bool,
    has_n: bool,
    has_n_localization: bool,
    has_gram_gender: bool,
}

struct EntryState {
    entry: VCardEntry,
    converted_to: Option<String>,
    map_name: bool,
}

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

impl State {
    fn new(vcard: &mut VCard) -> Self {
        let mut entries = AHashMap::with_capacity(vcard.entries.len());

        entries.extend([
            (
                Key::Property(JSContactProperty::Type),
                Value::Element(JSContactValue::Type(JSContactType::Card)),
            ),
            (
                Key::Property(JSContactProperty::Version),
                Value::Str("1.0".into()),
            ),
        ]);

        // Find the default language and the most "popular" alt ids
        let mut default_language = None;
        let mut alt_ids: AHashMap<(&VCardProperty, &str), usize> = AHashMap::new();
        for entry in &vcard.entries {
            match &entry.name {
                VCardProperty::Language => {
                    if let Some(VCardValue::Text(lang)) = entry.values.first() {
                        default_language = Some(lang.clone());
                    }
                }
                VCardProperty::N | VCardProperty::Adr => {
                    if let Some(alt_id) = entry.alt_id() {
                        *alt_ids.entry((&entry.name, alt_id)).or_default() += 1;
                    }
                }
                _ => (),
            }
        }

        // Find the alt ids with the highest count
        let mut name_alt_id = None;
        let mut name_alt_id_count = 0;
        for (&(prop, alt_id), &count) in &alt_ids {
            match prop {
                VCardProperty::N if count > name_alt_id_count => {
                    name_alt_id = Some(alt_id.to_string());
                    name_alt_id_count = count;
                }
                _ => (),
            }
        }

        // Move entries without a language to the top
        vcard.entries.sort_unstable_by_key(|entry| {
            let lang = entry.language();
            let weight = u32::from(lang.is_some() && default_language.as_deref() != lang);

            match &entry.name {
                VCardProperty::Birthplace | VCardProperty::Deathplace | VCardProperty::Role => {
                    weight + 2
                }
                VCardProperty::Other(name) if name.eq_ignore_ascii_case("X-ABLabel") => weight + 3,
                _ => weight,
            }
        });

        Self {
            entries,
            default_language,
            localizations: Default::default(),
            prop_ids: Default::default(),
            vcard_converted_properties: Default::default(),
            vcard_properties: Default::default(),
            name_alt_id,
            has_fn: false,
            has_n: false,
            has_n_localization: false,
            has_fn_localization: false,
            has_gram_gender: false,
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn map_named_entry(
        &mut self,
        entry: &mut EntryState,
        extract: &[VCardParameterName],
        top_property_name: JSContactProperty,
        value_property_name: JSContactProperty,
        extra_properties: impl IntoIterator<
            Item = (
                Key<'static, JSContactProperty>,
                Value<'static, JSContactProperty, JSContactValue>,
            ),
        >,
    ) {
        let value = if !matches!(
            entry.entry.name,
            VCardProperty::Anniversary | VCardProperty::Bday | VCardProperty::Deathdate
        ) {
            entry.to_text()
        } else {
            entry.to_anniversary()
        };

        if let Some(value) = value {
            let mut params = self.extract_params(&mut entry.entry.params, extract);
            let prop_id = params.prop_id();
            let alt_id = params.alt_id();
            let sub_property = top_property_name.sub_property();

            // Locate the address to patch or reset language
            if let Some(language) = params.language() {
                if let Some(patch) = prop_id
                    .as_deref()
                    .filter(|prop_id| self.has_prop_id(&entry.entry.name, prop_id))
                    .or_else(|| {
                        self.find_prop_id(
                            &entry.entry.name,
                            entry.entry.group.as_deref(),
                            alt_id.as_deref(),
                        )
                    })
                    .map(|prop_id| {
                        if let Some(sub_property) = sub_property.as_ref() {
                            format!(
                                "{}/{}/{}/{}",
                                top_property_name.to_cow().as_ref(),
                                sub_property.to_cow().as_ref(),
                                prop_id,
                                value_property_name.to_cow().as_ref()
                            )
                        } else {
                            format!(
                                "{}/{}/{}",
                                top_property_name.to_cow().as_ref(),
                                prop_id,
                                value_property_name.to_cow().as_ref()
                            )
                        }
                    })
                {
                    entry.set_converted_to(&[
                        JSContactProperty::Localizations.to_cow().as_ref(),
                        language.as_str(),
                        patch.as_str(),
                    ]);

                    let localizations = self.localizations.entry(language).or_default();
                    let mut base_path = None;

                    for (prop, value) in params.into_iter(&entry.entry.name) {
                        let base_path = base_path.get_or_insert_with(|| {
                            patch.rsplit_once('/').map(|(base, _)| base).unwrap()
                        });
                        localizations.push((format!("{}/{}", base_path, prop.to_string()), value));
                    }

                    localizations.push((patch, value));
                    return;
                } else {
                    entry.entry.params.push(VCardParameter::Language(language));
                }
            }

            let mut entries = self.get_mut_object_or_insert(top_property_name.clone());
            if let Some(sub_property) = sub_property.clone() {
                entries = entries
                    .insert_or_get_mut(sub_property, Value::Object(Map::from(vec![])))
                    .as_object_mut()
                    .unwrap();
            }

            let mut obj = vec![(Key::Property(value_property_name.clone()), value)];
            for (key, value) in extra_properties {
                obj.push((key, value));
            }

            obj.extend(params.into_iter(&entry.entry.name));
            let prop_id = entries.insert_named(prop_id, Value::Object(Map::from(obj)));

            if let Some(sub_property) = sub_property {
                entry.set_converted_to(&[
                    top_property_name.to_cow().as_ref(),
                    sub_property.to_cow().as_ref(),
                    prop_id.as_str(),
                    value_property_name.to_cow().as_ref(),
                ]);
            } else {
                entry.set_converted_to(&[
                    top_property_name.to_cow().as_ref(),
                    prop_id.as_str(),
                    value_property_name.to_cow().as_ref(),
                ]);
            }

            self.track_prop(&entry.entry, top_property_name, alt_id, prop_id);
        }
    }

    fn extract_params(
        &self,
        params: &mut Vec<VCardParameter>,
        extract: &[VCardParameterName],
    ) -> ExtractedParams {
        let mut p = ExtractedParams::default();

        for param in std::mem::take(params) {
            match param {
                VCardParameter::Language(v)
                    if p.language.is_none()
                        && self.default_language.as_ref().is_none_or(|lang| lang != &v)
                        && extract.contains(&VCardParameterName::Language) =>
                {
                    p.language = Some(v);
                }
                VCardParameter::Pref(v)
                    if p.pref.is_none() && extract.contains(&VCardParameterName::Pref) =>
                {
                    p.pref = Some(v);
                }
                VCardParameter::Author(v)
                    if p.author.is_none() && extract.contains(&VCardParameterName::Author) =>
                {
                    p.author = Some(v);
                }
                VCardParameter::AuthorName(v)
                    if p.author_name.is_none()
                        && extract.contains(&VCardParameterName::AuthorName) =>
                {
                    p.author_name = Some(v);
                }
                VCardParameter::Mediatype(v)
                    if p.media_type.is_none()
                        && extract.contains(&VCardParameterName::Mediatype) =>
                {
                    p.media_type = Some(v);
                }
                VCardParameter::Calscale(v)
                    if p.calscale.is_none() && extract.contains(&VCardParameterName::Calscale) =>
                {
                    p.calscale = Some(v);
                }
                VCardParameter::SortAs(v)
                    if p.sort_as.is_none() && extract.contains(&VCardParameterName::SortAs) =>
                {
                    p.sort_as = Some(v);
                }
                VCardParameter::Geo(v)
                    if p.geo.is_none() && extract.contains(&VCardParameterName::Geo) =>
                {
                    p.geo = Some(v);
                }
                VCardParameter::Tz(v)
                    if p.tz.is_none() && extract.contains(&VCardParameterName::Tz) =>
                {
                    p.tz = Some(v);
                }
                VCardParameter::Index(v)
                    if p.index.is_none() && extract.contains(&VCardParameterName::Index) =>
                {
                    p.index = Some(v);
                }
                VCardParameter::Level(v)
                    if p.level.is_none() && extract.contains(&VCardParameterName::Level) =>
                {
                    p.level = Some(v);
                }
                VCardParameter::Cc(v)
                    if p.country_code.is_none() && extract.contains(&VCardParameterName::Cc) =>
                {
                    p.country_code = Some(v);
                }
                VCardParameter::Created(v)
                    if p.created.is_none() && extract.contains(&VCardParameterName::Created) =>
                {
                    p.created = Some(v);
                }
                VCardParameter::Label(v)
                    if p.label.is_none() && extract.contains(&VCardParameterName::Label) =>
                {
                    p.label = Some(v);
                }
                VCardParameter::Phonetic(v)
                    if p.phonetic_system.is_none()
                        && extract.contains(&VCardParameterName::Phonetic) =>
                {
                    p.phonetic_system = Some(v);
                }
                VCardParameter::Script(v)
                    if p.phonetic_script.is_none()
                        && extract.contains(&VCardParameterName::Script) =>
                {
                    p.phonetic_script = Some(v);
                }
                VCardParameter::ServiceType(v)
                    if p.service_type.is_none()
                        && extract.contains(&VCardParameterName::ServiceType) =>
                {
                    p.service_type = Some(v);
                }
                VCardParameter::Username(v)
                    if p.username.is_none() && extract.contains(&VCardParameterName::Username) =>
                {
                    p.username = Some(v);
                }
                VCardParameter::PropId(v)
                    if p.prop_id.is_none() && extract.contains(&VCardParameterName::PropId) =>
                {
                    p.prop_id = Some(v);
                }
                VCardParameter::Altid(v) if p.alt_id.is_none() => {
                    p.alt_id = Some(v.clone());
                    params.push(VCardParameter::Altid(v));
                }
                VCardParameter::Type(typ) if extract.contains(&VCardParameterName::Type) => {
                    if p.types.is_empty() {
                        p.types = typ;
                    } else {
                        p.types.extend(typ);
                    }
                }
                _ => {
                    params.push(param);
                }
            }
        }

        p
    }

    #[inline]
    fn get_mut_object_or_insert(
        &mut self,
        key: JSContactProperty,
    ) -> &mut Map<'static, JSContactProperty, JSContactValue> {
        self.entries
            .entry(Key::Property(key))
            .or_insert_with(|| Value::Object(Map::from(Vec::new())))
            .as_object_mut()
            .unwrap()
    }

    #[inline]
    fn has_property(&self, key: JSContactProperty) -> bool {
        self.entries.contains_key(&Key::Property(key))
    }

    fn add_conversion_props(&mut self, mut entry: EntryState) {
        if let Some(converted_to) = entry.converted_to.take() {
            if entry.map_name || !entry.entry.params.is_empty() || entry.entry.group.is_some() {
                let mut value_type = None;

                match self.vcard_converted_properties.entry(converted_to) {
                    Entry::Occupied(mut conv_prop) => {
                        entry.jcal_parameters(&mut conv_prop.get_mut().params, &mut value_type);
                    }
                    Entry::Vacant(conv_prop) => {
                        let mut params = VCardParams::default();
                        entry.jcal_parameters(&mut params, &mut value_type);
                        if let Some(value_type) = value_type {
                            params.0.insert(
                                VCardParameterName::Value,
                                vec![Value::Str(value_type.into_string())],
                            );
                        }
                        if !params.0.is_empty() || entry.map_name {
                            conv_prop.insert(VCardConvertedProperty {
                                name: if entry.map_name {
                                    Some(entry.entry.name)
                                } else {
                                    None
                                },
                                params,
                            });
                        }
                    }
                }
            }
        } else {
            let mut value_type = None;
            let mut params = VCardParams::default();

            entry.jcal_parameters(&mut params, &mut value_type);

            let values = if entry.entry.values.len() == 1 {
                entry
                    .entry
                    .values
                    .into_iter()
                    .next()
                    .unwrap()
                    .into_jscontact_value(value_type.as_ref())
            } else {
                let mut values = Vec::with_capacity(entry.entry.values.len());
                for value in entry.entry.values {
                    values.push(value.into_jscontact_value(value_type.as_ref()));
                }
                Value::Array(values)
            };
            self.vcard_properties.push(Value::Array(vec![
                Value::Str(entry.entry.name.into_string()),
                Value::Object(
                    params
                        .into_jscontact_value()
                        .unwrap_or(Map::from(Vec::new())),
                ),
                Value::Str(
                    value_type
                        .map(|v| v.into_string())
                        .unwrap_or(Cow::Borrowed("text")),
                ),
                values,
            ]));
        }
    }

    #[inline]
    fn track_prop(
        &mut self,
        entry: &VCardEntry,
        prop_js: JSContactProperty,
        alt_id: Option<String>,
        prop_id: String,
    ) {
        self.prop_ids.push(PropIdKey {
            prop_id,
            prop_js,
            prop: entry.name.clone(),
            group: entry.group.clone(),
            alt_id,
        });
    }

    fn find_prop_id(
        &self,
        prop: &VCardProperty,
        group: Option<&str>,
        alt_id: Option<&str>,
    ) -> Option<&str> {
        self.prop_ids
            .iter()
            .find(|p| {
                p.prop == *prop && p.group.as_deref() == group && p.alt_id.as_deref() == alt_id
            })
            .map(|p| p.prop_id.as_str())
    }
    fn find_entry_by_group(&self, group: Option<&str>) -> Option<&PropIdKey> {
        self.prop_ids.iter().find(|p| p.group.as_deref() == group)
    }

    fn has_prop_id(&self, prop: &VCardProperty, prop_id: &str) -> bool {
        self.prop_ids
            .iter()
            .any(|p| p.prop == *prop && p.prop_id == prop_id)
    }

    fn into_jscontact(mut self) -> JSContact<'static> {
        if !self.localizations.is_empty() {
            self.entries.insert(
                Key::Property(JSContactProperty::Localizations),
                Value::Object(
                    self.localizations
                        .into_iter()
                        .map(|(lang, locals)| {
                            (
                                Key::Owned(lang),
                                Value::Object(
                                    locals
                                        .into_iter()
                                        .map(|(key, value)| (Key::Owned(key), value))
                                        .collect(),
                                ),
                            )
                        })
                        .collect(),
                ),
            );
        }

        let mut vcard_obj = Map::from(Vec::new());
        if !self.vcard_converted_properties.is_empty() {
            let mut converted_properties =
                Map::from(Vec::with_capacity(self.vcard_converted_properties.len()));

            for (converted_to, props) in self.vcard_converted_properties {
                let mut obj = Map::from(Vec::with_capacity(2));
                if let Some(params) = props.params.into_jscontact_value() {
                    obj.insert(
                        Key::Property(JSContactProperty::Parameters),
                        Value::Object(params),
                    );
                }
                if let Some(name) = props.name {
                    obj.insert(
                        Key::Property(JSContactProperty::Name),
                        Value::Str(name.into_string()),
                    );
                }

                converted_properties.insert_unchecked(Key::Owned(converted_to), Value::Object(obj));
            }

            vcard_obj.insert_unchecked(
                Key::Property(JSContactProperty::ConvertedProperties),
                Value::Object(converted_properties),
            );
        }

        if !self.vcard_properties.is_empty() {
            vcard_obj.insert_unchecked(
                Key::Property(JSContactProperty::Properties),
                Value::Array(self.vcard_properties),
            );
        }

        if !vcard_obj.is_empty() {
            self.entries.insert(
                Key::Property(JSContactProperty::VCard),
                Value::Object(vcard_obj),
            );
        }

        JSContact(Value::Object(self.entries.into_iter().collect()))
    }
}

impl VCardValue {
    fn into_jscontact_value(
        self,
        value_type: Option<&VCardValueType>,
    ) -> Value<'static, JSContactProperty, JSContactValue> {
        match self {
            VCardValue::Text(v) => Value::Str(v.into()),
            VCardValue::Integer(v) => Value::Number(v.into()),
            VCardValue::Float(v) => Value::Number(v.into()),
            VCardValue::Boolean(v) => Value::Bool(v),
            VCardValue::PartialDateTime(v) => {
                let mut out = String::new();
                let _ = v.format_as_vcard(
                    &mut out,
                    value_type.unwrap_or(if v.has_date() && v.has_time() {
                        &VCardValueType::Timestamp
                    } else {
                        &VCardValueType::DateAndOrTime
                    }),
                );
                Value::Str(out.into())
            }
            VCardValue::Binary(v) => Value::Str(v.to_string().into()),
            VCardValue::Sex(v) => Value::Str(v.as_str().into()),
            VCardValue::GramGender(v) => Value::Str(v.as_str().into()),
            VCardValue::Kind(v) => Value::Str(v.as_str().into()),
        }
    }
}

#[allow(clippy::wrong_self_convention)]
impl EntryState {
    fn new(entry: VCardEntry) -> Self {
        Self {
            entry,
            converted_to: None,
            map_name: false,
        }
    }

    fn jcal_parameters(
        &mut self,
        params: &mut VCardParams,
        value_type: &mut Option<VCardValueType>,
    ) {
        if self.entry.params.is_empty() && self.entry.group.is_none() {
            return;
        }
        let (default_type, _) = self.entry.name.default_types();
        let default_type = match default_type {
            ValueType::Vcard(t) => t,
            ValueType::Kind | ValueType::Sex | ValueType::GramGender => VCardValueType::Text,
        };

        for param in std::mem::take(&mut self.entry.params) {
            match param {
                VCardParameter::Language(v) => params
                    .0
                    .entry(VCardParameterName::Language)
                    .or_default()
                    .push(v.into()),
                VCardParameter::Value(v) => {
                    if let Some(v) = v.into_iter().next() {
                        if v != default_type {
                            *value_type = Some(v);
                        }
                    }
                }
                VCardParameter::Pref(v) => params
                    .0
                    .entry(VCardParameterName::Pref)
                    .or_default()
                    .push(v.to_string().into()),
                VCardParameter::Altid(v) => params
                    .0
                    .entry(VCardParameterName::Altid)
                    .or_default()
                    .push(v.into()),
                VCardParameter::Pid(v) => {
                    params
                        .0
                        .entry(VCardParameterName::Pid)
                        .or_default()
                        .extend(v.into_iter().map(Into::into));
                }
                VCardParameter::Type(v) => {
                    params
                        .0
                        .entry(VCardParameterName::Type)
                        .or_default()
                        .extend(v.into_iter().map(|t| Value::from(t.into_string())));
                }
                VCardParameter::Mediatype(v) => params
                    .0
                    .entry(VCardParameterName::Mediatype)
                    .or_default()
                    .push(v.into()),
                VCardParameter::Calscale(v) => params
                    .0
                    .entry(VCardParameterName::Calscale)
                    .or_default()
                    .push(v.into_string().into()),
                VCardParameter::SortAs(v) => params
                    .0
                    .entry(VCardParameterName::SortAs)
                    .or_default()
                    .push(v.into()),
                VCardParameter::Geo(v) => params
                    .0
                    .entry(VCardParameterName::Geo)
                    .or_default()
                    .push(v.into()),
                VCardParameter::Tz(v) => params
                    .0
                    .entry(VCardParameterName::Tz)
                    .or_default()
                    .push(v.into()),
                VCardParameter::Index(v) => params
                    .0
                    .entry(VCardParameterName::Index)
                    .or_default()
                    .push(v.to_string().into()),
                VCardParameter::Level(v) => params
                    .0
                    .entry(VCardParameterName::Level)
                    .or_default()
                    .push(v.as_str().into()),
                VCardParameter::Group(_) => {
                    // The GROUP parameter (Section 7.1 of [RFC7095]) does not convert to JSContact.
                    // It exclusively is for use in jCard and MUST NOT be set in a vCard.
                }
                VCardParameter::Cc(v) => params
                    .0
                    .entry(VCardParameterName::Cc)
                    .or_default()
                    .push(v.into()),
                VCardParameter::Author(v) => params
                    .0
                    .entry(VCardParameterName::Author)
                    .or_default()
                    .push(v.into()),
                VCardParameter::AuthorName(v) => params
                    .0
                    .entry(VCardParameterName::AuthorName)
                    .or_default()
                    .push(v.into()),
                VCardParameter::Created(v) => params
                    .0
                    .entry(VCardParameterName::Created)
                    .or_default()
                    .push(v.into()),
                VCardParameter::Derived(v) => params
                    .0
                    .entry(VCardParameterName::Derived)
                    .or_default()
                    .push(v.to_string().into()),
                VCardParameter::Label(v) => params
                    .0
                    .entry(VCardParameterName::Label)
                    .or_default()
                    .push(v.into()),
                VCardParameter::Phonetic(v) => params
                    .0
                    .entry(VCardParameterName::Phonetic)
                    .or_default()
                    .push(v.into_string().into()),
                VCardParameter::PropId(v) => params
                    .0
                    .entry(VCardParameterName::PropId)
                    .or_default()
                    .push(v.into()),
                VCardParameter::Script(v) => params
                    .0
                    .entry(VCardParameterName::Script)
                    .or_default()
                    .push(v.into()),
                VCardParameter::ServiceType(v) => params
                    .0
                    .entry(VCardParameterName::ServiceType)
                    .or_default()
                    .push(v.into()),
                VCardParameter::Username(v) => params
                    .0
                    .entry(VCardParameterName::Username)
                    .or_default()
                    .push(v.into()),
                VCardParameter::Jsptr(v) => params
                    .0
                    .entry(VCardParameterName::Jsptr)
                    .or_default()
                    .push(v.into()),
                VCardParameter::Other(v) => {
                    if v.len() > 1 {
                        let mut v = v.into_iter();
                        let name = v.next().unwrap();

                        if !name.eq_ignore_ascii_case("group") {
                            params
                                .0
                                .entry(VCardParameterName::Other(name))
                                .or_default()
                                .extend(v.map(Into::into));
                        }
                    }
                }
            }
        }

        if let Some(group) = self.entry.group.take() {
            params
                .0
                .insert(VCardParameterName::Group, vec![group.into()]);
        }
    }

    fn set_converted_to(&mut self, converted_to: &[&str]) {
        self.converted_to = Some(JsonPointer::<JSContactProperty>::encode(converted_to));
    }

    fn set_map_name(&mut self) {
        self.map_name = true;
    }

    fn to_text(&mut self) -> Option<Value<'static, JSContactProperty, JSContactValue>> {
        self.to_string().map(Into::into)
    }

    fn to_string(&mut self) -> Option<Cow<'static, str>> {
        let mut values = std::mem::take(&mut self.entry.values).into_iter();
        match values.next()? {
            VCardValue::Text(v) => Some(Cow::Owned(v)),
            VCardValue::Binary(data) => Some(Cow::Owned(data.to_string())),
            VCardValue::Sex(v) => Some(v.as_str().into()),
            VCardValue::GramGender(v) => Some(v.as_str().into()),
            VCardValue::Kind(v) => Some(v.as_str().into()),
            other => {
                self.entry.values.push(other);
                self.entry.values.extend(values);
                None
            }
        }
    }

    fn text_parts(&mut self) -> impl Iterator<Item = String> + '_ {
        self.entry
            .values
            .drain(..)
            .filter_map(|value| value.into_text())
    }

    fn text_parts_borrowed(&self) -> impl Iterator<Item = &str> + '_ {
        self.entry.values.iter().filter_map(|value| value.as_text())
    }

    fn to_kind(&mut self) -> Option<Value<'static, JSContactProperty, JSContactValue>> {
        let mut values = std::mem::take(&mut self.entry.values).into_iter();
        match values.next()? {
            VCardValue::Kind(v) => Value::Element(JSContactValue::Kind(match v {
                VCardKind::Application => JSContactKind::Application,
                VCardKind::Device => JSContactKind::Device,
                VCardKind::Group => JSContactKind::Group,
                VCardKind::Individual => JSContactKind::Individual,
                VCardKind::Location => JSContactKind::Location,
                VCardKind::Org => JSContactKind::Org,
            }))
            .into(),
            VCardValue::Text(v) => Value::Str(v.into()).into(),
            other => {
                self.entry.values.push(other);
                self.entry.values.extend(values);
                None
            }
        }
    }

    fn to_gram_gender(&mut self) -> Option<Value<'static, JSContactProperty, JSContactValue>> {
        let mut values = std::mem::take(&mut self.entry.values).into_iter();
        match values.next()? {
            VCardValue::GramGender(v) => {
                Value::Element(JSContactValue::GrammaticalGender(match v {
                    VCardGramGender::Animate => JSContactGrammaticalGender::Animate,
                    VCardGramGender::Common => JSContactGrammaticalGender::Common,
                    VCardGramGender::Feminine => JSContactGrammaticalGender::Feminine,
                    VCardGramGender::Inanimate => JSContactGrammaticalGender::Inanimate,
                    VCardGramGender::Masculine => JSContactGrammaticalGender::Masculine,
                    VCardGramGender::Neuter => JSContactGrammaticalGender::Neuter,
                }))
                .into()
            }
            VCardValue::Text(v) => Value::Str(v.into()).into(),
            other => {
                self.entry.values.push(other);
                self.entry.values.extend(values);
                None
            }
        }
    }

    fn to_timestamp(&mut self) -> Option<Value<'static, JSContactProperty, JSContactValue>> {
        if let Some(VCardValue::PartialDateTime(dt)) = self.entry.values.first() {
            if let Some(timestamp) = dt.to_timestamp() {
                return Value::Element(JSContactValue::Timestamp(timestamp)).into();
            }
        }

        None
    }

    fn to_tz(&mut self) -> Option<Value<'static, JSContactProperty, JSContactValue>> {
        let mut values = std::mem::take(&mut self.entry.values).into_iter();
        match values.next()? {
            VCardValue::PartialDateTime(v) if v.has_zone() => {
                let hour = v.hour.unwrap_or_default();

                Value::Str(
                    if hour != 0 {
                        format!("Etc/GMT{}{}", if v.tz_minus { "+" } else { "-" }, hour)
                    } else {
                        "Etc/UTC".to_string()
                    }
                    .into(),
                )
                .into()
            }
            VCardValue::Text(v) => Value::Str(v.into()).into(),
            other => {
                self.entry.values.push(other);
                self.entry.values.extend(values);
                None
            }
        }
    }

    fn to_anniversary(&mut self) -> Option<Value<'static, JSContactProperty, JSContactValue>> {
        if let Some(VCardValue::PartialDateTime(dt)) = self.entry.values.first() {
            if let Some(timestamp) = dt.to_timestamp() {
                Some(Value::Object(Map::from(vec![
                    (
                        Key::Property(JSContactProperty::Type),
                        Value::Element(JSContactValue::Type(JSContactType::Timestamp)),
                    ),
                    (
                        Key::Property(JSContactProperty::Utc),
                        Value::Element(JSContactValue::Timestamp(timestamp)),
                    ),
                ])))
            } else if dt.day.is_some() || dt.month.is_some() || dt.year.is_some() {
                let mut props = Map::from(vec![(
                    Key::Property(JSContactProperty::Type),
                    Value::Element(JSContactValue::Type(JSContactType::PartialDate)),
                )]);
                if let Some(year) = dt.year {
                    props.insert_unchecked(
                        Key::Property(JSContactProperty::Year),
                        Value::Number((year as u64).into()),
                    );
                }
                if let Some(month) = dt.month {
                    props.insert_unchecked(
                        Key::Property(JSContactProperty::Month),
                        Value::Number((month as u64).into()),
                    );
                }
                if let Some(day) = dt.day {
                    props.insert_unchecked(
                        Key::Property(JSContactProperty::Day),
                        Value::Number((day as u64).into()),
                    );
                }

                Some(Value::Object(props))
            } else {
                None
            }
        } else {
            None
        }
    }

    fn non_default_language(&self, default_language: Option<&str>) -> Option<&str> {
        self.entry
            .language()
            .filter(|&lang| Some(lang) != default_language)
    }
}

impl VCardParams {
    pub fn into_jscontact_value(self) -> Option<Map<'static, JSContactProperty, JSContactValue>> {
        if !self.0.is_empty() {
            let mut obj = Map::from(Vec::with_capacity(self.0.len()));

            for (param, value) in self.0 {
                let value = if value.len() > 1 {
                    Value::Array(value)
                } else {
                    value.into_iter().next().unwrap()
                };
                obj.insert_unchecked(Key::from(param.into_string()), value);
            }
            Some(obj)
        } else {
            None
        }
    }
}

#[derive(Default)]
struct ExtractedParams {
    language: Option<String>,
    pref: Option<u32>,
    author: Option<String>,
    author_name: Option<String>,
    phonetic_system: Option<VCardPhonetic>,
    phonetic_script: Option<String>,
    media_type: Option<String>,
    calscale: Option<CalendarScale>,
    sort_as: Option<String>,
    prop_id: Option<String>,
    alt_id: Option<String>,
    types: Vec<VCardType>,
    geo: Option<String>,
    tz: Option<String>,
    index: Option<u32>,
    level: Option<VCardLevel>,
    country_code: Option<String>,
    created: Option<i64>,
    label: Option<String>,
    service_type: Option<String>,
    username: Option<String>,
}

impl ExtractedParams {
    fn prop_id(&mut self) -> Option<String> {
        self.prop_id.take()
    }

    fn alt_id(&mut self) -> Option<String> {
        self.alt_id.take()
    }

    pub fn language(&mut self) -> Option<String> {
        self.language.take()
    }

    pub fn types(&mut self) -> Vec<VCardType> {
        std::mem::take(&mut self.types)
    }

    #[allow(clippy::type_complexity)]
    fn into_iter(
        mut self,
        property: &VCardProperty,
    ) -> impl Iterator<
        Item = (
            Key<'static, JSContactProperty>,
            Value<'static, JSContactProperty, JSContactValue>,
        ),
    > {
        let mut contexts: Option<
            Vec<(
                Key<'static, JSContactProperty>,
                Value<'static, JSContactProperty, JSContactValue>,
            )>,
        > = None;
        let mut features: Option<
            Vec<(
                Key<'static, JSContactProperty>,
                Value<'static, JSContactProperty, JSContactValue>,
            )>,
        > = None;

        if !self.types.is_empty() {
            let is_phone = matches!(property, VCardProperty::Tel);

            for typ in std::mem::take(&mut self.types) {
                if is_phone
                    && matches!(
                        typ,
                        VCardType::Fax
                            | VCardType::Cell
                            | VCardType::Video
                            | VCardType::Pager
                            | VCardType::Textphone
                            | VCardType::MainNumber
                            | VCardType::Text
                            | VCardType::Voice
                    )
                {
                    features.get_or_insert_default()
                } else {
                    contexts.get_or_insert_default()
                }
                .push((Key::from(typ.into_string()), Value::Bool(true)));
            }
        }

        let author = if self.author.is_some() || self.author_name.is_some() {
            Value::Object(Map::from_iter(
                [
                    self.author_name.map(|name| {
                        (
                            Key::Property(JSContactProperty::Name),
                            Value::Str(name.into()),
                        )
                    }),
                    self.author.map(|author| {
                        (
                            Key::Property(JSContactProperty::Uri),
                            Value::Str(author.into()),
                        )
                    }),
                ]
                .into_iter()
                .flatten(),
            ))
            .into()
        } else {
            None
        };

        [
            (
                Key::Property(JSContactProperty::Contexts),
                contexts.map(|v| Value::Object(Map::from(v))),
            ),
            (
                Key::Property(JSContactProperty::Features),
                features.map(|v| Value::Object(Map::from(v))),
            ),
            (
                Key::Property(JSContactProperty::Language),
                self.language.map(Into::into).map(Value::Str),
            ),
            (
                Key::Property(JSContactProperty::Pref),
                self.pref.map(|v| Value::Number((v as u64).into())),
            ),
            (Key::Property(JSContactProperty::Author), author),
            (
                Key::Property(JSContactProperty::MediaType),
                self.media_type.map(Into::into).map(Value::Str),
            ),
            (
                Key::Property(JSContactProperty::PhoneticSystem),
                self.phonetic_system.map(|v| match v {
                    VCardPhonetic::Ipa => {
                        Value::Element(JSContactValue::PhoneticSystem(JSContactPhoneticSystem::Ipa))
                    }
                    VCardPhonetic::Jyut => Value::Element(JSContactValue::PhoneticSystem(
                        JSContactPhoneticSystem::Jyut,
                    )),
                    VCardPhonetic::Piny => Value::Element(JSContactValue::PhoneticSystem(
                        JSContactPhoneticSystem::Piny,
                    )),
                    VCardPhonetic::Script => Value::Element(JSContactValue::PhoneticSystem(
                        JSContactPhoneticSystem::Script,
                    )),
                    VCardPhonetic::Other(value) => Value::Str(value.into()),
                }),
            ),
            (
                Key::Property(JSContactProperty::PhoneticScript),
                self.phonetic_script.map(Into::into).map(Value::Str),
            ),
            (
                Key::Property(JSContactProperty::CalendarScale),
                self.calscale
                    .map(|v| Value::Element(JSContactValue::CalendarScale(v))),
            ),
            (
                Key::Property(JSContactProperty::SortAs),
                self.sort_as.map(|sort_as| {
                    if matches!(property, VCardProperty::N) {
                        if let Some((surname, given)) =
                            sort_as.split_once(',').and_then(|(surname, given)| {
                                let surname = surname.trim();
                                let given = given.trim();

                                if surname.is_empty() || given.is_empty() {
                                    None
                                } else {
                                    Some((surname.to_string(), given.to_string()))
                                }
                            })
                        {
                            return Value::Object(Map::from_iter(vec![
                                (
                                    Key::Property(JSContactProperty::SortAsKind(
                                        JSContactKind::Surname,
                                    )),
                                    Value::Str(surname.into()),
                                ),
                                (
                                    Key::Property(JSContactProperty::SortAsKind(
                                        JSContactKind::Given,
                                    )),
                                    Value::Str(given.into()),
                                ),
                            ]));
                        }
                    }

                    Value::Object(Map::from_iter(vec![(
                        Key::Property(JSContactProperty::SortAsKind(JSContactKind::Surname)),
                        Value::Str(sort_as.into()),
                    )]))
                }),
            ),
            (
                Key::Property(JSContactProperty::Coordinates),
                self.geo.map(Into::into).map(Value::Str),
            ),
            (
                Key::Property(JSContactProperty::TimeZone),
                self.tz.map(Into::into).map(Value::Str),
            ),
            (
                Key::Property(JSContactProperty::ListAs),
                self.index.map(|v| Value::Number((v as u64).into())),
            ),
            (
                Key::Property(JSContactProperty::Level),
                self.level.map(|v| {
                    Value::Element(JSContactValue::Level(match v {
                        VCardLevel::Beginner | VCardLevel::Low => JSContactLevel::Low,
                        VCardLevel::Average | VCardLevel::Medium => JSContactLevel::Medium,
                        VCardLevel::Expert | VCardLevel::High => JSContactLevel::High,
                    }))
                }),
            ),
            (
                Key::Property(JSContactProperty::CountryCode),
                self.country_code.map(Into::into).map(Value::Str),
            ),
            (
                Key::Property(JSContactProperty::Created),
                self.created
                    .map(|v| Value::Element(JSContactValue::Timestamp(v))),
            ),
            (
                Key::Property(if matches!(property, VCardProperty::Adr) {
                    JSContactProperty::Full
                } else {
                    JSContactProperty::Label
                }),
                self.label.map(Into::into).map(Value::Str),
            ),
            (
                Key::Property(JSContactProperty::Service),
                self.service_type.map(Into::into).map(Value::Str),
            ),
            (
                Key::Property(JSContactProperty::User),
                self.username.map(Into::into).map(Value::Str),
            ),
        ]
        .into_iter()
        .filter_map(|(k, v)| v.map(|v| (k, v)))
    }
}

impl JSContactProperty {
    fn sub_property(&self) -> Option<JSContactProperty> {
        match self {
            JSContactProperty::SpeakToAs => Some(JSContactProperty::Pronouns),
            _ => None,
        }
    }
}

trait GetObjectOrCreate {
    fn get_mut_object_or_insert(
        &mut self,
        key: JSContactProperty,
    ) -> &mut Map<'static, JSContactProperty, JSContactValue>;
}

impl GetObjectOrCreate
    for AHashMap<Key<'static, JSContactProperty>, Value<'static, JSContactProperty, JSContactValue>>
{
    #[inline]
    fn get_mut_object_or_insert(
        &mut self,
        key: JSContactProperty,
    ) -> &mut Map<'static, JSContactProperty, JSContactValue> {
        self.entry(Key::Property(key))
            .or_insert_with(|| Value::Object(Map::from(Vec::new())))
            .as_object_mut()
            .unwrap()
    }
}
