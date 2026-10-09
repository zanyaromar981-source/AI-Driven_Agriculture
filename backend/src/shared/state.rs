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
            EditFarmerUseCase, EditProfileUseCase, ListFarmersUseCase, RegisterFarmerUseCase,
            RemoveFarmerUseCase, RequestSignInCodeUseCase, VerifySignInCodeUseCase,
            ViewFarmerUseCase, ViewProfileUseCase,
        },
        farms::app::use_cases::{
            ListAllFarmsUseCase, ListFarmsUseCase, RegisterFarmForFarmerUseCase,
            RegisterFarmUseCase, RemoveAnyFarmUseCase, RemoveFarmUseCase, RenameFarmUseCase,
            RepaintFarmCellsUseCase, ViewAnyFarmUseCase, ViewFarmUseCase,
        },
        fires::app::use_cases::{
            CorrectFireUseCase, CreateFireUseCase, ListFiresUseCase, ListStoredFiresUseCase,
            RecordFireUseCase, RemoveFireUseCase, ViewStoredFireUseCase,
        },
        insights::app::use_cases::{
            CorrectFarmInsightUseCase, CreateFarmInsightUseCase, ListFarmCoverageUseCase,
            RecordFarmInsightUseCase, RemoveFarmInsightUseCase, ViewFarmInsightsUseCase,
            ViewStoredFarmInsightsUseCase,
        },
        outlooks::app::use_cases::{
            RecordOutlookRunUseCase, RecordZoneOutlookUseCase, ViewSeasonOutlookUseCase,
            ViewZoneOutlookUseCase,
        },
        staff::app::use_cases::{
            AddStaffUseCase, CreateOwnerUseCase, CreateRoleUseCase, DeleteRoleUseCase,
            EditRoleUseCase, EditStaffUseCase, IdentifyStaffUseCase, ListRolesUseCase,
            ListStaffUseCase, RemoveStaffUseCase, SignInUseCase, ViewRoleUseCase, ViewStaffUseCase,
        },
        water::app::use_cases::{
            RemoveWaterPlanEntryUseCase, SetWaterPlanEntryUseCase, ViewWaterPlanUseCase,
        },
        zones::app::use_cases::{
            CompareYearsUseCase, CreateSubZoneReadingUseCase, CreateZoneReadingUseCase,
            DeleteSubZoneReadingUseCase, DeleteZoneReadingUseCase, ListSubZoneReadingsUseCase,
            ListZoneReadingsUseCase, ListZonesUseCase, RecordSubZoneReadingUseCase,
            RecordZoneReadingUseCase, UpdateSubZoneReadingUseCase, UpdateZoneReadingUseCase,
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
    pub list_all_farms_use_case: Arc<ListAllFarmsUseCase>,
    pub view_any_farm_use_case: Arc<ViewAnyFarmUseCase>,
    pub register_farm_for_farmer_use_case: Arc<RegisterFarmForFarmerUseCase>,
    pub rename_farm_use_case: Arc<RenameFarmUseCase>,
    pub remove_any_farm_use_case: Arc<RemoveAnyFarmUseCase>,
}

#[derive(Clone)]
pub struct FarmerFeature {
    pub request_sign_in_code_use_case: Arc<RequestSignInCodeUseCase>,
    pub verify_sign_in_code_use_case: Arc<VerifySignInCodeUseCase>,
    pub view_profile_use_case: Arc<ViewProfileUseCase>,
    pub edit_profile_use_case: Arc<EditProfileUseCase>,
    pub list_farmers_use_case: Arc<ListFarmersUseCase>,
    pub view_farmer_use_case: Arc<ViewFarmerUseCase>,
    pub register_farmer_use_case: Arc<RegisterFarmerUseCase>,
    pub edit_farmer_use_case: Arc<EditFarmerUseCase>,
    pub remove_farmer_use_case: Arc<RemoveFarmerUseCase>,
}

#[derive(Clone)]
pub struct FireFeature {
    pub list_fires_use_case: Arc<ListFiresUseCase>,
    pub record_fire_use_case: Arc<RecordFireUseCase>,
    pub list_stored_fires_use_case: Arc<ListStoredFiresUseCase>,
    pub view_stored_fire_use_case: Arc<ViewStoredFireUseCase>,
    pub create_fire_use_case: Arc<CreateFireUseCase>,
    pub correct_fire_use_case: Arc<CorrectFireUseCase>,
    pub remove_fire_use_case: Arc<RemoveFireUseCase>,
}

#[derive(Clone)]
pub struct InsightFeature {
    pub view_farm_insights_use_case: Arc<ViewFarmInsightsUseCase>,
    pub record_farm_insight_use_case: Arc<RecordFarmInsightUseCase>,
    pub list_farm_coverage_use_case: Arc<ListFarmCoverageUseCase>,
    pub view_stored_farm_insights_use_case: Arc<ViewStoredFarmInsightsUseCase>,
    pub create_farm_insight_use_case: Arc<CreateFarmInsightUseCase>,
    pub correct_farm_insight_use_case: Arc<CorrectFarmInsightUseCase>,
    pub remove_farm_insight_use_case: Arc<RemoveFarmInsightUseCase>,
}

#[derive(Clone)]
pub struct ZoneFeature {
    pub view_region_overview_use_case: Arc<ViewRegionOverviewUseCase>,
    pub view_zone_use_case: Arc<ViewZoneUseCase>,
    pub compare_years_use_case: Arc<CompareYearsUseCase>,
    pub record_zone_reading_use_case: Arc<RecordZoneReadingUseCase>,
    pub record_sub_zone_reading_use_case: Arc<RecordSubZoneReadingUseCase>,
    pub list_zones_use_case: Arc<ListZonesUseCase>,
    pub list_zone_readings_use_case: Arc<ListZoneReadingsUseCase>,
    pub create_zone_reading_use_case: Arc<CreateZoneReadingUseCase>,
    pub update_zone_reading_use_case: Arc<UpdateZoneReadingUseCase>,
    pub delete_zone_reading_use_case: Arc<DeleteZoneReadingUseCase>,
    pub list_sub_zone_readings_use_case: Arc<ListSubZoneReadingsUseCase>,
    pub create_sub_zone_reading_use_case: Arc<CreateSubZoneReadingUseCase>,
    pub update_sub_zone_reading_use_case: Arc<UpdateSubZoneReadingUseCase>,
    pub delete_sub_zone_reading_use_case: Arc<DeleteSubZoneReadingUseCase>,
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
pub struct StaffFeature {
    pub sign_in_use_case: Arc<SignInUseCase>,
    pub identify_staff_use_case: Arc<IdentifyStaffUseCase>,
    pub list_roles_use_case: Arc<ListRolesUseCase>,
    pub view_role_use_case: Arc<ViewRoleUseCase>,
    pub create_role_use_case: Arc<CreateRoleUseCase>,
    pub edit_role_use_case: Arc<EditRoleUseCase>,
    pub delete_role_use_case: Arc<DeleteRoleUseCase>,
    pub list_staff_use_case: Arc<ListStaffUseCase>,
    pub view_staff_use_case: Arc<ViewStaffUseCase>,
    pub add_staff_use_case: Arc<AddStaffUseCase>,
    pub edit_staff_use_case: Arc<EditStaffUseCase>,
    pub remove_staff_use_case: Arc<RemoveStaffUseCase>,
    pub create_owner_use_case: Arc<CreateOwnerUseCase>,
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
    pub staff: StaffFeature,
}

#[derive(Clone)]
pub struct AppState {
    pub startup_time: u64,
    pub features: Arc<Features>,
    pub database: DatabaseConnection,
    pub config: Config,
}
