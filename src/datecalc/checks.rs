/*
 * SPDX-FileCopyrightText: 2021 Fredrik Meringdal, Ralph Bisschops <https://github.com/fmeringdal/rust-rrule>
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

use super::{error::ValidationError, validate::YEAR_RANGE};

pub(crate) fn check_year_range(year: i32) -> Result<(), ValidationError> {
    if YEAR_RANGE.contains(&year) {
        Ok(())
    } else {
        Err(ValidationError::InvalidFieldValueRange {
            field: "YEAR".into(),
            value: year.to_string(),
            start_idx: YEAR_RANGE.start().to_string(),
            end_idx: YEAR_RANGE.end().to_string(),
        })
    }
}
