/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

use ahash::AHashMap;
use jmap_tools::{JsonPointer, Key, Map, Value};

use crate::common::timezone::ZonedDateTime;
use crate::{
    common::{
        blob::{BlobIdFn, BlobIdGenerator, BlobIds, BlobOptions, NoBlobIds},
        jsprop::ordered::OrderedMap,
        timezone::Tz,
    },
    icalendar::{
        ICalendarComponentType, ICalendarEntry, ICalendarParameterName, ICalendarProperty,
    },
    jscalendar::{
        JSCalendarDateTime, JSCalendarId, JSCalendarProperty, JSCalendarValue,
        ext::JSCalendarObjectExt,
    },
};

pub mod convert;
pub mod params;
pub mod props;

const PROPERTY_MAP_CAPACITY: usize = 16;

#[derive(Default)]
#[allow(clippy::type_complexity)]
struct State<I: JSCalendarId, B: JSCalendarId> {
    component_type: ICalendarComponentType,
    entries: PropertyMap<I, B>,
    ical_converted_properties: OrderedMap<String, ICalendarConvertedProperty<I, B>>,
    ical_properties: Vec<Value<'static, JSCalendarProperty<I>, JSCalendarValue<I, B>>>,
    ical_components: Option<Value<'static, JSCalendarProperty<I>, JSCalendarValue<I, B>>>,
    recurrence_overrides: Vec<(JSCalendarDateTime, State<I, B>)>,
    patch_objects: Vec<(
        JsonPointer<JSCalendarProperty<I>>,
        Value<'static, JSCalendarProperty<I>, JSCalendarValue<I, B>>,
    )>,
    link_ids: LinkIds,
    parameters: Map<'static, JSCalendarProperty<I>, JSCalendarValue<I, B>>,
    jsid: Option<String>,
    uid: Option<String>,
    recurrence_id: Option<ZonedDateTime>,
    recurrence_id_is_date: bool,
    due: Option<ZonedDateTime>,
    tz_start: Option<Tz>,
    tz_end: Option<Tz>,
    has_dates: bool,
    has_end: bool,
    map_component: bool,
    is_recurrence_instance: bool,
    include_ical_components: bool,
}

struct PropertyMap<I: JSCalendarId, B: JSCalendarId>(
    Map<'static, JSCalendarProperty<I>, JSCalendarValue<I, B>>,
);

struct PropertyEntry<'a, I: JSCalendarId, B: JSCalendarId> {
    map: &'a mut Map<'static, JSCalendarProperty<I>, JSCalendarValue<I, B>>,
    key: Key<'static, JSCalendarProperty<I>>,
}

impl<I: JSCalendarId, B: JSCalendarId> Default for PropertyMap<I, B> {
    fn default() -> Self {
        Self(Map::from(Vec::with_capacity(PROPERTY_MAP_CAPACITY)))
    }
}

impl<I: JSCalendarId, B: JSCalendarId> PropertyMap<I, B> {
    #[inline]
    fn insert(
        &mut self,
        key: Key<'static, JSCalendarProperty<I>>,
        value: Value<'static, JSCalendarProperty<I>, JSCalendarValue<I, B>>,
    ) -> Option<Value<'static, JSCalendarProperty<I>, JSCalendarValue<I, B>>> {
        self.0.upsert(key, value)
    }

    #[inline]
    fn entry(&mut self, key: Key<'static, JSCalendarProperty<I>>) -> PropertyEntry<'_, I, B> {
        PropertyEntry {
            map: &mut self.0,
            key,
        }
    }

    #[inline]
    fn get(
        &self,
        key: &Key<'_, JSCalendarProperty<I>>,
    ) -> Option<&Value<'static, JSCalendarProperty<I>, JSCalendarValue<I, B>>> {
        self.0.lookup(key)
    }

    #[inline]
    fn contains_key(&self, key: &Key<'_, JSCalendarProperty<I>>) -> bool {
        self.0.key_position(key).is_some()
    }

    fn retain(
        &mut self,
        mut keep: impl FnMut(
            &Key<'static, JSCalendarProperty<I>>,
            &mut Value<'static, JSCalendarProperty<I>, JSCalendarValue<I, B>>,
        ) -> bool,
    ) {
        self.0
            .as_mut_vec()
            .retain_mut(|(key, value)| keep(key, value));
    }

    #[inline]
    fn into_map(self) -> Map<'static, JSCalendarProperty<I>, JSCalendarValue<I, B>> {
        self.0
    }
}

impl<'a, I: JSCalendarId, B: JSCalendarId> PropertyEntry<'a, I, B> {
    #[inline]
    fn or_insert_with(
        self,
        value: impl FnOnce() -> Value<'static, JSCalendarProperty<I>, JSCalendarValue<I, B>>,
    ) -> &'a mut Value<'static, JSCalendarProperty<I>, JSCalendarValue<I, B>> {
        let position = match self.map.key_position(&self.key) {
            Some(position) => position,
            None => {
                self.map.insert_unchecked(self.key, value());
                self.map.len() - 1
            }
        };
        &mut self.map.as_mut_vec()[position].1
    }
}

#[derive(Debug, Default)]
struct ICalendarConvertedProperty<I: JSCalendarId, B: JSCalendarId> {
    name: Option<ICalendarProperty>,
    params: ICalendarParams<I, B>,
}

#[derive(Debug, Default)]
struct ICalendarParams<I: JSCalendarId, B: JSCalendarId>(
    Vec<(ICalendarParameterName, ParamValues<I, B>)>,
);

#[derive(Debug)]
enum ParamValues<I: JSCalendarId, B: JSCalendarId> {
    One(Value<'static, JSCalendarProperty<I>, JSCalendarValue<I, B>>),
    Many(Vec<Value<'static, JSCalendarProperty<I>, JSCalendarValue<I, B>>>),
}

#[derive(Debug, Default)]
struct LinkIds(AHashMap<String, LinkId>);

#[derive(Debug, Clone, Copy)]
struct LinkId {
    position: usize,
    next_suffix: u32,
}

#[derive(Debug, Clone)]
struct EntryState {
    entry: ICalendarEntry,
    converted_to: Option<ConvertedTo>,
    map_name: bool,
    keep_converted_path: bool,
}

#[derive(Debug, Clone)]
enum ConvertedTo {
    Name(&'static str),
    Pointer(String),
}

#[derive(Debug, Clone, Copy)]
pub struct ImportOptions<G = NoBlobIds> {
    include_ical_components: bool,
    return_first: bool,
    blobs: BlobOptions<G>,
}

pub(super) struct ImportContext<'a, B> {
    include_ical_components: bool,
    return_first: bool,
    blob_ids: Option<BlobIds<'a, B>>,
    task_series: Vec<(String, convert::TaskAnchors)>,
}

impl Default for ImportOptions {
    fn default() -> Self {
        Self {
            include_ical_components: true,
            return_first: false,
            blobs: BlobOptions::default(),
        }
    }
}

impl ImportOptions {
    pub fn new() -> Self {
        Self::default()
    }
}

impl<G> ImportOptions<G> {
    pub fn include_ical_components(mut self, include: bool) -> Self {
        self.include_ical_components = include;
        self
    }

    pub fn return_first(mut self, return_first: bool) -> Self {
        self.return_first = return_first;
        self
    }

    pub fn with_blob_ids<B, F>(self, blob_ids: F) -> ImportOptions<BlobIdFn<F>>
    where
        F: FnMut(&[u8]) -> Option<B>,
    {
        self.with_blob_id_generator(BlobIdFn(blob_ids))
    }

    pub fn with_blob_id_generator<T>(self, blob_id_generator: T) -> ImportOptions<T> {
        ImportOptions {
            include_ical_components: self.include_ical_components,
            return_first: self.return_first,
            blobs: self.blobs.with_handler(blob_id_generator),
        }
    }

    pub(super) fn context<B: Clone>(&mut self) -> ImportContext<'_, B>
    where
        G: BlobIdGenerator<B>,
    {
        ImportContext {
            include_ical_components: self.include_ical_components,
            return_first: self.return_first,
            blob_ids: self.blobs.blob_ids(),
            task_series: Vec::new(),
        }
    }
}
