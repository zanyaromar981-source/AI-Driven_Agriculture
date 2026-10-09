use std::sync::Arc;

use chrono::Utc;

use super::locate::zone_named;
use crate::{
    app::Pagination,
    features::zones::{
        app::{AppError, ZoneRepository},
        domain::{Month, MonthRange, ZoneReading, ZoneSlug},
    },
};

pub struct ListZoneReadingsInput {
    pub zone_slug: ZoneSlug,
    pub from: Option<Month>,
    pub to: Option<Month>,
    pub pagination: Pagination,
}

pub struct ListZoneReadingsUseCase {
    repository: Arc<dyn ZoneRepository>,
}

impl ListZoneReadingsUseCase {
    pub fn new(repository: Arc<dyn ZoneRepository>) -> Self {
        Self { repository }
    }

    /// One page of the zone's stored readings, newest month first, and how
    /// many the range holds in all.
    pub async fn execute(
        &self,
        input: ListZoneReadingsInput,
    ) -> Result<(Vec<ZoneReading>, u64), AppError> {
        let range = MonthRange::new(
            input.from,
            input.to,
            Month::containing(Utc::now().date_naive()),
        )?;

        let zone = zone_named(self.repository.as_ref(), &input.zone_slug).await?;

        self.repository
            .find_readings_by_zone_in_range(*zone.id(), range, &input.pagination)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        app::AppError as GlobalAppError,
        features::zones::{
            app::testing::{FakeZoneRepository, RepositoryCall, a_month, a_reading, a_slug},
            domain::ZoneError,
        },
    };

    fn input(slug: &str, from: Option<&str>, to: Option<&str>) -> ListZoneReadingsInput {
        ListZoneReadingsInput {
            zone_slug: a_slug(slug),
            from: from.map(a_month),
            to: to.map(a_month),
            pagination: Pagination::new(1, 100),
        }
    }

    fn seeded() -> FakeZoneRepository {
        FakeZoneRepository::seeded().with_readings(vec![
            a_reading(2, "2025-11", 40),
            a_reading(2, "2026-02", 60),
            a_reading(2, "2026-01", 50),
            a_reading(2, "2026-04", 70),
            a_reading(1, "2026-02", 10),
        ])
    }

    #[tokio::test]
    async fn lists_that_zones_readings_in_the_range_newest_first() {
        let repository = seeded();
        let use_case = ListZoneReadingsUseCase::new(Arc::new(repository.clone()));

        let (readings, count) = use_case
            .execute(input("kalar", Some("2026-01"), Some("2026-03")))
            .await
            .expect("listed");

        assert_eq!(
            readings
                .iter()
                .map(|reading| String::from(reading.month()))
                .collect::<Vec<_>>(),
            vec!["2026-02", "2026-01"]
        );
        assert_eq!(count, 2);
        assert_eq!(
            repository.calls(),
            vec![
                RepositoryCall::FindZoneBySlug {
                    slug: "kalar".to_string(),
                },
                RepositoryCall::FindReadingsByZoneInRange {
                    zone_id: 2,
                    from: "2026-01".to_string(),
                    to: "2026-03".to_string(),
                    page: 1,
                    rows_per_page: 100,
                },
            ]
        );
    }

    #[tokio::test]
    async fn the_count_is_of_the_whole_range_not_of_the_page() {
        let use_case = ListZoneReadingsUseCase::new(Arc::new(seeded()));

        let (readings, count) = use_case
            .execute(ListZoneReadingsInput {
                pagination: Pagination::new(2, 1),
                ..input("kalar", Some("2025-01"), Some("2026-12"))
            })
            .await
            .expect("listed");

        assert_eq!(count, 4);
        assert_eq!(readings.len(), 1);
        assert_eq!(String::from(readings[0].month()), "2026-02");
    }

    #[tokio::test]
    async fn with_no_bounds_it_asks_for_the_24_months_ending_now() {
        let repository = seeded();
        let use_case = ListZoneReadingsUseCase::new(Arc::new(repository.clone()));
        let current = Month::containing(Utc::now().date_naive());

        use_case
            .execute(input("kalar", None, None))
            .await
            .expect("listed");

        assert_eq!(
            repository.calls().last(),
            Some(&RepositoryCall::FindReadingsByZoneInRange {
                zone_id: 2,
                from: String::from(current.months_earlier(23).expect("month")),
                to: String::from(current),
                page: 1,
                rows_per_page: 100,
            })
        );
    }

    #[tokio::test]
    async fn a_from_after_the_to_is_refused_before_anything_is_read() {
        let repository = seeded();
        let use_case = ListZoneReadingsUseCase::new(Arc::new(repository.clone()));

        let result = use_case
            .execute(input("kalar", Some("2026-05"), Some("2026-03")))
            .await;

        assert!(matches!(
            result,
            Err(AppError::Zone(ZoneError::FromAfterTo))
        ));
        assert!(repository.calls().is_empty());
    }

    #[tokio::test]
    async fn an_unknown_zone_is_not_found() {
        let use_case = ListZoneReadingsUseCase::new(Arc::new(seeded()));

        assert!(matches!(
            use_case.execute(input("atlantis", None, None)).await,
            Err(AppError::GlobalAppError(GlobalAppError::NotFound))
        ));
    }
}
