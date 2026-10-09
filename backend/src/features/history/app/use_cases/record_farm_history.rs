use std::sync::Arc;

use chrono::{DateTime, Utc};

use crate::{
    app::AppError as GlobalAppError,
    features::history::{
        app::{AppError, HistoryFarmDirectory, HistoryRepository},
        domain::{HistorySource, Metric, Month, SeriesUpload},
    },
};

pub struct RecordFarmHistoryInput {
    pub farm_id: i32,
    pub metric: Metric,
    pub unit: String,
    pub source: HistorySource,
    pub as_of: DateTime<Utc>,
    pub points: Vec<(Month, f64)>,
}

pub struct RecordFarmHistoryUseCase {
    repository: Arc<dyn HistoryRepository>,
    directory: Arc<dyn HistoryFarmDirectory>,
}

impl RecordFarmHistoryUseCase {
    pub fn new(
        repository: Arc<dyn HistoryRepository>,
        directory: Arc<dyn HistoryFarmDirectory>,
    ) -> Self {
        Self {
            repository,
            directory,
        }
    }

    /// Stores the months a data job pushes for one metric of a farm. Months
    /// already stored that the push does not name are kept: the daily
    /// update sends only the recent ones. A push read from its source before
    /// the stored one, such as a slow copy of the job, replaces nothing and
    /// only fills months that are missing. The farm is looked up only to
    /// answer "not found" for one that does not exist; months written for a
    /// farm deleted at that very moment are never shown to anyone, because
    /// every read starts from a farm that exists.
    pub async fn execute(
        &self,
        input: RecordFarmHistoryInput,
        now: DateTime<Utc>,
    ) -> Result<SeriesUpload, AppError> {
        let upload = SeriesUpload::new(
            input.farm_id,
            input.metric,
            &input.unit,
            input.source,
            input.as_of,
            input.points,
            now,
        )?;

        if !self.directory.exists(*upload.farm_id()).await? {
            return Err(GlobalAppError::NotFound.into());
        }

        let newest = self.repository.store(&upload).await?;

        tracing::info!(
            farm_id = *upload.farm_id(),
            metric = %String::from(*upload.metric()),
            points = upload.points().len(),
            as_of = %upload.as_of(),
            replaced_stored_months = newest,
            "farm history recorded"
        );

        Ok(upload)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::history::{
        app::testing::{Call, FARM_ID, Fakes, a_month, a_series, a_source, now},
        domain::HistoryError,
    };

    fn input(unit: &str, points: &[(&str, f64)]) -> RecordFarmHistoryInput {
        RecordFarmHistoryInput {
            farm_id: FARM_ID,
            metric: Metric::RainMm,
            unit: unit.to_string(),
            source: a_source(),
            as_of: now(),
            points: points
                .iter()
                .map(|(raw, value)| (a_month(raw), *value))
                .collect(),
        }
    }

    fn use_case(fakes: &Fakes) -> RecordFarmHistoryUseCase {
        RecordFarmHistoryUseCase::new(Arc::new(fakes.clone()), Arc::new(fakes.clone()))
    }

    #[tokio::test]
    async fn stores_the_months_for_an_existing_farm_in_one_call() {
        let fakes = Fakes::new().with_site(FARM_ID);

        let stored = use_case(&fakes)
            .execute(input("mm", &[("2026-08", 0.0), ("2026-09", 1.4)]), now())
            .await
            .expect("stored");

        assert_eq!(stored.points().len(), 2);
        assert_eq!(
            fakes.calls(),
            vec![
                Call::Exists { farm_id: FARM_ID },
                Call::Store {
                    farm_id: FARM_ID,
                    metric: Metric::RainMm,
                    points: 2,
                },
            ],
            "what is stored already is not read before the write"
        );
    }

    #[tokio::test]
    async fn months_the_push_does_not_name_are_kept_and_named_ones_replaced() {
        let fakes = Fakes::new().with_site(FARM_ID).with_stored(
            FARM_ID,
            a_series(Metric::RainMm, &[("2016-10", 31.2), ("2026-08", 0.3)]),
        );

        use_case(&fakes)
            .execute(input("mm", &[("2026-08", 0.0), ("2026-09", 1.4)]), now())
            .await
            .expect("stored");

        let kept = fakes.stored(FARM_ID, Metric::RainMm).expect("series");
        let points: Vec<(String, f64)> = kept
            .points()
            .iter()
            .map(|point| (point.month().to_string(), *point.value()))
            .collect();

        assert_eq!(
            points,
            vec![
                ("2016-10".to_string(), 31.2),
                ("2026-08".to_string(), 0.0),
                ("2026-09".to_string(), 1.4),
            ],
            "the daily update must not wipe the ten years before it"
        );
    }

    #[tokio::test]
    async fn a_push_read_before_the_stored_one_fills_holes_and_replaces_nothing() {
        // The stored series was read at `now()`.
        let fakes = Fakes::new()
            .with_site(FARM_ID)
            .with_stored(FARM_ID, a_series(Metric::RainMm, &[("2026-09", 1.4)]));

        use_case(&fakes)
            .execute(
                RecordFarmHistoryInput {
                    as_of: now() - chrono::Duration::hours(3),
                    ..input("mm", &[("2026-08", 0.3), ("2026-09", 9.9)])
                },
                now(),
            )
            .await
            .expect("stored");

        let kept = fakes.stored(FARM_ID, Metric::RainMm).expect("series");
        let points: Vec<(String, f64)> = kept
            .points()
            .iter()
            .map(|point| (point.month().to_string(), *point.value()))
            .collect();

        assert_eq!(
            points,
            vec![("2026-08".to_string(), 0.3), ("2026-09".to_string(), 1.4)],
            "a slow copy of the job must not put an older value back"
        );
        assert_eq!(*kept.as_of(), now(), "the newer facts stay");
    }

    #[tokio::test]
    async fn the_same_push_twice_leaves_the_same_series() {
        let fakes = Fakes::new().with_site(FARM_ID);
        let use_case = use_case(&fakes);

        for _ in 0..2 {
            use_case
                .execute(input("mm", &[("2026-09", 1.4)]), now())
                .await
                .expect("stored");
        }

        assert_eq!(
            fakes
                .stored(FARM_ID, Metric::RainMm)
                .expect("series")
                .points()
                .len(),
            1
        );
    }

    #[tokio::test]
    async fn a_farm_that_does_not_exist_is_not_found_and_nothing_is_written() {
        let fakes = Fakes::new();

        let result = use_case(&fakes)
            .execute(input("mm", &[("2026-09", 1.4)]), now())
            .await;

        assert!(matches!(
            result,
            Err(AppError::GlobalAppError(GlobalAppError::NotFound))
        ));
        assert_eq!(fakes.calls(), vec![Call::Exists { farm_id: FARM_ID }]);
    }

    #[tokio::test]
    async fn a_push_that_breaks_a_rule_is_refused_before_anything_is_asked() {
        let fakes = Fakes::new().with_site(FARM_ID);
        let use_case = use_case(&fakes);

        assert!(matches!(
            use_case
                .execute(input("cm", &[("2026-09", 1.4)]), now())
                .await,
            Err(AppError::History(HistoryError::UnitMismatch { .. }))
        ));
        assert!(matches!(
            use_case
                .execute(input("mm", &[("2026-09", 1.4), ("2026-09", 1.5)]), now())
                .await,
            Err(AppError::History(HistoryError::DuplicateMonth(_)))
        ));
        assert!(matches!(
            use_case
                .execute(input("mm", &[("2026-09", -3.0)]), now())
                .await,
            Err(AppError::History(HistoryError::ValueOutOfRange { .. }))
        ));
        assert!(fakes.calls().is_empty());
    }

    #[tokio::test]
    async fn a_repository_failure_surfaces() {
        let fakes = Fakes::new().with_site(FARM_ID).failing();

        assert!(
            use_case(&fakes)
                .execute(input("mm", &[("2026-09", 1.4)]), now())
                .await
                .is_err()
        );
    }
}
