/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

use crate::{
    common::{CalendarScale, IanaType},
    jscontact::{JSContactProperty, JSContactValue},
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

#[allow(clippy::type_complexity)]
struct State {
    entries: AHashMap<
        Key<'static, JSContactProperty>,
        Value<'static, JSContactProperty, JSContactValue>,
    >,
    vcard_converted_properties: AHashMap<String, VCardConvertedProperty>,
    vcard_properties: Vec<Value<'static, JSContactProperty, JSContactValue>>,
    patch_objects: Vec<(
        JsonPointer<JSContactProperty>,
        Value<'static, JSContactProperty, JSContactValue>,
    )>,
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

#[derive(Debug, Clone)]
struct EntryState {
    entry: VCardEntry,
    converted_to: Option<String>,
    map_name: bool,
}

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
