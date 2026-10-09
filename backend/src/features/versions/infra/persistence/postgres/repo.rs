use async_trait::async_trait;
use sea_orm::{DatabaseConnection, DbErr, EntityTrait, QueryOrder};

use crate::{
    app::AppError as GlobalAppError,
    features::versions::{
        app::{AppError, VersionRepository},
        domain::{DataVersion, Topic},
        infra::persistence::postgres::entities::data_versions,
    },
};

fn database_error(error: DbErr) -> AppError {
    tracing::error!(%error, "version repository query failed");

    GlobalAppError::DatabaseError(error.to_string()).into()
}

#[derive(Debug)]
pub struct VersionPostgresRepository {
    conn: DatabaseConnection,
}

impl VersionPostgresRepository {
    pub fn new(conn: DatabaseConnection) -> Self {
        Self { conn }
    }
}

#[async_trait]
impl VersionRepository for VersionPostgresRepository {
    async fn find_all(&self) -> Result<Vec<DataVersion>, AppError> {
        let models = data_versions::Entity::find()
            .order_by_asc(data_versions::Column::Topic)
            .all(&self.conn)
            .await
            .map_err(database_error)?;

        // A row whose topic this build does not know is skipped, so an older
        // server keeps answering after a newer migration added a topic.
        Ok(models
            .into_iter()
            .filter_map(|model| {
                Topic::parse(&model.topic).map(|topic| {
                    DataVersion::rehydrate(topic, model.version, model.changed_at.and_utc())
                })
            })
            .collect())
    }
}
