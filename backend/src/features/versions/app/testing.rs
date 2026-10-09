use async_trait::async_trait;

use crate::features::versions::{
    app::{AppError, VersionRepository},
    domain::{DataVersion, Topic},
};

#[derive(Debug, Clone, Default)]
pub struct FakeVersionRepository {
    fail_with_database_error: bool,
}

impl FakeVersionRepository {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn failing() -> Self {
        Self {
            fail_with_database_error: true,
        }
    }
}

#[async_trait]
impl VersionRepository for FakeVersionRepository {
    async fn find_all(&self) -> Result<Vec<DataVersion>, AppError> {
        if self.fail_with_database_error {
            return Err(crate::app::AppError::DatabaseError("fake".to_string()).into());
        }

        Ok(Topic::ALL
            .into_iter()
            .zip(1i64..)
            .map(|(topic, version)| DataVersion::rehydrate(topic, version, chrono::Utc::now()))
            .collect())
    }
}
