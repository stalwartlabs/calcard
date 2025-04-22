mod checks;
mod counter_date;
mod easter;
pub mod error;
pub(crate) mod filters;
pub(crate) mod iterinfo;
mod masks;
mod monthinfo;
mod pos_list;
pub mod rrule;
pub(crate) mod rrule_iter;
mod utils;
mod validate;
mod yearinfo;

use crate::common::timezone::Tz;
use chrono::{Datelike, Duration, NaiveTime, Timelike};
use iterinfo::IterInfo;
use pos_list::build_pos_list;
pub(crate) use rrule_iter::RRuleIter;

/// Prevent loops when searching for the next event in the iterator.
/// If after X number of iterations it still has not found an event,
/// we can assume it will not find an event.
static MAX_ITER_LOOP: u32 = 100_000;

#[inline(always)]
pub(crate) fn duration_from_midnight(time: NaiveTime) -> Duration {
    Duration::hours(i64::from(time.hour()))
        + Duration::minutes(i64::from(time.minute()))
        + Duration::seconds(i64::from(time.second()))
}

#[inline(always)]
pub(crate) fn get_month(dt: &chrono::DateTime<Tz>) -> u8 {
    dt.month() as u8
}

#[inline(always)]
pub(crate) fn get_day(dt: &chrono::DateTime<Tz>) -> i8 {
    dt.day() as i8
}

#[inline(always)]
pub(crate) fn get_hour(dt: &chrono::DateTime<Tz>) -> u8 {
    dt.hour() as u8
}

#[inline(always)]
pub(crate) fn get_minute(dt: &chrono::DateTime<Tz>) -> u8 {
    dt.minute() as u8
}

#[inline(always)]
pub(crate) fn get_second(dt: &chrono::DateTime<Tz>) -> u8 {
    dt.second() as u8
}
