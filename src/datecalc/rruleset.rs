use super::{rrule::RRule, timezone::Tz, utils::collect_with_error};
use chrono::DateTime;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RRuleSet {
    pub(crate) rrule: Vec<RRule>,
    pub(crate) rdate: Vec<DateTime<Tz>>,
    pub(crate) exrule: Vec<RRule>,
    pub(crate) exdate: Vec<DateTime<Tz>>,
    pub(crate) dt_start: DateTime<Tz>,
    pub(crate) before: Option<DateTime<Tz>>,
    pub(crate) after: Option<DateTime<Tz>>,
    pub(crate) limited: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RRuleResult {
    pub dates: Vec<DateTime<Tz>>,
    pub limited: bool,
}

impl RRuleSet {
    pub fn new(dt_start: DateTime<Tz>) -> Self {
        Self {
            dt_start,
            rrule: vec![],
            rdate: vec![],
            exrule: vec![],
            exdate: vec![],
            before: None,
            after: None,
            limited: false,
        }
    }

    pub fn limit(mut self) -> Self {
        self.limited = true;
        self
    }

    pub fn before(mut self, dt: DateTime<Tz>) -> Self {
        self.before = Some(dt);
        self
    }

    pub fn after(mut self, dt: DateTime<Tz>) -> Self {
        self.after = Some(dt);
        self
    }

    pub fn rrule(mut self, rrule: RRule) -> Self {
        self.rrule.push(rrule);
        self
    }

    pub fn exrule(mut self, rrule: RRule) -> Self {
        self.exrule.push(rrule);
        self
    }

    pub fn rdate(mut self, rdate: DateTime<Tz>) -> Self {
        self.rdate.push(rdate);
        self
    }

    pub fn exdate(mut self, exdate: DateTime<Tz>) -> Self {
        self.exdate.push(exdate);
        self
    }

    pub fn set_rrules(mut self, rrules: Vec<RRule>) -> Self {
        self.rrule = rrules;
        self
    }

    pub fn set_exrules(mut self, exrules: Vec<RRule>) -> Self {
        self.exrule = exrules;
        self
    }

    pub fn set_rdates(mut self, rdates: Vec<DateTime<Tz>>) -> Self {
        self.rdate = rdates;
        self
    }

    pub fn set_exdates(mut self, exdates: Vec<DateTime<Tz>>) -> Self {
        self.exdate = exdates;
        self
    }

    pub fn get_rrule(&self) -> &Vec<RRule> {
        &self.rrule
    }

    pub fn get_exrule(&self) -> &Vec<RRule> {
        &self.exrule
    }

    pub fn get_rdate(&self) -> &Vec<DateTime<Tz>> {
        &self.rdate
    }

    pub fn get_exdate(&self) -> &Vec<DateTime<Tz>> {
        &self.exdate
    }

    pub fn get_dt_start(&self) -> &DateTime<Tz> {
        &self.dt_start
    }

    pub fn all(mut self, limit: u16) -> RRuleResult {
        self.limited = true;
        collect_with_error(
            self.into_iter(),
            &self.after,
            &self.before,
            true,
            Some(limit),
        )
    }

    pub fn all_unchecked(self) -> Vec<DateTime<Tz>> {
        collect_with_error(self.into_iter(), &self.after, &self.before, true, None).dates
    }
}
