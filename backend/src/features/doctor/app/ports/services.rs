use async_trait::async_trait;

use crate::{
    features::doctor::{
        app::AppError,
        domain::{Consultation, DoctorReply, FarmBrief, HistoryTopic},
    },
    shared::Phone,
};

/// The farm a question is about, when it belongs to the phone. Owned by the
/// farms feature; `None` means there is no such farm for this owner.
#[async_trait]
pub trait FarmBriefs: Send + Sync + std::fmt::Debug {
    async fn brief_for(&self, farm_id: i32, owner: &Phone) -> Result<Option<FarmBrief>, AppError>;
}

/// What the data jobs already know about a farm, topic by topic, in the
/// order the app shows them. Owned by the insights feature.
#[async_trait]
pub trait FarmHistory: Send + Sync + std::fmt::Debug {
    async fn history_of(&self, farm_id: i32) -> Result<Vec<HistoryTopic>, AppError>;
}

/// The local Doctor service, which reads the farm, its history and the
/// question, and answers. The backend never calls an AI itself.
#[async_trait]
pub trait Doctor: Send + Sync + std::fmt::Debug {
    /// `AppError::DoctorNotReady` when the service says it cannot answer
    /// yet, `AppError::DoctorFailed` for every other failure.
    async fn ask(&self, consultation: &Consultation) -> Result<DoctorReply, AppError>;
}
