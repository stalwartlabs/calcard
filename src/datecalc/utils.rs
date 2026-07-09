/*
 * SPDX-FileCopyrightText: 2021 Fredrik Meringdal, Ralph Bisschops <https://github.com/fmeringdal/rust-rrule>
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

use super::duration_from_midnight;
use crate::common::timezone::Tz;
use chrono::{NaiveDate, NaiveTime, Utc};
use std::ops;

const DAY_SECS: i64 = 24 * 60 * 60;

#[inline(always)]
pub(crate) fn date_from_ordinal(ordinal: i64) -> NaiveDate {
    chrono::DateTime::<Utc>::from_timestamp(ordinal * DAY_SECS, 0)
        .unwrap_or_default()
        .date_naive()
}

#[inline(always)]
pub(crate) fn days_since_unix_epoch(date: &chrono::DateTime<Utc>) -> i64 {
    date.timestamp() / DAY_SECS
}

#[inline(always)]
pub(crate) fn is_leap_year(year: i32) -> bool {
    year.trailing_zeros() >= 2 && (year % 25 != 0 || year.trailing_zeros() >= 4)
}

#[inline(always)]
pub(crate) fn get_year_len(year: i32) -> u16 {
    if is_leap_year(year) { 366 } else { 365 }
}

pub(crate) trait DifferentSigns {
    fn different_sign(a: Self, b: Self) -> bool;
}

macro_rules! impl_different_signs {
    ($($ty:ty),*) => {
        $(
        impl DifferentSigns for $ty {
            fn different_sign(a: Self, b: Self) -> bool {
                a > 0 && b < 0 || a < 0 && b > 0
            }
        })*
    };
    (@unsigned, $($ty:ty),*) => {
        $(
        impl DifferentSigns for $ty {
            fn different_sign(_: Self, _: Self) -> bool {
                false
            }
        })*
    };
}
impl_different_signs!(isize, i32, i16);
impl_different_signs!(@unsigned, usize, u32, u16, u8);

pub(crate) fn pymod<T>(a: T, b: T) -> T
where
    T: DifferentSigns + Copy + ops::Rem<Output = T> + ops::Add<Output = T>,
{
    let r = a % b;
    // If r and b differ in sign, add b to wrap the result to the correct sign.
    if T::different_sign(r, b) { r + b } else { r }
}

pub(crate) fn add_time_to_date(
    tz: Tz,
    date: NaiveDate,
    time: NaiveTime,
) -> Option<chrono::DateTime<Tz>> {
    if let Some(dt) = date.and_time(time).and_local_timezone(tz).single() {
        return Some(dt);
    }
    // If the day is a daylight saving time, the above code might not work, and we
    // can try to get a valid datetime by adding the `time` as a duration instead.
    let dt = date.and_hms_opt(0, 0, 0)?.and_local_timezone(tz).single()?;
    let day_duration = duration_from_midnight(time);
    dt.checked_add_signed(day_duration)
}
