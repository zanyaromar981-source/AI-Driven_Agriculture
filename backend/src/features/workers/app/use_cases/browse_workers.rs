use std::sync::Arc;

use crate::{
    app::Pagination,
    features::workers::{
        app::{AccountDirectory, AppError, WorkerFilter, WorkerRepository},
        domain::{CostIqd, CostPer, GeoPoint, ListedWorker, WorkerSearch, ZoneSlug},
    },
};

pub struct BrowseWorkersInput {
    pub zone: Option<ZoneSlug>,
    pub search: Option<WorkerSearch>,
    pub max_cost: Option<CostIqd>,
    pub cost_per: Option<CostPer>,
    /// Where the reader stands. Given, the list is ordered nearest first.
    pub near: Option<GeoPoint>,
    pub pagination: Pagination,
}

pub struct BrowseWorkersUseCase {
    repository: Arc<dyn WorkerRepository>,
    accounts: Arc<dyn AccountDirectory>,
}

impl BrowseWorkersUseCase {
    pub fn new(repository: Arc<dyn WorkerRepository>, accounts: Arc<dyn AccountDirectory>) -> Self {
        Self {
            repository,
            accounts,
        }
    }

    /// Returns one page of the workers a farmer may call and how many match
    /// in all. Only available cards are shown, and never the card of an
    /// account staff have blocked. The blocked phones are left out by the
    /// query itself, so the pages and the count stay true.
    pub async fn execute(
        &self,
        input: BrowseWorkersInput,
    ) -> Result<(Vec<ListedWorker>, u64), AppError> {
        let filter = WorkerFilter {
            zone: input.zone,
            search: input.search,
            max_cost: input.max_cost,
            cost_per: input.cost_per,
            available: Some(true),
            excluding: self.accounts.blocked_phones().await?,
        };

        let (workers, count) = self
            .repository
            .find_page(&filter, input.near.as_ref(), &input.pagination)
            .await?;

        let listed: Vec<ListedWorker> = workers
            .into_iter()
            .map(|worker| worker.seen_from(input.near.as_ref()))
            .collect();

        tracing::debug!(returned = listed.len(), count, "workers browsed");

        Ok((listed, count))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::workers::app::testing::{
        Call, Fakes, OTHER_PHONE, PHONE, a_phone, a_worker, unavailable,
    };

    const THIRD_PHONE: &str = "+9647809998877";

    fn input() -> BrowseWorkersInput {
        BrowseWorkersInput {
            zone: None,
            search: None,
            max_cost: None,
            cost_per: None,
            near: None,
            pagination: Pagination::new(1, 20),
        }
    }

    fn use_case(fakes: &Fakes) -> BrowseWorkersUseCase {
        BrowseWorkersUseCase::new(Arc::new(fakes.clone()), Arc::new(fakes.clone()))
    }

    #[tokio::test]
    async fn shows_available_cards_only_and_asks_for_exactly_that() {
        let fakes = Fakes::new()
            .with_stored(a_worker(1, PHONE))
            .with_stored(unavailable(&a_worker(2, OTHER_PHONE)));

        let (listed, count) = use_case(&fakes).execute(input()).await.expect("listed");

        assert_eq!(count, 1);
        assert_eq!(*listed[0].worker().id(), Some(1));
        assert_eq!(*listed[0].distance_km(), None);
        assert_eq!(
            fakes.calls(),
            vec![
                Call::BlockedPhones,
                Call::FindPage {
                    filter: WorkerFilter {
                        available: Some(true),
                        ..WorkerFilter::default()
                    },
                    near: None,
                    page: 1,
                },
            ]
        );
    }

    #[tokio::test]
    async fn the_card_of_a_blocked_account_is_left_out_by_the_query() {
        let fakes = Fakes::new()
            .with_stored(a_worker(1, PHONE))
            .with_stored(a_worker(2, OTHER_PHONE))
            .with_stored(a_worker(3, THIRD_PHONE))
            .with_blocked(OTHER_PHONE);

        let (listed, count) = use_case(&fakes).execute(input()).await.expect("listed");

        let ids: Vec<Option<i32>> = listed.iter().map(|card| *card.worker().id()).collect();
        assert_eq!(ids, vec![Some(1), Some(3)]);
        assert_eq!(count, 2, "the count must not include the hidden card");
        assert!(matches!(
            &fakes.calls()[1],
            Call::FindPage { filter, .. } if filter.excluding == vec![a_phone(OTHER_PHONE)]
        ));
    }

    #[tokio::test]
    async fn the_callers_point_orders_the_list_and_puts_a_distance_on_each_card() {
        let fakes = Fakes::new().with_stored(a_worker(1, PHONE));
        let near = GeoPoint::anywhere(35.5572, 45.4356).expect("point");

        let (listed, _) = use_case(&fakes)
            .execute(BrowseWorkersInput {
                near: Some(near),
                ..input()
            })
            .await
            .expect("listed");

        assert_eq!(
            *listed[0].distance_km(),
            None,
            "this card has no point, so no distance"
        );
        assert!(matches!(
            &fakes.calls()[1],
            Call::FindPage { near: Some(point), .. } if *point == near
        ));
    }

    #[tokio::test]
    async fn nothing_is_listed_when_the_blocked_accounts_cannot_be_read() {
        let fakes = Fakes::new().with_stored(a_worker(1, PHONE)).failing();

        assert!(use_case(&fakes).execute(input()).await.is_err());
        assert_eq!(
            fakes.calls(),
            vec![Call::BlockedPhones],
            "a failure must not fall back to showing every card"
        );
    }
}
