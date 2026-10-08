use std::time::Duration;

use sea_orm::{ConnectOptions, DatabaseConnection, DbErr, SqlxPostgresConnector};

use crate::infra::config::{Config, Database};

#[derive(Clone, Debug)]
pub struct DBConnector(DatabaseConnection);

impl DBConnector {
    pub async fn init(config: &Database) -> Result<Self, DbErr> {
        let mut options = ConnectOptions::new(config.url.clone());

        options
            .max_connections(config.max_connections)
            .min_connections(config.min_connections)
            .idle_timeout(Duration::from_secs(300))
            .max_lifetime(Duration::from_secs(1800))
            .acquire_timeout(Duration::from_secs(8))
            .sqlx_logging(true)
            .connect_timeout(Duration::from_secs(8));

        let conn = SqlxPostgresConnector::connect(options).await?;

        Ok(Self(conn))
    }

    pub fn conn(&self) -> &DatabaseConnection {
        &self.0
    }

    pub fn into_conn(self) -> DatabaseConnection {
        self.0
    }

    pub fn conn_clone(&self) -> DatabaseConnection {
        self.0.clone()
    }

    pub fn from_conn(conn: DatabaseConnection) -> Self {
        Self(conn)
    }

    pub async fn close(self) -> Result<(), DbErr> {
        self.0.close().await
    }
}

pub async fn postgres_init(
    config: &Config,
) -> Result<DBConnector, Box<dyn std::error::Error + Send + Sync>> {
    let db_context = DBConnector::init(&config.database).await?;

    Ok(db_context)
}
