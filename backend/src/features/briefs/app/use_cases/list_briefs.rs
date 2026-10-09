use std::sync::Arc;

use chrono::{NaiveDate, Utc};

use crate::features::briefs::{
    app::{AppError, BriefRepository},
    domain::{BriefScope, DailyBrief, DayRange},
};

pub struct ListBriefsInput {
    pub scope: BriefScope,
    /// None = far enough back to hold the last fourteen days.
    pub from: Option<NaiveDate>,
    /// None = up to now.
    pub to: Option<NaiveDate>,
}

pub struct ListBriefsUseCase {
    repository: Arc<dyn BriefRepository>,
}

impl ListBriefsUseCase {
    pub fn new(repository: Arc<dyn BriefRepository>) -> Self {
        Self { repository }
    }

    /// Returns the scope's briefs for the days asked for, newest first.
    pub async fn execute(&self, input: ListBriefsInput) -> Result<Vec<DailyBrief>, AppError> {
        self.execute_on(input, Utc::now().date_naive()).await
    }

    /// The range is settled before anything is read, so a range that is too
    /// long never reaches the database.
    pub async fn execute_on(
        &self,
        input: ListBriefsInput,
        today: NaiveDate,
    ) -> Result<Vec<DailyBrief>, AppError> {
        let range = DayRange::resolve(input.from, input.to, today)?;

        let briefs = self.repository.find_between(&input.scope, &range).await?;

        tracing::debug!(
            scope = input.scope.as_str(),
            from = %range.from(),
            to = %range.to(),
            returned = briefs.len(),
            "briefs listed"
        );

        Ok(briefs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::briefs::{
        app::testing::{Call, Fakes, a_brief, a_day, a_scope},
        domain::BriefError,
    };

    fn input(from: Option<u32>, to: Option<u32>) -> ListBriefsInput {
        ListBriefsInput {
            scope: a_scope("region"),
            from: from.map(a_day),
            to: to.map(a_day),
        }
    }

    #[tokio::test]
    async fn lists_the_days_of_the_range_newest_first_for_that_scope_only() {
        let fakes = Fakes::new()
            .with_stored(a_brief(3, "region"))
            .with_stored(a_brief(5, "region"))
            .with_stored(a_brief(4, "region"))
            .with_stored(a_brief(4, "kalar"))
            .with_stored(a_brief(6, "region"));
        let use_case = ListBriefsUseCase::new(Arc::new(fakes.clone()));

        let briefs = use_case
            .execute_on(input(Some(3), Some(5)), a_day(9))
            .await
            .expect("briefs");

        let days: Vec<NaiveDate> = briefs.iter().map(|brief| *brief.day()).collect();

        assert_eq!(days, vec![a_day(5), a_day(4), a_day(3)]);
        assert_eq!(
            fakes.calls(),
            vec![Call::FindBetween {
                scope: "region".to_string(),
                from: a_day(3),
                to: a_day(5),
            }]
        );
    }

    #[tokio::test]
    async fn with_no_range_it_asks_for_the_last_fourteen_days() {
        let fakes = Fakes::new();
        let use_case = ListBriefsUseCase::new(Arc::new(fakes.clone()));

        use_case
            .execute_on(input(None, None), a_day(20))
            .await
            .expect("briefs");

        assert_eq!(
            fakes.calls(),
            vec![Call::FindBetween {
                scope: "region".to_string(),
                from: a_day(7),
                to: a_day(21),
            }]
        );
    }

    #[tokio::test]
    async fn no_briefs_in_the_range_is_an_empty_list_not_an_error() {
        let use_case = ListBriefsUseCase::new(Arc::new(Fakes::new()));

        assert!(
            use_case
                .execute_on(input(None, None), a_day(9))
                .await
                .expect("briefs")
                .is_empty()
        );
    }

    #[tokio::test]
    async fn a_range_that_ends_before_it_starts_reads_nothing() {
        let fakes = Fakes::new();
        let use_case = ListBriefsUseCase::new(Arc::new(fakes.clone()));

        let result = use_case.execute_on(input(Some(5), Some(3)), a_day(9)).await;

        assert!(matches!(
            result,
            Err(AppError::Brief(BriefError::RangeEndsBeforeItStarts))
        ));
        assert!(fakes.calls().is_empty());
    }

    #[tokio::test]
    async fn a_range_longer_than_ninety_two_days_reads_nothing() {
        let fakes = Fakes::new();
        let use_case = ListBriefsUseCase::new(Arc::new(fakes.clone()));

        let result = use_case
            .execute_on(
                ListBriefsInput {
                    scope: a_scope("region"),
                    from: NaiveDate::from_ymd_opt(2026, 1, 1),
                    to: Some(a_day(9)),
                },
                a_day(9),
            )
            .await;

        assert!(matches!(
            result,
            Err(AppError::Brief(BriefError::RangeTooLong { .. }))
        ));
        assert!(fakes.calls().is_empty());
    }

    #[tokio::test]
    async fn a_repository_failure_surfaces() {
        let use_case = ListBriefsUseCase::new(Arc::new(Fakes::new().failing()));

        assert!(use_case.execute(input(None, None)).await.is_err());
    }
}
