use chrono::NaiveDate;
use getset::Getters;

use crate::{
    features::plans::domain::{AlertLevel, AlertType, PlanError, PlanText},
    shared::DomainError,
};

/// A warning on one day of the plan. Whether that day lies inside the plan
/// is the plan's rule, not the alert's.
#[derive(Clone, Debug, PartialEq, Getters)]
#[getset(get = "pub")]
pub struct PlanAlert {
    alert_type: AlertType,
    day: NaiveDate,
    /// The number behind the alert, for example the night's lowest
    /// temperature. None when the alert has no single number.
    value: Option<f64>,
    level: AlertLevel,
    ku: PlanText,
    en: PlanText,
}

impl PlanAlert {
    pub fn new(
        alert_type: AlertType,
        day: NaiveDate,
        value: Option<f64>,
        level: AlertLevel,
        ku: PlanText,
        en: PlanText,
    ) -> Result<Self, PlanError> {
        if value.is_some_and(|value| !value.is_finite()) {
            return Err(DomainError::InvalidValue(
                "Alert value must be a finite number".to_string(),
            )
            .into());
        }

        Ok(Self {
            alert_type,
            day,
            value,
            level,
            ku,
            en,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text() -> PlanText {
        PlanText::new("Frost -3 °C Sun night".to_string()).expect("text")
    }

    fn alert(value: Option<f64>) -> Result<PlanAlert, PlanError> {
        PlanAlert::new(
            AlertType::Frost,
            NaiveDate::from_ymd_opt(2026, 10, 11).expect("date"),
            value,
            AlertLevel::Alarm,
            text(),
            text(),
        )
    }

    #[test]
    fn an_alert_keeps_its_number_as_it_was_pushed() {
        assert_eq!(*alert(Some(-3.2)).expect("alert").value(), Some(-3.2));
    }

    #[test]
    fn an_alert_may_have_no_number() {
        assert!(alert(None).expect("alert").value().is_none());
    }

    #[test]
    fn a_number_that_is_not_finite_is_refused() {
        assert!(alert(Some(f64::NAN)).is_err());
        assert!(alert(Some(f64::INFINITY)).is_err());
    }
}
