use std::sync::Arc;

use chrono::{NaiveDate, Utc};

use crate::{
    app::AppError as GlobalAppError,
    features::alerts::{
        app::{AlertFarms, AlertRepository, AppError},
        domain::{Alert, AlertConfidence, AlertKey, AlertLevel, AlertSource, AlertText, AlertType},
    },
};

pub struct RecordAlertInput {
    pub farm_id: i32,
    pub key: AlertKey,
    pub alert_type: AlertType,
    pub day: NaiveDate,
    pub level: AlertLevel,
    pub confidence: AlertConfidence,
    pub ku: AlertText,
    pub en: AlertText,
    pub action_ku: AlertText,
    pub action_en: AlertText,
    pub source: AlertSource,
}

pub struct RecordAlertUseCase {
    repository: Arc<dyn AlertRepository>,
    farms: Arc<dyn AlertFarms>,
}

impl RecordAlertUseCase {
    pub fn new(repository: Arc<dyn AlertRepository>, farms: Arc<dyn AlertFarms>) -> Self {
        Self { repository, farms }
    }

    /// Stores a job's alert for a farm under its key. Sending the same key
    /// again replaces the wording and never unticks or un-pushes the alert.
    pub async fn execute(&self, input: RecordAlertInput) -> Result<Alert, AppError> {
        // A job gets a clear 404 for a farm that is gone. The read does not
        // guard the write: an alert stored for a farm deleted a moment later
        // is never served, because every read starts from a farm.
        if self.farms.owner_of(input.farm_id).await?.is_none() {
            tracing::info!(farm_id = input.farm_id, "alert refused: no such farm");

            return Err(GlobalAppError::NotFound.into());
        }

        let alert = Alert::new(
            input.farm_id,
            input.key,
            input.alert_type,
            input.day,
            input.level,
            input.confidence,
            input.ku,
            input.en,
            input.action_ku,
            input.action_en,
            input.source,
            Utc::now(),
        )?;

        let stored = self.repository.upsert(&alert).await?;

        tracing::info!(
            farm_id = *stored.farm_id(),
            key = stored.key().as_str(),
            level = %String::from(*stored.level()),
            "alert stored"
        );

        Ok(stored)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::alerts::app::testing::{Call, FARM_ID, Fakes, an_input};

    fn use_case(fakes: &Fakes) -> RecordAlertUseCase {
        RecordAlertUseCase::new(Arc::new(fakes.clone()), Arc::new(fakes.clone()))
    }

    #[tokio::test]
    async fn stores_a_new_alert_unticked_and_unpushed() {
        let fakes = Fakes::new();

        let alert = use_case(&fakes)
            .execute(an_input(FARM_ID, "frost:2026-10-11", "Frost"))
            .await
            .expect("stored");

        assert!(alert.id().is_some());
        assert!(!alert.done() && !alert.pushed());
        assert_eq!(
            fakes.calls(),
            vec![
                Call::OwnerOf { farm_id: FARM_ID },
                Call::Upsert {
                    farm_id: FARM_ID,
                    key: "frost:2026-10-11".to_string()
                },
            ]
        );
    }

    #[tokio::test]
    async fn the_same_key_again_replaces_the_text_but_keeps_the_tick_and_the_push() {
        let fakes = Fakes::new();
        let use_case = use_case(&fakes);

        let first = use_case
            .execute(an_input(FARM_ID, "frost:2026-10-11", "Frost"))
            .await
            .expect("first");
        let id = first.id().expect("id");

        fakes.tick_and_push(id);

        let second = use_case
            .execute(an_input(FARM_ID, "frost:2026-10-11", "Hard frost"))
            .await
            .expect("second");

        assert_eq!(*second.id(), Some(id), "no second alert for the same key");
        assert_eq!(second.en().as_str(), "Hard frost");
        assert!(
            *second.done() && *second.pushed(),
            "a job running again must not untick what the farmer ticked"
        );
        assert_eq!(fakes.stored_count(), 1);
    }

    #[tokio::test]
    async fn an_unknown_farm_is_not_found_and_nothing_is_stored() {
        let fakes = Fakes::new();

        let error = use_case(&fakes)
            .execute(an_input(999, "frost:2026-10-11", "Frost"))
            .await
            .expect_err("refused");

        assert!(matches!(
            error,
            AppError::GlobalAppError(GlobalAppError::NotFound)
        ));
        assert_eq!(fakes.stored_count(), 0);
    }
}
