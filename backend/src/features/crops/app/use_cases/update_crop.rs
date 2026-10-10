use std::sync::Arc;

use chrono::{DateTime, Utc};

use crate::{
    app::{AppError as GlobalAppError, StaffContext},
    features::crops::{
        app::{AppError, CropRepository, CropUsage},
        domain::{Crop, CropCode, CropDetails, CropError},
    },
};

pub struct UpdateCropUseCase {
    repository: Arc<dyn CropRepository>,
    trade: Arc<dyn CropUsage>,
}

impl UpdateCropUseCase {
    /// `trade` says whether a listing or a price names a product: the only
    /// things stored that count in its unit.
    pub fn new(repository: Arc<dyn CropRepository>, trade: Arc<dyn CropUsage>) -> Self {
        Self { repository, trade }
    }

    /// A staff member renames a crop, recolours it, reorders it, moves it
    /// to another group or switches it on or off. The code names it and
    /// never changes.
    ///
    /// The unit may change only while no listing and no price names the
    /// product: 12 trays must not become 12 kg. As with a delete, the
    /// trade shares no key with this table, so this is asked, not
    /// enforced: a listing posted between the question and the update is
    /// not seen. It keeps the unit it was posted with, which is stored on
    /// the listing itself.
    pub async fn execute(
        &self,
        actor: &StaffContext,
        code: CropCode,
        details: CropDetails,
        now: DateTime<Utc>,
    ) -> Result<Crop, AppError> {
        // Read only to learn whether the unit changes; whether the crop
        // exists is decided by the update below.
        let unit_changes = self
            .repository
            .find_by_code(&code)
            .await?
            .is_some_and(|current| current.details().unit != details.unit);

        if unit_changes && self.trade.is_used(&code).await? {
            tracing::info!(
                staff_id = *actor.staff_id(),
                crop = code.as_str(),
                "crop update refused: listings or prices count in its unit"
            );

            return Err(CropError::UnitInUse.into());
        }

        let Some(crop) = self.repository.update(&code, &details, now).await? else {
            tracing::info!(
                staff_id = *actor.staff_id(),
                crop = code.as_str(),
                "crop update refused: no such crop"
            );

            return Err(GlobalAppError::NotFound.into());
        };

        tracing::info!(
            staff_id = *actor.staff_id(),
            crop = code.as_str(),
            active = crop.details().active,
            "crop updated"
        );

        Ok(crop)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::crops::{
        app::testing::{
            FakeCropRepository, FakeCropUsage, RepositoryCall, a_crop, at, code, details, staff,
        },
        domain::{CropDetails, ProductGroup, ProductUnit},
    };

    fn use_case(repository: &FakeCropRepository, trade: &FakeCropUsage) -> UpdateCropUseCase {
        UpdateCropUseCase::new(Arc::new(repository.clone()), Arc::new(trade.clone()))
    }

    fn by_the_head(name: &str) -> CropDetails {
        CropDetails {
            group: ProductGroup::Animals,
            unit: ProductUnit::Head,
            ..details(name, 5, true)
        }
    }

    #[tokio::test]
    async fn a_unit_listings_or_prices_count_in_cannot_change() {
        let repository = FakeCropRepository::holding(vec![a_crop("wheat", 10, true)]);
        let trade = FakeCropUsage::using(&["wheat"]);

        let result = use_case(&repository, &trade)
            .execute(&staff(), code("wheat"), by_the_head("Wheat"), at(11))
            .await;

        assert!(matches!(result, Err(AppError::Crop(CropError::UnitInUse))));
        assert_eq!(
            repository.stored("wheat").expect("crop").details().unit,
            ProductUnit::Kg,
            "nothing was written"
        );
        assert!(
            !repository
                .calls()
                .iter()
                .any(|call| matches!(call, RepositoryCall::Update { .. }))
        );
    }

    #[tokio::test]
    async fn the_unit_of_a_product_nothing_trades_may_change() {
        let repository = FakeCropRepository::holding(vec![a_crop("wheat", 10, true)]);
        let trade = FakeCropUsage::default();

        let crop = use_case(&repository, &trade)
            .execute(&staff(), code("wheat"), by_the_head("Wheat"), at(11))
            .await
            .expect("crop");

        assert_eq!(crop.details().unit, ProductUnit::Head);
        assert_eq!(trade.asked(), vec!["wheat".to_string()]);
    }

    #[tokio::test]
    async fn a_traded_product_keeps_everything_else_editable_and_the_trade_is_not_asked() {
        let repository = FakeCropRepository::holding(vec![a_crop("wheat", 10, true)]);
        let trade = FakeCropUsage::using(&["wheat"]);
        let renamed = CropDetails {
            group: ProductGroup::NutsDried,
            ..details("Bread wheat", 5, false)
        };

        let crop = use_case(&repository, &trade)
            .execute(&staff(), code("wheat"), renamed.clone(), at(11))
            .await
            .expect("crop");

        assert_eq!(crop.details(), &renamed);
        assert!(trade.asked().is_empty(), "the unit did not change");
    }

    #[tokio::test]
    async fn not_knowing_whether_the_product_is_traded_refuses_the_unit_change() {
        let repository = FakeCropRepository::holding(vec![a_crop("wheat", 10, true)]);

        let result = use_case(&repository, &FakeCropUsage::failing())
            .execute(&staff(), code("wheat"), by_the_head("Wheat"), at(11))
            .await;

        assert!(result.is_err());
        assert_eq!(
            repository.stored("wheat").expect("crop").details().unit,
            ProductUnit::Kg
        );
    }

    #[tokio::test]
    async fn replaces_everything_but_the_code_and_the_day_it_was_added() {
        let repository = FakeCropRepository::holding(vec![a_crop("wheat", 10, true)]);

        let crop = use_case(&repository, &FakeCropUsage::default())
            .execute(
                &staff(),
                code("wheat"),
                details("Bread wheat", 5, false),
                at(11),
            )
            .await
            .expect("crop");

        assert_eq!(crop.code().as_str(), "wheat");
        assert_eq!(crop.details(), &details("Bread wheat", 5, false));
        assert_eq!(*crop.created_at(), at(8), "the creation time is kept");
        assert_eq!(*crop.updated_at(), at(11));
        assert_eq!(
            repository.calls(),
            vec![
                RepositoryCall::FindByCode {
                    code: "wheat".to_string()
                },
                RepositoryCall::Update {
                    code: "wheat".to_string(),
                    details: details("Bread wheat", 5, false),
                }
            ],
            "the crop is read for its unit, and the update decides"
        );
    }

    #[tokio::test]
    async fn a_missing_crop_is_not_found_and_nothing_is_created() {
        let repository = FakeCropRepository::new();

        let result = use_case(&repository, &FakeCropUsage::default())
            .execute(&staff(), code("rice"), details("Rice", 5, true), at(11))
            .await;

        assert!(matches!(
            result,
            Err(AppError::GlobalAppError(GlobalAppError::NotFound))
        ));
        assert!(repository.stored("rice").is_none());
    }

    #[tokio::test]
    async fn a_repository_failure_surfaces() {
        assert!(
            use_case(&FakeCropRepository::failing(), &FakeCropUsage::default())
                .execute(&staff(), code("rice"), details("Rice", 5, true), at(11))
                .await
                .is_err()
        );
    }
}
