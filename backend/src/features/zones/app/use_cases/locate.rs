use crate::{
    app::AppError as GlobalAppError,
    features::zones::{
        app::{AppError, ZoneRepository},
        domain::{SubZone, Zone, ZoneSlug},
    },
};

/// The zone a dashboard route names, or not found. Zones are reference data
/// that the dashboard never changes, so reading one before a write decides
/// nothing about that write.
pub(super) async fn zone_named(
    repository: &dyn ZoneRepository,
    slug: &ZoneSlug,
) -> Result<Zone, AppError> {
    repository
        .find_zone_by_slug(slug)
        .await?
        .ok_or_else(|| GlobalAppError::NotFound.into())
}

/// The sub-zone of that zone. One that belongs to another zone is not found.
pub(super) async fn sub_zone_named(
    repository: &dyn ZoneRepository,
    zone_slug: &ZoneSlug,
    sub_zone_slug: &ZoneSlug,
) -> Result<SubZone, AppError> {
    let zone = zone_named(repository, zone_slug).await?;

    repository
        .find_sub_zone_by_slug(*zone.id(), sub_zone_slug)
        .await?
        .ok_or_else(|| GlobalAppError::NotFound.into())
}
