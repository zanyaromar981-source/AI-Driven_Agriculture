use std::sync::Arc;

use crate::features::messages::{
    app::{AppError, MessageRepository},
    domain::MessageCounts,
};

pub struct CountMessagesUseCase {
    repository: Arc<dyn MessageRepository>,
}

impl CountMessagesUseCase {
    pub fn new(repository: Arc<dyn MessageRepository>) -> Self {
        Self { repository }
    }

    /// How many messages are in each state, for the inbox badge.
    pub async fn execute(&self) -> Result<MessageCounts, AppError> {
        self.repository.count_by_state().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::messages::{
        app::testing::{Call, FARMER_ID, Fakes, a_message, a_message_of},
        domain::MessageState,
    };

    #[tokio::test]
    async fn counts_every_state_in_one_call() {
        let fakes = Fakes::new()
            .with_stored(a_message(1))
            .with_stored(a_message(2))
            .with_stored(a_message_of(3, FARMER_ID, MessageState::Closed));

        let counts = CountMessagesUseCase::new(Arc::new(fakes.clone()))
            .execute()
            .await
            .expect("counts");

        assert_eq!(*counts.new(), 2);
        assert_eq!(*counts.read(), 0);
        assert_eq!(*counts.closed(), 1);
        assert_eq!(fakes.calls(), vec![Call::CountByState]);
    }

    #[tokio::test]
    async fn a_repository_failure_surfaces() {
        assert!(
            CountMessagesUseCase::new(Arc::new(Fakes::new().failing()))
                .execute()
                .await
                .is_err()
        );
    }
}
