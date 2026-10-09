use std::sync::Arc;

use crate::{
    app::Pagination,
    features::workers::{
        app::{AppError, WorkerFilter, WorkerRepository},
        domain::{CostIqd, CostPer, GeoPoint, ListedWorker, WorkerSearch, ZoneSlug},
    },
};

pub struct ListAllWorkersInput {
    pub zone: Option<ZoneSlug>,
    pub search: Option<WorkerSearch>,
    pub max_cost: Option<CostIqd>,
    pub cost_per: Option<CostPer>,
    /// `None` shows the cards on the list and the ones taken off it.
    pub available: Option<bool>,
    pub near: Option<GeoPoint>,
    pub pagination: Pagination,
}

pub struct ListAllWorkersUseCase {
    repository: Arc<dyn WorkerRepository>,
}

impl ListAllWorkersUseCase {
    pub fn new(repository: Arc<dyn WorkerRepository>) -> Self {
        Self { repository }
    }

    /// Returns one page of every card for Ministry staff: also the ones
    /// that are not available and the ones of blocked accounts, since staff
    /// must be able to see what they may want to remove.
    pub async fn execute(
        &self,
        input: ListAllWorkersInput,
    ) -> Result<(Vec<ListedWorker>, u64), AppError> {
        let filter = WorkerFilter {
            zone: input.zone,
            search: input.search,
            max_cost: input.max_cost,
            cost_per: input.cost_per,
            available: input.available,
            excluding: Vec::new(),
        };

        let (workers, count) = self
            .repository
            .find_page(&filter, input.near.as_ref(), &input.pagination)
            .await?;

        tracing::debug!(returned = workers.len(), count, "workers listed for staff");

        Ok((
            workers
                .into_iter()
                .map(|worker| worker.seen_from(input.near.as_ref()))
                .collect(),
            count,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::workers::app::testing::{
        Call, Fakes, OTHER_PHONE, PHONE, a_worker, unavailable,
    };

    fn input(available: Option<bool>) -> ListAllWorkersInput {
        ListAllWorkersInput {
            zone: None,
            search: None,
            max_cost: None,
            cost_per: None,
            available,
            near: None,
            pagination: Pagination::new(2, 20),
        }
    }

    fn fakes() -> Fakes {
        Fakes::new()
            .with_stored(a_worker(1, PHONE))
            .with_stored(unavailable(&a_worker(2, OTHER_PHONE)))
            .with_blocked(PHONE)
    }

    #[tokio::test]
    async fn staff_see_every_card_and_no_blocked_account_is_asked_for() {
        let fakes = fakes();

        let (listed, count) = ListAllWorkersUseCase::new(Arc::new(fakes.clone()))
            .execute(input(None))
            .await
            .expect("listed");

        assert_eq!(count, 2);
        assert_eq!(listed.len(), 2);
        assert_eq!(
            fakes.calls(),
            vec![Call::FindPage {
                filter: WorkerFilter::default(),
                near: None,
                page: 2,
            }]
        );
    }

    #[tokio::test]
    async fn staff_may_ask_for_the_cards_taken_off_the_list() {
        let (listed, count) = ListAllWorkersUseCase::new(Arc::new(fakes()))
            .execute(input(Some(false)))
            .await
            .expect("listed");

        assert_eq!(count, 1);
        assert_eq!(*listed[0].worker().id(), Some(2));
    }
}
