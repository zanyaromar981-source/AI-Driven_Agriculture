use std::sync::Arc;

use crate::{
    features::farms::{
        app::{
            FarmRepository,
            use_cases::{
                ListFarmsUseCase, RegisterFarmUseCase, RemoveFarmUseCase, RepaintFarmCellsUseCase,
            },
        },
        infra::FarmPostgresRepository,
    },
    infra::{Config, DBConnector},
    shared::{FarmFeature, Features},
};

pub async fn di_init(
    config: &Config,
    db_context: DBConnector,
) -> Result<Features, Box<dyn std::error::Error + Send + Sync>> {
    let farm_repository: Arc<dyn FarmRepository> =
        Arc::new(FarmPostgresRepository::new(db_context.conn_clone()));

    let farm = FarmFeature {
        register_farm_use_case: Arc::new(RegisterFarmUseCase::new(
            farm_repository.clone(),
            config.farm.max_farms_per_user,
            config.farm.max_cells_per_farm,
        )),
        list_farms_use_case: Arc::new(ListFarmsUseCase::new(farm_repository.clone())),
        repaint_farm_cells_use_case: Arc::new(RepaintFarmCellsUseCase::new(
            farm_repository.clone(),
        )),
        remove_farm_use_case: Arc::new(RemoveFarmUseCase::new(farm_repository)),
    };

    Ok(Features { farm })
}
