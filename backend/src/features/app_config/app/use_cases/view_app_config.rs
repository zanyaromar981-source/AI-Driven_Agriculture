use std::sync::Arc;

use crate::features::app_config::{
    app::{AppConfigRepository, AppError},
    domain::AppConfig,
};

pub struct ViewAppConfigUseCase {
    repository: Arc<dyn AppConfigRepository>,
}

impl ViewAppConfigUseCase {
    pub fn new(repository: Arc<dyn AppConfigRepository>) -> Self {
        Self { repository }
    }

    /// Returns the config as it is stored, for the app at start and for the
    /// staff page that edits it.
    pub async fn execute(&self) -> Result<AppConfig, AppError> {
        self.repository.find().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::app_config::app::testing::{Call, Fakes, settings};

    #[tokio::test]
    async fn returns_the_stored_config() {
        let fakes = Fakes::new();

        let config = ViewAppConfigUseCase::new(Arc::new(fakes.clone()))
            .execute()
            .await
            .expect("config");

        assert_eq!(config.settings(), &settings());
        assert_eq!(fakes.calls(), vec![Call::Find]);
    }

    #[tokio::test]
    async fn a_repository_failure_surfaces() {
        let result = ViewAppConfigUseCase::new(Arc::new(Fakes::new().failing()))
            .execute()
            .await;

        assert!(result.is_err());
    }
}
