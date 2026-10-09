use std::env;
use std::path::Path;

fn main() {
    /* IMPORTANT: For the entity generation to work, these two things must be done:
        1. Run cargo clean
        2. Run PROFILE=debug cargo build
    */

    // Only run if we're building the main crate and in development mode
    if env::var("CARGO_PKG_NAME").unwrap() == "farm-doctor-api" {
        // Check if we're in development mode
        let is_dev = env::var("PROFILE").unwrap_or_default() == "debug"
            || env::var("DEV").is_ok()
            || env::var("CARGO_ENV").unwrap_or_default() == "development";

        if is_dev {
            // Load environment variables from .env file
            dotenvy::dotenv().ok();
            // Check if we need to regenerate entities
            let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

            // Try to get database URL from environment variables
            let database_url = match (
                env::var("DATABASE__USERNAME"),
                env::var("DATABASE__PASSWORD"),
                env::var("DATABASE__HOST"),
                env::var("DATABASE__PORT"),
                env::var("DATABASE__NAME"),
            ) {
                (Ok(username), Ok(password), Ok(host), Ok(port), Ok(name)) => {
                    format!(
                        "postgres://{}:{}@{}:{}/{}",
                        username, password, host, port, name
                    )
                }
                (Ok(username), Ok(password), Ok(host), _, Ok(name)) => {
                    format!(
                        "postgres://{}:{}@{}:5432/{}",
                        username, password, host, name
                    )
                }
                _ => {
                    eprintln!(
                        "Warning: Database environment variables not found, skipping entity generation"
                    );
                    return;
                }
            };

            // One invocation per feature: `-t` limits generation to that
            // feature's tables so each entity lands inside the feature that
            // owns it, instead of one shared module outside the features.
            let entity_targets = [
                (
                    "farms,farm_cells",
                    "src/features/farms/infra/persistence/postgres/entities",
                ),
                (
                    "farmers,sign_in_challenges,letters",
                    "src/features/farmers/infra/persistence/postgres/entities",
                ),
                (
                    "fires",
                    "src/features/fires/infra/persistence/postgres/entities",
                ),
                (
                    "farm_insights",
                    "src/features/insights/infra/persistence/postgres/entities",
                ),
                (
                    "zones,sub_zones,zone_readings,sub_zone_readings",
                    "src/features/zones/infra/persistence/postgres/entities",
                ),
                (
                    "dams,dam_readings",
                    "src/features/dams/infra/persistence/postgres/entities",
                ),
                (
                    "season_outlooks,outlook_runs",
                    "src/features/outlooks/infra/persistence/postgres/entities",
                ),
                (
                    "water_plan_entries",
                    "src/features/water/infra/persistence/postgres/entities",
                ),
                (
                    "alwa_markets,alwa_prices,alwa_listings,alwa_offers",
                    "src/features/alwa/infra/persistence/postgres/entities",
                ),
                (
                    "staff,roles,role_permissions,staff_roles",
                    "src/features/staff/infra/persistence/postgres/entities",
                ),
                (
                    "daily_briefs,farm_brief_zones",
                    "src/features/briefs/infra/persistence/postgres/entities",
                ),
                (
                    "data_versions",
                    "src/features/versions/infra/persistence/postgres/entities",
                ),
            ];

            for (table, output_dir) in entity_targets {
                let entity_path = Path::new(&manifest_dir).join(output_dir);

                let status = std::process::Command::new("sea-orm-cli")
                    .args([
                        "generate",
                        "entity",
                        "--database-url",
                        &database_url,
                        "-t",
                        table,
                        "-o",
                        entity_path.to_str().unwrap(),
                        "--with-copy-enums",
                        "--date-time-crate",
                        "chrono",
                        "--with-prelude",
                        "none",
                        "--impl-active-model-behavior=true",
                    ])
                    .status();

                match status {
                    Ok(status) if !status.success() => {
                        eprintln!(
                            "Warning: sea-orm-cli failed for tables '{table}', using existing entity"
                        );
                    }
                    Err(_) => {
                        eprintln!("Warning: sea-orm-cli not found, using existing entities");
                        break;
                    }
                    _ => {}
                }
            }
        }
    }
}
