use std::sync::Arc;

use crate::features::app_config::app::{AppConfigRepository, AppError};

/// Answers one question for the public farm totals route
/// (`GET /v1/stats/farms`), which another feature owns: may it answer?
pub struct PublicFarmTotalsUseCase {
    repository: Arc<dyn AppConfigRepository>,
}

impl PublicFarmTotalsUseCase {
    pub fn new(repository: Arc<dyn AppConfigRepository>) -> Self {
        Self { repository }
    }

    /// Whether staff have left `public_farm_totals` on. Read from the
    /// stored config each time, so switching it off takes effect at once.
    pub async fn is_public_farm_totals_on(&self) -> Result<bool, AppError> {
        Ok(self.repository.find().await?.settings().public_farm_totals)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::app_config::app::testing::Fakes;

    #[tokio::test]
    async fn the_answer_is_the_stored_switch() {
        for on in [true, false] {
            let fakes = Fakes::new().with_public_farm_totals(on);

            let answer = PublicFarmTotalsUseCase::new(Arc::new(fakes))
                .is_public_farm_totals_on()
                .await
                .expect("answer");

            assert_eq!(answer, on);
        }
    }

    #[tokio::test]
    async fn a_failure_is_an_error_not_a_yes_or_a_no() {
        let result = PublicFarmTotalsUseCase::new(Arc::new(Fakes::new().failing()))
            .is_public_farm_totals_on()
            .await;

        assert!(result.is_err());
    }
}
