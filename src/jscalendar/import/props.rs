/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

use crate::{
    common::{
        Data, IanaString, IanaType, LinkRelation,
        blob::GeneratedBlobId,
        jsprop::text::{IntoAsciiLowercase, PointerString},
        timezone::{Tz, ZonedDateTime},
    },
    icalendar::{
        ICalendar, ICalendarComponentType, ICalendarDuration, ICalendarEntry,
        ICalendarParameterName, ICalendarParameterValue, ICalendarProperty, ICalendarValue,
        ICalendarValueType, Uri,
    },
    jscalendar::{
        JSCALENDAR_VERSION, JSCalendarDateTime, JSCalendarId, JSCalendarPrivacy,
        JSCalendarProperty, JSCalendarType, JSCalendarValue,
        ext::{JSCalendarKeyExt, JSCalendarObjectExt, JSCalendarValueExt},
        import::{
            ConvertedTo, EntryState, ICalendarConvertedProperty, ICalendarParams, LinkId, LinkIds,
            State, params::ExtractParams,
        },
        overrides::{Inherited, OverrideDiff},
        uuid5,
    },
};
use ahash::AHashMap;
use jmap_tools::{JsonPointerItem, Key, Map, Property, Value};
use std::{borrow::Cow, collections::hash_map::Entry, mem};

impl<I: JSCalendarId, B: JSCalendarId> State<I, B> {
    pub(super) fn map_named_entry(
        &mut self,
        entry: &mut EntryState,
        extract: &[ICalendarParameterName],
        top_property_name: JSCalendarProperty<I>,
        values: impl IntoIterator<
            Item = (
                Key<'static, JSCalendarProperty<I>>,
                Value<'static, JSCalendarProperty<I>, JSCalendarValue<I, B>>,
            ),
        >,
    ) {
        self.map_named_entry_with_id(entry, extract, top_property_name, values, None)
    }

    pub(super) fn map_named_entry_with_id(
        &mut self,
        entry: &mut EntryState,
        extract: &[ICalendarParameterName],
        top_property_name: JSCalendarProperty<I>,
        values: impl IntoIterator<
            Item = (
                Key<'static, JSCalendarProperty<I>>,
                Value<'static, JSCalendarProperty<I>, JSCalendarValue<I, B>>,
            ),
        >,
        default_id: Option<String>,
    ) {
        // Obtain main property and value
        let mut values = values.into_iter().peekable();
        let (property, value) = match values.peek() {
            Some((property, Value::Str(s))) => (property, s.as_ref()),
            Some((property, _)) => (property, "unknown"),
            _ => {
                panic!("Cannot generate jsid without a value");
            }
        };

        let mut parameters = mem::take(&mut self.parameters);
        let is_link = top_property_name == JSCalendarProperty::Links;
        let js_id = match parameters.extract_params(&mut entry.entry, extract) {
            Some(js_id) => js_id,
            None => {
                let js_id = default_id.unwrap_or_else(|| uuid5(value));
                if is_link {
                    self.link_ids.unique(js_id)
                } else {
                    js_id
                }
            }
        };

        entry.set_converted_to(|| {
            String::from_pointer([
                top_property_name.to_cow().as_ref(),
                js_id.as_str(),
                property.to_string().as_ref(),
            ])
        });

        let capacity = values.size_hint().1.unwrap_or_default() + parameters.len();
        let new_object = || Value::Object(Map::from(Vec::with_capacity(capacity)));
        if let Some(obj) = self
            .entries
            .entry(Key::Property(top_property_name))
            .or_insert_with(Value::new_object)
            .as_object_mut()
            .and_then(|objects| {
                if is_link {
                    self.link_ids.entry(objects, js_id, new_object)
                } else {
                    Some(objects.upsert_or_get_mut(Key::Owned(js_id), new_object))
                }
            })
            .and_then(Value::as_object_mut)
        {
            for (key, value) in values.chain(parameters.as_mut_vec().drain(..)) {
                if let Some(current_value) = obj.lookup_mut(&key) {
                    match (value, current_value) {
                        (Value::Object(new_obj), Value::Object(existing_obj)) => {
                            for (key, value) in new_obj.into_vec() {
                                existing_obj.upsert(key, value);
                            }
                        }
                        (value, current_value) => {
                            *current_value = value;
                        }
                    }
                } else {
                    obj.insert_unchecked(key, value);
                }
            }
        }
        parameters.as_mut_vec().clear();
        self.parameters = parameters;
    }

    pub(super) fn map_blob_link(
        &mut self,
        entry: &mut EntryState,
        generated: GeneratedBlobId<B>,
        content_type: Option<String>,
        default_id: Option<String>,
        value_type: ICalendarValueType,
    ) {
        let (rel, extract): (_, &[ICalendarParameterName]) = match (&entry.entry.name, value_type) {
            (ICalendarProperty::Image, ICalendarValueType::Binary) => (
                LinkRelation::Icon,
                &[
                    ICalendarParameterName::Display,
                    ICalendarParameterName::Fmttype,
                    ICalendarParameterName::Filename,
                    ICalendarParameterName::Linkrel,
                    ICalendarParameterName::Jsid,
                ],
            ),
            (ICalendarProperty::Image, _) => (
                LinkRelation::Icon,
                &[
                    ICalendarParameterName::Display,
                    ICalendarParameterName::Fmttype,
                    ICalendarParameterName::Filename,
                    ICalendarParameterName::Jsid,
                ],
            ),
            (_, ICalendarValueType::Binary) => (
                LinkRelation::Enclosure,
                &[
                    ICalendarParameterName::Fmttype,
                    ICalendarParameterName::Filename,
                    ICalendarParameterName::Linkrel,
                    ICalendarParameterName::Jsid,
                ],
            ),
            _ => (
                LinkRelation::Enclosure,
                &[
                    ICalendarParameterName::Fmttype,
                    ICalendarParameterName::Filename,
                    ICalendarParameterName::Jsid,
                ],
            ),
        };
        entry.entry.params.retain(|param| {
            !matches!(
                param.name,
                ICalendarParameterName::Value | ICalendarParameterName::Size
            )
        });
        entry.set_map_name();

        self.map_named_entry_with_id(
            entry,
            extract,
            JSCalendarProperty::Links,
            [
                Some((
                    Key::Property(JSCalendarProperty::BlobId),
                    Value::Element(JSCalendarValue::BlobId(generated.blob_id)),
                )),
                Some((
                    Key::Property(JSCalendarProperty::Type),
                    Value::Element(JSCalendarValue::Type(JSCalendarType::Link)),
                )),
                Some((
                    Key::Property(JSCalendarProperty::Rel),
                    Value::Element(JSCalendarValue::LinkRelation(rel)),
                )),
                Some((
                    Key::Property(JSCalendarProperty::Size),
                    Value::Number((generated.size as u64).into()),
                )),
                content_type.map(|content_type| {
                    (
                        Key::Property(JSCalendarProperty::ContentType),
                        Value::Str(content_type.into()),
                    )
                }),
            ]
            .into_iter()
            .flatten(),
            default_id,
        );
    }

    pub(super) fn insert_recurrence_override(
        &mut self,
        key: Key<'static, JSCalendarProperty<I>>,
        value: Value<'static, JSCalendarProperty<I>, JSCalendarValue<I, B>>,
    ) {
        self.entries
            .entry(Key::Property(JSCalendarProperty::RecurrenceOverrides))
            .or_insert_with(Value::new_object)
            .as_object_mut()
            .unwrap()
            .upsert(key, value);
    }

    pub(super) fn add_period_conversion_prop(&mut self, converted_to: String) {
        if self.include_ical_components {
            let mut params = ICalendarParams::default();
            params.push(
                ICalendarParameterName::Value,
                Value::Str(ICalendarValueType::Period.as_str().into()),
            );
            self.ical_converted_properties.insert_if_absent(
                converted_to,
                ICalendarConvertedProperty {
                    name: Some(ICalendarProperty::Rdate),
                    params,
                },
            );
        }
    }

    pub(super) fn add_conversion_props(&mut self, mut entry: EntryState) {
        if self.include_ical_components {
            if let Some(converted_to) = entry.converted_to.take() {
                if entry.map_name || !entry.entry.params.is_empty() {
                    let mut value_type = None;

                    let converted_to = converted_to.into_string();
                    match self.ical_converted_properties.get_mut(&converted_to) {
                        Some(conv_prop) => {
                            entry.jcal_parameters(&mut conv_prop.params, &mut value_type);
                        }
                        None => {
                            let mut params = ICalendarParams::default();
                            entry.jcal_parameters(&mut params, &mut value_type);
                            if let Some(value_type) = value_type {
                                params.push(
                                    ICalendarParameterName::Value,
                                    Value::Str(value_type.into_string()),
                                );
                            }
                            if !params.is_empty() || entry.map_name {
                                self.ical_converted_properties.push(
                                    converted_to,
                                    ICalendarConvertedProperty {
                                        name: if entry.map_name {
                                            Some(entry.entry.name)
                                        } else {
                                            None
                                        },
                                        params,
                                    },
                                );
                            }
                        }
                    }
                }
            } else {
                self.ical_properties.push(entry.into_jcal());
            }
        }
    }

    pub(super) fn into_object(
        mut self,
    ) -> Value<'static, JSCalendarProperty<I>, JSCalendarValue<I, B>> {
        let mut ical_obj = Map::from(Vec::new());
        if !self.ical_converted_properties.is_empty() {
            let mut converted_properties =
                Map::from(Vec::with_capacity(self.ical_converted_properties.len()));

            for (converted_to, props) in self.ical_converted_properties {
                let mut obj = Map::from(Vec::with_capacity(2));
                if let Some(params) = props.params.into_jscalendar_value() {
                    obj.upsert(
                        Key::Property(JSCalendarProperty::Parameters),
                        Value::Object(params),
                    );
                }
                if let Some(name) = props.name {
                    obj.upsert(
                        Key::Property(JSCalendarProperty::Name),
                        Value::Str(name.into_string().into_ascii_lowercase().into()),
                    );
                }

                converted_properties.insert_unchecked(Key::Owned(converted_to), Value::Object(obj));
            }

            ical_obj.insert_unchecked(
                Key::Property(JSCalendarProperty::ConvertedProperties),
                Value::Object(converted_properties),
            );
        }

        if !self.ical_properties.is_empty() {
            ical_obj.insert_unchecked(
                Key::Property(JSCalendarProperty::Properties),
                Value::Array(self.ical_properties),
            );
        }

        if let Some(components) = self.ical_components {
            ical_obj.insert_unchecked(Key::Property(JSCalendarProperty::Components), components);
        }

        if !ical_obj.is_empty() || self.map_component {
            ical_obj.insert_unchecked(
                Key::Property(JSCalendarProperty::Name),
                Value::Str(self.component_type.as_str().to_ascii_lowercase().into()),
            );
            self.entries.insert(
                Key::Property(JSCalendarProperty::ICalendar),
                Value::Object(ical_obj),
            );
        }

        if self.has_dates {
            /*
             If a date-time value does not have a timezone, then the timezone is not set
             in JSCalendar.
            */

            if let Some(tz) = self.tz_start.and_then(|tz| tz.name()) {
                self.entries
                    .insert(Key::Property(JSCalendarProperty::TimeZone), Value::Str(tz));
            }

            if self.tz_end.is_some()
                && self.tz_start.is_some()
                && self.tz_end != self.tz_start
                && let Some(tz) = self.tz_end.and_then(|tz| tz.name())
            {
                self.entries.insert(
                    Key::Property(JSCalendarProperty::EndTimeZone),
                    Value::Str(tz),
                );
            }

            if let Some(recurrence_id) = self.recurrence_id {
                self.entries.insert(
                    Key::Property(JSCalendarProperty::RecurrenceId),
                    Value::Element(JSCalendarValue::DateTime(JSCalendarDateTime::new(
                        recurrence_id.naive_timestamp(),
                        true,
                    ))),
                );
                if !self.is_recurrence_instance
                    && let Some(tz) = recurrence_id
                        .timezone()
                        .to_resolved()
                        .and_then(|tz| tz.name())
                {
                    self.entries.insert(
                        Key::Property(JSCalendarProperty::RecurrenceIdTimeZone),
                        Value::Str(tz),
                    );
                }
            }
        }

        if let Some(uid) = self.uid {
            self.entries.insert(
                Key::Property(JSCalendarProperty::Uid),
                Value::Str(uid.into()),
            );
        }

        if !self.is_recurrence_instance
            && let Some(component_type) = self.component_type.to_jscalendar_type()
        {
            self.entries.insert(
                Key::Property(JSCalendarProperty::Type),
                Value::Element(JSCalendarValue::Type(component_type)),
            );
        }

        let mut obj = Value::Object(self.entries.into_map());
        let (override_patches, patches): (Vec<_>, Vec<_>) =
            self.patch_objects.into_iter().partition(|(pointer, _)| {
                !self.recurrence_overrides.is_empty()
                    && matches!(
                        pointer.first(),
                        Some(JsonPointerItem::Key(Key::Property(
                            JSCalendarProperty::RecurrenceOverrides
                        )))
                    )
            });
        for (pointer, patch) in patches {
            obj.apply_jsprop(pointer.as_slice(), patch);
        }

        if !self.recurrence_overrides.is_empty()
            && let Value::Object(base) = &mut obj
        {
            let overrides_key = Key::Property(JSCalendarProperty::RecurrenceOverrides);
            let mut overrides = match base.key_position(&overrides_key) {
                Some(pos) => base.as_mut_vec().remove(pos).1,
                None => Value::new_object(),
            };
            if let Value::Object(overrides) = &mut overrides {
                let diff = OverrideDiff::new(base);
                let duration_key = Key::Property(JSCalendarProperty::Duration);
                let mut slots = overrides
                    .as_vec()
                    .iter()
                    .enumerate()
                    .filter_map(|(position, (key, value))| match key {
                        Key::Property(JSCalendarProperty::DateTime(recurrence_id)) => Some((
                            *recurrence_id,
                            (
                                position,
                                value
                                    .as_object()
                                    .and_then(|period| period.lookup(&duration_key))
                                    .cloned(),
                            ),
                        )),
                        _ => None,
                    })
                    .collect::<AHashMap<_, _>>();
                for (recurrence_id, recurrence) in self.recurrence_overrides {
                    let inherited = recurrence.inherited();
                    let instance = recurrence.into_object().into_object().unwrap_or_default();
                    let mut patch = diff.diff(recurrence_id, instance, inherited);
                    match slots.entry(recurrence_id) {
                        Entry::Occupied(slot) => {
                            let (position, period_duration) = slot.get();
                            if inherited == Inherited::StartAndEnd
                                && let Some(duration) = period_duration
                            {
                                patch.upsert(duration_key.clone(), duration.clone());
                            }
                            if let Some((_, value)) = overrides.as_mut_vec().get_mut(*position) {
                                *value = Value::Object(patch);
                            }
                        }
                        Entry::Vacant(slot) => {
                            slot.insert((overrides.as_vec().len(), None));
                            overrides.insert_unchecked(
                                Key::Property(JSCalendarProperty::DateTime(recurrence_id)),
                                Value::Object(patch),
                            );
                        }
                    }
                }
            }
            base.insert_unchecked(overrides_key, overrides);
        }

        for (pointer, patch) in override_patches {
            obj.apply_jsprop(pointer.as_slice(), patch);
        }

        obj
    }

    pub(super) fn into_root_object(
        self,
    ) -> Value<'static, JSCalendarProperty<I>, JSCalendarValue<I, B>> {
        let has_version = matches!(
            self.component_type,
            ICalendarComponentType::VCalendar
                | ICalendarComponentType::VEvent
                | ICalendarComponentType::VTodo
        );
        let mut obj = self.into_object();
        if has_version && let Value::Object(map) = &mut obj {
            map.upsert(
                Key::Property(JSCalendarProperty::Version),
                Value::Str(JSCALENDAR_VERSION.into()),
            );
        }
        obj
    }

    pub(super) fn set_map_component(&mut self) {
        self.map_component = true;
    }

    pub(super) fn set_is_recurrence_instance(&mut self) {
        self.is_recurrence_instance = true;
    }

    fn inherited(&self) -> Inherited {
        if self
            .entries
            .contains_key(&Key::Property(JSCalendarProperty::Start))
        {
            Inherited::Nothing
        } else if self.has_end {
            Inherited::Start
        } else {
            Inherited::StartAndEnd
        }
    }

    pub(super) fn has_start_or_due(&self) -> bool {
        self.entries
            .contains_key(&Key::Property(JSCalendarProperty::Start))
            || self
                .entries
                .contains_key(&Key::Property(JSCalendarProperty::Due))
    }

    pub(super) fn inherit_time_zone(&mut self, tz: Option<Tz>) {
        if self
            .entries
            .contains_key(&Key::Property(JSCalendarProperty::Start))
            || (tz.is_none() && self.due.is_some())
        {
            return;
        }
        if let Some(due) = self.due {
            self.entries.insert(
                Key::Property(JSCalendarProperty::Due),
                Value::Element(JSCalendarValue::DateTime(JSCalendarDateTime::local_in(
                    due, tz,
                ))),
            );
        }
        self.tz_start = tz;
    }

    pub(super) fn start_at_recurrence_id(&mut self) {
        if let Some(recurrence_id) = self.recurrence_id
            && !self.has_start_or_due()
        {
            self.entries.insert(
                Key::Property(JSCalendarProperty::Start),
                Value::Element(JSCalendarValue::DateTime(JSCalendarDateTime::new(
                    recurrence_id.naive_timestamp(),
                    true,
                ))),
            );
            if self.recurrence_id_is_date {
                self.default_to_one_day();
            }
        }
    }

    pub(super) fn default_to_one_day(&mut self) {
        if self.has_end || self.component_type != ICalendarComponentType::VEvent {
            return;
        }
        self.entries.insert(
            Key::Property(JSCalendarProperty::Duration),
            Value::Element(JSCalendarValue::Duration(ICalendarDuration::from_days(1))),
        );
        if self.include_ical_components {
            self.ical_converted_properties.insert(
                JSCalendarProperty::Duration::<I>.to_string().into_owned(),
                ICalendarConvertedProperty {
                    name: Some(ICalendarProperty::Dtstart),
                    ..Default::default()
                },
            );
        }
    }

    pub(super) fn recurrence_sequence(&self) -> Option<i64> {
        self.recurrence_id.map(|_| {
            self.entries
                .get(&Key::Property(JSCalendarProperty::Sequence))
                .and_then(Value::as_i64)
                .unwrap_or_default()
        })
    }

    pub(super) fn privacy(&self) -> Option<JSCalendarPrivacy> {
        match self
            .entries
            .get(&Key::Property(JSCalendarProperty::Privacy))?
        {
            Value::Element(JSCalendarValue::Privacy(privacy)) => Some(*privacy),
            _ => Some(JSCalendarPrivacy::Private),
        }
    }

    pub(super) fn raise_privacy(&mut self, privacy: JSCalendarPrivacy) {
        if self.privacy().unwrap_or(JSCalendarPrivacy::Public) < privacy {
            self.entries.insert(
                Key::Property(JSCalendarProperty::Privacy),
                Value::Element(JSCalendarValue::Privacy(privacy)),
            );
            self.patch_objects.retain(|(pointer, _)| {
                !matches!(
                    pointer.first(),
                    Some(JsonPointerItem::Key(Key::Property(
                        JSCalendarProperty::Privacy
                    )))
                )
            });
        }
    }

    pub(super) fn merge_privacy(&mut self, privacy: JSCalendarPrivacy) {
        let privacy = self
            .privacy()
            .map_or(privacy, |current| current.max(privacy));
        self.entries.insert(
            Key::Property(JSCalendarProperty::Privacy),
            Value::Element(JSCalendarValue::Privacy(privacy)),
        );
    }

    pub(super) fn remove_forbidden_override_patches(&mut self) {
        self.entries.retain(|key, _| {
            !matches!(key, Key::Property(property) if property.is_forbidden_override_patch())
        });
        self.patch_objects
            .retain(|(pointer, _)| !JSCalendarProperty::is_forbidden_override_pointer(pointer));
        self.ical_converted_properties.retain(|converted_to, _| {
            !Key::<JSCalendarProperty<I>>::Borrowed(converted_to).is_series_converted_property()
        });
    }

    pub(super) fn find_participant_by_address(&self, address: &str) -> Option<String> {
        self.entries
            .get(&Key::Property(JSCalendarProperty::Participants))?
            .as_object()?
            .iter()
            .find_map(|(key, value)| {
                match value
                    .as_object()?
                    .lookup(&Key::Property(JSCalendarProperty::CalendarAddress))?
                {
                    Value::Str(value) if value == address => Some(key.to_string().into_owned()),
                    _ => None,
                }
            })
    }

    #[inline]
    pub(super) fn get_mut_object_or_insert(
        &mut self,
        key: JSCalendarProperty<I>,
    ) -> &mut Map<'static, JSCalendarProperty<I>, JSCalendarValue<I, B>> {
        self.entries
            .entry(Key::Property(key))
            .or_insert_with(|| Value::Object(Map::from(Vec::new())))
            .as_object_mut()
            .unwrap()
    }
}

impl JSCalendarDateTime {
    pub(super) fn local_in(dt: ZonedDateTime, tz: Option<Tz>) -> Self {
        JSCalendarDateTime::new(
            if dt.timezone().is_floating() || tz == Some(dt.timezone()) {
                dt.naive_timestamp()
            } else {
                dt.with_timezone(tz.unwrap_or_default()).naive_timestamp()
            },
            true,
        )
    }
}

impl EntryState {
    pub(super) fn new(entry: ICalendarEntry, keep_converted_path: bool) -> Self {
        Self {
            entry,
            converted_to: None,
            map_name: false,
            keep_converted_path,
        }
    }

    pub(super) fn set_converted_to(&mut self, converted_to: impl FnOnce() -> String) {
        self.converted_to = Some(ConvertedTo::Pointer(
            if self.keep_converted_path && (self.map_name || !self.entry.params.is_empty()) {
                converted_to()
            } else {
                String::new()
            },
        ));
    }

    pub(super) fn set_converted_to_property<I: JSCalendarId>(
        &mut self,
        property: &JSCalendarProperty<I>,
    ) {
        self.converted_to = Some(match property.static_name() {
            Some(name) => ConvertedTo::Name(name),
            None => ConvertedTo::Pointer(String::from_pointer([property.to_string().as_ref()])),
        });
    }

    pub(super) fn set_map_name(&mut self) {
        self.map_name = true;
    }

    pub(super) fn into_jcal<I: JSCalendarId, B: JSCalendarId>(
        mut self,
    ) -> Value<'static, JSCalendarProperty<I>, JSCalendarValue<I, B>> {
        let mut value_type = None;
        let mut params = ICalendarParams::default();

        self.jcal_parameters(&mut params, &mut value_type);

        let values = if self.entry.values.len() == 1 {
            self.entry
                .values
                .into_iter()
                .next()
                .unwrap()
                .into_jscalendar_value(value_type.as_ref())
        } else {
            let mut values = Vec::with_capacity(self.entry.values.len());
            for value in self.entry.values {
                values.push(value.into_jscalendar_value(value_type.as_ref()));
            }
            Value::Array(values)
        };
        Value::Array(vec![
            Value::Str(self.entry.name.into_string().into_ascii_lowercase().into()),
            Value::Object(
                params
                    .into_jscalendar_value()
                    .unwrap_or(Map::from(Vec::new())),
            ),
            Value::Str(
                value_type
                    .map(|v| v.into_string())
                    .unwrap_or(Cow::Borrowed("unknown")),
            ),
            values,
        ])
    }

    pub(super) fn jcal_parameters<I: JSCalendarId, B: JSCalendarId>(
        &mut self,
        params: &mut ICalendarParams<I, B>,
        value_type: &mut Option<IanaType<ICalendarValueType, String>>,
    ) {
        if self.entry.params.is_empty() {
            return;
        }
        let (default_type, _) = self.entry.name.default_types();
        let default_type = default_type.unwrap_ical();

        for param in std::mem::take(&mut self.entry.params) {
            if matches!(param.name, ICalendarParameterName::Value) {
                if let Some(v) = param
                    .value
                    .into_value_type()
                    .filter(|v| !v.is_iana_and(|v| v == &default_type))
                {
                    *value_type = Some(v);
                }
            } else if let (ICalendarParameterName::Range, ICalendarParameterValue::Bool(true)) =
                (&param.name, &param.value)
            {
                params.push(param.name, Value::Str("THISANDFUTURE".into()));
            } else if let Some(value) = param.value.into_text() {
                params.push(param.name, Value::Str(value));
            }
        }
    }
}

impl ConvertedTo {
    fn into_string(self) -> String {
        match self {
            ConvertedTo::Name(name) => String::from_pointer([name]),
            ConvertedTo::Pointer(pointer) => pointer,
        }
    }
}

impl LinkIds {
    fn unique(&mut self, base: String) -> String {
        let Some(mut suffix) = self.0.get(&base).map(|link| link.next_suffix) else {
            return base;
        };
        let unique = loop {
            let candidate = format!("{base}-{suffix}");
            suffix = suffix.saturating_add(1);
            if !self.0.contains_key(&candidate) {
                break candidate;
            }
        };
        if let Some(link) = self.0.get_mut(&base) {
            link.next_suffix = suffix;
        }
        unique
    }

    fn entry<'x, I: JSCalendarId, B: JSCalendarId>(
        &mut self,
        links: &'x mut Map<'static, JSCalendarProperty<I>, JSCalendarValue<I, B>>,
        key: String,
        value: impl FnOnce() -> Value<'static, JSCalendarProperty<I>, JSCalendarValue<I, B>>,
    ) -> Option<&'x mut Value<'static, JSCalendarProperty<I>, JSCalendarValue<I, B>>> {
        let position = match self.0.get(&key) {
            Some(link) => link.position,
            None => {
                let position = links.len();
                links.insert_unchecked(Key::Owned(key.clone()), value());
                self.0.insert(
                    key,
                    LinkId {
                        position,
                        next_suffix: 2,
                    },
                );
                position
            }
        };
        links.as_mut_vec().get_mut(position).map(|(_, value)| value)
    }
}

impl ICalendar {
    pub(super) fn binary_link_sizes(&self) -> impl Iterator<Item = usize> + '_ {
        self.blob_binaries().map(<[u8]>::len)
    }
}

impl ICalendarEntry {
    pub fn participant_id(&self) -> Option<Cow<'_, str>> {
        if let Some(jsid) = self.jsid() {
            return Some(Cow::Borrowed(jsid));
        }
        match self.values.first()? {
            ICalendarValue::Text(address) => Some(Cow::Owned(uuid5(address))),
            ICalendarValue::Uri(Uri::Location(address)) => Some(Cow::Owned(uuid5(address))),
            ICalendarValue::Uri(uri) => Some(Cow::Owned(uuid5(uri.to_unwrapped_string()))),
            _ => None,
        }
    }
}

pub(super) struct ICalendarBinary {
    pub(super) data: Vec<u8>,
    pub(super) content_type: Option<String>,
    pub(super) is_data_uri: bool,
}

impl ICalendarBinary {
    pub(super) fn into_value(self) -> ICalendarValue {
        if self.is_data_uri {
            ICalendarValue::Uri(Uri::Data(Box::new(Data {
                content_type: self.content_type,
                data: self.data,
            })))
        } else {
            ICalendarValue::Binary(self.data)
        }
    }
}

impl ICalendarValue {
    pub(super) fn is_binary(&self) -> bool {
        self.binary_bytes().is_some()
    }

    pub(super) fn into_binary(self) -> Option<ICalendarBinary> {
        match self {
            ICalendarValue::Binary(data) => Some(ICalendarBinary {
                data,
                content_type: None,
                is_data_uri: false,
            }),
            ICalendarValue::Uri(Uri::Data(data)) => Some(ICalendarBinary {
                data: data.data,
                content_type: data.content_type,
                is_data_uri: true,
            }),
            _ => None,
        }
    }

    pub(super) fn uri_to_string(self, media_type: Option<String>) -> Self {
        match self {
            ICalendarValue::Uri(uri) => ICalendarValue::Text(uri.into_unwrapped_string()),
            ICalendarValue::Binary(data) => ICalendarValue::Text(
                Uri::Data(Box::new(Data {
                    content_type: media_type,
                    data,
                }))
                .into_unwrapped_string(),
            ),
            other => other,
        }
    }
}
