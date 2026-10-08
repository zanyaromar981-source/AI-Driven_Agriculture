use std::sync::Arc;

use getset::Getters;
use opentelemetry_sdk::trace::SdkTracerProvider;

use crate::{
    infra::{Config, di_init, postgres_init, telemetry},
    shared::AppState,
};

#[derive(Clone, Getters)]
#[getset(get = "pub")]
pub struct BootstrappedApp {
    state: AppState,
    tracer_provider: Option<SdkTracerProvider>,
}

impl BootstrappedApp {
    pub async fn start() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let config = Config::from_env();

        let tracer_provider = telemetry::init(&config)?;

        let db_context = postgres_init(&config).await?;

        tracing::info!(
            database = %config.database.name,
            host = %config.database.host,
            max_connections = config.database.max_connections,
            otel_exporting = tracer_provider.is_some(),
            "database connected"
        );

        let startup_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let database = db_context.conn_clone();

        let features = di_init(&config, db_context).await?;

        tracing::info!(
            issuer = %config.auth.issuer,
            audience = %config.auth.audience,
            max_farms_per_user = config.farm.max_farms_per_user,
            max_cells_per_farm = config.farm.max_cells_per_farm,
            "application bootstrapped"
        );

        let state = AppState {
            startup_time,
            config,
            database,
            features: Arc::new(features),
        };

        Ok(Self {
            state,
            tracer_provider,
        })
    }
}
