/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

use crate::{
    common::{CalendarScale, Data, PartialDateTime},
    jscontact::{
        JSContact, JSContactGrammaticalGender, JSContactKind, JSContactPhoneticSystem,
        JSContactProperty, JSContactValue,
    },
    vcard::{
        VCard, VCardEntry, VCardGramGender, VCardKind, VCardParameter, VCardPhonetic,
        VCardProperty, VCardType, VCardValue, VCardValueType,
    },
};
use jmap_tools::{Key, Value};

impl JSContact<'_> {
    pub fn into_vcard(self) -> Option<VCard> {
        let mut vcard = VCard::default();

        for (property, value) in self.0.into_object()?.into_vec() {
            let Key::Property(property) = property else {
                continue;
            };

            match property {
                JSContactProperty::Uid => {
                    if let Some(text) = value.into_string() {
                        vcard
                            .entries
                            .push(VCardEntry::new(VCardProperty::Uid).with_value(text));
                    }
                }
                JSContactProperty::Kind => {
                    if let Some(kind) = convert_value(value) {
                        vcard
                            .entries
                            .push(VCardEntry::new(VCardProperty::Kind).with_value(kind));
                    }
                }
                JSContactProperty::Directories => {
                    for (name, value) in value.into_expanded_object() {
                        let mut entry = VCardEntry::new(VCardProperty::OrgDirectory)
                            .with_param(VCardParameter::PropId(name.into_string()));
                        let mut value_type = None;

                        for (prop, value) in value.into_expanded_object() {
                            match prop {
                                Key::Property(JSContactProperty::Kind) => {
                                    if matches!(
                                        value,
                                        Value::Element(JSContactValue::Kind(JSContactKind::Entry))
                                    ) {
                                        entry.name = VCardProperty::Source;
                                    }
                                }
                                Key::Property(JSContactProperty::Uri) => {
                                    if let Some(text) = value.into_string() {
                                        entry.values.push(VCardValue::Text(text));
                                        value_type = Some(VCardValueType::Uri);
                                    }
                                }
                                Key::Property(JSContactProperty::Pref) => {
                                    if let Some(pref) = value.as_u64() {
                                        entry.params.push(VCardParameter::Pref(pref as u32));
                                    }
                                }
                                Key::Property(JSContactProperty::ListAs) => {
                                    if let Some(index) = value.as_u64() {
                                        entry.params.push(VCardParameter::Index(index as u32));
                                    }
                                }
                                Key::Property(JSContactProperty::MediaType) => {
                                    if let Some(text) = value.into_string() {
                                        entry.params.push(VCardParameter::Mediatype(text));
                                    }
                                }
                                Key::Property(JSContactProperty::Label) => {
                                    if let Some(text) = value.into_string() {
                                        entry.values.push(VCardValue::Text(text));
                                    }
                                }
                                Key::Property(JSContactProperty::Contexts) => {
                                    if let Some(types) = convert_types(value) {
                                        entry.params.push(VCardParameter::Type(types));
                                    }
                                }
                                _ => {}
                            }
                        }

                        if !entry.values.is_empty() {
                            if let Some(value_type) = value_type {
                                entry.params.push(VCardParameter::Value(vec![value_type]));
                            }
                            vcard.entries.push(entry);
                        }
                    }
                }
                JSContactProperty::Anniversaries => {
                    for (name, value) in value.into_expanded_object() {
                        let vcard_properties = value
                            .as_object()
                            .and_then(|obj| obj.get(&Key::Property(JSContactProperty::Kind)))
                            .and_then(|v| match v {
                                Value::Element(JSContactValue::Kind(JSContactKind::Birth)) => {
                                    Some((VCardProperty::Bday, Some(VCardProperty::Birthplace)))
                                }
                                Value::Element(JSContactValue::Kind(JSContactKind::Death)) => Some(
                                    (VCardProperty::Deathdate, Some(VCardProperty::Deathplace)),
                                ),
                                Value::Element(JSContactValue::Kind(JSContactKind::Wedding)) => {
                                    Some((VCardProperty::Anniversary, None))
                                }
                                _ => None,
                            });

                        if let Some((vcard_property, mut vcard_place_property)) = vcard_properties {
                            let mut date = None;
                            let mut place = None;
                            let mut calscale = None;

                            for (prop, value) in value.into_expanded_object() {
                                match prop {
                                    Key::Property(JSContactProperty::Kind) => {}
                                    Key::Property(JSContactProperty::Date) => {
                                        if let Some((date_, calscale_)) = convert_anniversary(value)
                                        {
                                            date = Some(date_);
                                            calscale = calscale_;
                                        }
                                    }
                                    Key::Property(JSContactProperty::Place)
                                        if vcard_place_property.is_some()
                                            && value.is_object_and_contains_any_key(&[
                                                Key::Property(JSContactProperty::Full),
                                                Key::Property(JSContactProperty::Coordinates),
                                            ]) =>
                                    {
                                        for (prop, value) in value.into_expanded_object() {
                                            match prop {
                                                Key::Property(JSContactProperty::Full)
                                                    if vcard_place_property.is_some() =>
                                                {
                                                    if let Some(text) = value.into_string() {
                                                        place = Some(
                                                            VCardEntry::new(
                                                                vcard_place_property
                                                                    .take()
                                                                    .unwrap(),
                                                            )
                                                            .with_value(text),
                                                        );
                                                    }
                                                }
                                                Key::Property(JSContactProperty::Coordinates)
                                                    if vcard_place_property.is_some() =>
                                                {
                                                    if let Some(text) = value.into_string() {
                                                        place = Some(
                                                            VCardEntry::new(
                                                                vcard_place_property
                                                                    .take()
                                                                    .unwrap(),
                                                            )
                                                            .with_value(text)
                                                            .with_param(VCardParameter::Value(
                                                                vec![VCardValueType::Uri],
                                                            )),
                                                        );
                                                    }
                                                }
                                                _ => {
                                                    let todo = "map unsupported vCard properties";
                                                }
                                            }
                                        }
                                    }
                                    _ => {
                                        let todo = "map unsupported vCard properties";
                                    }
                                }
                            }

                            // Add place if it exists
                            if let Some(place) = place {
                                vcard.entries.push(place.with_param(VCardParameter::PropId(
                                    name.to_string().into_owned(),
                                )));
                            }

                            // Add date
                            if let Some(date) = date {
                                let mut entry = VCardEntry::new(vcard_property);
                                if let Some(calscale) = calscale {
                                    entry.params.push(VCardParameter::Calscale(calscale));
                                }
                                vcard.entries.push(
                                    entry
                                        .with_param(VCardParameter::PropId(name.into_string()))
                                        .with_value(VCardValue::PartialDateTime(date)),
                                );
                            }
                        } else {
                            let todo = "map unsupported vCard properties";
                        }
                    }
                }
                JSContactProperty::Name => {
                    let mut params = Vec::new();
                    let mut parts: Vec<Option<String>> = vec![None; 7];
                    let mut num_parts = 0;

                    for (key, value) in value.into_expanded_object() {
                        match key {
                            Key::Property(JSContactProperty::Components) => {
                                for components in value.into_array().unwrap_or_default() {
                                    let mut comp_value = None;
                                    let mut comp_pos = None;

                                    for (key, value) in components.into_expanded_object() {
                                        match key {
                                            Key::Property(JSContactProperty::Kind) => {
                                                comp_pos = match value {
                                                    Value::Element(JSContactValue::Kind(
                                                        JSContactKind::Surname,
                                                    )) => 0,
                                                    Value::Element(JSContactValue::Kind(
                                                        JSContactKind::Given,
                                                    )) => 1,
                                                    Value::Element(JSContactValue::Kind(
                                                        JSContactKind::Given2,
                                                    )) => 2,
                                                    Value::Element(JSContactValue::Kind(
                                                        JSContactKind::Title,
                                                    )) => 3,
                                                    Value::Element(JSContactValue::Kind(
                                                        JSContactKind::Credential,
                                                    )) => 4,
                                                    Value::Element(JSContactValue::Kind(
                                                        JSContactKind::Surname2,
                                                    )) => 5,
                                                    Value::Element(JSContactValue::Kind(
                                                        JSContactKind::Generation,
                                                    )) => 6,
                                                    _ => continue,
                                                }
                                                .into();
                                            }
                                            Key::Property(JSContactProperty::Value) => {
                                                comp_value = value.into_string();
                                            }
                                            _ => {
                                                let todo = "map unsupported vCard properties";
                                            }
                                        }
                                    }

                                    if let (Some(comp_value), Some(comp_pos)) =
                                        (comp_value, comp_pos)
                                    {
                                        let part = &mut parts[comp_pos];
                                        if let Some(part) = part {
                                            *part = format!("{part},{comp_value}");
                                        } else {
                                            *part = Some(comp_value);
                                        }
                                        num_parts = num_parts.max(comp_pos + 1);
                                    }
                                }
                            }
                            Key::Property(JSContactProperty::Full) => {
                                if let Some(text) = value.into_string() {
                                    vcard
                                        .entries
                                        .push(VCardEntry::new(VCardProperty::Fn).with_value(text));
                                }
                            }
                            Key::Property(JSContactProperty::SortAs) => {
                                let mut surname = None;
                                let mut given = None;

                                for (key, value) in value.into_expanded_object() {
                                    match key {
                                        Key::Property(JSContactProperty::SortAsKind(
                                            JSContactKind::Surname,
                                        )) => {
                                            surname = value.into_string();
                                        }
                                        Key::Property(JSContactProperty::SortAsKind(
                                            JSContactKind::Given,
                                        )) => {
                                            given = value.into_string();
                                        }
                                        _ => {
                                            let todo = "map unsupported vCard properties";
                                        }
                                    }
                                }

                                if let (Some(surname), Some(given)) =
                                    (surname.as_ref(), given.as_ref())
                                {
                                    params.push(VCardParameter::SortAs(format!(
                                        "{surname}, {given}"
                                    )));
                                } else if let Some(surname) = surname.or(given) {
                                    params.push(VCardParameter::SortAs(surname));
                                }
                            }
                            Key::Property(JSContactProperty::PhoneticSystem) => {
                                if let Ok(phonetic_system) = VCardPhonetic::try_from(value) {
                                    params.push(VCardParameter::Phonetic(phonetic_system));
                                }
                            }
                            Key::Property(JSContactProperty::PhoneticScript) => {
                                if let Some(phonetic_script) = value.into_string() {
                                    params.push(VCardParameter::Script(phonetic_script));
                                }
                            }
                            _ => {
                                let todo = "map unsupported vCard properties";
                            }
                        }
                    }

                    if num_parts > 0 {
                        let mut n = String::with_capacity(16);
                        for (pos, part) in parts.into_iter().take(num_parts).enumerate() {
                            if pos > 0 {
                                n.push(';');
                            }
                            if let Some(part) = part {
                                n.push_str(&part);
                            }
                        }

                        vcard.entries.push(
                            VCardEntry::new(VCardProperty::N)
                                .with_params(params)
                                .with_value(n),
                        );
                    }
                }
                JSContactProperty::SpeakToAs => {
                    for (key, value) in value.into_expanded_object() {
                        match key {
                            Key::Property(JSContactProperty::GrammaticalGender) => {
                                if let Some(gender) = convert_value(value) {
                                    vcard.entries.push(
                                        VCardEntry::new(VCardProperty::Gramgender)
                                            .with_value(gender),
                                    );
                                }
                            }
                            Key::Property(JSContactProperty::Pronouns) => {
                                for (name, value) in value.into_expanded_object() {
                                    let mut entry = VCardEntry::new(VCardProperty::Pronouns)
                                        .with_param(VCardParameter::PropId(name.into_string()));

                                    for (prop, value) in value.into_expanded_object() {
                                        match prop {
                                            Key::Property(JSContactProperty::Pronouns) => {
                                                if let Some(pronoun_type) = convert_value(value) {
                                                    entry.values = vec![pronoun_type];
                                                }
                                            }
                                            Key::Property(JSContactProperty::Pref) => {
                                                if let Some(pref) = value.as_i64() {
                                                    entry.add_param(VCardParameter::Pref(
                                                        pref as u32,
                                                    ));
                                                }
                                            }
                                            _ => {
                                                let todo = "map unsupported vCard properties";
                                            }
                                        }
                                    }

                                    if !entry.values.is_empty() {
                                        vcard.entries.push(entry);
                                    }
                                }
                            }
                            _ => {
                                let todo = "map unsupported vCard properties";
                            }
                        }
                    }
                }
                JSContactProperty::Nicknames => {
                    for (name, value) in value.into_expanded_object() {
                        let mut entry = VCardEntry::new(VCardProperty::Nickname)
                            .with_param(VCardParameter::PropId(name.into_string()));

                        for (prop, value) in value.into_expanded_object() {
                            match prop {
                                Key::Property(JSContactProperty::Name) => {
                                    if let Some(name) = convert_value(value) {
                                        entry.values = vec![name];
                                    }
                                }
                                Key::Property(JSContactProperty::Pref) => {
                                    if let Some(pref) = value.as_i64() {
                                        entry.add_param(VCardParameter::Pref(pref as u32));
                                    }
                                }
                                Key::Property(JSContactProperty::Contexts) => {
                                    if let Some(types) = convert_types(value) {
                                        entry.params.push(VCardParameter::Type(types));
                                    }
                                }
                                _ => {
                                    let todo = "map unsupported vCard properties";
                                }
                            }
                        }

                        if !entry.values.is_empty() {
                            vcard.entries.push(entry);
                        }
                    }
                }
                JSContactProperty::Media => {
                    for (name, value) in value.into_expanded_object() {
                        let entry_type = value
                            .as_object()
                            .and_then(|obj| obj.get(&Key::Property(JSContactProperty::Kind)))
                            .and_then(|v| match v {
                                Value::Element(JSContactValue::Kind(JSContactKind::Photo)) => {
                                    Some(VCardProperty::Photo)
                                }
                                Value::Element(JSContactValue::Kind(JSContactKind::Logo)) => {
                                    Some(VCardProperty::Logo)
                                }
                                Value::Element(JSContactValue::Kind(JSContactKind::Sound)) => {
                                    Some(VCardProperty::Sound)
                                }
                                _ => None,
                            });

                        if let Some(entry_type) = entry_type {
                            let mut entry = VCardEntry::new(entry_type)
                                .with_param(VCardParameter::PropId(name.into_string()));

                            for (prop, value) in value.into_expanded_object() {
                                match prop {
                                    Key::Property(JSContactProperty::Uri) => {
                                        if let Some(uri) = value.into_string() {
                                            let uri = match Data::try_parse(uri.as_bytes()) {
                                                Some(data) => VCardValue::Binary(data),
                                                None => VCardValue::Text(uri),
                                            };

                                            entry.values = vec![uri];
                                        }
                                    }
                                    Key::Property(JSContactProperty::MediaType) => {
                                        if let Some(text) = value.into_string() {
                                            entry.params.push(VCardParameter::Mediatype(text));
                                        }
                                    }
                                    Key::Property(JSContactProperty::Label) => {
                                        if let Some(text) = value.into_string() {
                                            entry.params.push(VCardParameter::Label(text));
                                        }
                                    }
                                    Key::Property(JSContactProperty::Pref) => {
                                        if let Some(pref) = value.as_i64() {
                                            entry.add_param(VCardParameter::Pref(pref as u32));
                                        }
                                    }
                                    Key::Property(JSContactProperty::Kind) => {}
                                    _ => {
                                        let todo = "map unsupported vCard properties";
                                    }
                                }
                            }

                            if !entry.values.is_empty() {
                                vcard.entries.push(entry);
                            }
                        } else {
                            let todo = "map unsupported vCard properties";
                        }
                    }
                }
                JSContactProperty::Addresses => {
                    for (name, value) in value.into_expanded_object() {
                        let mut params = vec![VCardParameter::PropId(name.into_string())];
                        let mut parts: Vec<Option<String>> = vec![None; 18];
                        let mut num_parts = 0;

                        for (prop, value) in value.into_expanded_object() {
                            match prop {
                                Key::Property(JSContactProperty::Components) => {
                                    for components in value.into_array().unwrap_or_default() {
                                        let mut comp_value = None;
                                        let mut comp_pos = None;

                                        for (key, value) in components.into_expanded_object() {
                                            match key {
                                                Key::Property(JSContactProperty::Kind) => {
                                                    comp_pos = match value {
                                                        Value::Element(JSContactValue::Kind(
                                                            JSContactKind::PostOfficeBox,
                                                        )) => 0,
                                                        Value::Element(JSContactValue::Kind(
                                                            JSContactKind::Apartment,
                                                        )) => 1,
                                                        Value::Element(JSContactValue::Kind(
                                                            JSContactKind::Name,
                                                        )) => 2,
                                                        Value::Element(JSContactValue::Kind(
                                                            JSContactKind::Locality,
                                                        )) => 3,
                                                        Value::Element(JSContactValue::Kind(
                                                            JSContactKind::Region,
                                                        )) => 4,
                                                        Value::Element(JSContactValue::Kind(
                                                            JSContactKind::Postcode,
                                                        )) => 5,
                                                        Value::Element(JSContactValue::Kind(
                                                            JSContactKind::Country,
                                                        )) => 6,
                                                        Value::Element(JSContactValue::Kind(
                                                            JSContactKind::Room,
                                                        )) => 7,
                                                        Value::Element(JSContactValue::Kind(
                                                            JSContactKind::Floor,
                                                        )) => 9,
                                                        Value::Element(JSContactValue::Kind(
                                                            JSContactKind::Number,
                                                        )) => 10,
                                                        Value::Element(JSContactValue::Kind(
                                                            JSContactKind::Building,
                                                        )) => 12,
                                                        Value::Element(JSContactValue::Kind(
                                                            JSContactKind::Block,
                                                        )) => 13,
                                                        Value::Element(JSContactValue::Kind(
                                                            JSContactKind::Subdistrict,
                                                        )) => 14,
                                                        Value::Element(JSContactValue::Kind(
                                                            JSContactKind::District,
                                                        )) => 15,
                                                        Value::Element(JSContactValue::Kind(
                                                            JSContactKind::Landmark,
                                                        )) => 16,
                                                        Value::Element(JSContactValue::Kind(
                                                            JSContactKind::Direction,
                                                        )) => 17,
                                                        _ => continue,
                                                    }
                                                    .into();
                                                }
                                                Key::Property(JSContactProperty::Value) => {
                                                    comp_value = value.into_string();
                                                }
                                                _ => {
                                                    let todo = "map unsupported vCard properties";
                                                }
                                            }
                                        }

                                        if let (Some(comp_value), Some(comp_pos)) =
                                            (comp_value, comp_pos)
                                        {
                                            let part = &mut parts[comp_pos];
                                            if let Some(part) = part {
                                                *part = format!("{part},{comp_value}");
                                            } else {
                                                *part = Some(comp_value);
                                            }
                                            num_parts = num_parts.max(comp_pos + 1);
                                        }
                                    }
                                }
                                Key::Property(JSContactProperty::Label) => {
                                    if let Some(value) = value.into_string() {
                                        params.push(VCardParameter::Label(value));
                                    }
                                }
                                Key::Property(JSContactProperty::Coordinates) => {
                                    if let Some(value) = value.into_string() {
                                        params.push(VCardParameter::Geo(value));
                                    }
                                }
                                Key::Property(JSContactProperty::TimeZone) => {
                                    if let Some(value) = value.into_string() {
                                        params.push(VCardParameter::Tz(value));
                                    }
                                }
                                Key::Property(JSContactProperty::CountryCode) => {
                                    if let Some(value) = value.into_string() {
                                        params.push(VCardParameter::Cc(value));
                                    }
                                }
                                Key::Property(JSContactProperty::Pref) => {
                                    if let Some(value) = value.as_i64() {
                                        params.push(VCardParameter::Pref(value as u32));
                                    }
                                }
                                Key::Property(JSContactProperty::PhoneticSystem) => {
                                    if let Ok(phonetic_system) = VCardPhonetic::try_from(value) {
                                        params.push(VCardParameter::Phonetic(phonetic_system));
                                    }
                                }
                                Key::Property(JSContactProperty::PhoneticScript) => {
                                    if let Some(value) = value.into_string() {
                                        params.push(VCardParameter::Script(value));
                                    }
                                }
                                Key::Property(JSContactProperty::Contexts) => {
                                    if let Some(types) = convert_types(value) {
                                        params.push(VCardParameter::Type(types));
                                    }
                                }
                                _ => {
                                    let todo = "map unsupported vCard properties";
                                }
                            }
                        }

                        let mut n = String::with_capacity(16);
                        for (pos, part) in parts.into_iter().take(num_parts).enumerate() {
                            if pos > 0 {
                                n.push(';');
                            }
                            if let Some(part) = part {
                                n.push_str(&part);
                            }
                        }

                        vcard.entries.push(
                            VCardEntry::new(VCardProperty::Adr)
                                .with_params(params)
                                .with_value(n),
                        );
                    }
                }

                JSContactProperty::Language => todo!(),
                JSContactProperty::Members => todo!(),
                JSContactProperty::ProdId => todo!(),
                JSContactProperty::Created => todo!(),
                JSContactProperty::Updated => todo!(),
                JSContactProperty::Organizations => todo!(),
                JSContactProperty::Titles => todo!(),
                JSContactProperty::Emails => todo!(),
                JSContactProperty::OnlineServices => todo!(),
                JSContactProperty::Phones => todo!(),
                JSContactProperty::PreferredLanguages => todo!(),
                JSContactProperty::Calendars => todo!(),
                JSContactProperty::SchedulingAddresses => todo!(),
                JSContactProperty::Links => todo!(),
                JSContactProperty::Localizations => todo!(),
                JSContactProperty::Keywords => todo!(),
                JSContactProperty::Notes => todo!(),
                JSContactProperty::PersonalInfo => todo!(),
                JSContactProperty::RelatedTo => todo!(),
                JSContactProperty::VCardName => todo!(),
                JSContactProperty::VCardParams => todo!(),
                JSContactProperty::VCardProps => todo!(),
                _ => (),
            }
        }

        Some(vcard)
    }
}

pub fn convert_anniversary(
    value: Value<'_, JSContactProperty, JSContactValue>,
) -> Option<(PartialDateTime, Option<CalendarScale>)> {
    let mut date = PartialDateTime::default();
    let mut calendar_scale = None;

    for (key, value) in value.into_object()?.into_vec() {
        match key {
            Key::Property(JSContactProperty::Day) => {
                if let Value::Number(day) = value {
                    date.day = Some(day.cast_to_i64() as u8);
                }
            }
            Key::Property(JSContactProperty::Month) => {
                if let Value::Number(month) = value {
                    date.month = Some(month.cast_to_i64() as u8);
                }
            }
            Key::Property(JSContactProperty::Year) => {
                if let Value::Number(year) = value {
                    date.year = Some(year.cast_to_i64() as u16);
                }
            }
            Key::Property(JSContactProperty::CalendarScale) => {
                if let Value::Element(JSContactValue::CalendarScale(scale)) = value {
                    calendar_scale = Some(scale);
                }
            }
            Key::Property(JSContactProperty::Utc) => {
                if let Value::Element(JSContactValue::Timestamp(timestamp)) = value {
                    return Some((PartialDateTime::from_utc_timestamp(timestamp), None));
                }
            }
            _ => {}
        }
    }

    if date.year.is_some() || date.month.is_some() || date.day.is_some() {
        Some((date, calendar_scale))
    } else {
        None
    }
}

fn convert_value(value: Value<'_, JSContactProperty, JSContactValue>) -> Option<VCardValue> {
    match value {
        Value::Element(e) => match e {
            JSContactValue::Timestamp(t) => {
                VCardValue::PartialDateTime(PartialDateTime::from_utc_timestamp(t)).into()
            }
            JSContactValue::GrammaticalGender(g) => VCardValue::GramGender(match g {
                JSContactGrammaticalGender::Animate => VCardGramGender::Animate,
                JSContactGrammaticalGender::Common => VCardGramGender::Common,
                JSContactGrammaticalGender::Feminine => VCardGramGender::Feminine,
                JSContactGrammaticalGender::Inanimate => VCardGramGender::Inanimate,
                JSContactGrammaticalGender::Masculine => VCardGramGender::Masculine,
                JSContactGrammaticalGender::Neuter => VCardGramGender::Neuter,
            })
            .into(),
            JSContactValue::Kind(k) => match k {
                JSContactKind::Individual => VCardValue::Kind(VCardKind::Individual).into(),
                JSContactKind::Group => VCardValue::Kind(VCardKind::Group).into(),
                JSContactKind::Location => VCardValue::Kind(VCardKind::Location).into(),
                JSContactKind::Org => VCardValue::Kind(VCardKind::Org).into(),
                JSContactKind::Application => VCardValue::Kind(VCardKind::Application).into(),
                JSContactKind::Device => VCardValue::Kind(VCardKind::Device).into(),
                _ => None,
            },
            JSContactValue::Level(_)
            | JSContactValue::Type(_)
            | JSContactValue::Relation(_)
            | JSContactValue::PhoneticSystem(_)
            | JSContactValue::CalendarScale(_) => None,
        },
        Value::Str(s) => VCardValue::Text(s.into_owned()).into(),
        Value::Bool(b) => VCardValue::Boolean(b).into(),
        Value::Number(n) => match n.try_cast_to_i64() {
            Ok(i) => VCardValue::Integer(i),
            Err(f) => VCardValue::Float(f),
        }
        .into(),
        _ => None,
    }
}

fn convert_types(value: Value<'_, JSContactProperty, JSContactValue>) -> Option<Vec<VCardType>> {
    let mut types = Vec::new();
    for (typ, set) in value.into_expanded_boolean_set() {
        if set {
            let typ = typ.to_string();
            match VCardType::try_from(typ.as_ref().as_bytes()) {
                Ok(typ) => types.push(typ),
                Err(_) => {
                    types.push(VCardType::Other(typ.into_owned()));
                }
            }
        }
    }
    if !types.is_empty() {
        Some(types)
    } else {
        None
    }
}

impl TryFrom<Value<'_, JSContactProperty, JSContactValue>> for VCardPhonetic {
    type Error = ();

    fn try_from(value: Value<'_, JSContactProperty, JSContactValue>) -> Result<Self, Self::Error> {
        match value {
            Value::Element(JSContactValue::PhoneticSystem(system)) => Ok(match system {
                JSContactPhoneticSystem::Ipa => VCardPhonetic::Ipa,
                JSContactPhoneticSystem::Jyut => VCardPhonetic::Jyut,
                JSContactPhoneticSystem::Piny => VCardPhonetic::Piny,
                JSContactPhoneticSystem::Script => VCardPhonetic::Script,
            }),
            Value::Str(text) => Ok(VCardPhonetic::Other(text.into_owned())),
            _ => Err(()),
        }
    }
}
