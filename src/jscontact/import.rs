/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

use crate::{
    common::CalendarScale,
    jscontact::{
        JSContact, JSContactKind, JSContactLevel, JSContactPhoneticSystem, JSContactProperty,
        JSContactType, JSContactValue,
    },
    vcard::{
        VCard, VCardEntry, VCardKind, VCardLevel, VCardParameter, VCardParameterName,
        VCardPhonetic, VCardProperty, VCardType, VCardValue, VCardValueType,
    },
};
use ahash::AHashMap;
use jmap_tools::{Key, Map, Value};
use std::collections::HashMap;

impl VCard {
    pub fn into_jscontact(mut self) -> JSContact<'static> {
        let mut entries = AHashMap::with_capacity(self.entries.len());

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

        // Find default language and move birthplace and deathplace to the bottom
        let mut default_language = None;
        let mut name_alt_id: Option<String> = None;
        self.entries
            .sort_unstable_by_key(|entry| match &entry.name {
                VCardProperty::Birthplace | VCardProperty::Deathplace => 2,
                VCardProperty::Language => {
                    if let Some(VCardValue::Text(lang)) = entry.values.first() {
                        default_language = Some(lang.clone());
                    }
                    0
                }
                VCardProperty::N => {
                    if name_alt_id.is_none() {
                        name_alt_id = entry.alt_id().map(|id| id.to_string());
                    }
                    // Names without language are processed first
                    entry.language().is_some() as i32
                }
                _ => entry.language().is_some() as i32,
            });

        let mut localizations: HashMap<String, Vec<(String, String)>> = HashMap::new();
        let mut name: AHashMap<
            Key<'_, JSContactProperty>,
            Value<'_, JSContactProperty, JSContactValue>,
        > = AHashMap::with_capacity(2);

        for entry in self.entries {
            let mut did_map = false;
            let mut vcard_params = Vec::new();

            match &entry.name {
                VCardProperty::Uid
                    if !entries.contains_key(&Key::Property(JSContactProperty::Uid)) =>
                {
                    if let Some(VCardValue::Text(text)) = entry.values.into_iter().next() {
                        entries.insert(
                            Key::Property(JSContactProperty::Uid),
                            Value::Str(text.into()),
                        );
                        did_map = true;
                    }
                }
                VCardProperty::Kind
                    if !entries.contains_key(&Key::Property(JSContactProperty::Kind)) =>
                {
                    match entry.values.into_iter().next() {
                        Some(VCardValue::Kind(kind)) => {
                            entries.insert(
                                Key::Property(JSContactProperty::Kind),
                                Value::Element(JSContactValue::Kind(match kind {
                                    VCardKind::Application => JSContactKind::Application,
                                    VCardKind::Device => JSContactKind::Device,
                                    VCardKind::Group => JSContactKind::Group,
                                    VCardKind::Individual => JSContactKind::Individual,
                                    VCardKind::Location => JSContactKind::Location,
                                    VCardKind::Org => JSContactKind::Org,
                                })),
                            );
                            did_map = true;
                        }
                        Some(VCardValue::Text(text)) => {
                            entries.insert(
                                Key::Property(JSContactProperty::Kind),
                                Value::Str(text.into()),
                            );
                            did_map = true;
                        }
                        _ => (),
                    }
                }

                VCardProperty::Source => {
                    if let Some(text) = entry.values.into_iter().next().and_then(|v| v.into_text())
                    {
                        let entries =
                            entries.get_mut_object_or_insert(JSContactProperty::Directories);

                        let mut obj = Map::from(vec![
                            (
                                Key::Property(JSContactProperty::Kind),
                                Value::Element(JSContactValue::Kind(JSContactKind::Entry)),
                            ),
                            (
                                Key::Property(JSContactProperty::Uri),
                                Value::Str(text.into()),
                            ),
                        ]);

                        let mut props = extract_params(
                            entry.params,
                            &[
                                VCardParameterName::PropId,
                                VCardParameterName::Pref,
                                VCardParameterName::Mediatype,
                            ],
                            &mut vcard_params,
                        );
                        let prop_id = props.prop_id();
                        obj.extend(props.into_iterator(&entry.name));

                        entries.insert_named(prop_id, Value::Object(obj));

                        did_map = true;
                    }
                }
                VCardProperty::OrgDirectory => {
                    if let Some(text) = entry.values.into_iter().next().and_then(|v| v.into_text())
                    {
                        let entries =
                            entries.get_mut_object_or_insert(JSContactProperty::Directories);

                        let mut obj = Map::from(vec![
                            (
                                Key::Property(JSContactProperty::Kind),
                                Value::Element(JSContactValue::Kind(JSContactKind::Directory)),
                            ),
                            (
                                Key::Property(JSContactProperty::Uri),
                                Value::Str(text.into()),
                            ),
                        ]);

                        let mut props = extract_params(
                            entry.params,
                            &[
                                VCardParameterName::PropId,
                                VCardParameterName::Pref,
                                VCardParameterName::Index,
                                VCardParameterName::Label,
                            ],
                            &mut vcard_params,
                        );
                        let prop_id = props.prop_id();

                        if !props.types.is_empty() {
                            obj.insert_unchecked(
                                Key::Property(JSContactProperty::Contexts),
                                Value::new_boolean_set(
                                    props
                                        .types()
                                        .into_iter()
                                        .map(|t| (Key::Owned(t.as_str().to_string()), true)),
                                ),
                            );
                        }

                        entries.insert_named(prop_id, Value::Object(obj));
                        did_map = true;
                    }
                }
                VCardProperty::Anniversary | VCardProperty::Bday | VCardProperty::Deathdate => {
                    if let Some(mut obj) = entry.to_jscontact_anniversary() {
                        let entries =
                            entries.get_mut_object_or_insert(JSContactProperty::Anniversaries);

                        obj.insert_unchecked(
                            JSContactProperty::Kind,
                            Value::Element(JSContactValue::Kind(match &entry.name {
                                VCardProperty::Anniversary => JSContactKind::Wedding,
                                VCardProperty::Bday => JSContactKind::Birth,
                                VCardProperty::Deathdate => JSContactKind::Death,
                                _ => unreachable!(),
                            })),
                        );

                        let mut props = extract_params(
                            entry.params,
                            &[VCardParameterName::PropId, VCardParameterName::Calscale],
                            &mut vcard_params,
                        );
                        let prop_id = props.prop_id();
                        obj.extend(props.into_iterator(&entry.name));

                        entries.insert_named(prop_id, Value::Object(obj));
                        did_map = true;
                    }
                }
                VCardProperty::Birthplace | VCardProperty::Deathplace => {
                    // Only text and GEO can be mapped
                    let is_geo = matches!(entry.values.first(), Some(VCardValue::Text(uri)) if uri.starts_with("geo:"));
                    if is_geo || !entry.is_type(&VCardValueType::Uri) {
                        if let Some(text) =
                            entry.values.into_iter().next().and_then(|v| v.into_text())
                        {
                            // Extract language and value type
                            let mut props = extract_params(
                                entry.params,
                                &[VCardParameterName::Language, VCardParameterName::PropId],
                                &mut vcard_params,
                            );
                            if default_language == props.language {
                                props.language = None;
                            }

                            let kind = Value::Element(JSContactValue::Kind(
                                if entry.name == VCardProperty::Birthplace {
                                    JSContactKind::Birth
                                } else {
                                    JSContactKind::Death
                                },
                            ));

                            if let Some(lang) = props.language {
                                let prop_id = if let Some(prop_id) = props.prop_id {
                                    prop_id
                                } else {
                                    entries
                                        .iter()
                                        .find_map(|(k, v)| {
                                            let obj = v.as_object()?;
                                            if obj.iter().any(|(k, v)| {
                                                k == &Key::Property(JSContactProperty::Kind)
                                                    && v == &kind
                                            }) {
                                                Some(k.to_string().into_owned())
                                            } else {
                                                None
                                            }
                                        })
                                        .unwrap_or_else(|| format!("k{}", entries.len() + 1))
                                };
                                localizations.entry(lang).or_default().push((
                                    format!(
                                        "{}/{}/{}/{}",
                                        JSContactProperty::Anniversaries.as_str(),
                                        prop_id,
                                        JSContactProperty::Place.as_str(),
                                        JSContactProperty::Full.as_str()
                                    ),
                                    text,
                                ));
                            } else {
                                let entries = entries
                                    .get_mut_object_or_insert(JSContactProperty::Anniversaries);

                                // Place needs to be wrapped in an option to dance around the borrow checker
                                let mut place = Some(Map::from(vec![(
                                    Key::Property(if !is_geo {
                                        JSContactProperty::Full
                                    } else {
                                        JSContactProperty::Coordinates
                                    }),
                                    Value::Str(text.into()),
                                )]));

                                if let Some(anniversary) =
                                    props.prop_id.clone().and_then(|prop_id| {
                                        entries
                                            .get_mut(&Key::Owned(prop_id))
                                            .and_then(|v| v.as_object_mut())
                                    })
                                {
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
                                    entries.insert_named(props.prop_id, Value::Object(place));
                                }
                            }
                            did_map = true;
                        }
                    }
                }
                VCardProperty::Fn => {
                    if !name.contains_key(&Key::Property(JSContactProperty::Full))
                        || entry.language().is_some_and(|lang| {
                            default_language
                                .as_ref()
                                .is_none_or(|default_language| default_language != lang)
                                && localizations.get(lang).is_none_or(|locals| {
                                    !locals.iter().any(|(k, _)| k == "name/full")
                                })
                        })
                    {
                        if let Some(VCardValue::Text(text)) = entry.values.into_iter().next() {
                            if let Some(lang) = extract_params(
                                entry.params,
                                &[VCardParameterName::Language],
                                &mut vcard_params,
                            )
                            .language
                            {
                                localizations.entry(lang).or_default().push((
                                    format!(
                                        "{}/{}",
                                        JSContactProperty::Name.as_str(),
                                        JSContactProperty::Full.as_str()
                                    ),
                                    text,
                                ));
                            } else {
                                name.insert(
                                    Key::Property(JSContactProperty::Name),
                                    Value::Str(text.into()),
                                );
                            }
                            did_map = true;
                        }
                    }
                }
                VCardProperty::N => {
                    /*
                      Only process this N component if it matches the main ALTID and:
                      - It is the first N component
                      - Or it has a language that has not been processed yet
                    */
                    if name_alt_id.as_deref() == entry.alt_id()
                        && (!name.contains_key(&Key::Property(JSContactProperty::Components))
                            || entry.language().is_some_and(|lang| {
                                default_language
                                    .as_ref()
                                    .is_none_or(|default_language| default_language != lang)
                                    && localizations.get(lang).is_none_or(|locals| {
                                        !locals
                                            .iter()
                                            .any(|(k, _)| !k.starts_with("name/components/"))
                                    })
                            }))
                    {
                        let params = extract_params(
                            entry.params,
                            &[
                                VCardParameterName::Language,
                                VCardParameterName::Phonetic,
                                VCardParameterName::Script,
                                VCardParameterName::SortAs,
                            ],
                            &mut vcard_params,
                        );
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

                        for (comp_id, value) in
                            entry.values.iter().filter_map(|v| v.as_text()).enumerate()
                        {
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

                        if let Some(lang) = params.language.filter(|_| {
                            name.contains_key(&Key::Property(JSContactProperty::Components))
                        }) {
                            let locale = localizations.entry(lang).or_default();
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
                                        value.to_string(),
                                    ));
                                }
                            }

                            if let Some(script) = params.phonetic_system {
                                // Both JSContact and vCard use the same phonetic system names
                                locale.push((
                                    format!(
                                        "{}/{}",
                                        JSContactProperty::Name.as_str(),
                                        JSContactProperty::PhoneticSystem.as_str(),
                                    ),
                                    script.as_str().to_string(),
                                ));
                            }

                            if let Some(script) = params.phonetic_script {
                                locale.push((
                                    format!(
                                        "{}/{}",
                                        JSContactProperty::Name.as_str(),
                                        JSContactProperty::PhoneticScript.as_str(),
                                    ),
                                    script,
                                ));
                            }

                            if let Some(sort_as) = params.sort_as {
                                // Can't be converted in localizations
                                vcard_params.push(VCardParameter::SortAs(sort_as));
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

                            if let Some(system) = params.phonetic_system {
                                name.insert(
                                    Key::Property(JSContactProperty::PhoneticSystem),
                                    match system {
                                        VCardPhonetic::Ipa => {
                                            Value::Element(JSContactValue::PhoneticSystem(
                                                JSContactPhoneticSystem::Ipa,
                                            ))
                                        }
                                        VCardPhonetic::Jyut => {
                                            Value::Element(JSContactValue::PhoneticSystem(
                                                JSContactPhoneticSystem::Jyut,
                                            ))
                                        }
                                        VCardPhonetic::Piny => {
                                            Value::Element(JSContactValue::PhoneticSystem(
                                                JSContactPhoneticSystem::Piny,
                                            ))
                                        }
                                        VCardPhonetic::Script => {
                                            Value::Element(JSContactValue::PhoneticSystem(
                                                JSContactPhoneticSystem::Script,
                                            ))
                                        }
                                        VCardPhonetic::Other(value) => Value::Str(value.into()),
                                    },
                                );
                            }

                            if let Some(script) = params.phonetic_script {
                                name.insert(
                                    Key::Property(JSContactProperty::PhoneticScript),
                                    Value::Str(script.into()),
                                );
                            }

                            if let Some(sort_as) = params.sort_as {
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
                                    name.insert(
                                        Key::Property(JSContactProperty::SortAs),
                                        Value::Object(Map::from_iter(vec![
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
                                        ])),
                                    );
                                } else {
                                    name.insert(
                                        Key::Property(JSContactProperty::SortAs),
                                        Value::Object(Map::from_iter(vec![(
                                            Key::Property(JSContactProperty::SortAsKind(
                                                JSContactKind::Surname,
                                            )),
                                            Value::Str(sort_as.into()),
                                        )])),
                                    );
                                }
                            }

                            name.insert(
                                Key::Property(JSContactProperty::Components),
                                Value::Array(array),
                            );
                        }

                        did_map = true;
                    }
                }
                VCardProperty::Gramgender => {}
                VCardProperty::Pronouns => {}

                VCardProperty::Nickname => todo!(),
                VCardProperty::Photo => todo!(),
                VCardProperty::Adr => todo!(),
                VCardProperty::Tel => todo!(),
                VCardProperty::Email => todo!(),
                VCardProperty::Impp => todo!(),
                VCardProperty::Lang => todo!(),
                VCardProperty::Tz => todo!(),
                VCardProperty::Geo => todo!(),
                VCardProperty::Title => todo!(),
                VCardProperty::Role => todo!(),
                VCardProperty::Logo => todo!(),
                VCardProperty::Org => todo!(),
                VCardProperty::Member => todo!(),
                VCardProperty::Related => todo!(),
                VCardProperty::Categories => todo!(),
                VCardProperty::Note => todo!(),
                VCardProperty::Prodid => todo!(),
                VCardProperty::Rev => todo!(),
                VCardProperty::Sound => todo!(),
                VCardProperty::Clientpidmap => todo!(),
                VCardProperty::Url => todo!(),
                VCardProperty::Version => todo!(),
                VCardProperty::Key => todo!(),
                VCardProperty::Fburl => todo!(),
                VCardProperty::Caladruri => todo!(),
                VCardProperty::Caluri => todo!(),
                VCardProperty::Expertise => todo!(),
                VCardProperty::Hobby => todo!(),
                VCardProperty::Interest => todo!(),
                VCardProperty::ContactUri => todo!(),
                VCardProperty::Created => todo!(),
                VCardProperty::Language => todo!(),
                VCardProperty::Socialprofile => todo!(),
                VCardProperty::Jsprop => todo!(),

                VCardProperty::Begin | VCardProperty::End => {
                    continue;
                }
                _ => {}
            }

            if !did_map {
                // Duplicate or unsupported property, add as vCardProps
                let todo = "convert to vCardProps";
            }
        }

        if !name.is_empty() {
            entries.insert(
                Key::Property(JSContactProperty::Name),
                Value::Object(name.into_iter().collect()),
            );
        }

        if !localizations.is_empty() {
            entries.insert(
                Key::Property(JSContactProperty::Localizations),
                Value::Object(
                    localizations
                        .into_iter()
                        .map(|(lang, locals)| {
                            (
                                Key::Owned(lang),
                                Value::Object(
                                    locals
                                        .into_iter()
                                        .map(|(key, value)| {
                                            (Key::Owned(key), Value::Str(value.into()))
                                        })
                                        .collect(),
                                ),
                            )
                        })
                        .collect(),
                ),
            );
        }

        JSContact(Value::Object(entries.into_iter().collect()))
    }
}

impl VCardEntry {
    fn to_jscontact_anniversary(&self) -> Option<Map<'static, JSContactProperty, JSContactValue>> {
        if let Some(VCardValue::PartialDateTime(dt)) = self.values.first() {
            if let Some(timestamp) = dt.to_timestamp() {
                Some(Map::from(vec![
                    (
                        Key::Property(JSContactProperty::Type),
                        Value::Element(JSContactValue::Type(JSContactType::Timestamp)),
                    ),
                    (
                        Key::Property(JSContactProperty::Utc),
                        Value::Element(JSContactValue::Timestamp(timestamp)),
                    ),
                ]))
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

                Some(props)
            } else {
                None
            }
        } else {
            None
        }
    }
}

fn extract_params(
    params: Vec<VCardParameter>,
    extract: &[VCardParameterName],
    skipped: &mut Vec<VCardParameter>,
) -> ExtractedParams {
    let mut p = ExtractedParams::default();

    for param in params {
        match param {
            VCardParameter::Language(v)
                if p.language.is_none() && extract.contains(&VCardParameterName::Language) =>
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
                if p.author_name.is_none() && extract.contains(&VCardParameterName::AuthorName) =>
            {
                p.author_name = Some(v);
            }
            VCardParameter::Mediatype(v)
                if p.media_type.is_none() && extract.contains(&VCardParameterName::Mediatype) =>
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
                if p.phonetic_script.is_none() && extract.contains(&VCardParameterName::Script) =>
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
            VCardParameter::Type(typ) if extract.contains(&VCardParameterName::Type) => {
                if p.types.is_empty() {
                    p.types = typ;
                } else {
                    p.types.extend(typ);
                }
            }
            _ => {
                skipped.push(param);
            }
        }
    }

    p
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

    fn types(&mut self) -> Vec<VCardType> {
        std::mem::take(&mut self.types)
    }

    fn into_iterator(
        self,
        property: &VCardProperty,
    ) -> impl Iterator<
        Item = (
            Key<'static, JSContactProperty>,
            Value<'static, JSContactProperty, JSContactValue>,
        ),
    > {
        [
            (
                Key::Property(JSContactProperty::Language),
                self.language.map(Into::into).map(Value::Str),
            ),
            (
                Key::Property(JSContactProperty::Pref),
                self.pref.map(|v| Value::Number((v as u64).into())),
            ),
            (
                Key::Property(JSContactProperty::Uri),
                self.author.map(Into::into).map(Value::Str),
            ),
            (
                Key::Property(JSContactProperty::Name),
                self.author_name.map(Into::into).map(Value::Str),
            ),
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
                self.sort_as.map(Into::into).map(Value::Str),
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
