use std::sync::Arc;

use sea_orm::DatabaseConnection;

use crate::{
    features::{
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
pub struct Features {
    pub farm: FarmFeature,
    pub farmer: FarmerFeature,
    pub fire: FireFeature,
    pub insight: InsightFeature,
}

#[derive(Clone)]
pub struct AppState {
    pub startup_time: u64,
    pub features: Arc<Features>,
    pub database: DatabaseConnection,
    pub config: Config,
}
