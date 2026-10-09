use chrono::NaiveDateTime;
use sea_orm::ActiveValue::{NotSet, Set};
use serde::{Deserialize, Serialize};

use crate::{
    app::AppError as GlobalAppError,
    features::briefs::{
        app::AppError,
        domain::{
            Author, BriefPoint, BriefScope, BriefSource, BriefSummary, DailyBrief, FarmZone,
            Headline, PointLevel, PointText, SourceTitle, SourceUrl,
        },
        infra::persistence::postgres::entities::{daily_briefs, farm_brief_zones},
    },
};

/// One point as it is kept in the `points` JSON column.
#[derive(Serialize, Deserialize)]
struct StoredPoint {
    level: String,
    text_en: String,
    text_ku: String,
}

/// One source as it is kept in the `sources` JSON column.
#[derive(Serialize, Deserialize)]
struct StoredSource {
    title: String,
    url: String,
}

pub(super) fn stored_points(points: &[BriefPoint]) -> serde_json::Value {
    let points: Vec<StoredPoint> = points
        .iter()
        .map(|point| StoredPoint {
            level: (*point.level()).into(),
            text_en: point.text_en().into(),
            text_ku: point.text_ku().into(),
        })
        .collect();

    serde_json::json!(points)
}

pub(super) fn stored_sources(sources: &[BriefSource]) -> serde_json::Value {
    let sources: Vec<StoredSource> = sources
        .iter()
        .map(|source| StoredSource {
            title: source.title().into(),
            url: source.url().into(),
        })
        .collect();

    serde_json::json!(sources)
}

fn points_from(stored: serde_json::Value) -> Result<Vec<BriefPoint>, AppError> {
    let points: Vec<StoredPoint> = serde_json::from_value(stored).map_err(|error| {
        GlobalAppError::MissingValue(format!("Stored brief points are not readable: {error}"))
    })?;

    points
        .into_iter()
        .map(|point| {
            Ok(BriefPoint::new(
                PointLevel::try_from(point.level.as_str())?,
                PointText::new(point.text_en)?,
                PointText::new(point.text_ku)?,
            ))
        })
        .collect()
}

fn sources_from(stored: serde_json::Value) -> Result<Vec<BriefSource>, AppError> {
    let sources: Vec<StoredSource> = serde_json::from_value(stored).map_err(|error| {
        GlobalAppError::MissingValue(format!("Stored brief sources are not readable: {error}"))
    })?;

    sources
        .into_iter()
        .map(|source| {
            Ok(BriefSource::new(
                SourceTitle::new(source.title)?,
                SourceUrl::new(source.url)?,
            ))
        })
        .collect()
}

impl TryFrom<daily_briefs::Model> for DailyBrief {
    type Error = AppError;

    fn try_from(model: daily_briefs::Model) -> Result<Self, Self::Error> {
        Ok(DailyBrief::rehydrate(
            model.id,
            model.day,
            BriefScope::new(model.scope)?,
            Headline::new(model.headline_en)?,
            Headline::new(model.headline_ku)?,
            BriefSummary::new(model.summary_en)?,
            BriefSummary::new(model.summary_ku)?,
            points_from(model.points)?,
            sources_from(model.sources)?,
            Author::new(model.author)?,
            model.generated_at.and_utc(),
            model.updated_at.and_utc(),
        ))
    }
}

impl From<&DailyBrief> for daily_briefs::ActiveModel {
    fn from(brief: &DailyBrief) -> Self {
        daily_briefs::ActiveModel {
            id: match *brief.id() {
                Some(id) => Set(id),
                None => NotSet,
            },
            day: Set(*brief.day()),
            scope: Set(brief.scope().into()),
            headline_en: Set(brief.headline_en().into()),
            headline_ku: Set(brief.headline_ku().into()),
            summary_en: Set(brief.summary_en().into()),
            summary_ku: Set(brief.summary_ku().into()),
            points: Set(stored_points(brief.points())),
            sources: Set(stored_sources(brief.sources())),
            author: Set(brief.author().into()),
            generated_at: Set(brief.generated_at().naive_utc()),
            updated_at: Set(brief.updated_at().naive_utc()),
        }
    }
}

/// A farm's district with the moment it was recorded, ready to be written.
impl From<(&FarmZone, NaiveDateTime)> for farm_brief_zones::ActiveModel {
    fn from((zone, recorded_at): (&FarmZone, NaiveDateTime)) -> Self {
        farm_brief_zones::ActiveModel {
            id: NotSet,
            farm_id: Set(*zone.farm_id()),
            zone_slug: Set(zone.zone_slug().into()),
            updated_at: Set(recorded_at),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn point(level: PointLevel, text_en: &str) -> BriefPoint {
        BriefPoint::new(
            level,
            PointText::new(text_en.to_string()).expect("text"),
            PointText::new("باران دواکەوتووە".to_string()).expect("text"),
        )
    }

    #[test]
    fn points_survive_the_trip_through_the_json_column_in_order() {
        let points = vec![
            point(PointLevel::Alarm, "Two fires"),
            point(PointLevel::Info, "Prices steady"),
        ];

        assert_eq!(points_from(stored_points(&points)).expect("points"), points);
    }

    #[test]
    fn sources_survive_the_trip_through_the_json_column() {
        let sources = vec![BriefSource::new(
            SourceTitle::new("FAO crop calendar".to_string()).expect("title"),
            SourceUrl::new("https://example.org/calendar".to_string()).expect("url"),
        )];

        assert_eq!(
            sources_from(stored_sources(&sources)).expect("sources"),
            sources
        );
    }

    #[test]
    fn an_empty_list_is_stored_as_an_empty_array_not_as_null() {
        assert_eq!(stored_points(&[]), serde_json::json!([]));
        assert_eq!(stored_sources(&[]), serde_json::json!([]));
    }

    #[test]
    fn a_json_column_that_is_not_what_was_written_is_an_error_not_an_empty_brief() {
        assert!(points_from(serde_json::json!({"level": "info"})).is_err());
        assert!(points_from(serde_json::json!([{"level": "info"}])).is_err());
        assert!(
            points_from(serde_json::json!([
                {"level": "danger", "text_en": "a", "text_ku": "b"}
            ]))
            .is_err()
        );
        assert!(sources_from(serde_json::json!([{"title": "a", "url": "nowhere"}])).is_err());
    }
}
