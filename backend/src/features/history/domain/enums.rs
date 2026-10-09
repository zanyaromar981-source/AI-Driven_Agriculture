use crate::{features::history::domain::HistoryError, shared::DomainError};

/// How the months of one year become that year's figure.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum YearRule {
    /// The months are added up, so a year counts only when all twelve are
    /// there: eleven months of rain is not a year of rain.
    Sum,
    /// The months are averaged, over however many the year has.
    Mean,
}

/// One thing that is kept month by month for a farm. The variants are
/// declared in the order the app lists them, and that order sorts a farm's
/// series.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Metric {
    RainMm,
    TempMaxC,
    TempMinC,
    /// Reference evapotranspiration: the water a well-watered grass field
    /// would lose.
    Et0Mm,
    SoilMoisture,
    /// NDVI.
    Greenness,
    GroundwaterPct,
}

impl Metric {
    pub const ALL: [Metric; 7] = [
        Metric::RainMm,
        Metric::TempMaxC,
        Metric::TempMinC,
        Metric::Et0Mm,
        Metric::SoilMoisture,
        Metric::Greenness,
        Metric::GroundwaterPct,
    ];

    /// The one unit a metric is stored in. A push in another unit is
    /// refused rather than converted.
    pub const fn unit(self) -> &'static str {
        match self {
            Metric::RainMm | Metric::Et0Mm => "mm",
            Metric::TempMaxC | Metric::TempMinC => "°C",
            Metric::SoilMoisture => "m3/m3",
            Metric::Greenness => "ndvi",
            Metric::GroundwaterPct => "percentile",
        }
    }

    pub const fn year_rule(self) -> YearRule {
        match self {
            Metric::RainMm | Metric::Et0Mm => YearRule::Sum,
            Metric::TempMaxC
            | Metric::TempMinC
            | Metric::SoilMoisture
            | Metric::Greenness
            | Metric::GroundwaterPct => YearRule::Mean,
        }
    }

    /// The lowest and highest value one month can have. Wide enough for any
    /// real month anywhere, narrow enough to stop a wrong unit or a fill
    /// value from being stored as a measurement.
    pub const fn monthly_range(self) -> (f64, f64) {
        match self {
            Metric::RainMm => (0.0, 2_000.0),
            Metric::TempMaxC | Metric::TempMinC => (-60.0, 60.0),
            Metric::Et0Mm => (0.0, 500.0),
            Metric::SoilMoisture => (0.0, 1.0),
            Metric::Greenness => (-0.2, 1.0),
            Metric::GroundwaterPct => (0.0, 100.0),
        }
    }

    /// The metrics named by a comma-separated list, each once, in the order
    /// they are declared whatever order they were asked in.
    pub fn parse_list(raw: &str) -> Result<Vec<Metric>, HistoryError> {
        let mut asked = raw
            .split(',')
            .map(|name| Metric::try_from(name.trim()))
            .collect::<Result<Vec<_>, _>>()?;

        asked.sort();
        asked.dedup();

        Ok(asked)
    }
}

impl From<Metric> for String {
    fn from(value: Metric) -> Self {
        match value {
            Metric::RainMm => "rain_mm".to_string(),
            Metric::TempMaxC => "temp_max_c".to_string(),
            Metric::TempMinC => "temp_min_c".to_string(),
            Metric::Et0Mm => "et0_mm".to_string(),
            Metric::SoilMoisture => "soil_moisture".to_string(),
            Metric::Greenness => "greenness".to_string(),
            Metric::GroundwaterPct => "groundwater_pct".to_string(),
        }
    }
}

impl TryFrom<&str> for Metric {
    type Error = HistoryError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "rain_mm" => Ok(Metric::RainMm),
            "temp_max_c" => Ok(Metric::TempMaxC),
            "temp_min_c" => Ok(Metric::TempMinC),
            "et0_mm" => Ok(Metric::Et0Mm),
            "soil_moisture" => Ok(Metric::SoilMoisture),
            "greenness" => Ok(Metric::Greenness),
            "groundwater_pct" => Ok(Metric::GroundwaterPct),
            _ => Err(DomainError::InvalidValue(format!("Invalid metric: {value}")).into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every variant must survive a trip to the database and back. A mismatch
    /// between the two directions corrupts rows silently rather than failing.
    #[test]
    fn every_metric_round_trips() {
        for metric in Metric::ALL {
            let stored = String::from(metric);
            let parsed = Metric::try_from(stored.as_str())
                .unwrap_or_else(|err| panic!("{metric:?} stored as {stored:?} but {err:?}"));

            assert_eq!(parsed, metric, "{stored:?} did not round trip");
        }
    }

    #[test]
    fn the_stored_form_is_lower_case_snake_case() {
        assert_eq!(String::from(Metric::TempMaxC), "temp_max_c");
        assert_eq!(String::from(Metric::GroundwaterPct), "groundwater_pct");
    }

    #[test]
    fn an_unknown_stored_value_is_rejected_rather_than_defaulted() {
        assert!(Metric::try_from("wind").is_err());
        assert!(Metric::try_from("Rain_mm").is_err());
        assert!(Metric::try_from("").is_err());
    }

    #[test]
    fn totals_are_summed_over_a_year_and_states_are_averaged() {
        assert_eq!(Metric::RainMm.year_rule(), YearRule::Sum);
        assert_eq!(Metric::Et0Mm.year_rule(), YearRule::Sum);

        for metric in [
            Metric::TempMaxC,
            Metric::TempMinC,
            Metric::SoilMoisture,
            Metric::Greenness,
            Metric::GroundwaterPct,
        ] {
            assert_eq!(metric.year_rule(), YearRule::Mean, "{metric:?}");
        }
    }

    #[test]
    fn every_metric_has_a_unit_and_a_range_that_is_not_empty() {
        for metric in Metric::ALL {
            let (min, max) = metric.monthly_range();

            assert!(!metric.unit().is_empty(), "{metric:?}");
            assert!(min < max, "{metric:?}");
        }
    }

    #[test]
    fn a_list_is_read_in_declared_order_with_repeats_dropped() {
        assert_eq!(
            Metric::parse_list("greenness, rain_mm,greenness").expect("metrics"),
            vec![Metric::RainMm, Metric::Greenness]
        );
    }

    #[test]
    fn a_list_with_an_unknown_or_empty_name_is_refused_as_a_whole() {
        assert!(Metric::parse_list("rain_mm,wind").is_err());
        assert!(Metric::parse_list("").is_err());
        assert!(Metric::parse_list("rain_mm,").is_err());
    }
}
