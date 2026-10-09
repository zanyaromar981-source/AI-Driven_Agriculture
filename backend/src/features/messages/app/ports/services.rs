use async_trait::async_trait;

use crate::{
    features::messages::{
        app::AppError,
        domain::{FarmCard, FarmerContact, SearchText},
    },
    shared::Phone,
};

/// Who the farmers are. Owned by the farmers feature; a message keeps only
/// the farmer's id.
#[async_trait]
pub trait SenderDirectory: Send + Sync + std::fmt::Debug {
    /// The id of the farmer with that phone, `None` when there is none.
    async fn farmer_id_of(&self, phone: &Phone) -> Result<Option<i32>, AppError>;

    /// The name and phone of each of the farmers that still exist. A farmer
    /// staff removed is simply not in the answer.
    async fn contacts_of(&self, farmer_ids: &[i32]) -> Result<Vec<FarmerContact>, AppError>;

    /// The ids of the farmers whose name or phone contains the text.
    async fn ids_matching(&self, search: &SearchText) -> Result<Vec<i32>, AppError>;
}

/// The farms messages are about. Owned by the farms feature.
#[async_trait]
pub trait MessageFarmDirectory: Send + Sync + std::fmt::Debug {
    async fn is_owned_by(&self, farm_id: i32, phone: &Phone) -> Result<bool, AppError>;

    /// The name and place of each of the farms that still exist.
    async fn cards_of(&self, farm_ids: &[i32]) -> Result<Vec<FarmCard>, AppError>;

    /// The ids of the farms in that governorate and district. Each part
    /// that is given must hold.
    async fn ids_in_place(
        &self,
        governorate: Option<&str>,
        zone_slug: Option<&str>,
    ) -> Result<Vec<i32>, AppError>;
}
