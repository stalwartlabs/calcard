/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

use crate::{
    jscontact::{JSContact, JSContactKind, JSContactProperty, JSContactType, JSContactValue},
    vcard::{VCard, VCardKind, VCardProperty, VCardValue},
};
use jmap_tools::{Key, Value};

impl VCard {
    pub fn into_jscontact(self) -> JSContact<'static> {
        let mut entries = Vec::with_capacity(self.entries.len());

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

        for entry in self.entries {
            match entry.name {
                VCardProperty::Uid => {
                    if let Some(VCardValue::Text(text)) = entry.values.into_iter().next() {
                        entries.push((
                            Key::Property(JSContactProperty::Uid),
                            Value::Str(text.into()),
                        ));
                    }
                }
                VCardProperty::Kind => match entry.values.into_iter().next() {
                    Some(VCardValue::Kind(kind)) => {
                        entries.push((
                            Key::Property(JSContactProperty::Kind),
                            Value::Element(JSContactValue::Kind(match kind {
                                VCardKind::Application => JSContactKind::Application,
                                VCardKind::Device => JSContactKind::Device,
                                VCardKind::Group => JSContactKind::Group,
                                VCardKind::Individual => JSContactKind::Individual,
                                VCardKind::Location => JSContactKind::Location,
                                VCardKind::Org => JSContactKind::Org,
                            })),
                        ));
                    }
                    Some(VCardValue::Text(text)) => {
                        entries.push((
                            Key::Property(JSContactProperty::Kind),
                            Value::Str(text.into()),
                        ));
                    }
                    _ => (),
                },

                VCardProperty::Source => todo!(),

                VCardProperty::Xml => todo!(),
                VCardProperty::Fn => todo!(),
                VCardProperty::N => todo!(),
                VCardProperty::Nickname => todo!(),
                VCardProperty::Photo => todo!(),
                VCardProperty::Bday => todo!(),
                VCardProperty::Anniversary => todo!(),
                VCardProperty::Gender => todo!(),
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
                VCardProperty::Birthplace => todo!(),
                VCardProperty::Deathplace => todo!(),
                VCardProperty::Deathdate => todo!(),
                VCardProperty::Expertise => todo!(),
                VCardProperty::Hobby => todo!(),
                VCardProperty::Interest => todo!(),
                VCardProperty::OrgDirectory => todo!(),
                VCardProperty::ContactUri => todo!(),
                VCardProperty::Created => todo!(),
                VCardProperty::Gramgender => todo!(),
                VCardProperty::Language => todo!(),
                VCardProperty::Pronouns => todo!(),
                VCardProperty::Socialprofile => todo!(),
                VCardProperty::Jsprop => todo!(),
                VCardProperty::Other(_) => todo!(),

                VCardProperty::Begin | VCardProperty::End => (),
            }
        }

        JSContact(Value::Object(entries.into()))
    }
}
