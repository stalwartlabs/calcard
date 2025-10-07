/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

use crate::{
    common::{CalendarScale, IanaType},
    jscontact::{JSContactId, JSContactProperty, JSContactValue},
    vcard::{
        Jscomp, VCardEntry, VCardLevel, VCardParameterName, VCardPhonetic, VCardProperty, VCardType,
    },
};
use ahash::AHashMap;
use jmap_tools::{JsonPointer, Key, Map, Value};
use std::collections::HashMap;

pub mod convert;
pub mod entry;
pub mod params;
pub mod props;

#[derive(Debug, Clone, Copy)]
pub struct ConversionOptions {
    pub include_vcard_parameters: bool,
}

#[allow(clippy::type_complexity)]
struct State<I, B>
where
    I: JSContactId,
    B: JSContactId,
{
    entries: AHashMap<
        Key<'static, JSContactProperty<I>>,
        Value<'static, JSContactProperty<I>, JSContactValue<I, B>>,
    >,
    vcard_converted_properties: AHashMap<String, VCardConvertedProperty<I, B>>,
    vcard_properties: Vec<Value<'static, JSContactProperty<I>, JSContactValue<I, B>>>,
    patch_objects: Vec<(
        JsonPointer<JSContactProperty<I>>,
        Value<'static, JSContactProperty<I>, JSContactValue<I, B>>,
    )>,
    localizations: HashMap<
        String,
        Vec<(
            String,
            Value<'static, JSContactProperty<I>, JSContactValue<I, B>>,
        )>,
    >,
    default_language: Option<String>,
    prop_ids: Vec<PropIdKey<I>>,
    name_alt_id: Option<String>,
    has_fn: bool,
    has_fn_localization: bool,
    has_n: bool,
    has_n_localization: bool,
    has_gram_gender: bool,
    include_vcard_converted: bool,
}

#[derive(Debug, Clone)]
struct EntryState {
    entry: VCardEntry,
    converted_to: Option<String>,
    map_name: bool,
}

struct PropIdKey<I>
where
    I: JSContactId,
{
    prop: VCardProperty,
    prop_js: JSContactProperty<I>,
    group: Option<String>,
    alt_id: Option<String>,
    prop_id: String,
}

#[derive(Debug, Default)]
struct VCardConvertedProperty<I, B>
where
    I: JSContactId,
    B: JSContactId,
{
    name: Option<VCardProperty>,
    params: VCardParams<I, B>,
}

#[derive(Debug, Default)]
#[allow(clippy::type_complexity)]
struct VCardParams<I, B>(
    AHashMap<VCardParameterName, Vec<Value<'static, JSContactProperty<I>, JSContactValue<I, B>>>>,
)
where
    I: JSContactId,
    B: JSContactId;

#[derive(Default)]
struct ExtractedParams {
    language: Option<String>,
    pref: Option<u32>,
    author: Option<String>,
    author_name: Option<String>,
    phonetic_system: Option<IanaType<VCardPhonetic, String>>,
    phonetic_script: Option<String>,
    media_type: Option<String>,
    calscale: Option<IanaType<CalendarScale, String>>,
    sort_as: Option<String>,
    prop_id: Option<String>,
    alt_id: Option<String>,
    types: Vec<IanaType<VCardType, String>>,
    geo: Option<String>,
    tz: Option<String>,
    index: Option<u32>,
    level: Option<IanaType<VCardLevel, String>>,
    country_code: Option<String>,
    created: Option<i64>,
    label: Option<String>,
    service_type: Option<String>,
    username: Option<String>,
    jscomps: Vec<Jscomp>,
}

trait GetObjectOrCreate<I, B>
where
    I: JSContactId,
    B: JSContactId,
{
    fn get_mut_object_or_insert(
        &mut self,
        key: JSContactProperty<I>,
    ) -> &mut Map<'static, JSContactProperty<I>, JSContactValue<I, B>>;
}

impl<I, B> GetObjectOrCreate<I, B>
    for AHashMap<
        Key<'static, JSContactProperty<I>>,
        Value<'static, JSContactProperty<I>, JSContactValue<I, B>>,
    >
where
    I: JSContactId,
    B: JSContactId,
{
    #[inline]
    fn get_mut_object_or_insert(
        &mut self,
        key: JSContactProperty<I>,
    ) -> &mut Map<'static, JSContactProperty<I>, JSContactValue<I, B>> {
        self.entry(Key::Property(key))
            .or_insert_with(|| Value::Object(Map::from(Vec::new())))
            .as_object_mut()
            .unwrap()
    }
}

impl Default for ConversionOptions {
    fn default() -> Self {
        Self {
            include_vcard_parameters: true,
        }
    }
}

impl ConversionOptions {
    pub fn include_vcard_parameters(mut self, include: bool) -> Self {
        self.include_vcard_parameters = include;
        self
    }
}
