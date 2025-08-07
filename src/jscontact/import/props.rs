/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

use crate::{
    jscontact::{
        import::{
            EntryState, ExtractedParams, PropIdKey, State, VCardConvertedProperty, VCardParams,
        },
        JSContact, JSContactProperty, JSContactType, JSContactValue,
    },
    vcard::{
        VCard, VCardEntry, VCardParameter, VCardParameterName, VCardProperty, VCardValue,
        VCardValueType,
    },
};
use ahash::AHashMap;
use jmap_tools::{Key, Map, Property, Value};
use std::{borrow::Cow, collections::hash_map::Entry};

impl State {
    pub(super) fn new(vcard: &mut VCard) -> Self {
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
    pub(super) fn map_named_entry(
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

    pub(super) fn extract_params(
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
    pub(super) fn get_mut_object_or_insert(
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
    pub(super) fn has_property(&self, key: JSContactProperty) -> bool {
        self.entries.contains_key(&Key::Property(key))
    }

    pub(super) fn add_conversion_props(&mut self, mut entry: EntryState) {
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
    pub(super) fn track_prop(
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

    pub(super) fn find_prop_id(
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

    pub(super) fn find_entry_by_group(&self, group: Option<&str>) -> Option<&PropIdKey> {
        self.prop_ids.iter().find(|p| p.group.as_deref() == group)
    }

    pub(super) fn has_prop_id(&self, prop: &VCardProperty, prop_id: &str) -> bool {
        self.prop_ids
            .iter()
            .any(|p| p.prop == *prop && p.prop_id == prop_id)
    }

    pub(super) fn into_jscontact(mut self) -> JSContact<'static> {
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

impl JSContactProperty {
    pub(super) fn sub_property(&self) -> Option<JSContactProperty> {
        match self {
            JSContactProperty::SpeakToAs => Some(JSContactProperty::Pronouns),
            _ => None,
        }
    }
}

impl VCardValue {
    pub(super) fn into_jscontact_value(
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
