use std::sync::Arc;

use sea_orm::DatabaseConnection;

use crate::features::alwa::app::use_cases::{
    CreateMarketUseCase, CreatePriceUseCase, DeleteListingUseCase, DeleteMarketUseCase,
    DeletePriceUseCase, ListAllListingsUseCase, ListStoredPricesUseCase, ModerateListingUseCase,
    UpdateMarketUseCase, UpdatePriceUseCase,
};
use crate::{
    features::{
        alwa::app::use_cases::{
            AcceptOfferUseCase, BrowseListingsUseCase, CancelListingUseCase, ListDealsUseCase,
            ListMarketsUseCase, ListMyListingsUseCase, ListMyOffersUseCase, MakeOfferUseCase,
            PostListingUseCase, RecordPriceUseCase, ViewListingUseCase, ViewMarketPricesUseCase,
            ViewPriceHistoryUseCase,
        },
        briefs::app::use_cases::{
            CorrectBriefUseCase, DeleteBriefUseCase, ListBriefsUseCase, ListStoredBriefsUseCase,
            RecordBriefUseCase, RecordFarmZonesUseCase, RemoveBriefUseCase, ViewFarmBriefUseCase,
            ViewLatestBriefUseCase,
        },
        dams::app::use_cases::{
            CreateDamReadingUseCase, DeleteDamReadingUseCase, ListDamReadingsUseCase,
            ListDamsUseCase, ListReferenceDamsUseCase, RecordDamReadingUseCase,
            UpdateDamReadingUseCase, ViewDamHistoryUseCase,
        },
        doctor::app::use_cases::AskDoctorUseCase,
        farmers::app::use_cases::{
            EditFarmerUseCase, EditProfileUseCase, EnsureFarmerUseCase, IdentifyFarmerUseCase,
            IssueLetterUseCase, ListFarmersUseCase, RegisterFarmerUseCase, RemoveFarmerUseCase,
            RequestSignInCodeUseCase, VerifySignInCodeUseCase, ViewFarmerUseCase,
            ViewLetterUseCase, ViewProfileUseCase,
        },
        farms::app::use_cases::{
            EditFarmUseCase, ListAllFarmsUseCase, ListFarmsUseCase, RegisterFarmForFarmerUseCase,
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
            CreateOutlookRunUseCase, CreateZoneOutlookUseCase, DeleteOutlookRunUseCase,
            DeleteZoneOutlookUseCase, ListOutlookRunsUseCase, ListZoneOutlooksUseCase,
            RecordOutlookRunUseCase, RecordZoneOutlookUseCase, UpdateOutlookRunUseCase,
            UpdateZoneOutlookUseCase, ViewSeasonOutlookUseCase, ViewZoneOutlookUseCase,
        },
        staff::app::use_cases::{
            AddStaffUseCase, CreateOwnerUseCase, CreateRoleUseCase, DeleteRoleUseCase,
            EditOwnProfileUseCase, EditRoleUseCase, EditStaffUseCase, IdentifyStaffUseCase,
            ListRolesUseCase, ListStaffUseCase, RemoveStaffUseCase, SignInUseCase, ViewRoleUseCase,
            ViewStaffUseCase,
        },
        water::app::use_cases::{
            CreateWaterPlanEntryUseCase, DeleteWaterPlanEntryUseCase, ListWaterPlanEntriesUseCase,
            ListWaterSeasonsUseCase, RemoveWaterPlanEntryUseCase, SetWaterPlanEntryUseCase,
            UpdateWaterPlanEntryUseCase, ViewWaterPlanUseCase,
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
    pub edit_farm_use_case: Arc<EditFarmUseCase>,
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
    pub identify_farmer_use_case: Arc<IdentifyFarmerUseCase>,
    pub ensure_farmer_use_case: Arc<EnsureFarmerUseCase>,
    pub view_profile_use_case: Arc<ViewProfileUseCase>,
    pub edit_profile_use_case: Arc<EditProfileUseCase>,
    pub list_farmers_use_case: Arc<ListFarmersUseCase>,
    pub view_farmer_use_case: Arc<ViewFarmerUseCase>,
    pub register_farmer_use_case: Arc<RegisterFarmerUseCase>,
    pub edit_farmer_use_case: Arc<EditFarmerUseCase>,
    pub remove_farmer_use_case: Arc<RemoveFarmerUseCase>,
    pub issue_letter_use_case: Arc<IssueLetterUseCase>,
    pub view_letter_use_case: Arc<ViewLetterUseCase>,
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
    pub list_reference_dams_use_case: Arc<ListReferenceDamsUseCase>,
    pub list_dam_readings_use_case: Arc<ListDamReadingsUseCase>,
    pub create_dam_reading_use_case: Arc<CreateDamReadingUseCase>,
    pub update_dam_reading_use_case: Arc<UpdateDamReadingUseCase>,
    pub delete_dam_reading_use_case: Arc<DeleteDamReadingUseCase>,
}

#[derive(Clone)]
pub struct OutlookFeature {
    pub view_season_outlook_use_case: Arc<ViewSeasonOutlookUseCase>,
    pub view_zone_outlook_use_case: Arc<ViewZoneOutlookUseCase>,
    pub record_zone_outlook_use_case: Arc<RecordZoneOutlookUseCase>,
    pub record_outlook_run_use_case: Arc<RecordOutlookRunUseCase>,
    pub list_zone_outlooks_use_case: Arc<ListZoneOutlooksUseCase>,
    pub create_zone_outlook_use_case: Arc<CreateZoneOutlookUseCase>,
    pub update_zone_outlook_use_case: Arc<UpdateZoneOutlookUseCase>,
    pub delete_zone_outlook_use_case: Arc<DeleteZoneOutlookUseCase>,
    pub list_outlook_runs_use_case: Arc<ListOutlookRunsUseCase>,
    pub create_outlook_run_use_case: Arc<CreateOutlookRunUseCase>,
    pub update_outlook_run_use_case: Arc<UpdateOutlookRunUseCase>,
    pub delete_outlook_run_use_case: Arc<DeleteOutlookRunUseCase>,
}

#[derive(Clone)]
pub struct WaterFeature {
    pub view_water_plan_use_case: Arc<ViewWaterPlanUseCase>,
    pub set_water_plan_entry_use_case: Arc<SetWaterPlanEntryUseCase>,
    pub remove_water_plan_entry_use_case: Arc<RemoveWaterPlanEntryUseCase>,
    pub list_water_seasons_use_case: Arc<ListWaterSeasonsUseCase>,
    pub list_water_plan_entries_use_case: Arc<ListWaterPlanEntriesUseCase>,
    pub create_water_plan_entry_use_case: Arc<CreateWaterPlanEntryUseCase>,
    pub update_water_plan_entry_use_case: Arc<UpdateWaterPlanEntryUseCase>,
    pub delete_water_plan_entry_use_case: Arc<DeleteWaterPlanEntryUseCase>,
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
    pub create_market_use_case: Arc<CreateMarketUseCase>,
    pub update_market_use_case: Arc<UpdateMarketUseCase>,
    pub delete_market_use_case: Arc<DeleteMarketUseCase>,
    pub list_stored_prices_use_case: Arc<ListStoredPricesUseCase>,
    pub create_price_use_case: Arc<CreatePriceUseCase>,
    pub update_price_use_case: Arc<UpdatePriceUseCase>,
    pub delete_price_use_case: Arc<DeletePriceUseCase>,
    pub list_all_listings_use_case: Arc<ListAllListingsUseCase>,
    pub moderate_listing_use_case: Arc<ModerateListingUseCase>,
    pub delete_listing_use_case: Arc<DeleteListingUseCase>,
}

#[derive(Clone)]
pub struct DoctorFeature {
    pub ask_doctor_use_case: Arc<AskDoctorUseCase>,
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
    pub edit_own_profile_use_case: Arc<EditOwnProfileUseCase>,
}

#[derive(Clone)]
pub struct BriefFeature {
    pub view_latest_brief_use_case: Arc<ViewLatestBriefUseCase>,
    pub list_briefs_use_case: Arc<ListBriefsUseCase>,
    pub view_farm_brief_use_case: Arc<ViewFarmBriefUseCase>,
    pub record_brief_use_case: Arc<RecordBriefUseCase>,
    pub record_farm_zones_use_case: Arc<RecordFarmZonesUseCase>,
    pub delete_brief_use_case: Arc<DeleteBriefUseCase>,
    pub list_stored_briefs_use_case: Arc<ListStoredBriefsUseCase>,
    pub correct_brief_use_case: Arc<CorrectBriefUseCase>,
    pub remove_brief_use_case: Arc<RemoveBriefUseCase>,
}

#[derive(Clone)]
pub struct VersionFeature {
    pub list_versions_use_case: Arc<crate::features::versions::app::use_cases::ListVersionsUseCase>,
}

#[derive(Clone)]
pub struct RuleFeature {
    pub list_rules_use_case: Arc<crate::features::rules::app::use_cases::ListRulesUseCase>,
    pub change_rule_use_case: Arc<crate::features::rules::app::use_cases::ChangeRuleUseCase>,
    pub view_rule_history_use_case:
        Arc<crate::features::rules::app::use_cases::ViewRuleHistoryUseCase>,
}

#[derive(Clone)]
pub struct JobFeature {
    pub record_job_run_use_case: Arc<crate::features::jobs::app::use_cases::RecordJobRunUseCase>,
    pub view_jobs_use_case: Arc<crate::features::jobs::app::use_cases::ViewJobsUseCase>,
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
    pub doctor: DoctorFeature,
    pub staff: StaffFeature,
    pub brief: BriefFeature,
    pub version: VersionFeature,
    pub rule: RuleFeature,
    pub job: JobFeature,
}

#[derive(Clone)]
pub struct AppState {
    pub startup_time: u64,
    pub features: Arc<Features>,
    pub database: DatabaseConnection,
    pub config: Config,
}
