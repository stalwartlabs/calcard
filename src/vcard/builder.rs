/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

use crate::vcard::{VCardEntry, VCardKind, VCardParameter, VCardProperty, VCardValue};

impl VCardEntry {
    pub fn new(name: VCardProperty) -> Self {
        Self {
            group: None,
            name,
            params: vec![],
            values: vec![],
        }
    }

    pub fn with_group(mut self, group: Option<String>) -> Self {
        self.group = group;
        self
    }

    pub fn with_value(mut self, value: impl Into<VCardValue>) -> Self {
        self.values.push(value.into());
        self
    }

    pub fn with_param(mut self, param: impl Into<VCardParameter>) -> Self {
        self.params.push(param.into());
        self
    }
}

impl From<String> for VCardValue {
    fn from(text: String) -> Self {
        VCardValue::Text(text)
    }
}

impl From<VCardKind> for VCardValue {
    fn from(kind: VCardKind) -> Self {
        VCardValue::Kind(kind)
    }
}
