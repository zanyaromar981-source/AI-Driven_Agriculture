use std::collections::{BTreeMap, HashMap, HashSet};

use chrono::{DateTime, Utc};
use getset::Getters;

use crate::{
    features::history::domain::{HistoryError, HistorySource, Metric, Month, YearRule},
    shared::DomainError,
};

const MIN_POINTS: usize = 1;
/// Twenty years of months: a first fill of ten years fits twice over.
const MAX_POINTS: usize = 240;

/// The value of one metric for one month.
#[derive(Clone, Copy, Debug, PartialEq, Getters)]
#[getset(get = "pub")]
pub struct MonthlyPoint {
    month: Month,
    value: f64,
}

impl MonthlyPoint {
    /// Reconstruct from persisted state.
    pub fn rehydrate(month: Month, value: f64) -> Self {
        Self { month, value }
    }
}

/// What a data job pushes for one metric of one farm: some months and the
/// facts about the series they belong to.
#[derive(Clone, Debug, Getters)]
#[getset(get = "pub")]
pub struct SeriesUpload {
    farm_id: i32,
    metric: Metric,
    source: HistorySource,
    /// When the job read the numbers from its source.
    as_of: DateTime<Utc>,
    /// Earliest month first.
    points: Vec<MonthlyPoint>,
}

impl SeriesUpload {
    /// `now` is the moment of the push: a month that has not begun yet
    /// cannot have been measured.
    pub fn new(
        farm_id: i32,
        metric: Metric,
        unit: &str,
        source: HistorySource,
        as_of: DateTime<Utc>,
        points: Vec<(Month, f64)>,
        now: DateTime<Utc>,
    ) -> Result<Self, HistoryError> {
        if farm_id < 1 {
            return Err(
                DomainError::InvalidValue("Farm id must be a positive number".to_string()).into(),
            );
        }

        if unit.trim() != metric.unit() {
            return Err(HistoryError::UnitMismatch {
                metric: metric.into(),
                expected: metric.unit(),
            });
        }

        if !(MIN_POINTS..=MAX_POINTS).contains(&points.len()) {
            return Err(HistoryError::PointCount {
                min: MIN_POINTS,
                max: MAX_POINTS,
            });
        }

        let current = Month::containing(now.date_naive())?;
        let (min, max) = metric.monthly_range();
        let mut seen = HashSet::new();

        for (month, value) in &points {
            if !seen.insert(*month) {
                return Err(HistoryError::DuplicateMonth(month.to_string()));
            }

            if *month > current {
                return Err(DomainError::InvalidValue(format!(
                    "The month {month} has not begun yet"
                ))
                .into());
            }

            // A value that is not a number fails this test as well.
            if !(min..=max).contains(value) {
                return Err(HistoryError::ValueOutOfRange {
                    metric: metric.into(),
                    month: month.to_string(),
                    value: *value,
                    min,
                    max,
                });
            }
        }

        let mut points: Vec<MonthlyPoint> = points
            .into_iter()
            .map(|(month, value)| MonthlyPoint { month, value })
            .collect();
        points.sort_by_key(|point| point.month);

        Ok(Self {
            farm_id,
            metric,
            source,
            as_of,
            points,
        })
    }
}

/// The figure of one calendar year, with the number of months it is made
/// from so a reader can tell a whole year from a part of one.
#[derive(Clone, Copy, Debug, PartialEq, Getters)]
#[getset(get = "pub")]
pub struct YearFigure {
    year: i32,
    value: f64,
    months: u32,
}

/// One metric of one farm as it is read back: the stored months of a
/// window with the facts of the series.
#[derive(Clone, Debug, PartialEq, Getters)]
#[getset(get = "pub")]
pub struct Series {
    metric: Metric,
    source: HistorySource,
    as_of: DateTime<Utc>,
    /// Earliest month first. Months nobody measured are simply not there.
    points: Vec<MonthlyPoint>,
}

impl Series {
    /// Reconstruct from persisted state.
    pub fn rehydrate(
        metric: Metric,
        source: HistorySource,
        as_of: DateTime<Utc>,
        mut points: Vec<MonthlyPoint>,
    ) -> Self {
        points.sort_by_key(|point| point.month);

        Self {
            metric,
            source,
            as_of,
            points,
        }
    }

    /// One figure per calendar year, earliest first. A metric that is added
    /// up gets a figure only for a year with all twelve months; a metric
    /// that is averaged gets one for every year with at least one month,
    /// averaged over the months it has.
    pub fn years(&self) -> Vec<YearFigure> {
        let mut per_year: BTreeMap<i32, Vec<f64>> = BTreeMap::new();

        for point in &self.points {
            per_year
                .entry(point.month.year())
                .or_default()
                .push(point.value);
        }

        per_year
            .into_iter()
            .filter_map(|(year, values)| {
                let months = values.len() as u32;
                let total: f64 = values.iter().sum();

                let value = match self.metric.year_rule() {
                    YearRule::Sum if months == 12 => total,
                    YearRule::Sum => return None,
                    YearRule::Mean => total / f64::from(months),
                };

                Some(YearFigure {
                    year,
                    value: tidy(value),
                    months,
                })
            })
            .collect()
    }

    /// The usual value of each calendar month, January first: the mean of
    /// that month over the years the series holds. `None` for a calendar
    /// month the series has no value for.
    pub fn normal(&self) -> [Option<f64>; 12] {
        let mut totals = [(0.0_f64, 0_u32); 12];

        for point in &self.points {
            let slot = &mut totals[point.month.number() as usize - 1];
            slot.0 += point.value;
            slot.1 += 1;
        }

        totals.map(|(total, count)| (count > 0).then(|| tidy(total / f64::from(count))))
    }
}

/// Drops the noise binary sums leave far behind the decimal point, so 512.4
/// is not served as 512.4000000000001.
fn tidy(value: f64) -> f64 {
    (value * 10_000.0).round() / 10_000.0
}

/// Where one farm is and since when it exists, as the farms feature reports
/// it. It has no owner on purpose: the data jobs never learn whose farm it
/// is.
#[derive(Clone, Debug, PartialEq, Getters)]
#[getset(get = "pub")]
pub struct FarmSite {
    farm_id: i32,
    lat: f64,
    lon: f64,
    created_at: DateTime<Utc>,
}

impl FarmSite {
    /// Reconstruct from the farms feature's state.
    pub fn rehydrate(farm_id: i32, lat: f64, lon: f64, created_at: DateTime<Utc>) -> Self {
        Self {
            farm_id,
            lat,
            lon,
            created_at,
        }
    }
}

/// How much of one metric of one farm is stored.
#[derive(Clone, Copy, Debug, PartialEq, Getters)]
#[getset(get = "pub")]
pub struct MetricCoverage {
    farm_id: i32,
    metric: Metric,
    first_month: Month,
    last_month: Month,
    months: u32,
    as_of: DateTime<Utc>,
}

impl MetricCoverage {
    /// Reconstruct from persisted state.
    pub fn rehydrate(
        farm_id: i32,
        metric: Metric,
        first_month: Month,
        last_month: Month,
        months: u32,
        as_of: DateTime<Utc>,
    ) -> Self {
        Self {
            farm_id,
            metric,
            first_month,
            last_month,
            months,
            as_of,
        }
    }
}

/// One farm with what is stored of each metric, so the data job can tell
/// what is missing.
#[derive(Clone, Debug, PartialEq, Getters)]
#[getset(get = "pub")]
pub struct FarmCoverage {
    site: FarmSite,
    /// In the order the metrics are declared. A metric with nothing stored
    /// is not listed.
    metrics: Vec<MetricCoverage>,
}

impl FarmCoverage {
    /// Pairs every farm with its own metrics and keeps the farms in the
    /// order they came. Coverage whose farm is not listed, for example
    /// months pushed while the farm was being deleted, is ignored.
    pub fn assemble(sites: Vec<FarmSite>, stored: Vec<MetricCoverage>) -> Vec<Self> {
        let mut per_farm: HashMap<i32, Vec<MetricCoverage>> = HashMap::new();

        for coverage in stored {
            per_farm.entry(coverage.farm_id).or_default().push(coverage);
        }

        sites
            .into_iter()
            .map(|site| {
                let mut metrics = per_farm.remove(&site.farm_id).unwrap_or_default();
                metrics.sort_by_key(|coverage| coverage.metric);

                Self { site, metrics }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use super::*;

    fn month(raw: &str) -> Month {
        Month::parse(raw).expect("month")
    }

    fn now() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 10, 9, 12, 0, 0).unwrap()
    }

    fn source() -> HistorySource {
        HistorySource::new("ERA5-Land reanalysis via Open-Meteo, about 9 km".to_string())
            .expect("source")
    }

    fn upload(
        metric: Metric,
        unit: &str,
        points: Vec<(&str, f64)>,
    ) -> Result<SeriesUpload, HistoryError> {
        SeriesUpload::new(
            7,
            metric,
            unit,
            source(),
            now(),
            points
                .into_iter()
                .map(|(raw, value)| (month(raw), value))
                .collect(),
            now(),
        )
    }

    fn series(metric: Metric, points: Vec<(&str, f64)>) -> Series {
        Series::rehydrate(
            metric,
            source(),
            now(),
            points
                .into_iter()
                .map(|(raw, value)| MonthlyPoint::rehydrate(month(raw), value))
                .collect(),
        )
    }

    /// Twelve months of one year, each with the same value.
    fn whole_year(year: i32, value: f64) -> Vec<(String, f64)> {
        (1..=12)
            .map(|number| (format!("{year}-{number:02}"), value))
            .collect()
    }

    fn borrowed(points: &[(String, f64)]) -> Vec<(&str, f64)> {
        points
            .iter()
            .map(|(raw, value)| (raw.as_str(), *value))
            .collect()
    }

    #[test]
    fn a_push_keeps_its_points_earliest_month_first() {
        let upload = upload(
            Metric::RainMm,
            "mm",
            vec![("2026-09", 0.0), ("2016-10", 31.2), ("2020-01", 140.5)],
        )
        .expect("upload");

        let months: Vec<String> = upload
            .points()
            .iter()
            .map(|point| point.month().to_string())
            .collect();

        assert_eq!(months, vec!["2016-10", "2020-01", "2026-09"]);
    }

    #[test]
    fn a_unit_other_than_the_metrics_own_is_refused() {
        assert!(matches!(
            upload(Metric::RainMm, "cm", vec![("2020-01", 14.0)]),
            Err(HistoryError::UnitMismatch { expected: "mm", .. })
        ));
        assert!(
            upload(Metric::TempMaxC, "°C", vec![("2020-01", 14.0)]).is_ok(),
            "the degree sign is part of the unit"
        );
    }

    #[test]
    fn a_push_carries_1_to_240_points() {
        let many = |count: usize| -> Vec<(Month, f64)> {
            (0..count as u32)
                .map(|back| (month("2026-09").back(back), 1.0))
                .collect()
        };
        let build =
            |points| SeriesUpload::new(7, Metric::RainMm, "mm", source(), now(), points, now());

        assert!(matches!(
            build(vec![]),
            Err(HistoryError::PointCount { min: 1, max: 240 })
        ));
        assert!(build(many(240)).is_ok());
        assert!(matches!(
            build(many(241)),
            Err(HistoryError::PointCount { .. })
        ));
    }

    #[test]
    fn the_same_month_twice_in_one_push_is_refused() {
        let result = upload(
            Metric::RainMm,
            "mm",
            vec![("2020-01", 14.0), ("2020-02", 3.0), ("2020-01", 15.0)],
        );

        assert!(
            matches!(result, Err(HistoryError::DuplicateMonth(month)) if month == "2020-01"),
            "which of the two values is meant cannot be known"
        );
    }

    #[test]
    fn each_metric_refuses_a_value_outside_its_own_range() {
        let cases = [
            (Metric::RainMm, -0.1, 2_000.1),
            (Metric::TempMaxC, -60.1, 60.1),
            (Metric::TempMinC, -60.1, 60.1),
            (Metric::Et0Mm, -0.1, 500.1),
            (Metric::SoilMoisture, -0.01, 1.01),
            (Metric::Greenness, -0.21, 1.01),
            (Metric::GroundwaterPct, -0.1, 100.1),
        ];

        for (metric, too_low, too_high) in cases {
            let (min, max) = metric.monthly_range();

            for value in [too_low, too_high] {
                assert!(
                    matches!(
                        upload(metric, metric.unit(), vec![("2020-01", value)]),
                        Err(HistoryError::ValueOutOfRange { .. })
                    ),
                    "{metric:?} accepted {value}"
                );
            }

            for value in [min, max] {
                assert!(
                    upload(metric, metric.unit(), vec![("2020-01", value)]).is_ok(),
                    "{metric:?} refused its own limit {value}"
                );
            }
        }
    }

    #[test]
    fn a_value_that_is_not_a_number_is_refused() {
        for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert!(upload(Metric::TempMinC, "°C", vec![("2020-01", value)]).is_err());
        }
    }

    #[test]
    fn the_running_month_may_be_pushed_but_a_month_to_come_may_not() {
        assert!(upload(Metric::RainMm, "mm", vec![("2026-10", 2.0)]).is_ok());
        assert!(upload(Metric::RainMm, "mm", vec![("2026-11", 2.0)]).is_err());
    }

    #[test]
    fn a_farm_id_below_one_cannot_name_a_farm() {
        let build = |farm_id| {
            SeriesUpload::new(
                farm_id,
                Metric::RainMm,
                "mm",
                source(),
                now(),
                vec![(month("2020-01"), 1.0)],
                now(),
            )
        };

        assert!(build(0).is_err());
        assert!(build(-4).is_err());
    }

    #[test]
    fn a_summed_metric_gets_a_year_figure_only_for_a_year_with_all_twelve_months() {
        let mut points = whole_year(2024, 50.0);
        points.extend(whole_year(2025, 10.0).into_iter().take(11));
        points.push(("2026-01".to_string(), 80.0));

        let years = series(Metric::RainMm, borrowed(&points)).years();

        assert_eq!(
            years,
            vec![YearFigure {
                year: 2024,
                value: 600.0,
                months: 12,
            }],
            "eleven months of rain must not be shown as the rain of a year"
        );
    }

    #[test]
    fn an_averaged_metric_gets_a_figure_for_every_year_with_a_month_and_says_how_many() {
        let years = series(
            Metric::Greenness,
            vec![("2024-03", 0.6), ("2024-04", 0.8), ("2025-07", 0.2)],
        )
        .years();

        assert_eq!(
            years,
            vec![
                YearFigure {
                    year: 2024,
                    value: 0.7,
                    months: 2,
                },
                YearFigure {
                    year: 2025,
                    value: 0.2,
                    months: 1,
                },
            ]
        );
    }

    #[test]
    fn a_year_figure_of_a_summed_metric_is_the_sum_and_of_an_averaged_one_the_mean() {
        let points = whole_year(2024, 30.0);

        assert_eq!(
            *series(Metric::Et0Mm, borrowed(&points)).years()[0].value(),
            360.0
        );
        assert_eq!(
            *series(Metric::TempMaxC, borrowed(&points)).years()[0].value(),
            30.0
        );
    }

    #[test]
    fn year_figures_come_earliest_year_first_and_carry_no_float_noise() {
        let years = series(
            Metric::SoilMoisture,
            vec![("2025-01", 0.1), ("2024-01", 0.1), ("2024-02", 0.2)],
        )
        .years();

        assert_eq!(years[0].year, 2024);
        assert_eq!(years[1].year, 2025);
        assert_eq!(
            years[0].value, 0.15,
            "0.1 + 0.2 is not exactly 0.3 in binary"
        );
    }

    #[test]
    fn the_normal_is_the_mean_of_each_calendar_month_january_first() {
        let normal = series(
            Metric::RainMm,
            vec![
                ("2024-01", 100.0),
                ("2025-01", 140.0),
                ("2024-07", 0.0),
                ("2025-12", 90.0),
            ],
        )
        .normal();

        assert_eq!(normal[0], Some(120.0));
        assert_eq!(normal[6], Some(0.0), "a dry month is a zero, not a gap");
        assert_eq!(normal[11], Some(90.0));
        assert_eq!(
            normal.iter().filter(|month| month.is_none()).count(),
            9,
            "a calendar month with no value is unknown, not zero"
        );
    }

    #[test]
    fn a_series_without_points_has_no_years_and_no_normal() {
        let empty = series(Metric::RainMm, vec![]);

        assert!(empty.years().is_empty());
        assert!(empty.normal().iter().all(Option::is_none));
    }

    fn site(farm_id: i32) -> FarmSite {
        FarmSite::rehydrate(farm_id, 35.56, 45.43, now())
    }

    fn coverage(farm_id: i32, metric: Metric) -> MetricCoverage {
        MetricCoverage::rehydrate(
            farm_id,
            metric,
            month("2016-10"),
            month("2026-09"),
            120,
            now(),
        )
    }

    #[test]
    fn coverage_gives_each_farm_its_own_metrics_in_declared_order() {
        let farms = FarmCoverage::assemble(
            vec![site(1), site(2)],
            vec![
                coverage(2, Metric::Greenness),
                coverage(1, Metric::RainMm),
                coverage(2, Metric::RainMm),
            ],
        );

        let metrics = |index: usize| -> Vec<Metric> {
            farms[index]
                .metrics()
                .iter()
                .map(|coverage| *coverage.metric())
                .collect()
        };

        assert_eq!(metrics(0), vec![Metric::RainMm]);
        assert_eq!(metrics(1), vec![Metric::RainMm, Metric::Greenness]);
    }

    #[test]
    fn a_farm_with_nothing_stored_is_still_listed() {
        let farms = FarmCoverage::assemble(vec![site(1)], vec![]);

        assert_eq!(farms.len(), 1);
        assert!(
            farms[0].metrics().is_empty(),
            "a new farm is exactly the one the job has to fill"
        );
    }

    #[test]
    fn coverage_of_a_farm_that_is_gone_is_left_out() {
        let farms = FarmCoverage::assemble(vec![site(1)], vec![coverage(99, Metric::RainMm)]);

        assert_eq!(farms.len(), 1);
        assert!(farms[0].metrics().is_empty());
    }
}
