use std::sync::Arc;

use sea_orm::DatabaseConnection;

use crate::{
    features::farms::app::use_cases::{
        ListFarmsUseCase, RegisterFarmUseCase, RemoveFarmUseCase, RepaintFarmCellsUseCase,
    },
    infra::Config,
};

#[derive(Clone)]
pub struct FarmFeature {
    pub register_farm_use_case: Arc<RegisterFarmUseCase>,
    pub list_farms_use_case: Arc<ListFarmsUseCase>,
    pub remove_farm_use_case: Arc<RemoveFarmUseCase>,
    pub repaint_farm_cells_use_case: Arc<RepaintFarmCellsUseCase>,
}

#[derive(Clone)]
pub struct Features {
    pub farm: FarmFeature,
}

#[derive(Clone)]
pub struct AppState {
    pub startup_time: u64,
    pub features: Arc<Features>,
    pub database: DatabaseConnection,
    pub config: Config,
}
