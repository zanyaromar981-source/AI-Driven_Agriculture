use std::sync::Arc;

use chrono::Utc;

use crate::{
    app::{AppError as GlobalAppError, StaffContext},
    features::farmers::{
        app::{AppError, FarmCounter, FarmerRecord, FarmerRepository},
        domain::FarmerChange,
    },
};

pub struct EditFarmerUseCase {
    farmers: Arc<dyn FarmerRepository>,
    farms: Arc<dyn FarmCounter>,
}

impl EditFarmerUseCase {
    pub fn new(farmers: Arc<dyn FarmerRepository>, farms: Arc<dyn FarmCounter>) -> Self {
        Self { farmers, farms }
    }

    pub async fn execute(
        &self,
        actor: &StaffContext,
        id: i32,
        change: FarmerChange,
    ) -> Result<FarmerRecord, AppError> {
        let Some(farmer) = self.farmers.update_by_id(id, &change, Utc::now()).await? else {
            tracing::info!(
                staff_id = *actor.staff_id(),
                farmer_id = id,
                "farmer edit by staff refused: no such farmer"
            );

            return Err(GlobalAppError::NotFound.into());
        };

        let farms_count = self.farms.count_for(farmer.phone()).await?;

        tracing::info!(
            staff_id = *actor.staff_id(),
            farmer_id = id,
            blocked = *farmer.blocked(),
            "farmer edited by staff"
        );

        Ok(FarmerRecord {
            farmer,
            farms_count,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::farmers::{
        app::testing::{Call, Fakes, PHONE, staff_context},
        domain::{FarmerDetails, FarmerName, Gender, Language, Village},
    };

    fn use_case(fakes: &Fakes) -> EditFarmerUseCase {
        EditFarmerUseCase::new(Arc::new(fakes.clone()), Arc::new(fakes.clone()))
    }

    fn input(name: Option<&str>) -> FarmerChange {
        FarmerChange {
            name: name.map(|name| FarmerName::new(name.to_string()).expect("name")),
            language: Language::English,
            details: FarmerDetails::default(),
            blocked: None,
        }
    }

    #[tokio::test]
    async fn replaces_the_name_and_language_and_keeps_the_phone() {
        let fakes = Fakes::new().with_farmer();

        let record = use_case(&fakes)
            .execute(&staff_context(), 1, input(Some("Hiwa K.")))
            .await
            .expect("farmer");

        assert_eq!(*record.farmer.language(), Language::English);
        assert_eq!(record.farmer.phone().as_str(), PHONE);
        assert_eq!(
            fakes.calls()[0],
            Call::UpdateFarmerById { id: 1 },
            "the update is the first call: no lookup decides before it"
        );
    }

    #[tokio::test]
    async fn a_null_name_clears_it() {
        let fakes = Fakes::new().with_farmer();

        let record = use_case(&fakes)
            .execute(&staff_context(), 1, input(None))
            .await
            .expect("farmer");

        assert!(record.farmer.name().is_none());
    }

    #[tokio::test]
    async fn stores_the_details_staff_recorded() {
        let fakes = Fakes::new().with_farmer();

        let record = use_case(&fakes)
            .execute(
                &staff_context(),
                1,
                FarmerChange {
                    details: FarmerDetails {
                        gender: Some(Gender::Female),
                        village: Some(Village::new("Sangaw".to_string()).expect("village")),
                        ..FarmerDetails::default()
                    },
                    ..input(None)
                },
            )
            .await
            .expect("farmer");

        assert_eq!(record.farmer.details().gender, Some(Gender::Female));
        assert_eq!(
            record
                .farmer
                .details()
                .village
                .as_ref()
                .map(Village::as_str),
            Some("Sangaw")
        );
    }

    #[tokio::test]
    async fn blocking_and_unblocking_happen_only_when_asked_for() {
        let fakes = Fakes::new().with_farmer();
        let use_case = use_case(&fakes);

        let blocked = use_case
            .execute(
                &staff_context(),
                1,
                FarmerChange {
                    blocked: Some(true),
                    ..input(None)
                },
            )
            .await
            .expect("farmer");
        assert!(*blocked.farmer.blocked());

        let renamed = use_case
            .execute(&staff_context(), 1, input(Some("Hiwa K.")))
            .await
            .expect("farmer");
        assert!(
            *renamed.farmer.blocked(),
            "an edit that does not mention blocking must not let the farmer back in"
        );

        let unblocked = use_case
            .execute(
                &staff_context(),
                1,
                FarmerChange {
                    blocked: Some(false),
                    ..input(None)
                },
            )
            .await
            .expect("farmer");
        assert!(!*unblocked.farmer.blocked());
    }

    #[tokio::test]
    async fn is_not_found_when_there_is_no_such_farmer() {
        let fakes = Fakes::new();

        assert!(matches!(
            use_case(&fakes)
                .execute(&staff_context(), 1, input(None))
                .await,
            Err(AppError::GlobalAppError(GlobalAppError::NotFound))
        ));
    }
}
