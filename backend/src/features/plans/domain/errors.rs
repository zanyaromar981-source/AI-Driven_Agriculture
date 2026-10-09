use chrono::NaiveDate;

use crate::shared::DomainError;

#[derive(thiserror::Error, Debug)]
pub enum PlanError {
    #[error("A plan covers between 1 and {max} days, never more")]
    DayCount { max: usize },

    #[error("rain_mm, tmin and tmax must have one entry per day, the same number each")]
    UnequalDays,

    #[error("The alert for {0} is on a day the plan does not cover")]
    AlertOutsidePlan(NaiveDate),

    #[error("A plan carries at most {max} alerts")]
    TooManyAlerts { max: usize },

    #[error("The decision {0} appears more than once in the plan")]
    DuplicateDecision(String),

    #[error("A plan cannot be issued in the future")]
    IssuedInTheFuture,

    #[error("A plan starts on the day it was issued, give or take a day")]
    StartsFarFromIssue,

    #[error("The farm has no plan yet")]
    NotReady,

    #[error("The farm's plan is too old to show")]
    Stale,

    #[error(transparent)]
    DomainError(#[from] DomainError),
}
