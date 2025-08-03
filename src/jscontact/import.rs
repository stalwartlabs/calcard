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
    },
};
use ahash::AHashMap;
use jmap_tools::{Key, Map, Value};
use std::collections::HashMap;

struct PropIdKey {
    prop: VCardProperty,
    prop_js: JSContactProperty,
    group: Option<String>,
    alt_id: Option<String>,
    prop_id: String,
}

#[allow(clippy::type_complexity)]
struct State {
    entries: AHashMap<
        Key<'static, JSContactProperty>,
        Value<'static, JSContactProperty, JSContactValue>,
    >,
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
    did_convert: bool,
}

impl VCard {
    pub fn into_jscontact(mut self) -> JSContact<'static> {
        let mut state = State::new(&mut self);

        for entry in self.entries {
            let mut entry = EntryState::new(entry);

            match &entry.entry.name {
                VCardProperty::Kind => {
                    if !state.has_property(JSContactProperty::Kind) {
                        if let Some(kind) = entry.to_kind() {
                            state
                                .entries
                                .insert(Key::Property(JSContactProperty::Kind), kind);
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
                            let patch_id = prop_id
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
                                state.localizations.entry(lang).or_default().push((
                                    format!(
                                        "{}/{}/{}/{}",
                                        JSContactProperty::Anniversaries.as_str(),
                                        patch_id.unwrap(),
                                        JSContactProperty::Place.as_str(),
                                        JSContactProperty::Full.as_str()
                                    ),
                                    text,
                                ));
                            } else {
                                // Place needs to be wrapped in an option to dance around the borrow checker
                                let mut place = Some(Map::from(vec![(
                                    Key::Property(if !is_geo {
                                        JSContactProperty::Full
                                    } else {
                                        JSContactProperty::Coordinates
                                    }),
                                    text,
                                )]));

                                if let Some(anniversary) = patch_id.and_then(|patch_id| {
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
                                    if let Some(anniversary) =
                                        entries.iter_mut().find_map(|(_, v)| {
                                            let obj = v.as_object_mut()?;
                                            if obj.iter().any(|(k, v)| {
                                                k == &Key::Property(JSContactProperty::Kind)
                                                    && v == &kind
                                            }) {
                                                Some(obj)
                                            } else {
                                                None
                                            }
                                        })
                                    {
                                        anniversary.insert(
                                            Key::Property(JSContactProperty::Place),
                                            Value::Object(place.take().unwrap()),
                                        );
                                    }
                                }

                                if let Some(place) = place {
                                    let prop_id =
                                        entries.insert_named(prop_id, Value::Object(place));
                                    state.track_prop(
                                        &entry.entry,
                                        JSContactProperty::Anniversaries,
                                        alt_id,
                                        prop_id,
                                    );
                                }
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
                                state.has_fn_localization = true;
                                state.localizations.entry(lang).or_default().push((
                                    format!(
                                        "{}/{}",
                                        JSContactProperty::Name.as_str(),
                                        JSContactProperty::Full.as_str()
                                    ),
                                    text,
                                ));
                            } else {
                                state.has_fn = true;
                                state
                                    .get_mut_object_or_insert(JSContactProperty::Name)
                                    .insert(Key::Property(JSContactProperty::Name), text);
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

                        entry.did_convert = true;

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
                        let components_iter = components.iter().filter(|(kind, value)| {
                            !matches!(kind, JSContactKind::Credential)
                                || !components.contains(&(JSContactKind::Generation, value))
                        });

                        let name = state
                            .entries
                            .get_mut_object_or_insert(JSContactProperty::Name);

                        if let Some(lang) = params.language() {
                            state.has_n_localization = true;
                            let locale = state.localizations.entry(lang).or_default();
                            let parts = name
                                .get(&Key::Property(JSContactProperty::Components))
                                .unwrap()
                                .as_array()
                                .unwrap();

                            for (kind, value) in components_iter {
                                if let Some(pos) = parts.iter().position(|obj| {
                                    obj.as_object().unwrap().contains_key_value(
                                        &Key::Property(JSContactProperty::Kind),
                                        &Value::Element(JSContactValue::Kind(*kind)),
                                    )
                                }) {
                                    locale.push((
                                        format!(
                                            "{}/{}/{}/{}",
                                            JSContactProperty::Name.as_str(),
                                            JSContactProperty::Components.as_str(),
                                            pos,
                                            JSContactProperty::Phonetic.as_str()
                                        ),
                                        Value::Str(value.to_string().into()),
                                    ));
                                }
                            }

                            for (prop, value) in params.into_iter(&entry.entry.name) {
                                locale.push((
                                    format!(
                                        "{}/{}",
                                        JSContactProperty::Name.as_str(),
                                        prop.to_string()
                                    ),
                                    value,
                                ));
                            }
                        } else {
                            let mut array = Vec::with_capacity(components.len());

                            for (kind, value) in components_iter {
                                array.push(Value::Object(Map::from(vec![
                                    (
                                        Key::Property(JSContactProperty::Kind),
                                        Value::Element(JSContactValue::Kind(*kind)),
                                    ),
                                    (
                                        Key::Property(JSContactProperty::Value),
                                        Value::Str(value.to_string().into()),
                                    ),
                                ])));
                            }

                            state.has_n = true;
                            name.extend(params.into_iter(&entry.entry.name));
                            name.insert(
                                Key::Property(JSContactProperty::Components),
                                Value::Array(array),
                            );
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
                            })
                            .and_then(|prop_id| {
                                state
                                    .entries
                                    .get(&Key::Property(JSContactProperty::Addresses))
                                    .and_then(|v| v.as_object())
                                    .and_then(|v| v.get(&Key::Borrowed(prop_id)))
                            })
                            .and_then(|v| v.as_object());

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

                    let components_iter =
                        entry.text_parts().enumerate().filter_map(|(i, value)| {
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
                            (kind, value).into()
                        });

                    if let Some(lang) = params.language() {
                        let prop_id = prop_id.as_deref().unwrap();
                        let locale = state.localizations.entry(lang).or_default();
                        let parts = addr_patch
                            .unwrap()
                            .get(&Key::Property(JSContactProperty::Components))
                            .unwrap()
                            .as_array()
                            .unwrap();

                        for (kind, value) in components_iter {
                            if let Some(pos) = parts.iter().position(|obj| {
                                obj.as_object().unwrap().contains_key_value(
                                    &Key::Property(JSContactProperty::Kind),
                                    &Value::Element(JSContactValue::Kind(kind)),
                                )
                            }) {
                                locale.push((
                                    format!(
                                        "{}/{}/{}/{}/{}",
                                        JSContactProperty::Addresses.as_str(),
                                        prop_id,
                                        JSContactProperty::Components.as_str(),
                                        pos,
                                        JSContactProperty::Phonetic.as_str()
                                    ),
                                    Value::Str(value.to_string().into()),
                                ));
                            }
                        }

                        for (prop_name, value) in params.into_iter(&entry.entry.name) {
                            locale.push((
                                format!(
                                    "{}/{}/{}",
                                    JSContactProperty::Addresses.as_str(),
                                    prop_id,
                                    prop_name.to_string()
                                ),
                                value,
                            ));
                        }
                    } else {
                        let entries = state.get_mut_object_or_insert(JSContactProperty::Addresses);
                        let mut addr = Map::from(Vec::with_capacity(4));
                        let mut array = Vec::with_capacity(6);

                        for (kind, value) in components_iter {
                            array.push(Value::Object(Map::from(vec![
                                (
                                    Key::Property(JSContactProperty::Kind),
                                    Value::Element(JSContactValue::Kind(kind)),
                                ),
                                (
                                    Key::Property(JSContactProperty::Value),
                                    Value::Str(value.into()),
                                ),
                            ])));
                        }

                        addr.extend(params.into_iter(&entry.entry.name));
                        addr.insert_unchecked(
                            Key::Property(JSContactProperty::Components),
                            Value::Array(array),
                        );

                        let prop_id = entries.insert_named(prop_id, Value::Object(addr));
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
                        [(
                            Key::Property(JSContactProperty::VCardName),
                            Value::Str("impp".into()),
                        )],
                    );
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
                    let key = match &entry.entry.name {
                        VCardProperty::Language => JSContactProperty::Language,
                        VCardProperty::Prodid => JSContactProperty::ProdId,
                        VCardProperty::Uid => JSContactProperty::Uid,
                        _ => unreachable!(),
                    };

                    if !state.has_property(key) {
                        if let Some(text) = entry.to_text() {
                            state.entries.insert(Key::Property(key), text);
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
                    state.map_named_entry(
                        &mut entry,
                        &[VCardParameterName::Type, VCardParameterName::SortAs],
                        JSContactProperty::Organizations,
                        JSContactProperty::Name,
                        [],
                    );
                }
                VCardProperty::Member | VCardProperty::Categories => {
                    let obj = state.get_mut_object_or_insert(
                        if entry.entry.name == VCardProperty::Member {
                            JSContactProperty::Members
                        } else {
                            JSContactProperty::Keywords
                        },
                    );

                    for key in entry.text_parts() {
                        obj.insert(Key::Owned(key), Value::Bool(true));
                    }
                }
                VCardProperty::Created | VCardProperty::Rev => {
                    let key = match &entry.entry.name {
                        VCardProperty::Created => JSContactProperty::Created,
                        VCardProperty::Rev => JSContactProperty::Updated,
                        _ => unreachable!(),
                    };

                    if !state.has_property(key) {
                        if let Some(text) = entry.to_timestamp() {
                            state.entries.insert(Key::Property(key), text);
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
                        let params = state
                            .extract_params(&mut entry.entry.params, &[VCardParameterName::Type]);
                        state
                            .get_mut_object_or_insert(JSContactProperty::RelatedTo)
                            .insert(
                                Key::Owned(text),
                                Value::Object(Map::from(vec![(
                                    Key::Property(JSContactProperty::Relation),
                                    Value::Object(Map::from_iter(params.types.iter().map(|t| {
                                        (Key::Owned(t.as_str().to_string()), Value::Bool(true))
                                    }))),
                                )])),
                            );
                    }
                }
                VCardProperty::Tz | VCardProperty::Geo => {
                    let (key, value) = match &entry.entry.name {
                        VCardProperty::Tz => (JSContactProperty::TimeZone, entry.to_tz()),
                        VCardProperty::Geo => (JSContactProperty::Coordinates, entry.to_text()),
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
                            .and_then(|prop_id| addresses.get_mut(&Key::Owned(prop_id)))
                            .and_then(|v| v.as_object_mut())
                            .filter(|v| !v.contains_key(&Key::Property(key)))
                        {
                            addr.insert(key, value);
                        } else {
                            addresses.insert_named(
                                None,
                                Value::Object(Map::from(vec![(Key::Property(key), value)])),
                            );
                        }
                    }
                }
                VCardProperty::Other(name)
                    if name.eq_ignore_ascii_case("X-ABLabel") && entry.entry.group.is_some() =>
                {
                    if let Some(prop) = state.find_entry_by_group(entry.entry.group.as_deref()) {
                        let prop_id = prop.prop_id.to_string();
                        let prop = prop.prop_js;

                        if let Some(obj) = state
                            .entries
                            .get_mut(&Key::Property(prop))
                            .and_then(|v| v.as_object_mut())
                            .and_then(|v| v.get_mut(&Key::Owned(prop_id)))
                            .and_then(|v| {
                                v.as_object_mut().filter(|v| {
                                    !v.contains_key(&Key::Property(JSContactProperty::Label))
                                })
                            })
                        {
                            if let Some(value) = entry.to_text() {
                                obj.insert_unchecked(
                                    Key::Property(JSContactProperty::Label),
                                    value,
                                );
                            }
                        }
                    }
                }
                VCardProperty::Jsprop => {
                    let todo = "map";
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

            if !entry.did_convert {
                // Duplicate or unsupported property, add as vCardProps
                let todo = "convert to vCardProps";
            }
        }

        state.into_jscontact()
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
            prop_ids: Vec::new(),
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
                        if let Some(sub_property) = top_property_name.sub_property() {
                            format!(
                                "{}/{}/{}/{}",
                                top_property_name.as_str(),
                                sub_property.as_str(),
                                prop_id,
                                value_property_name.as_str()
                            )
                        } else {
                            format!(
                                "{}/{}/{}",
                                top_property_name.as_str(),
                                prop_id,
                                value_property_name.as_str()
                            )
                        }
                    })
                {
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

            let mut entries = self.get_mut_object_or_insert(top_property_name);
            if let Some(sub_property) = top_property_name.sub_property() {
                entries = entries
                    .insert_or_get_mut(sub_property, Value::Object(Map::from(vec![])))
                    .as_object_mut()
                    .unwrap();
            }
            let mut obj = vec![(Key::Property(value_property_name), value)];
            for (key, value) in extra_properties {
                obj.push((key, value));
            }

            obj.extend(params.into_iter(&entry.entry.name));
            let prop_id = entries.insert_named(prop_id, Value::Object(Map::from(obj)));
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

        JSContact(Value::Object(self.entries.into_iter().collect()))
    }
}

#[allow(clippy::wrong_self_convention)]
impl EntryState {
    fn new(entry: VCardEntry) -> Self {
        Self {
            entry,
            did_convert: false,
        }
    }

    fn to_text(&mut self) -> Option<Value<'static, JSContactProperty, JSContactValue>> {
        self.to_string().map(|s| Value::Str(s.into()))
    }

    fn to_string(&mut self) -> Option<String> {
        let mut values = std::mem::take(&mut self.entry.values).into_iter();
        match values.next()? {
            VCardValue::Text(v) => {
                self.did_convert = true;
                v.into()
            }
            VCardValue::Binary(data) => {
                self.did_convert = true;
                data.to_string().into()
            }
            VCardValue::Sex(v) => {
                self.did_convert = true;
                v.as_str().to_string().into()
            }
            VCardValue::GramGender(v) => {
                self.did_convert = true;
                v.as_str().to_string().into()
            }
            VCardValue::Kind(v) => {
                self.did_convert = true;
                v.as_str().to_string().into()
            }
            other => {
                self.entry.values.push(other);
                self.entry.values.extend(values);
                None
            }
        }
    }

    fn text_parts(&mut self) -> impl Iterator<Item = String> + '_ {
        self.did_convert = true;
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
            VCardValue::Kind(v) => {
                self.did_convert = true;
                Value::Element(JSContactValue::Kind(match v {
                    VCardKind::Application => JSContactKind::Application,
                    VCardKind::Device => JSContactKind::Device,
                    VCardKind::Group => JSContactKind::Group,
                    VCardKind::Individual => JSContactKind::Individual,
                    VCardKind::Location => JSContactKind::Location,
                    VCardKind::Org => JSContactKind::Org,
                }))
                .into()
            }
            VCardValue::Text(v) => {
                self.did_convert = true;
                Value::Str(v.into()).into()
            }
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
                self.did_convert = true;
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
            VCardValue::Text(v) => {
                self.did_convert = true;
                Value::Str(v.into()).into()
            }
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
                self.did_convert = true;
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
            VCardValue::Text(v) => {
                self.did_convert = true;
                Value::Str(v.into()).into()
            }
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
                self.did_convert = true;
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
                self.did_convert = true;
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
                .push((Key::Owned(typ.as_str().to_string()), Value::Bool(true)));
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
