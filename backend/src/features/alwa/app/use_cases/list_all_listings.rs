use std::sync::Arc;

use chrono::Utc;

use crate::{
    app::{AppError as GlobalAppError, Pagination},
    features::alwa::{
        app::{AlwaRepository, AppError, ModerationFilter, listing_cards::assemble_cards},
        domain::{Crop, ListingCard, ListingStatus, MarketSlug},
    },
    shared::Phone,
};

pub struct ListAllListingsInput {
    pub market: Option<MarketSlug>,
    pub crop: Option<Crop>,
    /// Left out: every status.
    pub status: Option<ListingStatus>,
    pub seller: Option<Phone>,
    pub pagination: Pagination,
}

pub struct ListAllListingsUseCase {
    repository: Arc<dyn AlwaRepository>,
}

impl ListAllListingsUseCase {
    pub fn new(repository: Arc<dyn AlwaRepository>) -> Self {
        Self { repository }
    }

    /// Returns one page of every seller's listings for staff to moderate,
    /// newest first, and how many match in all.
    pub async fn execute(
        &self,
        input: ListAllListingsInput,
    ) -> Result<(Vec<ListingCard>, u64), AppError> {
        let market_id = match &input.market {
            Some(slug) => {
                let Some(market) = self.repository.find_market_by_slug(slug).await? else {
                    tracing::info!(market = slug.as_str(), "listings refused: no such alwa");

                    return Err(GlobalAppError::NotFound.into());
                };

                Some(*market.id())
            }
            None => None,
        };

        let now = Utc::now();
        let filter = ModerationFilter {
            market_id,
            crop: input.crop,
            status: input.status,
            seller: input.seller,
        };

        let (listings, count) = self
            .repository
            .find_listings_for_moderation(&filter, now, &input.pagination)
            .await?;

        let cards = assemble_cards(self.repository.as_ref(), listings, now).await?;

        tracing::debug!(
            returned = cards.len(),
            count,
            "alwa listings listed for staff"
        );

        Ok((cards, count))
    }
}

#[cfg(test)]
mod tests {
    use chrono::Duration;

    use super::*;
    use crate::features::alwa::app::testing::{
        BUYER, FakeAlwaRepository, MARKET, OTHER_BUYER, RepositoryCall, SELLER, a_listing,
        a_sold_listing, an_open_listing, an_open_offer, market_slug, phone,
    };

    fn input() -> ListAllListingsInput {
        ListAllListingsInput {
            market: None,
            crop: None,
            status: None,
            seller: None,
            pagination: Pagination::new(1, 20),
        }
    }

    fn repository() -> FakeAlwaRepository {
        let open = an_open_listing(7);
        let (sold, sold_offers) = a_sold_listing(8);

        sold_offers
            .into_iter()
            .fold(FakeAlwaRepository::new(), FakeAlwaRepository::with_offer)
            .with_offer(an_open_offer(11, &open, BUYER, 900))
            .with_offer(an_open_offer(12, &open, OTHER_BUYER, 950))
            .with_listing(open)
            .with_listing(sold)
            .with_listing(a_listing(9, Utc::now() - Duration::days(5)))
    }

    #[tokio::test]
    async fn lists_listings_of_every_status_with_their_open_offer_count() {
        let use_case = ListAllListingsUseCase::new(Arc::new(repository()));

        let (cards, count) = use_case.execute(input()).await.expect("listings");

        assert_eq!(count, 3);
        assert_eq!(
            cards
                .iter()
                .map(|card| (*card.status(), card.open_offers()))
                .collect::<Vec<_>>(),
            vec![
                (ListingStatus::Open, 2),
                (ListingStatus::Sold, 0),
                (ListingStatus::Closed, 0),
            ]
        );
    }

    #[tokio::test]
    async fn narrows_by_the_status_a_reader_sees_and_by_seller() {
        let repository = repository();
        let use_case = ListAllListingsUseCase::new(Arc::new(repository.clone()));

        let (closed, _) = use_case
            .execute(ListAllListingsInput {
                status: Some(ListingStatus::Closed),
                ..input()
            })
            .await
            .expect("closed");

        assert_eq!(closed.len(), 1);
        assert_eq!(
            *closed[0].listing().id(),
            Some(9),
            "a listing past its closing time is still stored as open"
        );

        let (of_buyer, count) = use_case
            .execute(ListAllListingsInput {
                seller: Some(phone(BUYER)),
                ..input()
            })
            .await
            .expect("of buyer");

        assert!(of_buyer.is_empty());
        assert_eq!(count, 0);

        let (of_seller, _) = use_case
            .execute(ListAllListingsInput {
                seller: Some(phone(SELLER)),
                market: Some(market_slug(MARKET)),
                ..input()
            })
            .await
            .expect("of seller");

        assert_eq!(of_seller.len(), 3);
        assert!(repository.calls().iter().any(|call| matches!(
            call,
            RepositoryCall::FindListingsForModeration { filter, .. }
                if filter.seller == Some(phone(SELLER)) && filter.market_id == Some(1)
        )));
    }

    #[tokio::test]
    async fn an_unknown_market_is_not_found() {
        let use_case = ListAllListingsUseCase::new(Arc::new(repository()));

        let result = use_case
            .execute(ListAllListingsInput {
                market: Some(market_slug("baghdad")),
                ..input()
            })
            .await;

        assert!(matches!(
            result,
            Err(AppError::GlobalAppError(GlobalAppError::NotFound))
        ));
    }
}
