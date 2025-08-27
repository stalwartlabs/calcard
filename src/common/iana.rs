/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

use std::borrow::Cow;

use crate::common::{IanaString, IanaType};

impl<I, O> IanaType<I, O> {
    pub fn iana(&self) -> Option<&I> {
        match self {
            IanaType::Iana(iana) => Some(iana),
            _ => None,
        }
    }

    pub fn other(&self) -> Option<&O> {
        match self {
            IanaType::Other(s) => Some(s),
            _ => None,
        }
    }

    pub fn into_iana(self) -> Option<I> {
        match self {
            IanaType::Iana(iana) => Some(iana),
            _ => None,
        }
    }

    pub fn is_iana(&self) -> bool {
        matches!(self, IanaType::Iana(_))
    }

    pub fn is_iana_and(&self, f: impl FnOnce(&I) -> bool) -> bool {
        match &self {
            IanaType::Iana(x) => f(x),
            _ => false,
        }
    }
}

impl<I, O> AsRef<str> for IanaType<I, O>
where
    I: IanaString,
    O: AsRef<str>,
{
    fn as_ref(&self) -> &str {
        match self {
            IanaType::Iana(iana) => iana.as_str(),
            IanaType::Other(s) => s.as_ref(),
        }
    }
}

impl<I> IanaType<I, String>
where
    I: IanaString,
{
    pub fn into_string(self) -> Cow<'static, str> {
        match self {
            IanaType::Iana(iana) => Cow::Borrowed(iana.as_str()),
            IanaType::Other(s) => Cow::Owned(s),
        }
    }
}

impl<I> From<I> for IanaType<I, String>
where
    I: IanaString,
{
    fn from(iana: I) -> Self {
        IanaType::Iana(iana)
    }
}

impl<I> From<String> for IanaType<I, String> {
    fn from(s: String) -> Self {
        IanaType::Other(s)
    }
}

impl<I, O> Default for IanaType<I, O>
where
    I: IanaString
        + std::fmt::Debug
        + Clone
        + PartialEq
        + Eq
        + PartialOrd
        + Ord
        + std::hash::Hash
        + Default,
    O: std::fmt::Debug + Clone + PartialEq + Eq + PartialOrd + Ord + std::hash::Hash,
{
    fn default() -> Self {
        IanaType::Iana(I::default())
    }
}
