/*
 * SPDX-FileCopyrightText: 2021 Fredrik Meringdal, Ralph Bisschops <https://github.com/fmeringdal/rust-rrule>
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

use super::masks::MASKS;
use super::utils::is_leap_year;

/// Implementation of the Gauss Easter Algorithm.
#[allow(clippy::many_single_char_names)]
pub(crate) fn easter(year: i32, offset: i16) -> i32 {
    let a = year % 19;
    let b = year / 100;
    let c = year % 100;
    let d = b / 4;
    let e = b % 4;
    let f = (b + 8) / 25;
    let g = (b - f + 1) / 3;
    let h = (19 * a + b - d - g + 15) % 30;
    let i = c / 4;
    let k = c % 4;
    let l = (32 + 2 * e + 2 * i - h - k) % 7;
    let m = (a + 11 * h + 22 * l) / 451;
    let month = ((h + l - 7 * m + 114) / 31) as usize;
    let day = ((h + l - 7 * m + 114) % 31) + 1 + i32::from(offset);

    let month_range_mask = if is_leap_year(year) {
        &MASKS.month_366_range
    } else {
        &MASKS.month_365_range
    };

    i32::from(month_range_mask[month - 1]) + day - 1
}
