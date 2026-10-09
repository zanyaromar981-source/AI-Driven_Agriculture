use std::sync::Arc;

use crate::features::dams::{
    app::{AppError, DamRepository},
    domain::Dam,
};

pub struct ListReferenceDamsUseCase {
    repository: Arc<dyn DamRepository>,
}

impl ListReferenceDamsUseCase {
    pub fn new(repository: Arc<dyn DamRepository>) -> Self {
        Self { repository }
    }

    /// Returns the dams as they are seeded, without readings: what an
    /// editing screen needs to offer the dams to pick from.
    pub async fn execute(&self) -> Result<Vec<Dam>, AppError> {
        let dams = self.repository.find_all().await?;

        tracing::debug!(returned = dams.len(), "reference dams listed");

        Ok(dams)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::dams::app::testing::{
        FakeDamRepository, RepositoryCall, a_reading, darbandikhan, day, dukan,
    };

    #[tokio::test]
    async fn the_dams_are_listed_with_one_query_and_no_reading_is_looked_up() {
        let repository = FakeDamRepository::holding(
            vec![dukan(), darbandikhan()],
            vec![a_reading(&dukan(), day(2026, 10, 1), 38.5)],
        );
        let use_case = ListReferenceDamsUseCase::new(Arc::new(repository.clone()));

        let dams = use_case.execute().await.expect("dams");

        assert_eq!(
            dams.iter()
                .map(|dam| dam.slug().as_str())
                .collect::<Vec<_>>(),
            vec!["dukan", "darbandikhan"]
        );
        assert_eq!(repository.calls(), vec![RepositoryCall::FindAll]);
    }

    #[tokio::test]
    async fn a_repository_failure_surfaces() {
        let use_case = ListReferenceDamsUseCase::new(Arc::new(FakeDamRepository::failing()));

        assert!(use_case.execute().await.is_err());
    }
}
