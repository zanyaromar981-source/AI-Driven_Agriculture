// Use Mimalloc as allocator
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use std::time::Duration;

use axum::{Router, middleware};
use clap::{Parser, Subcommand};
use farm_doctor_api::{
    features::{
        alwa::web::{
            ingest_routes as alwa_ingest_routes, public_routes as alwa_public_routes,
            routes as alwa_routes,
        },
        dams::web::{ingest_routes as dam_ingest_routes, public_routes as dam_public_routes},
        farmers::web::{public_routes as farmer_public_routes, routes as farmer_routes},
        farms::web::routes as farm_routes,
        fires::web::{ingest_routes as fire_ingest_routes, public_routes as fire_public_routes},
        insights::web::{ingest_routes as insight_ingest_routes, routes as insight_routes},
        outlooks::web::{
            ingest_routes as outlook_ingest_routes, public_routes as outlook_public_routes,
        },
        water::web::{ingest_routes as water_ingest_routes, public_routes as water_public_routes},
        zones::web::{ingest_routes as zone_ingest_routes, public_routes as zone_public_routes},
    },
    infra::{
        BootstrappedApp, Config,
        http::{auth, health_routes, service_key, swagger_ui},
        postgres_init, telemetry,
    },
    shared::{AppState, Phone, issue_jwt},
};
use migration::{Migrator, MigratorTrait};
use tokio::net::TcpListener;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Clone, Debug)]
enum Commands {
    /// HTTP API server (default).
    Serve,
    /// Run database migrations.
    Migrate,
    /// Print a sign-in token for a phone number, for local work with curl.
    Token {
        /// E.164 Iraqi mobile number, for example +9647501234567.
        phone: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Migrate) => return run_migrations().await,
        Some(Commands::Token { phone }) => return print_token(phone),
        Some(Commands::Serve) | None => {}
    }

    let bootstrapped_app = BootstrappedApp::start().await?;
    let state = bootstrapped_app.state().clone();

    let api = Router::<AppState>::new()
        .nest(
            "/v1",
            Router::new()
                .merge(farm_routes())
                .merge(farmer_routes())
                .merge(insight_routes())
                .merge(alwa_routes())
                .layer(middleware::from_fn_with_state(state.clone(), auth))
                .merge(farmer_public_routes())
                .merge(fire_public_routes())
                .merge(zone_public_routes())
                .merge(dam_public_routes())
                .merge(outlook_public_routes())
                .merge(water_public_routes())
                .merge(alwa_public_routes())
                .nest(
                    "/ingest",
                    Router::new()
                        .merge(fire_ingest_routes())
                        .merge(insight_ingest_routes())
                        .merge(zone_ingest_routes())
                        .merge(dam_ingest_routes())
                        .merge(outlook_ingest_routes())
                        .merge(water_ingest_routes())
                        .merge(alwa_ingest_routes())
                        .layer(middleware::from_fn_with_state(state.clone(), service_key)),
                ),
        )
        .merge(health_routes())
        .merge(swagger_ui())
        .layer(telemetry::http_trace_layer())
        .with_state(state.clone());

    let listener = TcpListener::bind(format!("0.0.0.0:{}", state.config.server.port)).await?;

    tracing::info!(port = state.config.server.port, "starting http server");

    axum::serve(listener, api)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    if let Some(tracer_provider) = bootstrapped_app.tracer_provider() {
        tracer_provider.shutdown()?;
    }

    Ok(())
}

async fn run_migrations() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let config = Config::from_env();
    let db_context = postgres_init(&config).await?;

    println!("Running database migrations...");

    Migrator::up(db_context.conn(), None).await?;

    println!("Database migrations completed successfully!");

    Ok(())
}

fn print_token(phone: String) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let config = Config::from_env();
    let phone = Phone::new(phone)?;

    let token = issue_jwt(
        phone.as_str(),
        &config.auth.jwt_secret,
        &config.auth.issuer,
        &config.auth.audience,
        Duration::from_secs(config.auth.token_ttl_days * 86_400),
    )?;

    println!("{token}");

    Ok(())
}

pub async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };
    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();
    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
