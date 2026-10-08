use getset::Getters;

use crate::{
    features::insights::domain::{InsightError, MeasureCode},
    shared::DomainError,
};

const MAX_UNIT_LENGTH: usize = 20;
const MAX_LABEL_LENGTH: usize = 60;

/// One named number of a reading, with the unit and the words the app shows
/// next to it.
#[derive(Clone, Debug, PartialEq, Getters)]
#[getset(get = "pub")]
pub struct Measure {
    code: MeasureCode,
    value: f64,
    /// Empty for a number without a unit, such as an index.
    unit: String,
    label_en: String,
    label_ku: Option<String>,
}

impl Measure {
    pub fn new(
        code: MeasureCode,
        value: f64,
        unit: String,
        label_en: String,
        label_ku: Option<String>,
    ) -> Result<Self, InsightError> {
        if !value.is_finite() {
            return Err(invalid("Measure value must be a finite number".to_string()));
        }

        let unit = unit.trim().to_string();

        if unit.chars().count() > MAX_UNIT_LENGTH {
            return Err(invalid(format!(
                "Measure unit must be {MAX_UNIT_LENGTH} characters max"
            )));
        }

        Ok(Self {
            code,
            value,
            unit,
            label_en: label(label_en)?,
            label_ku: label_ku.map(label).transpose()?,
        })
    }
}

fn label(value: String) -> Result<String, InsightError> {
    let value = value.trim().to_string();

    if value.is_empty() || value.chars().count() > MAX_LABEL_LENGTH {
        return Err(invalid(format!(
            "Measure label must be 1 to {MAX_LABEL_LENGTH} characters"
        )));
    }

    Ok(value)
}

fn invalid(detail: String) -> InsightError {
    DomainError::InvalidValue(detail).into()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn code() -> MeasureCode {
        MeasureCode::new("level_pct".to_string()).expect("code")
    }

    fn measure(
        value: f64,
        unit: &str,
        label_en: &str,
        label_ku: Option<&str>,
    ) -> Result<Measure, InsightError> {
        Measure::new(
            code(),
            value,
            unit.to_string(),
            label_en.to_string(),
            label_ku.map(str::to_string),
        )
    }

    #[test]
    fn accepts_a_measure_with_both_labels() {
        let measure = measure(54.2, "%", "Dam level", Some("ئاستی بەنداو")).expect("measure");

        assert_eq!(*measure.value(), 54.2);
        assert_eq!(measure.unit(), "%");
    }

    #[test]
    fn the_sorani_label_may_be_missing_but_the_english_one_may_not() {
        assert!(measure(1.0, "%", "Dam level", None).is_ok());
        assert!(measure(1.0, "%", "", None).is_err());
        assert!(measure(1.0, "%", "   ", None).is_err());
    }

    #[test]
    fn a_sorani_label_that_is_sent_must_say_something() {
        assert!(measure(1.0, "%", "Dam level", Some("  ")).is_err());
    }

    #[test]
    fn a_value_that_is_not_a_finite_number_is_refused() {
        assert!(measure(f64::NAN, "%", "Dam level", None).is_err());
        assert!(measure(f64::INFINITY, "%", "Dam level", None).is_err());
        assert!(measure(f64::NEG_INFINITY, "%", "Dam level", None).is_err());
    }

    #[test]
    fn a_negative_value_is_a_number_like_any_other() {
        assert!(measure(-3.5, "°C", "Night temperature", None).is_ok());
    }

    #[test]
    fn the_unit_may_be_empty_but_not_longer_than_twenty_characters() {
        assert!(measure(0.42, "", "Greenness index", None).is_ok());
        assert!(measure(1.0, &"u".repeat(MAX_UNIT_LENGTH), "Dam level", None).is_ok());
        assert!(measure(1.0, &"u".repeat(MAX_UNIT_LENGTH + 1), "Dam level", None).is_err());
    }

    #[test]
    fn the_label_limit_counts_characters_not_bytes() {
        let kurdish = "ئ".repeat(MAX_LABEL_LENGTH);

        assert!(measure(1.0, "%", "Dam level", Some(&kurdish)).is_ok());
        assert!(measure(1.0, "%", &"l".repeat(MAX_LABEL_LENGTH + 1), None).is_err());
    }
}
