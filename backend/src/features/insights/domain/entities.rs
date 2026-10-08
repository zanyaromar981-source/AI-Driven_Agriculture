use std::collections::{HashMap, HashSet};

use chrono::{DateTime, NaiveDate, Utc};
use getset::Getters;

use crate::{
    features::insights::domain::{
        Confidence, InsightError, InsightSource, Measure, Summary, Topic,
    },
    shared::DomainError,
};

const MIN_MEASURES: usize = 1;
const MAX_MEASURES: usize = 20;

/// The current reading of one topic for one farm. A farm has at most one per
/// topic: the next push replaces it.
#[derive(Clone, Debug, Getters)]
#[getset(get = "pub")]
pub struct FarmInsight {
    /// None = new (not yet persisted), Some = existing (persisted)
    id: Option<i32>,
    farm_id: i32,
    topic: Topic,
    /// The day the numbers describe, which can be days before the push.
    as_of: NaiveDate,
    source: InsightSource,
    confidence: Confidence,
    summary_en: Option<Summary>,
    summary_ku: Option<Summary>,
    measures: Vec<Measure>,
    updated_at: DateTime<Utc>,
}

impl FarmInsight {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        farm_id: i32,
        topic: Topic,
        as_of: NaiveDate,
        source: InsightSource,
        confidence: Confidence,
        summary_en: Option<Summary>,
        summary_ku: Option<Summary>,
        measures: Vec<Measure>,
    ) -> Result<Self, InsightError> {
        if farm_id < 1 {
            return Err(
                DomainError::InvalidValue("Farm id must be a positive number".to_string()).into(),
            );
        }

        if !(MIN_MEASURES..=MAX_MEASURES).contains(&measures.len()) {
            return Err(InsightError::MeasureCount {
                min: MIN_MEASURES,
                max: MAX_MEASURES,
            });
        }

        let mut seen = HashSet::new();

        for measure in &measures {
            if !seen.insert(measure.code()) {
                return Err(InsightError::DuplicateMeasureCode(measure.code().into()));
            }
        }

        Ok(Self {
            id: None,
            farm_id,
            topic,
            as_of,
            source,
            confidence,
            summary_en,
            summary_ku,
            measures,
            updated_at: Utc::now(),
        })
    }

    /// Reconstruct from persisted state.
    #[allow(clippy::too_many_arguments)]
    pub fn rehydrate(
        id: i32,
        farm_id: i32,
        topic: Topic,
        as_of: NaiveDate,
        source: InsightSource,
        confidence: Confidence,
        summary_en: Option<Summary>,
        summary_ku: Option<Summary>,
        measures: Vec<Measure>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id: Some(id),
            farm_id,
            topic,
            as_of,
            source,
            confidence,
            summary_en,
            summary_ku,
            measures,
            updated_at,
        }
    }
}

/// Where one farm is and how large, as the farms feature reports it. It has
/// no owner on purpose: the data jobs never learn whose farm it is.
#[derive(Clone, Debug, PartialEq, Getters)]
#[getset(get = "pub")]
pub struct FarmSite {
    farm_id: i32,
    lat: f64,
    lon: f64,
    area_dunam: f64,
}

impl FarmSite {
    /// Reconstruct from the farms feature's state.
    pub fn rehydrate(farm_id: i32, lat: f64, lon: f64, area_dunam: f64) -> Self {
        Self {
            farm_id,
            lat,
            lon,
            area_dunam,
        }
    }
}

/// Which topic of which farm has a reading, and for which day.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Getters)]
#[getset(get = "pub")]
pub struct ReadingStamp {
    farm_id: i32,
    topic: Topic,
    as_of: NaiveDate,
}

impl ReadingStamp {
    /// Reconstruct from persisted state.
    pub fn rehydrate(farm_id: i32, topic: Topic, as_of: NaiveDate) -> Self {
        Self {
            farm_id,
            topic,
            as_of,
        }
    }
}

/// One farm with the readings it already has, so a data job can tell which
/// ones are missing or old.
#[derive(Clone, Debug, PartialEq, Getters)]
#[getset(get = "pub")]
pub struct FarmCoverage {
    site: FarmSite,
    /// In the order the topics are declared.
    readings: Vec<(Topic, NaiveDate)>,
}

impl FarmCoverage {
    /// Pairs every farm with its own readings and keeps the farms in the
    /// order they came. A reading whose farm is not listed, for example one
    /// left behind by a deleted farm, is ignored.
    pub fn assemble(sites: Vec<FarmSite>, stamps: Vec<ReadingStamp>) -> Vec<Self> {
        let mut per_farm: HashMap<i32, Vec<(Topic, NaiveDate)>> = HashMap::new();

        for stamp in stamps {
            per_farm
                .entry(stamp.farm_id)
                .or_default()
                .push((stamp.topic, stamp.as_of));
        }

        sites
            .into_iter()
            .map(|site| {
                let mut readings = per_farm.remove(&site.farm_id).unwrap_or_default();
                readings.sort_by_key(|(topic, _)| *topic);

                Self { site, readings }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::insights::domain::MeasureCode;

    fn day(day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 10, day).expect("date")
    }

    fn measure(code: &str) -> Measure {
        Measure::new(
            MeasureCode::new(code.to_string()).expect("code"),
            54.2,
            "%".to_string(),
            "Dam level".to_string(),
            None,
        )
        .expect("measure")
    }

    fn insight(farm_id: i32, measures: Vec<Measure>) -> Result<FarmInsight, InsightError> {
        FarmInsight::new(
            farm_id,
            Topic::SurfaceWater,
            day(7),
            InsightSource::new("Sentinel-2".to_string()).expect("source"),
            Confidence::Likely,
            None,
            None,
            measures,
        )
    }

    #[test]
    fn a_reading_with_one_measure_is_valid_and_not_yet_stored() {
        let insight = insight(7, vec![measure("level_pct")]).expect("insight");

        assert!(insight.id().is_none());
        assert_eq!(insight.measures().len(), 1);
    }

    #[test]
    fn a_reading_without_measures_says_nothing_and_is_refused() {
        assert!(matches!(
            insight(7, vec![]),
            Err(InsightError::MeasureCount { min: 1, max: 20 })
        ));
    }

    #[test]
    fn twenty_measures_is_the_most_a_reading_carries() {
        let measures = |count: usize| (0..count).map(|n| measure(&format!("m_{n}"))).collect();

        assert!(insight(7, measures(MAX_MEASURES)).is_ok());
        assert!(matches!(
            insight(7, measures(MAX_MEASURES + 1)),
            Err(InsightError::MeasureCount { .. })
        ));
    }

    #[test]
    fn the_same_code_twice_in_one_reading_is_refused() {
        let result = insight(
            7,
            vec![
                measure("level_pct"),
                measure("volume"),
                measure("level_pct"),
            ],
        );

        assert!(
            matches!(result, Err(InsightError::DuplicateMeasureCode(code)) if code == "level_pct"),
            "the app looks a measure up by its code, so a code names one measure"
        );
    }

    #[test]
    fn a_farm_id_below_one_cannot_name_a_farm() {
        assert!(insight(0, vec![measure("level_pct")]).is_err());
        assert!(insight(-4, vec![measure("level_pct")]).is_err());
    }

    #[test]
    fn coverage_gives_each_farm_its_own_readings_in_topic_order() {
        let coverage = FarmCoverage::assemble(
            vec![
                FarmSite::rehydrate(1, 36.0, 44.0, 12.0),
                FarmSite::rehydrate(2, 36.5, 44.5, 30.0),
            ],
            vec![
                ReadingStamp::rehydrate(2, Topic::Weather, day(8)),
                ReadingStamp::rehydrate(1, Topic::Rain, day(6)),
                ReadingStamp::rehydrate(2, Topic::SurfaceWater, day(7)),
            ],
        );

        assert_eq!(coverage.len(), 2);
        assert_eq!(coverage[0].readings(), &vec![(Topic::Rain, day(6))]);
        assert_eq!(
            coverage[1].readings(),
            &vec![(Topic::SurfaceWater, day(7)), (Topic::Weather, day(8))]
        );
    }

    #[test]
    fn a_farm_without_readings_is_still_listed() {
        let coverage =
            FarmCoverage::assemble(vec![FarmSite::rehydrate(1, 36.0, 44.0, 12.0)], vec![]);

        assert_eq!(coverage.len(), 1);
        assert!(
            coverage[0].readings().is_empty(),
            "a new farm is exactly the one the job has to compute for"
        );
    }

    #[test]
    fn a_reading_of_a_farm_that_is_gone_is_left_out() {
        let coverage = FarmCoverage::assemble(
            vec![FarmSite::rehydrate(1, 36.0, 44.0, 12.0)],
            vec![ReadingStamp::rehydrate(99, Topic::Rain, day(6))],
        );

        assert_eq!(coverage.len(), 1);
        assert!(coverage[0].readings().is_empty());
    }
}
