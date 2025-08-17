/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

use crate::{
    jscontact::{JSContactProperty, JSContactValue},
    vcard::VCard,
};
use jmap_tools::{Key, Value};

pub mod convert;
pub mod entry;
pub mod params;
pub mod props;

#[derive(Default)]
#[allow(clippy::type_complexity)]
struct State<'x> {
    pub(super) vcard: VCard,
    pub(super) js_props: Vec<(String, Value<'x, JSContactProperty, JSContactValue>)>,
    pub(super) converted_props: Vec<(
        Vec<Key<'static, JSContactProperty>>,
        Value<'x, JSContactProperty, JSContactValue>,
    )>,
    pub(super) converted_props_count: usize,
    pub(super) language: Option<String>,
}
