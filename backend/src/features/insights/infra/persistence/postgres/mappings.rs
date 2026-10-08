use chrono::NaiveDate;
use sea_orm::ActiveValue::{NotSet, Set};
use serde::{Deserialize, Serialize};

use crate::{
    app::AppError as GlobalAppError,
    features::insights::{
        app::AppError,
        domain::{
            Confidence, FarmInsight, InsightSource, Measure, MeasureCode, ReadingStamp, Summary,
            Topic,
        },
        infra::persistence::postgres::entities::farm_insights,
    },
};

/// One measure as it is kept in the `measures` JSON column.
#[derive(Serialize, Deserialize)]
struct StoredMeasure {
    code: String,
    value: f64,
    unit: String,
    label_en: String,
    label_ku: Option<String>,
}

impl From<&Measure> for StoredMeasure {
    fn from(measure: &Measure) -> Self {
        Self {
            code: measure.code().into(),
            value: *measure.value(),
            unit: measure.unit().clone(),
            label_en: measure.label_en().clone(),
            label_ku: measure.label_ku().clone(),
        }
    }
}

fn stored_measures(measures: &[Measure]) -> serde_json::Value {
    let measures: Vec<StoredMeasure> = measures.iter().map(StoredMeasure::from).collect();

    serde_json::json!(measures)
}

fn measures_from(stored: serde_json::Value) -> Result<Vec<Measure>, AppError> {
    let measures: Vec<StoredMeasure> = serde_json::from_value(stored).map_err(|error| {
        GlobalAppError::MissingValue(format!("Stored insight measures are not readable: {error}"))
    })?;

    measures
        .into_iter()
        .map(|measure| {
            Ok(Measure::new(
                MeasureCode::new(measure.code)?,
                measure.value,
                measure.unit,
                measure.label_en,
                measure.label_ku,
            )?)
        })
        .collect()
}

impl TryFrom<farm_insights::Model> for FarmInsight {
    type Error = AppError;

    fn try_from(model: farm_insights::Model) -> Result<Self, Self::Error> {
        Ok(FarmInsight::rehydrate(
            model.id,
            model.farm_id,
            Topic::try_from(model.topic.as_str())?,
            model.as_of,
            InsightSource::new(model.source)?,
            Confidence::try_from(model.confidence.as_str())?,
            model.summary_en.map(Summary::new).transpose()?,
            model.summary_ku.map(Summary::new).transpose()?,
            measures_from(model.measures)?,
            model.updated_at.and_utc(),
        ))
    }
}

impl From<&FarmInsight> for farm_insights::ActiveModel {
    fn from(insight: &FarmInsight) -> Self {
        farm_insights::ActiveModel {
            id: match *insight.id() {
                Some(id) => Set(id),
                None => NotSet,
            },
            farm_id: Set(*insight.farm_id()),
            topic: Set((*insight.topic()).into()),
            as_of: Set(*insight.as_of()),
            source: Set(insight.source().into()),
            confidence: Set((*insight.confidence()).into()),
            summary_en: Set(insight.summary_en().as_ref().map(Into::into)),
            summary_ku: Set(insight.summary_ku().as_ref().map(Into::into)),
            measures: Set(stored_measures(insight.measures())),
            updated_at: Set(insight.updated_at().naive_utc()),
        }
    }
}

impl TryFrom<(i32, String, NaiveDate)> for ReadingStamp {
    type Error = AppError;

    fn try_from((farm_id, topic, as_of): (i32, String, NaiveDate)) -> Result<Self, Self::Error> {
        Ok(ReadingStamp::rehydrate(
            farm_id,
            Topic::try_from(topic.as_str())?,
            as_of,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn measure(label_ku: Option<&str>) -> Measure {
        Measure::new(
            MeasureCode::new("level_pct".to_string()).expect("code"),
            54.2,
            "%".to_string(),
            "Dam level".to_string(),
            label_ku.map(str::to_string),
        )
        .expect("measure")
    }

    #[test]
    fn measures_survive_the_trip_through_the_json_column_in_order() {
        let measures = vec![measure(Some("ئاستی بەنداو")), measure(None)];

        let read_back = measures_from(stored_measures(&measures)).expect("measures");

        assert_eq!(read_back, measures);
    }

    #[test]
    fn a_json_column_that_is_not_a_list_of_measures_is_an_error_not_an_empty_reading() {
        assert!(measures_from(serde_json::json!({"code": "level_pct"})).is_err());
        assert!(measures_from(serde_json::json!([{"code": "level_pct"}])).is_err());
    }
}
