use chrono::{NaiveDate, NaiveDateTime};
use sea_orm::{ActiveValue::Set, FromQueryResult};

use crate::features::history::{
    app::AppError,
    domain::{HistorySource, Metric, MetricCoverage, Month, MonthlyPoint, Series, SeriesUpload},
    infra::persistence::postgres::entities::{farm_history, farm_history_series},
};

/// The stored span of one metric of one farm, as the grouped query returns
/// it.
#[derive(Debug, FromQueryResult)]
pub struct StoredSpan {
    pub farm_id: i32,
    pub metric: String,
    pub first_month: NaiveDate,
    pub last_month: NaiveDate,
    pub months: i64,
}

impl TryFrom<&farm_history::Model> for MonthlyPoint {
    type Error = AppError;

    fn try_from(model: &farm_history::Model) -> Result<Self, Self::Error> {
        Ok(MonthlyPoint::rehydrate(
            Month::containing(model.month)?,
            model.value,
        ))
    }
}

impl TryFrom<(farm_history_series::Model, Vec<MonthlyPoint>)> for Series {
    type Error = AppError;

    fn try_from(
        (model, points): (farm_history_series::Model, Vec<MonthlyPoint>),
    ) -> Result<Self, Self::Error> {
        Ok(Series::rehydrate(
            Metric::try_from(model.metric.as_str())?,
            HistorySource::new(model.source)?,
            model.as_of.and_utc(),
            points,
        ))
    }
}

impl TryFrom<(StoredSpan, NaiveDateTime)> for MetricCoverage {
    type Error = AppError;

    fn try_from((span, as_of): (StoredSpan, NaiveDateTime)) -> Result<Self, Self::Error> {
        Ok(MetricCoverage::rehydrate(
            span.farm_id,
            Metric::try_from(span.metric.as_str())?,
            Month::containing(span.first_month)?,
            Month::containing(span.last_month)?,
            u32::try_from(span.months).unwrap_or(u32::MAX),
            as_of.and_utc(),
        ))
    }
}

impl From<&SeriesUpload> for farm_history_series::ActiveModel {
    fn from(upload: &SeriesUpload) -> Self {
        farm_history_series::ActiveModel {
            farm_id: Set(*upload.farm_id()),
            metric: Set((*upload.metric()).into()),
            unit: Set(upload.metric().unit().to_string()),
            source: Set(upload.source().into()),
            as_of: Set(upload.as_of().naive_utc()),
        }
    }
}

/// One row per pushed month, earliest first.
pub fn point_active_models(upload: &SeriesUpload) -> Vec<farm_history::ActiveModel> {
    upload
        .points()
        .iter()
        .map(|point| farm_history::ActiveModel {
            farm_id: Set(*upload.farm_id()),
            metric: Set((*upload.metric()).into()),
            month: Set(point.month().first_day()),
            value: Set(*point.value()),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};

    use super::*;

    fn upload() -> SeriesUpload {
        let now = Utc.with_ymd_and_hms(2026, 10, 9, 12, 0, 0).unwrap();

        SeriesUpload::new(
            7,
            Metric::TempMaxC,
            "°C",
            HistorySource::new("ERA5-Land via Open-Meteo".to_string()).expect("source"),
            now,
            vec![
                (Month::parse("2026-09").expect("month"), 36.4),
                (Month::parse("2016-10").expect("month"), 27.9),
            ],
            now,
        )
        .expect("upload")
    }

    #[test]
    fn a_month_is_stored_as_its_first_day_and_read_back_as_the_same_month() {
        let models = point_active_models(&upload());

        assert_eq!(
            models[0].month.clone().unwrap(),
            NaiveDate::from_ymd_opt(2016, 10, 1).expect("date"),
            "earliest month first, so two pushes lock rows in the same order"
        );

        let stored = farm_history::Model {
            farm_id: 7,
            metric: "temp_max_c".to_string(),
            month: NaiveDate::from_ymd_opt(2016, 10, 1).expect("date"),
            value: 27.9,
        };
        let point = MonthlyPoint::try_from(&stored).expect("point");

        assert_eq!(point.month().to_string(), "2016-10");
        assert_eq!(*point.value(), 27.9);
    }

    #[test]
    fn the_series_row_carries_the_metrics_own_unit_and_a_naive_utc_time() {
        let model = farm_history_series::ActiveModel::from(&upload());

        assert_eq!(model.metric.unwrap(), "temp_max_c");
        assert_eq!(model.unit.unwrap(), "°C");
        assert_eq!(
            model.as_of.unwrap(),
            NaiveDate::from_ymd_opt(2026, 10, 9)
                .expect("date")
                .and_hms_opt(12, 0, 0)
                .expect("time")
        );
    }

    #[test]
    fn a_stored_metric_the_code_does_not_know_is_an_error_not_a_guess() {
        let span = StoredSpan {
            farm_id: 7,
            metric: "wind".to_string(),
            first_month: NaiveDate::from_ymd_opt(2016, 10, 1).expect("date"),
            last_month: NaiveDate::from_ymd_opt(2026, 9, 1).expect("date"),
            months: 120,
        };

        assert!(MetricCoverage::try_from((span, upload().as_of().naive_utc())).is_err());
    }
}
