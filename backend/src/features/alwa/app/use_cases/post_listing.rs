use std::sync::Arc;

use chrono::Utc;

use crate::{
    app::{AppError as GlobalAppError, AuthContext},
    features::alwa::{
        app::{
            AlwaRepository, AppError, CropDirectory, ZoneLocator, listing_cards::assemble_cards,
        },
        domain::{IdempotencyKey, Listing, ListingCard, ListingDraft, Market, MarketSlug},
    },
};

pub struct PostListingInput {
    /// Left out by the app: the nearest alwa to the listing's point is
    /// taken then.
    pub market: Option<MarketSlug>,
    pub draft: ListingDraft,
    pub idempotency_key: Option<IdempotencyKey>,
}

pub struct PostListingUseCase {
    repository: Arc<dyn AlwaRepository>,
    crops: Arc<dyn CropDirectory>,
    zones: Arc<dyn ZoneLocator>,
    max_open_listings_per_seller: u64,
}

impl PostListingUseCase {
    pub fn new(
        repository: Arc<dyn AlwaRepository>,
        crops: Arc<dyn CropDirectory>,
        zones: Arc<dyn ZoneLocator>,
        max_open_listings_per_seller: u64,
    ) -> Self {
        Self {
            repository,
            crops,
            zones,
            max_open_listings_per_seller,
        }
    }

    pub async fn execute(
        &self,
        auth_context: &AuthContext,
        input: PostListingInput,
    ) -> Result<ListingCard, AppError> {
        let seller = auth_context.user().phone();
        let now = Utc::now();

        // A repeat of a post that already went through (the app retries when
        // an answer is lost) gets the listing it created the first time.
        if let Some(key) = &input.idempotency_key
            && let Some(existing) = self
                .repository
                .find_listing_by_idempotency_key(seller, key)
                .await?
        {
            tracing::info!(
                listing_id = existing.id().unwrap_or_default(),
                "listing post repeated: returning the listing already posted"
            );

            return first_card(self.repository.as_ref(), existing, now).await;
        }

        // A new listing must name a crop staff have switched on. A repeat of
        // an earlier post never gets here: that listing was checked when it
        // was made.
        self.crops
            .active()
            .await?
            .allow(input.draft.crop)
            .inspect_err(
                |error| tracing::info!(%error, "listing refused: the crop is not in use"),
            )?;

        let market = self.market_for(&input).await?;

        let open = self
            .repository
            .count_open_listings_by_seller(seller, now)
            .await?;

        if open >= self.max_open_listings_per_seller {
            tracing::info!(
                open,
                max = self.max_open_listings_per_seller,
                "listing refused: open listing quota reached"
            );

            return Err(AppError::MaxOpenListingsReached(
                self.max_open_listings_per_seller,
            ));
        }

        let mut draft = input.draft;

        // The place is where the farmer stands, so the district is read off
        // the map. A district the caller named is kept as said.
        if draft.zone_slug.is_none()
            && let Some(point) = &draft.point
        {
            draft.zone_slug = self.zones.zone_of(point).await?;
        }

        let listing = Listing::new(seller.clone(), market.as_ref(), draft, now)?;
        let posted = match self
            .repository
            .create_listing(&listing, input.idempotency_key.as_ref())
            .await
        {
            Ok(posted) => posted,
            Err(error) => {
                // Two posts with one key can both pass the lookup above; the
                // database lets one in, and the other gets that listing.
                if let Some(key) = &input.idempotency_key
                    && let Some(existing) = self
                        .repository
                        .find_listing_by_idempotency_key(seller, key)
                        .await?
                {
                    return first_card(self.repository.as_ref(), existing, now).await;
                }

                return Err(error);
            }
        };

        tracing::info!(
            listing_id = posted.id().unwrap_or_default(),
            market = market.as_ref().map(|market| market.slug().as_str()),
            zone = posted.zone_slug().as_ref().map(|zone| zone.as_str()),
            crop = String::from(*posted.crop()),
            quantity_kg = posted.quantity().value(),
            open_after = open + 1,
            "listing posted"
        );

        first_card(self.repository.as_ref(), posted, now).await
    }
}

impl PostListingUseCase {
    /// The alwa the listing is at: the one named, else the nearest to the
    /// listing's point, else none.
    async fn market_for(&self, input: &PostListingInput) -> Result<Option<Market>, AppError> {
        if let Some(slug) = &input.market {
            let Some(market) = self.repository.find_market_by_slug(slug).await? else {
                tracing::info!(market = slug.as_str(), "listing refused: no such alwa");

                return Err(GlobalAppError::NotFound.into());
            };

            return Ok(Some(market));
        }

        let Some(point) = &input.draft.point else {
            return Ok(None);
        };

        let markets = self.repository.find_markets().await?;

        Ok(Market::nearest(&markets, point).cloned())
    }
}

async fn first_card(
    repository: &dyn AlwaRepository,
    listing: Listing,
    now: chrono::DateTime<Utc>,
) -> Result<ListingCard, AppError> {
    let cards = assemble_cards(repository, vec![listing], now).await?;

    cards.into_iter().next().ok_or_else(|| {
        GlobalAppError::MissingValue("The posted listing has no card".to_string()).into()
    })
}

#[cfg(test)]
mod tests {
    use chrono::Duration;

    use super::*;
    use crate::features::alwa::{
        app::testing::{
            ERBIL, FakeAlwaRepository, FakeCropDirectory, FakeZoneLocator, MARKET, RepositoryCall,
            SELLER, a_listing, a_listing_draft, an_open_listing, auth_context, market_slug, point,
        },
        domain::{AlwaError, ListingStatus, ZoneSlug},
    };

    const MAX: u64 = 3;

    fn input() -> PostListingInput {
        PostListingInput {
            market: Some(market_slug(MARKET)),
            draft: a_listing_draft(Utc::now() + Duration::days(2)),
            idempotency_key: None,
        }
    }

    fn holding(open: u64) -> FakeAlwaRepository {
        (0..open).fold(FakeAlwaRepository::new(), |repository, id| {
            repository.with_listing(an_open_listing(id as i32 + 1))
        })
    }

    #[tokio::test]
    async fn posts_an_open_listing_for_the_signed_in_phone() {
        let repository = holding(MAX - 1);
        let use_case = PostListingUseCase::new(
            Arc::new(repository.clone()),
            Arc::new(FakeCropDirectory::seeded()),
            Arc::new(FakeZoneLocator::nowhere()),
            MAX,
        );

        let card = use_case
            .execute(&auth_context(SELLER), input())
            .await
            .expect("card");

        assert_eq!(card.listing().seller_phone().as_str(), SELLER);
        assert_eq!(*card.status(), ListingStatus::Open);
        assert!(card.listing().id().is_some());
        assert!(card.offers().is_empty());
        assert!(repository.calls().contains(&RepositoryCall::CreateListing));
    }

    #[tokio::test]
    async fn rejects_once_the_open_listing_quota_is_reached() {
        let repository = holding(MAX);
        let use_case = PostListingUseCase::new(
            Arc::new(repository.clone()),
            Arc::new(FakeCropDirectory::seeded()),
            Arc::new(FakeZoneLocator::nowhere()),
            MAX,
        );

        let result = use_case.execute(&auth_context(SELLER), input()).await;

        assert!(matches!(result, Err(AppError::MaxOpenListingsReached(MAX))));
        assert!(
            !repository.wrote(),
            "a rejected listing must not be written"
        );
        assert!(
            repository
                .calls()
                .contains(&RepositoryCall::CountOpenListingsBySeller {
                    seller: SELLER.to_string()
                })
        );
    }

    #[tokio::test]
    async fn listings_that_are_no_longer_open_do_not_count_against_the_quota() {
        let repository =
            holding(MAX - 1).with_listing(a_listing(50, Utc::now() - Duration::days(5)));
        let use_case = PostListingUseCase::new(
            Arc::new(repository.clone()),
            Arc::new(FakeCropDirectory::seeded()),
            Arc::new(FakeZoneLocator::nowhere()),
            MAX,
        );

        assert!(
            use_case
                .execute(&auth_context(SELLER), input())
                .await
                .is_ok(),
            "a listing past its closing time is closed, so it frees its place"
        );
    }

    #[tokio::test]
    async fn a_closing_time_in_the_past_is_not_written() {
        let repository = FakeAlwaRepository::new();
        let use_case = PostListingUseCase::new(
            Arc::new(repository.clone()),
            Arc::new(FakeCropDirectory::seeded()),
            Arc::new(FakeZoneLocator::nowhere()),
            MAX,
        );

        let result = use_case
            .execute(
                &auth_context(SELLER),
                PostListingInput {
                    market: Some(market_slug(MARKET)),
                    draft: a_listing_draft(Utc::now() - Duration::minutes(1)),
                    idempotency_key: None,
                },
            )
            .await;

        assert!(matches!(
            result,
            Err(AppError::Alwa(AlwaError::BadClosingTime(_)))
        ));
        assert!(!repository.wrote());
    }

    #[tokio::test]
    async fn a_listing_at_an_unknown_alwa_is_not_found() {
        let repository = FakeAlwaRepository::new();
        let use_case = PostListingUseCase::new(
            Arc::new(repository.clone()),
            Arc::new(FakeCropDirectory::seeded()),
            Arc::new(FakeZoneLocator::nowhere()),
            MAX,
        );

        let result = use_case
            .execute(
                &auth_context(SELLER),
                PostListingInput {
                    market: Some(market_slug("baghdad")),
                    ..input()
                },
            )
            .await;

        assert!(matches!(
            result,
            Err(AppError::GlobalAppError(GlobalAppError::NotFound))
        ));
        assert!(!repository.wrote());
    }

    #[tokio::test]
    async fn a_crop_staff_added_later_can_be_put_on_sale() {
        let repository = FakeAlwaRepository::new();
        let use_case = PostListingUseCase::new(
            Arc::new(repository.clone()),
            Arc::new(FakeCropDirectory::with(&["rice"])),
            Arc::new(FakeZoneLocator::nowhere()),
            MAX,
        );
        let mut input = input();
        input.draft.crop = crate::features::alwa::domain::Crop::of("rice");

        let card = use_case
            .execute(&auth_context(SELLER), input)
            .await
            .expect("card");

        assert_eq!(card.listing().crop().as_str(), "rice");
    }

    #[tokio::test]
    async fn a_crop_that_is_unknown_or_switched_off_is_refused_by_name_and_nothing_is_posted() {
        let repository = FakeAlwaRepository::new();
        // The draft sells tomato, and tomato is not in the list.
        let crops = FakeCropDirectory::with(&["wheat"]);
        let use_case = PostListingUseCase::new(
            Arc::new(repository.clone()),
            Arc::new(crops.clone()),
            Arc::new(FakeZoneLocator::nowhere()),
            MAX,
        );

        let result = use_case.execute(&auth_context(SELLER), input()).await;

        assert!(matches!(
            result,
            Err(AppError::Alwa(AlwaError::UnknownCrop(code))) if code == "tomato"
        ));
        assert_eq!(crops.asked(), 1);
        assert!(!repository.calls().contains(&RepositoryCall::CreateListing));
    }

    #[tokio::test]
    async fn when_the_crop_list_cannot_be_read_nothing_is_posted() {
        let repository = FakeAlwaRepository::new();
        let use_case = PostListingUseCase::new(
            Arc::new(repository.clone()),
            Arc::new(FakeCropDirectory::failing()),
            Arc::new(FakeZoneLocator::nowhere()),
            MAX,
        );

        assert!(
            use_case
                .execute(&auth_context(SELLER), input())
                .await
                .is_err()
        );
        assert!(!repository.calls().contains(&RepositoryCall::CreateListing));
    }

    /// What the app sends: a crop, a quantity, a price, a closing time and
    /// the point the farmer stands at.
    fn app_input(lat: f64, lon: f64) -> PostListingInput {
        let mut draft = a_listing_draft(Utc::now() + Duration::days(14));
        draft.grade = None;
        draft.pickup = None;
        draft.seller_name = None;
        draft.point = Some(point(lat, lon));

        PostListingInput {
            market: None,
            draft,
            idempotency_key: None,
        }
    }

    fn use_case_with(
        repository: &FakeAlwaRepository,
        zones: &FakeZoneLocator,
    ) -> PostListingUseCase {
        PostListingUseCase::new(
            Arc::new(repository.clone()),
            Arc::new(FakeCropDirectory::seeded()),
            Arc::new(zones.clone()),
            MAX,
        )
    }

    #[tokio::test]
    async fn a_listing_with_only_a_point_gets_the_nearest_alwa_and_its_zone() {
        let repository = FakeAlwaRepository::new();
        let zones = FakeZoneLocator::everywhere("dashti-hawler");

        let card = use_case_with(&repository, &zones)
            .execute(&auth_context(SELLER), app_input(ERBIL.0 + 0.05, ERBIL.1))
            .await
            .expect("card");

        let listing = card.listing();
        assert_eq!(
            listing.market().as_ref().map(MarketSlug::as_str),
            Some("erbil"),
            "Erbil is the nearer of the two alwas"
        );
        assert_eq!(*listing.market_id(), Some(2));
        assert_eq!(
            listing.zone_slug().as_ref().map(ZoneSlug::as_str),
            Some("dashti-hawler")
        );
        assert_eq!(*listing.point(), Some(point(ERBIL.0 + 0.05, ERBIL.1)));
        assert_eq!(*listing.pickup(), None);
        assert_eq!(zones.asked(), vec![point(ERBIL.0 + 0.05, ERBIL.1)]);
    }

    #[tokio::test]
    async fn a_named_alwa_is_kept_even_when_another_is_nearer() {
        let repository = FakeAlwaRepository::new();
        let zones = FakeZoneLocator::nowhere();
        let mut input = app_input(ERBIL.0, ERBIL.1);
        input.market = Some(market_slug(MARKET));

        let card = use_case_with(&repository, &zones)
            .execute(&auth_context(SELLER), input)
            .await
            .expect("card");

        assert_eq!(
            card.listing().market().as_ref().map(MarketSlug::as_str),
            Some(MARKET)
        );
        assert!(
            !repository.calls().contains(&RepositoryCall::FindMarkets),
            "nothing to choose between"
        );
    }

    #[tokio::test]
    async fn a_listing_with_neither_an_alwa_nor_a_point_has_no_market_and_no_zone() {
        let repository = FakeAlwaRepository::new();
        let zones = FakeZoneLocator::everywhere("kalar");
        let mut input = app_input(ERBIL.0, ERBIL.1);
        input.draft.point = None;

        let card = use_case_with(&repository, &zones)
            .execute(&auth_context(SELLER), input)
            .await
            .expect("card");

        assert_eq!(*card.listing().market(), None);
        assert_eq!(*card.listing().zone_slug(), None);
        assert!(zones.asked().is_empty(), "there is no point to look up");
    }

    #[tokio::test]
    async fn a_point_outside_every_district_leaves_the_zone_empty() {
        let repository = FakeAlwaRepository::new();

        let card = use_case_with(&repository, &FakeZoneLocator::nowhere())
            .execute(&auth_context(SELLER), app_input(ERBIL.0, ERBIL.1))
            .await
            .expect("card");

        assert_eq!(*card.listing().zone_slug(), None);
        assert!(card.listing().market().is_some());
    }

    #[tokio::test]
    async fn a_zone_the_caller_named_is_not_replaced_by_the_one_on_the_map() {
        let repository = FakeAlwaRepository::new();
        let zones = FakeZoneLocator::everywhere("kalar");
        let mut input = app_input(ERBIL.0, ERBIL.1);
        input.draft.zone_slug = Some(ZoneSlug::new("chamchamal".to_string()).expect("zone"));

        let card = use_case_with(&repository, &zones)
            .execute(&auth_context(SELLER), input)
            .await
            .expect("card");

        assert_eq!(
            card.listing().zone_slug().as_ref().map(ZoneSlug::as_str),
            Some("chamchamal")
        );
        assert!(zones.asked().is_empty());
    }

    #[tokio::test]
    async fn when_the_zone_cannot_be_looked_up_nothing_is_posted() {
        let repository = FakeAlwaRepository::new();

        let result = use_case_with(&repository, &FakeZoneLocator::failing())
            .execute(&auth_context(SELLER), app_input(ERBIL.0, ERBIL.1))
            .await;

        assert!(result.is_err());
        assert!(!repository.calls().contains(&RepositoryCall::CreateListing));
    }
}
