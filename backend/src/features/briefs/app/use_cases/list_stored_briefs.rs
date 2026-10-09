use std::sync::Arc;

use chrono::NaiveDate;

use crate::{
    app::Pagination,
    features::briefs::{
        app::{AppError, BriefRepository},
        domain::{BriefError, BriefScope, DailyBrief},
    },
};

pub struct ListStoredBriefsInput {
    /// None = every scope.
    pub scope: Option<BriefScope>,
    /// None = from the first brief there is.
    pub from: Option<NaiveDate>,
    /// None = up to the last brief there is.
    pub to: Option<NaiveDate>,
    pub pagination: Pagination,
}

pub struct ListStoredBriefsUseCase {
    repository: Arc<dyn BriefRepository>,
}

impl ListStoredBriefsUseCase {
    pub fn new(repository: Arc<dyn BriefRepository>) -> Self {
        Self { repository }
    }

    /// Returns one page of the stored briefs for the dashboard, newest day
    /// first, and how many match in all. Unlike the public list, a missing
    /// date is not filled in and the range has no longest length: an editing
    /// screen must be able to reach every brief, and the page bounds the
    /// work instead.
    pub async fn execute(
        &self,
        input: ListStoredBriefsInput,
    ) -> Result<(Vec<DailyBrief>, u64), AppError> {
        if let (Some(from), Some(to)) = (input.from, input.to)
            && from > to
        {
            return Err(BriefError::RangeEndsBeforeItStarts.into());
        }

        let (briefs, count) = self
            .repository
            .find_page(
                input.scope.as_ref(),
                input.from,
                input.to,
                &input.pagination,
            )
            .await?;

        tracing::debug!(
            scope = input.scope.as_ref().map(|scope| scope.as_str()),
            returned = briefs.len(),
            count,
            "stored briefs listed"
        );

        Ok((briefs, count))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::briefs::app::testing::{Call, Fakes, a_brief, a_day, a_scope};

    fn fakes() -> Fakes {
        Fakes::new()
            .with_stored(a_brief(7, "region"))
            .with_stored(a_brief(9, "region"))
            .with_stored(a_brief(9, "kalar"))
            .with_stored(a_brief(8, "region"))
    }

    fn input(scope: Option<&str>, from: Option<u32>, to: Option<u32>) -> ListStoredBriefsInput {
        ListStoredBriefsInput {
            scope: scope.map(a_scope),
            from: from.map(a_day),
            to: to.map(a_day),
            pagination: Pagination::new(1, 20),
        }
    }

    fn keys(briefs: &[DailyBrief]) -> Vec<(NaiveDate, &str)> {
        briefs
            .iter()
            .map(|brief| (*brief.day(), brief.scope().as_str()))
            .collect()
    }

    #[tokio::test]
    async fn with_no_filter_it_lists_every_scope_newest_day_first() {
        let fakes = fakes();
        let use_case = ListStoredBriefsUseCase::new(Arc::new(fakes.clone()));

        let (briefs, count) = use_case
            .execute(input(None, None, None))
            .await
            .expect("briefs");

        assert_eq!(count, 4);
        assert_eq!(
            keys(&briefs),
            vec![
                (a_day(9), "kalar"),
                (a_day(9), "region"),
                (a_day(8), "region"),
                (a_day(7), "region"),
            ]
        );
        assert_eq!(
            fakes.calls(),
            vec![Call::FindPage {
                scope: None,
                from: None,
                to: None,
                page: 1,
            }]
        );
    }

    #[tokio::test]
    async fn the_scope_and_the_days_narrow_the_list() {
        let use_case = ListStoredBriefsUseCase::new(Arc::new(fakes()));

        let (briefs, count) = use_case
            .execute(input(Some("region"), Some(8), None))
            .await
            .expect("briefs");

        assert_eq!(count, 2);
        assert_eq!(
            keys(&briefs),
            vec![(a_day(9), "region"), (a_day(8), "region")]
        );
    }

    #[tokio::test]
    async fn the_count_is_of_every_page_not_only_the_one_returned() {
        let use_case = ListStoredBriefsUseCase::new(Arc::new(fakes()));

        let (briefs, count) = use_case
            .execute(ListStoredBriefsInput {
                pagination: Pagination::new(2, 3),
                ..input(None, None, None)
            })
            .await
            .expect("briefs");

        assert_eq!(count, 4);
        assert_eq!(keys(&briefs), vec![(a_day(7), "region")]);
    }

    #[tokio::test]
    async fn a_range_that_ends_before_it_starts_reads_nothing() {
        let fakes = fakes();
        let use_case = ListStoredBriefsUseCase::new(Arc::new(fakes.clone()));

        let result = use_case.execute(input(None, Some(9), Some(7))).await;

        assert!(matches!(
            result,
            Err(AppError::Brief(BriefError::RangeEndsBeforeItStarts))
        ));
        assert!(fakes.calls().is_empty());
    }

    #[tokio::test]
    async fn a_repository_failure_surfaces() {
        let use_case = ListStoredBriefsUseCase::new(Arc::new(Fakes::new().failing()));

        assert!(use_case.execute(input(None, None, None)).await.is_err());
    }
}
