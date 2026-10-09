use std::sync::Arc;

use crate::{
    app::{AppError as GlobalAppError, AuthContext},
    features::briefs::{
        app::{AppError, BriefFarmOwnership, BriefRepository},
        domain::{BriefScope, FarmBrief},
    },
};

pub struct ViewFarmBriefUseCase {
    repository: Arc<dyn BriefRepository>,
    ownership: Arc<dyn BriefFarmOwnership>,
}

impl ViewFarmBriefUseCase {
    pub fn new(
        repository: Arc<dyn BriefRepository>,
        ownership: Arc<dyn BriefFarmOwnership>,
    ) -> Self {
        Self {
            repository,
            ownership,
        }
    }

    /// Returns the newest brief of the district the farm is recorded in.
    /// When the farm has no district recorded yet, or that district has no
    /// brief yet, the newest brief of the whole region stands in. With
    /// nothing stored at all there is no brief, which is not an error. A
    /// farm of another phone is answered like one that does not exist.
    pub async fn execute(
        &self,
        auth_context: &AuthContext,
        farm_id: i32,
    ) -> Result<FarmBrief, AppError> {
        if !self
            .ownership
            .is_owned_by(farm_id, auth_context.user().phone())
            .await?
        {
            tracing::info!(farm_id, "brief refused: no such farm for this owner");

            return Err(GlobalAppError::NotFound.into());
        }

        let zone_slug = self.repository.find_farm_zone(farm_id).await?;

        let of_zone = match &zone_slug {
            Some(slug) => self.repository.find_latest(&BriefScope::zone(slug)).await?,
            None => None,
        };

        let brief = match of_zone {
            Some(brief) => Some(brief),
            None => self.repository.find_latest(&BriefScope::region()).await?,
        };

        tracing::debug!(
            farm_id,
            zone = zone_slug.as_ref().map(|slug| slug.as_str()),
            scope = brief.as_ref().map(|brief| brief.scope().as_str()),
            "farm brief viewed"
        );

        Ok(FarmBrief::new(farm_id, zone_slug, brief))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::briefs::app::testing::{
        Call, FARM_ID, Fakes, OWNER, a_brief, a_day, auth_context,
    };

    fn use_case(fakes: &Fakes) -> ViewFarmBriefUseCase {
        ViewFarmBriefUseCase::new(Arc::new(fakes.clone()), Arc::new(fakes.clone()))
    }

    fn owned() -> Call {
        Call::IsOwnedBy {
            farm_id: FARM_ID,
            phone: OWNER.to_string(),
        }
    }

    fn latest(scope: &str) -> Call {
        Call::FindLatest {
            scope: scope.to_string(),
        }
    }

    #[tokio::test]
    async fn shows_the_newest_brief_of_the_farms_own_zone() {
        let fakes = Fakes::new()
            .with_owned_farm()
            .with_farm_zone(FARM_ID, "chamchamal")
            .with_stored(a_brief(8, "chamchamal"))
            .with_stored(a_brief(9, "chamchamal"))
            .with_stored(a_brief(10, "region"));

        let farm_brief = use_case(&fakes)
            .execute(&auth_context(), FARM_ID)
            .await
            .expect("farm brief");

        let brief = farm_brief.brief().as_ref().expect("brief");

        assert_eq!(brief.scope().as_str(), "chamchamal");
        assert_eq!(
            *brief.day(),
            a_day(9),
            "the zone's own brief wins even when the region has a newer one"
        );
        assert_eq!(
            fakes.calls(),
            vec![
                owned(),
                Call::FindFarmZone { farm_id: FARM_ID },
                latest("chamchamal"),
            ],
            "ownership is checked for the signed-in phone before anything is read"
        );
    }

    #[tokio::test]
    async fn a_zone_without_a_brief_yet_falls_back_to_the_region() {
        let fakes = Fakes::new()
            .with_owned_farm()
            .with_farm_zone(FARM_ID, "chamchamal")
            .with_stored(a_brief(9, "kalar"))
            .with_stored(a_brief(8, "region"));

        let farm_brief = use_case(&fakes)
            .execute(&auth_context(), FARM_ID)
            .await
            .expect("farm brief");

        assert_eq!(
            farm_brief.zone_slug().as_ref().map(|slug| slug.as_str()),
            Some("chamchamal"),
            "the farm's zone is still told, so the app can say the brief is the region's"
        );
        assert_eq!(
            farm_brief.brief().as_ref().expect("brief").scope().as_str(),
            "region"
        );
        assert_eq!(
            fakes.calls(),
            vec![
                owned(),
                Call::FindFarmZone { farm_id: FARM_ID },
                latest("chamchamal"),
                latest("region"),
            ]
        );
    }

    #[tokio::test]
    async fn a_farm_with_no_recorded_zone_gets_the_region_brief() {
        let fakes = Fakes::new()
            .with_owned_farm()
            .with_farm_zone(99, "kalar")
            .with_stored(a_brief(9, "kalar"))
            .with_stored(a_brief(8, "region"));

        let farm_brief = use_case(&fakes)
            .execute(&auth_context(), FARM_ID)
            .await
            .expect("farm brief");

        assert!(farm_brief.zone_slug().is_none());
        assert_eq!(
            farm_brief.brief().as_ref().expect("brief").scope().as_str(),
            "region"
        );
        assert_eq!(
            fakes.calls(),
            vec![
                owned(),
                Call::FindFarmZone { farm_id: FARM_ID },
                latest("region"),
            ]
        );
    }

    #[tokio::test]
    async fn nothing_stored_at_all_is_no_brief_yet_not_an_error() {
        let fakes = Fakes::new()
            .with_owned_farm()
            .with_farm_zone(FARM_ID, "chamchamal");

        let farm_brief = use_case(&fakes)
            .execute(&auth_context(), FARM_ID)
            .await
            .expect("farm brief");

        assert!(farm_brief.brief().is_none());
        assert_eq!(*farm_brief.farm_id(), FARM_ID);
    }

    #[tokio::test]
    async fn is_not_found_when_the_user_does_not_own_the_farm_and_reads_nothing() {
        let fakes = Fakes::new()
            .with_owned_farm()
            .with_farm_zone(99, "kalar")
            .with_stored(a_brief(9, "kalar"));

        let result = use_case(&fakes).execute(&auth_context(), 99).await;

        assert!(matches!(
            result,
            Err(AppError::GlobalAppError(GlobalAppError::NotFound))
        ));
        assert_eq!(
            fakes.calls(),
            vec![Call::IsOwnedBy {
                farm_id: 99,
                phone: OWNER.to_string(),
            }],
            "not even the zone of another farmer's farm is looked up"
        );
    }

    #[tokio::test]
    async fn a_failed_ownership_check_surfaces_instead_of_reading() {
        let fakes = Fakes::new().with_owned_farm().failing();

        let result = use_case(&fakes).execute(&auth_context(), FARM_ID).await;

        assert!(result.is_err());
        assert_eq!(fakes.calls(), vec![owned()]);
    }
}
