use std::sync::Arc;

use chrono::{NaiveDate, Utc};

use crate::features::dams::{
    app::{AppError, DamRepository},
    domain::{Dam, DamReading, DamSlug, HistoryRange},
};

pub struct ViewDamHistoryInput {
    pub slug: DamSlug,
    pub from: Option<NaiveDate>,
    pub to: Option<NaiveDate>,
}

pub struct ViewDamHistoryUseCase {
    repository: Arc<dyn DamRepository>,
}

impl ViewDamHistoryUseCase {
    pub fn new(repository: Arc<dyn DamRepository>) -> Self {
        Self { repository }
    }

    /// Returns the dam and its readings in the asked range, oldest first.
    pub async fn execute(
        &self,
        input: ViewDamHistoryInput,
    ) -> Result<(Dam, Vec<DamReading>), AppError> {
        let range = HistoryRange::new(input.from, input.to, Utc::now().date_naive())?;

        let dam = self
            .repository
            .find_by_slug(&input.slug)
            .await?
            .ok_or_else(|| AppError::DamNotFound(String::from(&input.slug)))?;

        let readings = self
            .repository
            .find_readings_between(*dam.id(), range.from(), range.to())
            .await?;

        tracing::debug!(
            dam = input.slug.as_str(),
            from = %range.from(),
            to = %range.to(),
            returned = readings.len(),
            "dam history viewed"
        );

        Ok((dam, readings))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::dams::{
        app::testing::{FakeDamRepository, RepositoryCall, a_reading, day, dukan, slug},
        domain::DamError,
    };

    fn input(from: Option<NaiveDate>, to: Option<NaiveDate>) -> ViewDamHistoryInput {
        ViewDamHistoryInput {
            slug: slug("dukan"),
            from,
            to,
        }
    }

    #[tokio::test]
    async fn returns_the_readings_inside_the_range_oldest_first() {
        let repository = FakeDamRepository::holding(
            vec![dukan()],
            vec![
                a_reading(&dukan(), day(2024, 3, 1), 70.0),
                a_reading(&dukan(), day(2024, 1, 1), 60.0),
                a_reading(&dukan(), day(2023, 12, 31), 50.0),
                a_reading(&dukan(), day(2024, 4, 1), 80.0),
            ],
        );
        let use_case = ViewDamHistoryUseCase::new(Arc::new(repository.clone()));

        let (dam, readings) = use_case
            .execute(input(Some(day(2024, 1, 1)), Some(day(2024, 3, 1))))
            .await
            .expect("history");

        assert_eq!(dam.slug().as_str(), "dukan");
        assert_eq!(
            readings
                .iter()
                .map(|reading| *reading.day())
                .collect::<Vec<_>>(),
            vec![day(2024, 1, 1), day(2024, 3, 1)],
            "both ends of the range are included"
        );
        assert_eq!(
            repository.calls(),
            vec![
                RepositoryCall::FindBySlug {
                    slug: "dukan".to_string()
                },
                RepositoryCall::FindReadingsBetween {
                    dam_id: 1,
                    from: day(2024, 1, 1),
                    to: day(2024, 3, 1),
                },
            ]
        );
    }

    #[tokio::test]
    async fn without_dates_the_last_ten_years_are_asked_for() {
        let repository = FakeDamRepository::holding(vec![dukan()], vec![]);
        let use_case = ViewDamHistoryUseCase::new(Arc::new(repository.clone()));

        use_case.execute(input(None, None)).await.expect("history");

        let asked = repository.calls().into_iter().find_map(|call| match call {
            RepositoryCall::FindReadingsBetween { from, to, .. } => Some((from, to)),
            _ => None,
        });
        let (from, to) = asked.expect("the readings were asked for");

        assert_eq!(to, Utc::now().date_naive());
        assert_eq!(
            Some(from),
            to.checked_sub_months(chrono::Months::new(120)),
            "the default range is ten years"
        );
    }

    #[tokio::test]
    async fn an_unknown_dam_is_not_found() {
        let repository = FakeDamRepository::holding(vec![dukan()], vec![]);
        let use_case = ViewDamHistoryUseCase::new(Arc::new(repository.clone()));

        let result = use_case
            .execute(ViewDamHistoryInput {
                slug: slug("mosul"),
                from: None,
                to: None,
            })
            .await;

        assert!(matches!(result, Err(AppError::DamNotFound(slug)) if slug == "mosul"));
        assert_eq!(
            repository.calls(),
            vec![RepositoryCall::FindBySlug {
                slug: "mosul".to_string()
            }]
        );
    }

    #[tokio::test]
    async fn a_backwards_range_is_refused_before_any_query() {
        let repository = FakeDamRepository::holding(vec![dukan()], vec![]);
        let use_case = ViewDamHistoryUseCase::new(Arc::new(repository.clone()));

        let result = use_case
            .execute(input(Some(day(2024, 3, 1)), Some(day(2024, 1, 1))))
            .await;

        assert!(matches!(
            result,
            Err(AppError::Dam(DamError::RangeEndsBeforeItStarts))
        ));
        assert!(repository.calls().is_empty());
    }

    #[tokio::test]
    async fn a_repository_failure_surfaces() {
        let use_case = ViewDamHistoryUseCase::new(Arc::new(FakeDamRepository::failing()));

        assert!(use_case.execute(input(None, None)).await.is_err());
    }
}
