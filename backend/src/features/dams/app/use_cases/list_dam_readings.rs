use std::sync::Arc;

use chrono::NaiveDate;

use crate::{
    app::Pagination,
    features::dams::{
        app::{AppError, DamRepository},
        domain::{DamError, DamReading, DamSlug},
    },
};

pub struct ListDamReadingsInput {
    pub slug: DamSlug,
    /// None = from the first reading there is.
    pub from: Option<NaiveDate>,
    /// None = up to the last reading there is.
    pub to: Option<NaiveDate>,
    pub pagination: Pagination,
}

pub struct ListDamReadingsUseCase {
    repository: Arc<dyn DamRepository>,
}

impl ListDamReadingsUseCase {
    pub fn new(repository: Arc<dyn DamRepository>) -> Self {
        Self { repository }
    }

    /// Returns one page of the dam's stored readings, newest day first, and
    /// how many match in all. Unlike the public history, a missing date is
    /// not filled in: an editing screen must be able to reach every reading.
    pub async fn execute(
        &self,
        input: ListDamReadingsInput,
    ) -> Result<(Vec<DamReading>, u64), AppError> {
        if let (Some(from), Some(to)) = (input.from, input.to)
            && from > to
        {
            return Err(DamError::RangeEndsBeforeItStarts.into());
        }

        let dam = self
            .repository
            .find_by_slug(&input.slug)
            .await?
            .ok_or_else(|| AppError::DamNotFound(String::from(&input.slug)))?;

        let (readings, count) = self
            .repository
            .find_readings_page(*dam.id(), input.from, input.to, &input.pagination)
            .await?;

        tracing::debug!(
            dam = input.slug.as_str(),
            returned = readings.len(),
            count,
            "dam readings listed"
        );

        Ok((readings, count))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::dams::app::testing::{
        FakeDamRepository, RepositoryCall, a_reading, darbandikhan, day, dukan, slug,
    };

    fn repository() -> FakeDamRepository {
        FakeDamRepository::holding(
            vec![dukan(), darbandikhan()],
            vec![
                a_reading(&dukan(), day(2026, 8, 1), 45.0),
                a_reading(&dukan(), day(2026, 10, 1), 38.5),
                a_reading(&dukan(), day(2026, 9, 1), 40.0),
                a_reading(&darbandikhan(), day(2026, 10, 1), 20.0),
            ],
        )
    }

    fn input(
        from: Option<NaiveDate>,
        to: Option<NaiveDate>,
        pagination: Pagination,
    ) -> ListDamReadingsInput {
        ListDamReadingsInput {
            slug: slug("dukan"),
            from,
            to,
            pagination,
        }
    }

    #[tokio::test]
    async fn the_named_dams_readings_come_newest_first_with_the_count_of_all() {
        let repository = repository();
        let use_case = ListDamReadingsUseCase::new(Arc::new(repository.clone()));

        let (readings, count) = use_case
            .execute(input(None, None, Pagination::new(1, 2)))
            .await
            .expect("readings");

        assert_eq!(
            readings
                .iter()
                .map(|reading| *reading.day())
                .collect::<Vec<_>>(),
            vec![day(2026, 10, 1), day(2026, 9, 1)]
        );
        assert_eq!(count, 3, "the count covers every page, and only this dam");
        assert_eq!(
            repository.calls(),
            vec![
                RepositoryCall::FindBySlug {
                    slug: "dukan".to_string()
                },
                RepositoryCall::FindReadingsPage {
                    dam_id: 1,
                    from: None,
                    to: None,
                    page: 1,
                    rows_per_page: 2,
                },
            ],
            "missing dates must reach the repository as open ends, not as a default range"
        );
    }

    #[tokio::test]
    async fn the_dates_narrow_the_list_and_include_both_ends() {
        let use_case = ListDamReadingsUseCase::new(Arc::new(repository()));

        let (readings, count) = use_case
            .execute(input(
                Some(day(2026, 8, 1)),
                Some(day(2026, 9, 1)),
                Pagination::new(1, 100),
            ))
            .await
            .expect("readings");

        assert_eq!(
            readings
                .iter()
                .map(|reading| *reading.day())
                .collect::<Vec<_>>(),
            vec![day(2026, 9, 1), day(2026, 8, 1)]
        );
        assert_eq!(count, 2);
    }

    #[tokio::test]
    async fn an_unknown_dam_is_not_found() {
        let repository = repository();
        let use_case = ListDamReadingsUseCase::new(Arc::new(repository.clone()));

        let result = use_case
            .execute(ListDamReadingsInput {
                slug: slug("mosul"),
                from: None,
                to: None,
                pagination: Pagination::new(1, 100),
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
        let repository = repository();
        let use_case = ListDamReadingsUseCase::new(Arc::new(repository.clone()));

        let result = use_case
            .execute(input(
                Some(day(2026, 9, 1)),
                Some(day(2026, 8, 1)),
                Pagination::new(1, 100),
            ))
            .await;

        assert!(matches!(
            result,
            Err(AppError::Dam(DamError::RangeEndsBeforeItStarts))
        ));
        assert!(repository.calls().is_empty());
    }

    #[tokio::test]
    async fn a_repository_failure_surfaces() {
        let use_case = ListDamReadingsUseCase::new(Arc::new(FakeDamRepository::failing()));

        assert!(
            use_case
                .execute(input(None, None, Pagination::new(1, 100)))
                .await
                .is_err()
        );
    }
}
