use crate::common::timezone::Tz;

use super::utils::{add_time_to_date, date_from_ordinal, pymod};
use chrono::NaiveTime;

pub(crate) fn build_pos_list(
    by_set_pos: &[i32],
    dayset: &[usize],
    timeset: &[NaiveTime],
    year_ordinal: i64,
    tz: Tz,
) -> Vec<chrono::DateTime<Tz>> {
    let mut pos_list = vec![];

    if timeset.is_empty() {
        return vec![];
    }

    let timeset_len = (timeset.len()) as u32;
    let timeset_len_float = f64::from(timeset_len);
    let timeset_len_int = timeset_len as i32;
    for pos in by_set_pos {
        let pos = if *pos > 0 { pos - 1 } else { *pos };
        let day_pos = (f64::from(pos) / timeset_len_float).floor() as isize;
        let time_pos = (pymod(pos, timeset_len_int)) as usize;

        let day_idx = if day_pos < 0 {
            let dayset_len = dayset.len() as isize;
            let index = dayset_len + day_pos;
            match usize::try_from(index) {
                Ok(day_idx) => day_idx,
                Err(_) => continue,
            }
        } else {
            (day_pos) as usize
        };
        let day = match dayset.get(day_idx) {
            Some(day) => day,
            None => continue,
        };
        let day = *day as i64;

        // Get ordinal which is UTC
        let date = date_from_ordinal(year_ordinal + day);
        // Create new Date + Time combination
        // Use Time from `timeset`.
        let time = timeset[time_pos];
        let res = match add_time_to_date(tz, date, time) {
            Some(date) => date,
            None => continue,
        };

        if !pos_list.contains(&res) {
            pos_list.push(res);
        }
    }

    pos_list.sort();

    pos_list
}
