use thiserror::Error;

use crate::{
    app::{AppError as GlobalAppError, ErrorInfo, ErrorKind, ToErrorInfo},
    features::plans::domain::PlanError,
    shared::DomainError,
};

impl ToErrorInfo for PlanError {
    fn to_error_info(&self) -> ErrorInfo {
        let invalid = |code| ErrorInfo::with_code(ErrorKind::InvalidInput, code, self.to_string());

        match self {
            PlanError::DayCount { .. } => invalid("too_many_days"),
            PlanError::UnequalDays => invalid("unequal_days"),
            PlanError::AlertOutsidePlan(_) => invalid("alert_outside_plan"),
            PlanError::TooManyAlerts { .. } => invalid("too_many_alerts"),
            PlanError::DuplicateDecision(_) => invalid("duplicate_decision"),
            PlanError::IssuedInTheFuture => invalid("issued_in_future"),
            PlanError::StartsFarFromIssue => invalid("bad_from"),
            // Both are a 404 on purpose: the app answers a 404 with "10-day
            // plan coming soon" and drops the copy it kept, while a 5xx
            // makes it show that old copy again.
            PlanError::NotReady => {
                ErrorInfo::with_code(ErrorKind::NotFound, "plan_not_ready", self.to_string())
            }
            PlanError::Stale => {
                ErrorInfo::with_code(ErrorKind::NotFound, "plan_stale", self.to_string())
            }
            PlanError::DomainError(err) => err.to_error_info(),
        }
    }
}

#[derive(Error, Debug)]
pub enum AppError {
    #[error(transparent)]
    Plan(#[from] PlanError),

    #[error(transparent)]
    Domain(#[from] DomainError),

    #[error(transparent)]
    GlobalAppError(#[from] GlobalAppError),
}

impl ToErrorInfo for AppError {
    fn to_error_info(&self) -> ErrorInfo {
        match self {
            AppError::Plan(err) => err.to_error_info(),
            AppError::Domain(err) => err.to_error_info(),
            AppError::GlobalAppError(err) => err.to_error_info(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn more_than_ten_days_has_a_code_the_job_can_act_on() {
        let info = PlanError::DayCount { max: 10 }.to_error_info();

        assert_eq!(info.kind, ErrorKind::InvalidInput);
        assert_eq!(info.code, "too_many_days");
    }

    #[test]
    fn each_broken_rule_of_a_pushed_plan_has_its_own_code() {
        assert_eq!(PlanError::UnequalDays.to_error_info().code, "unequal_days");
        assert_eq!(
            PlanError::DuplicateDecision("sow_wait".to_string())
                .to_error_info()
                .code,
            "duplicate_decision"
        );
        assert_eq!(
            PlanError::IssuedInTheFuture.to_error_info().code,
            "issued_in_future"
        );
    }

    #[test]
    fn no_plan_yet_and_a_stale_plan_are_both_not_found_never_a_server_fault() {
        let not_ready = PlanError::NotReady.to_error_info();
        let stale = PlanError::Stale.to_error_info();

        assert_eq!(
            not_ready.kind,
            ErrorKind::NotFound,
            "the app shows its calm 'coming soon' line on a 404 only"
        );
        assert_eq!(not_ready.code, "plan_not_ready");
        assert_eq!(stale.kind, ErrorKind::NotFound);
        assert_eq!(stale.code, "plan_stale");
    }
}
