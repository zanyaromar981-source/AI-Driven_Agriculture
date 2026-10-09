use getset::Getters;

use crate::{
    features::plans::domain::{MAX_DAYS, PlanError},
    shared::DomainError,
};

/// A guard against a slip in a job, not a measured limit: no day in Iraq
/// has ever come near these.
const MAX_RAIN_MM: f64 = 1000.0;
const MIN_TEMPERATURE_C: f64 = -60.0;
const MAX_TEMPERATURE_C: f64 = 60.0;

/// The forecast numbers of a plan, one entry per day, the first entry for
/// the plan's first day. An entry is None when the weather service gave no
/// number for that day.
#[derive(Clone, Debug, PartialEq, Getters)]
#[getset(get = "pub")]
pub struct DailyValues {
    rain_mm: Vec<Option<f64>>,
    tmin: Vec<Option<f64>>,
    tmax: Vec<Option<f64>>,
}

impl DailyValues {
    /// `declared_days` is the `days` a job may send next to the lists; when
    /// it is there it has to agree with them.
    pub fn new(
        rain_mm: Vec<Option<f64>>,
        tmin: Vec<Option<f64>>,
        tmax: Vec<Option<f64>>,
        declared_days: Option<usize>,
    ) -> Result<Self, PlanError> {
        let days = rain_mm.len();

        // No forecast beyond ten days, ever: past that the numbers are not
        // worth a farmer's decision.
        if days == 0 || days > MAX_DAYS || declared_days.is_some_and(|declared| declared > MAX_DAYS)
        {
            return Err(PlanError::DayCount { max: MAX_DAYS });
        }

        if tmin.len() != days
            || tmax.len() != days
            || declared_days.is_some_and(|declared| declared != days)
        {
            return Err(PlanError::UnequalDays);
        }

        if !rain_mm
            .iter()
            .flatten()
            .all(|rain| (0.0..=MAX_RAIN_MM).contains(rain))
        {
            return Err(invalid(format!(
                "Rain must be between 0 and {MAX_RAIN_MM} mm"
            )));
        }

        if !tmin
            .iter()
            .chain(&tmax)
            .flatten()
            .all(|temperature| (MIN_TEMPERATURE_C..=MAX_TEMPERATURE_C).contains(temperature))
        {
            return Err(invalid(format!(
                "A temperature must be between {MIN_TEMPERATURE_C} and {MAX_TEMPERATURE_C} C"
            )));
        }

        Ok(Self {
            rain_mm,
            tmin,
            tmax,
        })
    }

    pub fn days(&self) -> usize {
        self.rain_mm.len()
    }

    /// The same numbers without the first `count` days. None when no day
    /// would be left.
    pub fn without_first(&self, count: usize) -> Option<Self> {
        if count >= self.days() {
            return None;
        }

        Some(Self {
            rain_mm: self.rain_mm[count..].to_vec(),
            tmin: self.tmin[count..].to_vec(),
            tmax: self.tmax[count..].to_vec(),
        })
    }
}

fn invalid(detail: String) -> PlanError {
    DomainError::InvalidValue(detail).into()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn numbers(days: usize) -> Vec<Option<f64>> {
        (0..days).map(|day| Some(day as f64)).collect()
    }

    fn values(days: usize) -> Result<DailyValues, PlanError> {
        DailyValues::new(numbers(days), numbers(days), numbers(days), None)
    }

    #[test]
    fn ten_days_is_the_most_a_plan_covers_never_more() {
        assert_eq!(values(10).expect("values").days(), 10);
        assert!(
            matches!(values(11), Err(PlanError::DayCount { max: 10 })),
            "no forecast beyond 10 days, ever"
        );
        assert!(matches!(values(16), Err(PlanError::DayCount { .. })));
    }

    #[test]
    fn a_declared_day_count_above_ten_is_refused_even_with_short_lists() {
        let result = DailyValues::new(numbers(3), numbers(3), numbers(3), Some(14));

        assert!(matches!(result, Err(PlanError::DayCount { max: 10 })));
    }

    #[test]
    fn a_plan_with_no_days_is_refused() {
        assert!(matches!(values(0), Err(PlanError::DayCount { .. })));
    }

    #[test]
    fn the_three_lists_must_have_the_same_number_of_days() {
        assert!(matches!(
            DailyValues::new(numbers(10), numbers(9), numbers(10), None),
            Err(PlanError::UnequalDays)
        ));
        assert!(matches!(
            DailyValues::new(numbers(10), numbers(10), numbers(8), None),
            Err(PlanError::UnequalDays)
        ));
    }

    #[test]
    fn a_declared_day_count_must_agree_with_the_lists() {
        assert!(DailyValues::new(numbers(7), numbers(7), numbers(7), Some(7)).is_ok());
        assert!(matches!(
            DailyValues::new(numbers(7), numbers(7), numbers(7), Some(10)),
            Err(PlanError::UnequalDays)
        ));
    }

    #[test]
    fn a_day_without_a_number_stays_empty_rather_than_becoming_zero() {
        let values = DailyValues::new(
            vec![Some(2.1), None],
            vec![None, Some(-3.0)],
            vec![Some(20.0), None],
            None,
        )
        .expect("values");

        assert_eq!(values.rain_mm(), &vec![Some(2.1), None]);
        assert_eq!(values.tmin(), &vec![None, Some(-3.0)]);
    }

    #[test]
    fn numbers_no_weather_could_give_are_refused() {
        let one = |value: f64| vec![Some(value)];

        assert!(DailyValues::new(one(-0.1), one(1.0), one(2.0), None).is_err());
        assert!(DailyValues::new(one(1001.0), one(1.0), one(2.0), None).is_err());
        assert!(DailyValues::new(one(0.0), one(-61.0), one(2.0), None).is_err());
        assert!(DailyValues::new(one(0.0), one(1.0), one(61.0), None).is_err());
        assert!(DailyValues::new(one(0.0), one(f64::NAN), one(2.0), None).is_err());
    }

    #[test]
    fn dropping_the_first_days_keeps_the_rest_in_order() {
        let rest = values(10)
            .expect("values")
            .without_first(2)
            .expect("eight days left");

        assert_eq!(rest.days(), 8);
        assert_eq!(rest.rain_mm()[0], Some(2.0));
        assert_eq!(rest.tmax()[7], Some(9.0));
    }

    #[test]
    fn dropping_every_day_leaves_nothing() {
        assert!(values(2).expect("values").without_first(2).is_none());
        assert!(values(2).expect("values").without_first(5).is_none());
    }
}
