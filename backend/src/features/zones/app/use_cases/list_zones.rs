use std::sync::Arc;

use crate::features::zones::{
    app::{AppError, ZoneRepository},
    domain::{SubZone, Zone},
};

pub struct ZoneWithSubZones {
    pub zone: Zone,
    pub sub_zones: Vec<SubZone>,
}

pub struct ListZonesUseCase {
    repository: Arc<dyn ZoneRepository>,
}

impl ListZonesUseCase {
    pub fn new(repository: Arc<dyn ZoneRepository>) -> Self {
        Self { repository }
    }

    /// Every zone with its sub-zones, in seed order, for the dashboard's
    /// pickers. Two queries whatever the number of zones.
    pub async fn execute(&self) -> Result<Vec<ZoneWithSubZones>, AppError> {
        let zones = self.repository.find_all_zones().await?;
        let sub_zones = self.repository.find_all_sub_zones().await?;

        Ok(zones
            .into_iter()
            .map(|zone| {
                let sub_zones = sub_zones
                    .iter()
                    .filter(|sub_zone| sub_zone.zone_id() == zone.id())
                    .cloned()
                    .collect();

                ZoneWithSubZones { zone, sub_zones }
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::zones::app::testing::{FakeZoneRepository, RepositoryCall};

    #[tokio::test]
    async fn lists_every_zone_with_only_its_own_sub_zones() {
        let repository = FakeZoneRepository::seeded();
        let use_case = ListZonesUseCase::new(Arc::new(repository.clone()));

        let listed = use_case.execute().await.expect("listed");

        let shape: Vec<(&str, Vec<&str>)> = listed
            .iter()
            .map(|row| {
                (
                    row.zone.slug().as_str(),
                    row.sub_zones
                        .iter()
                        .map(|sub_zone| sub_zone.slug().as_str())
                        .collect(),
                )
            })
            .collect();

        assert_eq!(
            shape,
            vec![
                (
                    "chamchamal",
                    vec!["markaz-chamchamal", "aghjalar", "sangaw"]
                ),
                ("kalar", vec!["markaz-kalar"]),
                ("qushtapa", vec![]),
            ]
        );
        assert_eq!(
            repository.calls(),
            vec![
                RepositoryCall::FindAllZones,
                RepositoryCall::FindAllSubZones
            ],
            "the sub-zones are read once, not once per zone"
        );
    }

    #[tokio::test]
    async fn a_repository_failure_surfaces() {
        let use_case = ListZonesUseCase::new(Arc::new(FakeZoneRepository::failing()));

        assert!(use_case.execute().await.is_err());
    }
}
