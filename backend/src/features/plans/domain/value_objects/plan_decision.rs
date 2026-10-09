use getset::Getters;

use crate::features::plans::domain::{DecisionCode, PlanText};

/// One thing to do, or not to do, in the days the plan covers.
#[derive(Clone, Debug, PartialEq, Eq, Getters)]
#[getset(get = "pub")]
pub struct PlanDecision {
    code: DecisionCode,
    ku: PlanText,
    en: PlanText,
}

impl PlanDecision {
    pub fn new(code: DecisionCode, ku: PlanText, en: PlanText) -> Self {
        Self { code, ku, en }
    }
}
