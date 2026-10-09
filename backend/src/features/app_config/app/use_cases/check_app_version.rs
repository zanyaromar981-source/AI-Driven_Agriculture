use std::sync::Arc;

use chrono::{DateTime, Duration, Utc};

use crate::{
    features::app_config::{
        app::{AppConfigRepository, AppError, AppFarmers, VersionGate},
        domain::AppVersion,
    },
    shared::Phone,
};

pub struct CheckAppVersionUseCase {
    repository: Arc<dyn AppConfigRepository>,
    farmers: Arc<dyn AppFarmers>,
    gate: Arc<VersionGate>,
}

impl CheckAppVersionUseCase {
    pub fn new(
        repository: Arc<dyn AppConfigRepository>,
        farmers: Arc<dyn AppFarmers>,
        gate: Arc<VersionGate>,
    ) -> Self {
        Self {
            repository,
            farmers,
            gate,
        }
    }

    /// Decides whether a request from the app may go on, from the version
    /// the app declared in `X-App-Version`. `phone` is the signed-in
    /// farmer's, when the route has one.
    ///
    /// It runs on every farmer request, so it reads the database only when
    /// what it remembers has grown stale: the oldest version allowed every
    /// 30 seconds, and one write per farmer and version per hour.
    pub async fn execute(
        &self,
        declared: Option<&str>,
        phone: Option<&Phone>,
    ) -> Result<(), AppError> {
        self.execute_at(declared, phone, Utc::now()).await
    }

    pub async fn execute_at(
        &self,
        declared: Option<&str>,
        phone: Option<&Phone>,
        now: DateTime<Utc>,
    ) -> Result<(), AppError> {
        // No header, or one that is not a version: older builds of the app
        // and tools like curl send none, and they are let through.
        let Some(version) = declared.and_then(|declared| AppVersion::new(declared).ok()) else {
            return Ok(());
        };

        // Noted before it is judged, so staff also see how many farmers
        // are still on a version that is refused.
        if let Some(phone) = phone {
            self.note(phone, version, now).await;
        }

        let Some(min_version) = self.min_version(now).await else {
            return Ok(());
        };

        if version < min_version {
            tracing::info!(
                %version,
                %min_version,
                "request refused: the app must be updated"
            );

            return Err(AppError::UpdateRequired {
                min_version: min_version.to_string(),
            });
        }

        Ok(())
    }

    /// The oldest version allowed, from memory when it was read in the last
    /// 30 seconds. When it cannot be read, the last value known is used, and
    /// with none known nothing is refused: a database fault must not look
    /// like "update your app".
    async fn min_version(&self, now: DateTime<Utc>) -> Option<AppVersion> {
        if let Some(min_version) = self.gate.fresh_min_version(now) {
            return Some(min_version);
        }

        match self.repository.find().await {
            Ok(config) => {
                let min_version = config.settings().min_version;

                self.gate.remember_min_version(min_version, now);

                Some(min_version)
            }
            Err(error) => {
                tracing::warn!(%error, "the oldest app version allowed could not be read");

                self.gate.last_min_version()
            }
        }
    }

    /// Writes down that this farmer used this version, once an hour at
    /// most. A failure here never fails the farmer's request.
    async fn note(&self, phone: &Phone, version: AppVersion, now: DateTime<Utc>) {
        if !self.gate.claim_sighting(phone.as_str(), version, now) {
            return;
        }

        let noted = match self.farmers.farmer_id_of(phone).await {
            Ok(Some(farmer_id)) => {
                self.repository
                    .record_sighting(
                        farmer_id,
                        &version,
                        now,
                        now - Duration::minutes(VersionGate::SIGHTING_GAP_MINUTES),
                    )
                    .await
            }
            // The farmer was removed after the token was checked: there is
            // nobody to note.
            Ok(None) => Ok(()),
            Err(error) => Err(error),
        };

        if let Err(error) = noted {
            tracing::warn!(%error, %version, "an app version in use could not be noted");

            self.gate.release_sighting(phone.as_str(), version);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::app_config::app::testing::{Call, FARMER_ID, Fakes, PHONE, a_time, phone};

    fn use_case(fakes: &Fakes) -> CheckAppVersionUseCase {
        CheckAppVersionUseCase::new(
            Arc::new(fakes.clone()),
            Arc::new(fakes.clone()),
            Arc::new(VersionGate::new()),
        )
    }

    fn finds(fakes: &Fakes) -> usize {
        fakes
            .calls()
            .iter()
            .filter(|call| matches!(call, Call::Find))
            .count()
    }

    fn sightings(fakes: &Fakes) -> usize {
        fakes
            .calls()
            .iter()
            .filter(|call| matches!(call, Call::RecordSighting { .. }))
            .count()
    }

    #[tokio::test]
    async fn a_version_below_the_oldest_allowed_must_update() {
        let fakes = Fakes::new().with_min_version("1.2.0");

        let result = use_case(&fakes)
            .execute_at(Some("1.1.9"), None, a_time())
            .await;

        let Err(AppError::UpdateRequired { min_version }) = result else {
            panic!("expected update required, got {result:?}");
        };

        assert_eq!(min_version, "1.2.0");
    }

    #[tokio::test]
    async fn the_oldest_allowed_version_itself_and_newer_ones_pass() {
        let fakes = Fakes::new().with_min_version("1.2.0");
        let use_case = use_case(&fakes);

        for declared in ["1.2.0", "1.2.1", "1.10.0", "2.0.0"] {
            assert!(
                use_case
                    .execute_at(Some(declared), None, a_time())
                    .await
                    .is_ok(),
                "{declared} must pass"
            );
        }
    }

    #[tokio::test]
    async fn the_comparison_is_by_number_not_by_text() {
        let fakes = Fakes::new().with_min_version("1.9.0");

        assert!(
            use_case(&fakes)
                .execute_at(Some("1.10.0"), None, a_time())
                .await
                .is_ok(),
            "1.10.0 is newer than 1.9.0"
        );
    }

    #[tokio::test]
    async fn no_header_is_let_through_without_reading_anything() {
        let fakes = Fakes::new().with_min_version("9.0.0");

        assert!(
            use_case(&fakes)
                .execute_at(None, Some(&phone()), a_time())
                .await
                .is_ok()
        );
        assert!(fakes.calls().is_empty());
    }

    #[tokio::test]
    async fn a_header_that_is_not_a_version_is_let_through() {
        let fakes = Fakes::new().with_min_version("9.0.0");
        let use_case = use_case(&fakes);

        for declared in ["", "abc", "1.0", "v1.0.0", "1.0.0-beta"] {
            assert!(
                use_case
                    .execute_at(Some(declared), Some(&phone()), a_time())
                    .await
                    .is_ok(),
                "{declared:?} must be let through"
            );
        }

        assert!(fakes.calls().is_empty());
    }

    #[tokio::test]
    async fn the_oldest_allowed_version_is_read_once_in_thirty_seconds() {
        let fakes = Fakes::new();
        let use_case = use_case(&fakes);

        for seconds in [0, 1, 10, 29] {
            use_case
                .execute_at(Some("1.0.0"), None, a_time() + Duration::seconds(seconds))
                .await
                .expect("pass");
        }

        assert_eq!(
            finds(&fakes),
            1,
            "the database is not asked on every request"
        );

        use_case
            .execute_at(Some("1.0.0"), None, a_time() + Duration::seconds(30))
            .await
            .expect("pass");

        assert_eq!(finds(&fakes), 2, "after 30 seconds it is read again");
    }

    #[tokio::test]
    async fn a_signed_in_farmer_is_noted_once_an_hour_per_version() {
        let fakes = Fakes::new();
        let use_case = use_case(&fakes);

        for minutes in [0, 1, 30, 59] {
            use_case
                .execute_at(
                    Some("1.0.3"),
                    Some(&phone()),
                    a_time() + Duration::minutes(minutes),
                )
                .await
                .expect("pass");
        }

        assert_eq!(sightings(&fakes), 1, "not a write on every request");
        assert!(fakes.calls().contains(&Call::FarmerIdOf {
            phone: PHONE.to_string()
        }));
        assert!(fakes.calls().contains(&Call::RecordSighting {
            farmer_id: FARMER_ID,
            version: "1.0.3".to_string(),
            unless_after: a_time() - Duration::minutes(60),
        }));

        use_case
            .execute_at(
                Some("1.0.3"),
                Some(&phone()),
                a_time() + Duration::minutes(60),
            )
            .await
            .expect("pass");
        use_case
            .execute_at(Some("1.1.0"), Some(&phone()), a_time())
            .await
            .expect("pass");

        assert_eq!(
            sightings(&fakes),
            3,
            "an hour later, and another version, are each noted"
        );
    }

    #[tokio::test]
    async fn a_request_with_no_farmer_is_checked_but_not_noted() {
        let fakes = Fakes::new();

        use_case(&fakes)
            .execute_at(Some("1.0.3"), None, a_time())
            .await
            .expect("pass");

        assert_eq!(fakes.calls(), vec![Call::Find]);
    }

    #[tokio::test]
    async fn a_refused_version_is_still_noted_so_staff_see_who_is_stuck() {
        let fakes = Fakes::new().with_min_version("2.0.0");

        let result = use_case(&fakes)
            .execute_at(Some("1.0.3"), Some(&phone()), a_time())
            .await;

        assert!(matches!(result, Err(AppError::UpdateRequired { .. })));
        assert_eq!(sightings(&fakes), 1);
    }

    #[tokio::test]
    async fn a_removed_farmer_is_not_noted() {
        let fakes = Fakes::new().without_farmer();

        use_case(&fakes)
            .execute_at(Some("1.0.3"), Some(&phone()), a_time())
            .await
            .expect("pass");

        assert_eq!(sightings(&fakes), 0);
    }

    #[tokio::test]
    async fn with_the_database_down_and_nothing_remembered_nobody_is_refused() {
        let fakes = Fakes::new().with_min_version("2.0.0").failing();

        let result = use_case(&fakes)
            .execute_at(Some("1.0.0"), Some(&phone()), a_time())
            .await;

        assert!(
            result.is_ok(),
            "a database fault must not tell a farmer to update"
        );
    }

    #[tokio::test]
    async fn with_the_database_down_the_last_known_version_still_holds() {
        let fakes = Fakes::new().with_min_version("2.0.0");
        let use_case = use_case(&fakes);

        use_case
            .execute_at(Some("2.0.0"), None, a_time())
            .await
            .expect("pass");

        fakes.set_failing(true);

        let result = use_case
            .execute_at(Some("1.0.0"), None, a_time() + Duration::minutes(5))
            .await;

        assert!(matches!(result, Err(AppError::UpdateRequired { .. })));
    }

    #[tokio::test]
    async fn a_sighting_that_failed_to_write_is_tried_again_on_the_next_request() {
        let fakes = Fakes::new();
        let use_case = use_case(&fakes);

        // The config is remembered first, so only the sighting fails below.
        use_case
            .execute_at(Some("1.0.3"), None, a_time())
            .await
            .expect("pass");

        fakes.set_failing(true);
        use_case
            .execute_at(Some("1.0.3"), Some(&phone()), a_time())
            .await
            .expect("a failed note never fails the request");

        fakes.set_failing(false);
        use_case
            .execute_at(Some("1.0.3"), Some(&phone()), a_time())
            .await
            .expect("pass");

        assert_eq!(
            sightings(&fakes),
            1,
            "the failed attempt stopped at the farmer lookup, the retry wrote"
        );
    }
}
