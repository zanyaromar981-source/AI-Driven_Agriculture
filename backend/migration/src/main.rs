use sea_orm_migration::prelude::*;

#[cfg(debug_assertions)]
fn load_env() {
    dotenvy::dotenv().ok();

    let database_host = std::env::var("DATABASE__HOST").unwrap();
    let database_port = std::env::var("DATABASE__PORT").unwrap_or_else(|_| "5432".to_string());
    let database_user = std::env::var("DATABASE__USERNAME").unwrap();
    let database_password = std::env::var("DATABASE__PASSWORD").unwrap();
    let database_name = std::env::var("DATABASE__NAME").unwrap();

    let database_url = format!(
        "postgres://{}:{}@{}:{}/{}",
        database_user, database_password, database_host, database_port, database_name
    );

    std::env::set_var("DATABASE_URL", database_url);
}

#[cfg(not(debug_assertions))]
fn load_env() {}

#[tokio::main]
async fn main() {
    load_env();
    cli::run_cli(migration::Migrator).await;
}
