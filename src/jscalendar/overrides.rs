/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

use crate::{
    common::{IanaParse, elements::Elements, jsprop::text::ConvertedKeys},
    icalendar::ICalendarProperty,
    jscalendar::{
        JSCalendarDateTime, JSCalendarId, JSCalendarProperty, JSCalendarValue,
        ext::{JSCalendarKeyExt, JSCalendarObjectExt, JSCalendarPatch},
    },
};
use ahash::AHashMap;
use jmap_tools::{JsonPointerHandler, JsonPointerItem, Key, Map, Value};
use std::{borrow::Cow, ptr, str::FromStr};

type JSCalendarMap<'x, I, B> = Map<'x, JSCalendarProperty<I>, JSCalendarValue<I, B>>;
type JSCalendarObject<'x, I, B> = Value<'x, JSCalendarProperty<I>, JSCalendarValue<I, B>>;

#[derive(Debug, Clone, Copy, Default)]
struct SeriesDates {
    start: Option<JSCalendarDateTime>,
    due: Option<JSCalendarDateTime>,
}

pub(crate) struct OverrideTemplate<'x, I: JSCalendarId, B: JSCalendarId> {
    entries: JSCalendarMap<'x, I, B>,
    dates: SeriesDates,
    has_participants: bool,
    uses: usize,
}

impl SeriesDates {
    fn new<I: JSCalendarId, B: JSCalendarId>(base: &JSCalendarMap<'_, I, B>) -> Self {
        let mut dates = SeriesDates::default();
        for (key, value) in base.iter() {
            match (key, value) {
                (
                    Key::Property(JSCalendarProperty::Start),
                    Value::Element(JSCalendarValue::DateTime(dt)),
                ) => dates.start = Some(*dt),
                (
                    Key::Property(JSCalendarProperty::Due),
                    Value::Element(JSCalendarValue::DateTime(dt)),
                ) => dates.due = Some(*dt),
                _ => {}
            }
        }
        dates
    }

    fn instance(
        &self,
        recurrence_id: JSCalendarDateTime,
    ) -> (Option<JSCalendarDateTime>, Option<JSCalendarDateTime>) {
        let Some(start) = self.start else {
            return (None, self.due);
        };
        let rid = JSCalendarDateTime::new(recurrence_id.timestamp, true);
        (
            Some(rid),
            self.due.map(|due| {
                JSCalendarDateTime::new(
                    due.timestamp + (rid.timestamp - start.timestamp),
                    due.is_local,
                )
            }),
        )
    }
}

trait OverrideMapExt<I: JSCalendarId>: Sized {
    fn last_object(&self, property: &JSCalendarProperty<I>) -> Option<&Self>;
    fn converted_override_dates(&self) -> Vec<JSCalendarDateTime>;
    fn reaches_template(&self, is_converted: impl FnMut() -> bool) -> bool;
}

impl<I: JSCalendarId, B: JSCalendarId> OverrideMapExt<I> for JSCalendarMap<'_, I, B> {
    fn last_object(&self, property: &JSCalendarProperty<I>) -> Option<&Self> {
        self.as_vec()
            .iter()
            .rev()
            .find_map(|(key, value)| match (key, value) {
                (Key::Property(key), Value::Object(obj)) if key == property => Some(obj),
                _ => None,
            })
    }

    fn converted_override_dates(&self) -> Vec<JSCalendarDateTime> {
        self.last_object(&JSCalendarProperty::ICalendar)
            .into_iter()
            .flat_map(|ical| ical.iter())
            .filter_map(|(key, value)| match (key, value) {
                (Key::Property(JSCalendarProperty::ConvertedProperties), Value::Object(props)) => {
                    Some(props.keys())
                }
                _ => None,
            })
            .flatten()
            .filter_map(|key| match key.clone().into_converted_keys().as_slice() {
                [
                    Key::Property(JSCalendarProperty::RecurrenceOverrides),
                    Key::Property(JSCalendarProperty::DateTime(dt)),
                    ..,
                ] => Some(*dt),
                _ => None,
            })
            .collect()
    }

    fn reaches_template(&self, mut is_converted: impl FnMut() -> bool) -> bool {
        let exported = || {
            self.iter().filter(|(key, value)| {
                !key.is_forbidden_override_key()
                    && (!matches!(key, Key::Property(JSCalendarProperty::Excluded))
                        || matches!(value, Value::Bool(true)))
            })
        };
        let mut members = exported();
        match (members.next(), members.next()) {
            (None, _) => false,
            (Some((key, Value::Element(JSCalendarValue::Duration(_)))), None)
                if key.same_key(&Key::Property(JSCalendarProperty::Duration)) && is_converted() =>
            {
                false
            }
            _ => !exported().any(|(key, value)| {
                matches!(value, Value::Bool(true))
                    && key.same_key(&Key::Property(JSCalendarProperty::Excluded))
            }),
        }
    }
}

impl<'x, I: JSCalendarId, B: JSCalendarId> OverrideTemplate<'x, I, B> {
    pub(crate) fn uses(
        base: &JSCalendarMap<'x, I, B>,
        overrides: &JSCalendarMap<'x, I, B>,
    ) -> usize {
        let instances = overrides
            .values()
            .filter(|patch| patch.is_instance_patch())
            .count();
        if instances == 0
            || !base
                .last_object(&JSCalendarProperty::RecurrenceOverrides)
                .is_some_and(|exported| ptr::eq(exported, overrides))
        {
            return instances;
        }
        let mut converted = None;
        let has_instance = overrides.iter().any(|(key, patch)| match (key, patch) {
            (Key::Property(JSCalendarProperty::DateTime(dt)), Value::Object(patch)) => patch
                .reaches_template(|| {
                    converted
                        .get_or_insert_with(|| base.converted_override_dates())
                        .contains(dt)
                }),
            _ => false,
        });
        if has_instance { instances } else { 0 }
    }

    pub(crate) fn new(base: &JSCalendarMap<'x, I, B>, uses: usize) -> Self {
        let mut entries = Map::from(Vec::with_capacity(base.len()));
        let mut has_participants = false;

        for (key, value) in base.iter() {
            match (key, value) {
                (Key::Property(property), _) if property.is_series_property() => {}
                (Key::Property(JSCalendarProperty::ICalendar), Value::Object(obj)) => {
                    if let Some(obj) = instance_ical_clone(obj) {
                        entries.insert_unchecked(key.clone(), Value::Object(obj));
                    }
                }
                (Key::Property(JSCalendarProperty::Participants), Value::Object(obj)) => {
                    has_participants = !obj.is_empty();
                    entries.insert_unchecked(key.clone(), value.clone());
                }
                _ => {
                    entries.insert_unchecked(key.clone(), value.clone());
                }
            }
        }

        Self {
            entries,
            dates: SeriesDates::new(base),
            has_participants,
            uses,
        }
    }

    pub(crate) fn blob_uses(&self) -> impl Iterator<Item = &B> {
        Elements::from_values(self.entries.values())
            .filter_map(|element| match element {
                JSCalendarValue::BlobId(blob_id) => Some(blob_id),
                _ => None,
            })
            .flat_map(|blob_id| std::iter::repeat_n(blob_id, self.uses))
    }

    pub(crate) fn instance(
        &mut self,
        recurrence_id: JSCalendarDateTime,
    ) -> JSCalendarMap<'x, I, B> {
        let (start, due) = self.dates.instance(recurrence_id);
        for (property, dt) in [
            (JSCalendarProperty::Start, start),
            (JSCalendarProperty::Due, due),
        ] {
            if let Some(dt) = dt
                && let Some(value) = self.entries.lookup_mut(&Key::Property(property))
            {
                *value = Value::Element(JSCalendarValue::DateTime(dt));
            }
        }

        self.uses = self.uses.saturating_sub(1);
        if self.uses == 0 {
            std::mem::take(&mut self.entries)
        } else {
            self.entries.clone()
        }
    }

    pub(crate) fn apply_patch(
        &self,
        instance: JSCalendarMap<'x, I, B>,
        patch: JSCalendarMap<'x, I, B>,
    ) -> Result<JSCalendarMap<'x, I, B>, String> {
        let mut instance = Value::Object(instance);
        let mut invalid_pointer = None;

        for (key, value) in patch.into_vec() {
            match key {
                Key::Property(JSCalendarProperty::Pointer(pointer)) => {
                    if !instance.patch_jptr(pointer.iter(), value) && invalid_pointer.is_none() {
                        invalid_pointer = Some(pointer.to_string());
                    }
                }
                Key::Property(JSCalendarProperty::Excluded) => {}
                key => {
                    if let Value::Object(entries) = &mut instance {
                        if value.is_null() {
                            if let Some(pos) = entries.key_position(&key) {
                                entries.as_mut_vec().remove(pos);
                            }
                        } else {
                            entries.upsert(key, value);
                        }
                    }
                }
            }
        }

        let Value::Object(mut instance) = instance else {
            return Err(invalid_pointer.unwrap_or_default());
        };

        if self.has_participants
            && instance
                .lookup(&Key::Property(JSCalendarProperty::Participants))
                .and_then(Value::as_object)
                .is_none_or(|participants| participants.is_empty())
        {
            instance.as_mut_vec().retain(|(key, _)| {
                !matches!(
                    key,
                    Key::Property(
                        JSCalendarProperty::OrganizerCalendarAddress | JSCalendarProperty::SentBy
                    )
                )
            });
        }

        match invalid_pointer {
            Some(pointer) => Err(pointer),
            None => Ok(instance),
        }
    }
}

fn instance_ical_clone<'x, I: JSCalendarId, B: JSCalendarId>(
    obj: &JSCalendarMap<'x, I, B>,
) -> Option<JSCalendarMap<'x, I, B>> {
    let mut obj = obj.clone();
    retain_instance_ical(&mut obj).then_some(obj)
}

fn retain_instance_ical<I: JSCalendarId, B: JSCalendarId>(
    obj: &mut JSCalendarMap<'_, I, B>,
) -> bool {
    let mut has_contents = false;
    obj.as_mut_vec().retain_mut(|(key, value)| {
        let keep = match (key, value) {
            (Key::Property(JSCalendarProperty::ConvertedProperties), Value::Object(props)) => {
                props
                    .as_mut_vec()
                    .retain(|(key, _)| !key.is_series_converted_property());
                !props.is_empty()
            }
            (Key::Property(JSCalendarProperty::Properties), Value::Array(props)) => {
                props.retain(|property| !is_series_ical_property(property));
                !props.is_empty()
            }
            (Key::Property(JSCalendarProperty::Name), _) => return true,
            _ => true,
        };
        has_contents |= keep;
        keep
    });
    has_contents
}

fn is_series_ical_property<I: JSCalendarId, B: JSCalendarId>(
    property: &JSCalendarObject<'_, I, B>,
) -> bool {
    property
        .as_array()
        .and_then(|items| items.first())
        .and_then(Value::as_str)
        .is_some_and(|name| {
            matches!(
                ICalendarProperty::parse(name.as_bytes()),
                Some(
                    ICalendarProperty::Rrule
                        | ICalendarProperty::Rdate
                        | ICalendarProperty::Exdate
                        | ICalendarProperty::Exrule
                        | ICalendarProperty::RecurrenceId
                )
            )
        })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Inherited {
    Nothing,
    Start,
    StartAndEnd,
}

impl Inherited {
    fn covers<I: JSCalendarId>(self, property: &JSCalendarProperty<I>) -> bool {
        match (self, property) {
            (Inherited::Nothing, _) => false,
            (
                _,
                JSCalendarProperty::Start
                | JSCalendarProperty::TimeZone
                | JSCalendarProperty::ShowWithoutTime,
            ) => true,
            (
                Inherited::StartAndEnd,
                JSCalendarProperty::Duration
                | JSCalendarProperty::Due
                | JSCalendarProperty::EndTimeZone,
            ) => true,
            _ => false,
        }
    }

    fn covers_key<I: JSCalendarId>(self, key: &Key<'_, JSCalendarProperty<I>>) -> bool {
        match key {
            Key::Property(JSCalendarProperty::Pointer(pointer)) => {
                let mut items = pointer
                    .iter()
                    .filter(|item| !matches!(item, JsonPointerItem::Root));
                matches!(
                    (items.next(), items.next()),
                    (Some(JsonPointerItem::Key(Key::Property(property))), None)
                        if self.covers(property)
                )
            }
            Key::Property(property) => self.covers(property),
            Key::Borrowed(_) | Key::Owned(_) => {
                JSCalendarProperty::<I>::from_str(key.to_string().as_ref())
                    .is_ok_and(|property| self.covers(&property))
            }
        }
    }
}

pub struct OverrideDiff<'a, I: JSCalendarId, B: JSCalendarId> {
    base: &'a JSCalendarMap<'static, I, B>,
    base_ical: Option<JSCalendarMap<'static, I, B>>,
    dates: SeriesDates,
}

impl<'a, I: JSCalendarId, B: JSCalendarId> OverrideDiff<'a, I, B> {
    pub fn new(base: &'a JSCalendarMap<'static, I, B>) -> Self {
        Self {
            base,
            base_ical: base
                .lookup(&Key::Property(JSCalendarProperty::ICalendar))
                .and_then(Value::as_object)
                .and_then(instance_ical_clone),
            dates: SeriesDates::new(base),
        }
    }

    pub fn diff(
        &self,
        recurrence_id: JSCalendarDateTime,
        mut instance: JSCalendarMap<'static, I, B>,
        inherited: Inherited,
    ) -> JSCalendarMap<'static, I, B> {
        let (start, due) = self.dates.instance(recurrence_id);
        let mut patch = Map::from(Vec::new());
        let ical_key = Key::Property(JSCalendarProperty::ICalendar);

        for (key, _) in self.base.iter() {
            if !key.is_ignored_override_key()
                && !matches!(
                    key,
                    Key::Property(JSCalendarProperty::Start | JSCalendarProperty::ICalendar)
                )
                && !inherited.covers_key(key)
                && instance.key_position(key).is_none()
            {
                patch.insert_unchecked(key.to_owned(), Value::Null);
            }
        }

        if inherited != Inherited::Nothing
            && let Some(base_ical) = &self.base_ical
        {
            inherit_date_conversions(&mut instance, base_ical, inherited);
        }
        let has_ical = match instance.lookup_mut(&ical_key) {
            Some(Value::Object(obj)) => retain_instance_ical(obj),
            _ => false,
        };
        if !has_ical {
            if let Some(pos) = instance.key_position(&ical_key) {
                instance.as_mut_vec().remove(pos);
            }
            if self.base_ical.is_some() {
                patch.insert_unchecked(ical_key, Value::Null);
            }
        }

        for (key, value) in instance.into_vec() {
            if key.is_ignored_override_key() {
                continue;
            }
            let base_value = match (&key, &value) {
                (
                    Key::Property(JSCalendarProperty::Start),
                    Value::Element(JSCalendarValue::DateTime(dt)),
                ) if start == Some(*dt) => continue,
                (
                    Key::Property(JSCalendarProperty::Due),
                    Value::Element(JSCalendarValue::DateTime(dt)),
                ) if due == Some(*dt) => continue,
                (Key::Property(JSCalendarProperty::ICalendar), Value::Object(ical))
                    if self
                        .base_ical
                        .as_ref()
                        .is_some_and(|base| same_members(base, ical)) =>
                {
                    continue;
                }
                (
                    Key::Property(
                        JSCalendarProperty::Start
                        | JSCalendarProperty::Due
                        | JSCalendarProperty::ICalendar,
                    ),
                    _,
                ) => None,
                (key, _) => self.base.lookup(key),
            };

            match (base_value, key, value) {
                (
                    Some(Value::Object(base_members)),
                    Key::Property(property),
                    Value::Object(members),
                ) if property.is_member_map() => {
                    diff_members(&mut patch, property, base_members, members);
                }
                (Some(base_value), _, value) if same_value(base_value, &value) => {}
                (_, key, value) => {
                    patch.insert_unchecked(key, value);
                }
            }
        }

        patch
    }
}

fn inherit_date_conversions<I: JSCalendarId, B: JSCalendarId>(
    instance: &mut JSCalendarMap<'static, I, B>,
    base_ical: &JSCalendarMap<'static, I, B>,
    inherited: Inherited,
) {
    let converted_key = Key::Property(JSCalendarProperty::ConvertedProperties);
    let Some(Value::Object(base_converted)) = base_ical.lookup(&converted_key) else {
        return;
    };
    let mut conversions = base_converted
        .iter()
        .filter(|(key, _)| inherited.covers_key(key))
        .peekable();
    if conversions.peek().is_none() {
        return;
    }

    let Value::Object(ical) = instance.upsert_or_get_mut(
        Key::Property(JSCalendarProperty::ICalendar),
        Value::new_object,
    ) else {
        return;
    };
    let name_key = Key::Property(JSCalendarProperty::Name);
    if ical.key_position(&name_key).is_none()
        && let Some(name) = base_ical.lookup(&name_key)
    {
        ical.insert_unchecked(name_key, name.clone());
    }
    let Value::Object(converted) = ical.upsert_or_get_mut(converted_key, Value::new_object) else {
        return;
    };
    for (key, value) in conversions {
        if converted.key_position(key).is_none() {
            converted.insert_unchecked(key.clone(), value.clone());
        }
    }
}

fn diff_members<I: JSCalendarId, B: JSCalendarId>(
    patch: &mut JSCalendarMap<'static, I, B>,
    property: JSCalendarProperty<I>,
    base_members: &JSCalendarMap<'static, I, B>,
    members: JSCalendarMap<'static, I, B>,
) {
    let base_members = base_members.as_vec();
    let mut matched = vec![false; base_members.len()];
    let mut next_pos = 0;
    let mut index: Option<AHashMap<Cow<'_, str>, usize>> = None;

    for (id, member) in members.into_vec() {
        let pos = if base_members
            .get(next_pos)
            .is_some_and(|(base_id, _)| base_id.same_key(&id))
        {
            Some(next_pos)
        } else {
            index
                .get_or_insert_with(|| {
                    base_members
                        .iter()
                        .enumerate()
                        .map(|(pos, (id, _))| (id.to_string(), pos))
                        .collect()
                })
                .get(id.to_string().as_ref())
                .copied()
        };

        let Some((pos, base_member)) =
            pos.and_then(|pos| base_members.get(pos).map(|(_, member)| (pos, member)))
        else {
            patch.insert_unchecked(property.member_pointer([id]), member);
            continue;
        };
        if let Some(matched) = matched.get_mut(pos) {
            *matched = true;
        }
        next_pos = pos + 1;

        match (base_member, member) {
            (base_member, member) if same_value(base_member, &member) => {}
            (Value::Object(base_member), Value::Object(member))
                if property == JSCalendarProperty::Participants
                    && base_member.lookup(&Key::Property(JSCalendarProperty::CalendarAddress))
                        != member.lookup(&Key::Property(JSCalendarProperty::CalendarAddress)) =>
            {
                patch.insert_unchecked(property.member_pointer([id]), Value::Object(member));
            }
            (Value::Object(base_member), Value::Object(member)) => {
                for (sub_property, _) in base_member.iter() {
                    if member.key_position(sub_property).is_none() {
                        patch.insert_unchecked(
                            property.member_pointer([id.clone(), sub_property.to_owned()]),
                            Value::Null,
                        );
                    }
                }
                for (sub_property, value) in member.into_vec() {
                    match (base_member.lookup(&sub_property), value) {
                        (Some(base_value), value) if same_value(base_value, &value) => {}
                        (Some(Value::Object(base_set)), Value::Object(set))
                            if is_boolean_set(base_set) && is_boolean_set(&set) =>
                        {
                            for (key, _) in base_set.iter() {
                                if set.key_position(key).is_none() {
                                    patch.insert_unchecked(
                                        property.member_pointer([
                                            id.clone(),
                                            sub_property.clone(),
                                            key.to_owned(),
                                        ]),
                                        Value::Null,
                                    );
                                }
                            }
                            for (key, value) in set.into_vec() {
                                if !base_set
                                    .lookup(&key)
                                    .is_some_and(|base| same_value(base, &value))
                                {
                                    patch.insert_unchecked(
                                        property.member_pointer([
                                            id.clone(),
                                            sub_property.clone(),
                                            key,
                                        ]),
                                        value,
                                    );
                                }
                            }
                        }
                        (_, value) => {
                            patch.insert_unchecked(
                                property.member_pointer([id.clone(), sub_property]),
                                value,
                            );
                        }
                    }
                }
            }
            (_, member) => {
                patch.insert_unchecked(property.member_pointer([id]), member);
            }
        }
    }

    for ((id, _), matched) in base_members.iter().zip(matched) {
        if !matched {
            patch.insert_unchecked(property.member_pointer([id.to_owned()]), Value::Null);
        }
    }
}

fn same_value<I: JSCalendarId, B: JSCalendarId>(
    a: &JSCalendarObject<'_, I, B>,
    b: &JSCalendarObject<'_, I, B>,
) -> bool {
    match (a, b) {
        (Value::Object(a), Value::Object(b)) => same_members(a, b),
        (Value::Array(a), Value::Array(b)) => {
            a.len() == b.len() && a.iter().zip(b).all(|(a, b)| same_value(a, b))
        }
        _ => a == b,
    }
}

fn same_members<I: JSCalendarId, B: JSCalendarId>(
    a: &JSCalendarMap<'_, I, B>,
    b: &JSCalendarMap<'_, I, B>,
) -> bool {
    if a.len() != b.len() {
        return false;
    }
    for (pos, ((a_key, a_value), (b_key, b_value))) in a.iter().zip(b.iter()).enumerate() {
        if !a_key.same_key(b_key) {
            let index = b
                .iter()
                .skip(pos)
                .map(|(key, value)| (key.to_string(), value))
                .collect::<AHashMap<_, _>>();
            return index.len() == a.len() - pos
                && a.iter().skip(pos).all(|(key, a_value)| {
                    index
                        .get(key.to_string().as_ref())
                        .is_some_and(|b_value| same_value(a_value, b_value))
                });
        } else if !same_value(a_value, b_value) {
            return false;
        }
    }
    true
}

fn is_boolean_set<I: JSCalendarId, B: JSCalendarId>(set: &JSCalendarMap<'_, I, B>) -> bool {
    set.values().all(|value| matches!(value, Value::Bool(_)))
}
