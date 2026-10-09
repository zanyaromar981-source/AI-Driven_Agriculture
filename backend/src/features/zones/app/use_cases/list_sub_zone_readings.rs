use std::sync::Arc;

use chrono::Utc;

use super::locate::sub_zone_named;
use crate::{
    app::Pagination,
    features::zones::{
        app::{AppError, ZoneRepository},
        domain::{Month, MonthRange, SubZoneReading, ZoneSlug},
    },
};

pub struct ListSubZoneReadingsInput {
    pub zone_slug: ZoneSlug,
    pub sub_zone_slug: ZoneSlug,
    pub from: Option<Month>,
    pub to: Option<Month>,
    pub pagination: Pagination,
}

pub struct ListSubZoneReadingsUseCase {
    repository: Arc<dyn ZoneRepository>,
}

impl ListSubZoneReadingsUseCase {
    pub fn new(repository: Arc<dyn ZoneRepository>) -> Self {
        Self { repository }
    }

    /// One page of the sub-zone's stored readings, newest month first, and
    /// how many the range holds in all.
    pub async fn execute(
        &self,
        input: ListSubZoneReadingsInput,
    ) -> Result<(Vec<SubZoneReading>, u64), AppError> {
        let range = MonthRange::new(
            input.from,
            input.to,
            Month::containing(Utc::now().date_naive()),
        )?;

        let sub_zone = sub_zone_named(
            self.repository.as_ref(),
            &input.zone_slug,
            &input.sub_zone_slug,
        )
        .await?;

        self.repository
            .find_sub_zone_readings_in_range(*sub_zone.id(), range, &input.pagination)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        app::AppError as GlobalAppError,
        features::zones::app::testing::{
            FakeZoneRepository, RepositoryCall, a_month, a_slug, a_sub_zone_reading,
        },
    };

    fn input(zone: &str, sub_zone: &str) -> ListSubZoneReadingsInput {
        ListSubZoneReadingsInput {
            zone_slug: a_slug(zone),
            sub_zone_slug: a_slug(sub_zone),
            from: Some(a_month("2026-01")),
            to: Some(a_month("2026-03")),
            pagination: Pagination::new(1, 100),
        }
    }

    fn seeded() -> FakeZoneRepository {
        FakeZoneRepository::seeded().with_sub_zone_readings(vec![
            a_sub_zone_reading(3, "2026-01", 50),
            a_sub_zone_reading(3, "2026-03", 70),
            a_sub_zone_reading(3, "2026-04", 80),
            a_sub_zone_reading(2, "2026-02", 10),
        ])
    }

    #[tokio::test]
    async fn lists_that_sub_zones_readings_in_the_range_newest_first() {
        let repository = seeded();
        let use_case = ListSubZoneReadingsUseCase::new(Arc::new(repository.clone()));

        let (readings, count) = use_case
            .execute(input("chamchamal", "sangaw"))
            .await
            .expect("listed");

        assert_eq!(
            readings
                .iter()
                .map(|reading| String::from(reading.month()))
                .collect::<Vec<_>>(),
            vec!["2026-03", "2026-01"]
        );
        assert_eq!(count, 2);
        assert_eq!(
            repository.calls().last(),
            Some(&RepositoryCall::FindSubZoneReadingsInRange {
                sub_zone_id: 3,
                from: "2026-01".to_string(),
                to: "2026-03".to_string(),
                page: 1,
                rows_per_page: 100,
            })
        );
    }

    #[tokio::test]
    async fn a_sub_zone_of_another_zone_is_not_found() {
        let use_case = ListSubZoneReadingsUseCase::new(Arc::new(seeded()));

        assert!(matches!(
            use_case.execute(input("kalar", "sangaw")).await,
            Err(AppError::GlobalAppError(GlobalAppError::NotFound))
        ));
    }

    #[tokio::test]
    async fn an_unknown_zone_is_not_found() {
        let use_case = ListSubZoneReadingsUseCase::new(Arc::new(seeded()));

        assert!(matches!(
            use_case.execute(input("atlantis", "sangaw")).await,
            Err(AppError::GlobalAppError(GlobalAppError::NotFound))
        ));
    }
}
