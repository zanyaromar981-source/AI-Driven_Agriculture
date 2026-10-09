use std::sync::Arc;

use chrono::Utc;

use crate::{
    app::StaffContext,
    features::app_config::{
        app::{AppConfigRepository, AppError, VersionGate},
        domain::{AppConfig, AppSettings},
    },
};

pub struct UpdateAppConfigUseCase {
    repository: Arc<dyn AppConfigRepository>,
    gate: Arc<VersionGate>,
}

impl UpdateAppConfigUseCase {
    pub fn new(repository: Arc<dyn AppConfigRepository>, gate: Arc<VersionGate>) -> Self {
        Self { repository, gate }
    }

    /// Replaces the whole config with what staff saved and returns it as
    /// stored. Saving the same settings twice leaves the same config.
    pub async fn execute(
        &self,
        actor: &StaffContext,
        settings: AppSettings,
    ) -> Result<AppConfig, AppError> {
        let now = Utc::now();

        let config = AppConfig::new(settings, *actor.staff_id(), now)?;

        let stored = self.repository.replace(&config).await?;

        // This server refuses old apps from the next request on, without
        // waiting for its remembered value to grow stale.
        self.gate
            .remember_min_version(stored.settings().min_version, now);

        tracing::info!(
            staff_id = *actor.staff_id(),
            latest_version = %stored.settings().latest_version,
            min_version = %stored.settings().min_version,
            maintenance = stored.settings().maintenance,
            "app config replaced by staff"
        );

        Ok(stored)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::app_config::{
        app::testing::{Call, Fakes, STAFF_ID, settings, staff_context, version},
        domain::AppConfigError,
    };

    fn use_case(fakes: &Fakes, gate: &Arc<VersionGate>) -> UpdateAppConfigUseCase {
        UpdateAppConfigUseCase::new(Arc::new(fakes.clone()), gate.clone())
    }

    #[tokio::test]
    async fn stores_what_staff_saved_with_who_saved_it() {
        let fakes = Fakes::new();
        let gate = Arc::new(VersionGate::new());

        let stored = use_case(&fakes, &gate)
            .execute(
                &staff_context(),
                AppSettings {
                    latest_version: version("1.3.0"),
                    min_version: version("1.2.0"),
                    ..settings()
                },
            )
            .await
            .expect("config");

        assert_eq!(*stored.updated_by(), Some(STAFF_ID));
        assert_eq!(fakes.stored().settings().min_version, version("1.2.0"));
        assert_eq!(
            fakes.calls(),
            vec![Call::Replace {
                min_version: "1.2.0".to_string(),
                updated_by: Some(STAFF_ID)
            }],
            "one write, with no read before it"
        );
    }

    #[tokio::test]
    async fn the_new_oldest_version_is_enforced_at_once() {
        let fakes = Fakes::new();
        let gate = Arc::new(VersionGate::new());

        use_case(&fakes, &gate)
            .execute(
                &staff_context(),
                AppSettings {
                    latest_version: version("1.3.0"),
                    min_version: version("1.2.0"),
                    ..settings()
                },
            )
            .await
            .expect("config");

        assert_eq!(gate.last_min_version(), Some(version("1.2.0")));
    }

    #[tokio::test]
    async fn settings_that_disagree_are_refused_and_nothing_is_written() {
        let fakes = Fakes::new();
        let gate = Arc::new(VersionGate::new());

        let result = use_case(&fakes, &gate)
            .execute(
                &staff_context(),
                AppSettings {
                    latest_version: version("1.9.0"),
                    min_version: version("1.10.0"),
                    ..settings()
                },
            )
            .await;

        assert!(matches!(
            result,
            Err(AppError::AppConfig(AppConfigError::MinAboveLatest { .. }))
        ));
        assert!(fakes.calls().is_empty());
        assert_eq!(gate.last_min_version(), None);
    }

    #[tokio::test]
    async fn a_failed_write_does_not_change_what_is_enforced() {
        let fakes = Fakes::new().failing();
        let gate = Arc::new(VersionGate::new());

        let result = use_case(&fakes, &gate)
            .execute(&staff_context(), settings())
            .await;

        assert!(result.is_err());
        assert_eq!(gate.last_min_version(), None);
    }
}
