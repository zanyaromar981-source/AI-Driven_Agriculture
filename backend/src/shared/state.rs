use std::sync::Arc;

use sea_orm::DatabaseConnection;

use crate::{
    features::{
        alwa::app::use_cases::{
            AcceptOfferUseCase, BrowseListingsUseCase, CancelListingUseCase, ListDealsUseCase,
            ListMarketsUseCase, ListMyListingsUseCase, ListMyOffersUseCase, MakeOfferUseCase,
            PostListingUseCase, RecordPriceUseCase, ViewListingUseCase, ViewMarketPricesUseCase,
            ViewPriceHistoryUseCase,
        },
        dams::app::use_cases::{ListDamsUseCase, RecordDamReadingUseCase, ViewDamHistoryUseCase},
        farmers::app::use_cases::{
            EditProfileUseCase, RequestSignInCodeUseCase, VerifySignInCodeUseCase,
            ViewProfileUseCase,
        },
        farms::app::use_cases::{
            ListFarmsUseCase, RegisterFarmUseCase, RemoveFarmUseCase, RepaintFarmCellsUseCase,
            ViewFarmUseCase,
        },
        fires::app::use_cases::{ListFiresUseCase, RecordFireUseCase},
        insights::app::use_cases::{
            ListFarmCoverageUseCase, RecordFarmInsightUseCase, ViewFarmInsightsUseCase,
        },
        outlooks::app::use_cases::{
            RecordOutlookRunUseCase, RecordZoneOutlookUseCase, ViewSeasonOutlookUseCase,
            ViewZoneOutlookUseCase,
        },
        water::app::use_cases::{
            RemoveWaterPlanEntryUseCase, SetWaterPlanEntryUseCase, ViewWaterPlanUseCase,
        },
        zones::app::use_cases::{
            CompareYearsUseCase, RecordSubZoneReadingUseCase, RecordZoneReadingUseCase,
            ViewRegionOverviewUseCase, ViewZoneUseCase,
        },
    },
    infra::Config,
};

#[derive(Clone)]
pub struct FarmFeature {
    pub register_farm_use_case: Arc<RegisterFarmUseCase>,
    pub list_farms_use_case: Arc<ListFarmsUseCase>,
    pub view_farm_use_case: Arc<ViewFarmUseCase>,
    pub remove_farm_use_case: Arc<RemoveFarmUseCase>,
    pub repaint_farm_cells_use_case: Arc<RepaintFarmCellsUseCase>,
}

#[derive(Clone)]
pub struct FarmerFeature {
    pub request_sign_in_code_use_case: Arc<RequestSignInCodeUseCase>,
    pub verify_sign_in_code_use_case: Arc<VerifySignInCodeUseCase>,
    pub view_profile_use_case: Arc<ViewProfileUseCase>,
    pub edit_profile_use_case: Arc<EditProfileUseCase>,
}

#[derive(Clone)]
pub struct FireFeature {
    pub list_fires_use_case: Arc<ListFiresUseCase>,
    pub record_fire_use_case: Arc<RecordFireUseCase>,
}

#[derive(Clone)]
pub struct InsightFeature {
    pub view_farm_insights_use_case: Arc<ViewFarmInsightsUseCase>,
    pub record_farm_insight_use_case: Arc<RecordFarmInsightUseCase>,
    pub list_farm_coverage_use_case: Arc<ListFarmCoverageUseCase>,
}

#[derive(Clone)]
pub struct ZoneFeature {
    pub view_region_overview_use_case: Arc<ViewRegionOverviewUseCase>,
    pub view_zone_use_case: Arc<ViewZoneUseCase>,
    pub compare_years_use_case: Arc<CompareYearsUseCase>,
    pub record_zone_reading_use_case: Arc<RecordZoneReadingUseCase>,
    pub record_sub_zone_reading_use_case: Arc<RecordSubZoneReadingUseCase>,
}

#[derive(Clone)]
pub struct DamFeature {
    pub list_dams_use_case: Arc<ListDamsUseCase>,
    pub view_dam_history_use_case: Arc<ViewDamHistoryUseCase>,
    pub record_dam_reading_use_case: Arc<RecordDamReadingUseCase>,
}

#[derive(Clone)]
pub struct OutlookFeature {
    pub view_season_outlook_use_case: Arc<ViewSeasonOutlookUseCase>,
    pub view_zone_outlook_use_case: Arc<ViewZoneOutlookUseCase>,
    pub record_zone_outlook_use_case: Arc<RecordZoneOutlookUseCase>,
    pub record_outlook_run_use_case: Arc<RecordOutlookRunUseCase>,
}

#[derive(Clone)]
pub struct WaterFeature {
    pub view_water_plan_use_case: Arc<ViewWaterPlanUseCase>,
    pub set_water_plan_entry_use_case: Arc<SetWaterPlanEntryUseCase>,
    pub remove_water_plan_entry_use_case: Arc<RemoveWaterPlanEntryUseCase>,
}

#[derive(Clone)]
pub struct AlwaFeature {
    pub list_markets_use_case: Arc<ListMarketsUseCase>,
    pub view_market_prices_use_case: Arc<ViewMarketPricesUseCase>,
    pub view_price_history_use_case: Arc<ViewPriceHistoryUseCase>,
    pub record_price_use_case: Arc<RecordPriceUseCase>,
    pub browse_listings_use_case: Arc<BrowseListingsUseCase>,
    pub view_listing_use_case: Arc<ViewListingUseCase>,
    pub list_deals_use_case: Arc<ListDealsUseCase>,
    pub post_listing_use_case: Arc<PostListingUseCase>,
    pub list_my_listings_use_case: Arc<ListMyListingsUseCase>,
    pub cancel_listing_use_case: Arc<CancelListingUseCase>,
    pub make_offer_use_case: Arc<MakeOfferUseCase>,
    pub accept_offer_use_case: Arc<AcceptOfferUseCase>,
    pub list_my_offers_use_case: Arc<ListMyOffersUseCase>,
}

#[derive(Clone)]
pub struct Features {
    pub farm: FarmFeature,
    pub farmer: FarmerFeature,
    pub fire: FireFeature,
    pub insight: InsightFeature,
    pub zone: ZoneFeature,
    pub dam: DamFeature,
    pub outlook: OutlookFeature,
    pub water: WaterFeature,
    pub alwa: AlwaFeature,
}

#[derive(Clone)]
pub struct AppState {
    pub startup_time: u64,
    pub features: Arc<Features>,
    pub database: DatabaseConnection,
    pub config: Config,
}
