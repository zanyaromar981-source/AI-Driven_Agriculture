use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    app::AppError as GlobalAppError,
    features::{
        farms::{
            app::{AppError, AreaDirectory},
            domain::{AreaNames, GovernorateName, SubZoneName, ZoneName},
        },
        zones::{app::ZoneRepository, domain::Governorate},
    },
};

/// Reads what the areas are called from the zones feature through its own
/// repository port, in the order that feature lists them: north to south.
#[derive(Debug)]
pub struct ZonesFeatureAreaDirectory {
    zones: Arc<dyn ZoneRepository>,
}

impl ZonesFeatureAreaDirectory {
    pub fn new(zones: Arc<dyn ZoneRepository>) -> Self {
        Self { zones }
    }
}

#[async_trait]
impl AreaDirectory for ZonesFeatureAreaDirectory {
    async fn names(&self) -> Result<AreaNames, AppError> {
        let failed = |error| {
            tracing::error!(%error, "reading area names for farm totals failed");

            AppError::from(GlobalAppError::InternalServerError)
        };

        let zones = self.zones.find_all_zones().await.map_err(failed)?;
        let sub_zones = self.zones.find_all_sub_zones().await.map_err(failed)?;

        Ok(AreaNames {
            governorates: Governorate::of(&zones)
                .into_iter()
                .map(|governorate| GovernorateName {
                    name_en: governorate.name_en().clone(),
                    name_ku: governorate.name_ku().clone(),
                })
                .collect(),
            sub_zones: sub_zones
                .iter()
                .filter_map(|sub_zone| {
                    let zone = zones.iter().find(|zone| zone.id() == sub_zone.zone_id())?;

                    Some(SubZoneName {
                        zone_slug: String::from(zone.slug()),
                        slug: String::from(sub_zone.slug()),
                        name_en: sub_zone.name_en().clone(),
                        name_ku: sub_zone.name_ku().clone(),
                    })
                })
                .collect(),
            zones: zones
                .iter()
                .map(|zone| ZoneName {
                    slug: String::from(zone.slug()),
                    name_en: zone.name_en().clone(),
                    name_ku: zone.name_ku().clone(),
                })
                .collect(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::zones::app::testing::{FakeZoneRepository, RepositoryCall};

    #[tokio::test]
    async fn the_names_come_from_the_zones_feature_in_its_order() {
        let zones = FakeZoneRepository::seeded();
        let directory = ZonesFeatureAreaDirectory::new(Arc::new(zones.clone()));

        let names = directory.names().await.expect("names");

        assert_eq!(
            names
                .governorates
                .iter()
                .map(|name| (name.name_en.as_str(), name.name_ku.as_str()))
                .collect::<Vec<_>>(),
            vec![("Sulaymaniyah", "سلێمانی"), ("Erbil", "هەولێر")]
        );
        assert_eq!(
            names
                .zones
                .iter()
                .map(|name| name.slug.as_str())
                .collect::<Vec<_>>(),
            vec!["chamchamal", "kalar", "qushtapa"]
        );
        assert_eq!(names.zones[1].name_ku, "کەلار");
        assert_eq!(
            names
                .sub_zones
                .iter()
                .map(|name| (name.zone_slug.as_str(), name.slug.as_str()))
                .collect::<Vec<_>>(),
            vec![
                ("chamchamal", "markaz-chamchamal"),
                ("chamchamal", "aghjalar"),
                ("chamchamal", "sangaw"),
                ("kalar", "markaz-kalar"),
            ]
        );
        assert_eq!(
            zones.calls(),
            vec![
                RepositoryCall::FindAllZones,
                RepositoryCall::FindAllSubZones
            ],
            "two reads whatever the number of areas, and never the shapes"
        );
    }

    #[tokio::test]
    async fn a_failure_in_the_zones_feature_is_a_server_fault() {
        let directory = ZonesFeatureAreaDirectory::new(Arc::new(FakeZoneRepository::failing()));

        assert!(matches!(
            directory.names().await,
            Err(AppError::GlobalAppError(
                GlobalAppError::InternalServerError
            ))
        ));
    }
}
