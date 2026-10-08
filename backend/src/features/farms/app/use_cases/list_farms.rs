use std::sync::Arc;

use crate::{
    app::{AuthContext, Pagination},
    features::farms::{
        app::{AppError, FarmRepository},
        domain::FarmSummary,
    },
};

pub struct ListFarmsUseCase {
    repository: Arc<dyn FarmRepository>,
}

impl ListFarmsUseCase {
    pub fn new(repository: Arc<dyn FarmRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(
        &self,
        auth_context: &AuthContext,
        pagination: &Pagination,
    ) -> Result<(Vec<FarmSummary>, Option<u64>), AppError> {
        let (farms, total_count) = self
            .repository
            .find_all_by_owner(auth_context.user().phone(), pagination)
            .await?;

        tracing::debug!(
            page = *pagination.page(),
            rows_per_page = *pagination.rows_per_page(),
            returned = farms.len(),
            counted = total_count.is_some(),
            "farms listed"
        );

        Ok((farms, total_count))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::farms::app::testing::{
        FakeFarmRepository, OWNER, RepositoryCall, a_farm, auth_context,
    };

    #[tokio::test]
    async fn lists_only_the_authenticated_owners_farms() {
        let repository = FakeFarmRepository::holding(a_farm());
        let use_case = ListFarmsUseCase::new(Arc::new(repository.clone()));

        let (rows, _) = use_case
            .execute(&auth_context(), &Pagination::new(1, 20))
            .await
            .expect("listing");

        assert_eq!(rows.len(), 1);
        assert_eq!(
            repository.calls(),
            vec![RepositoryCall::FindAllByOwner {
                owner: OWNER.to_string(),
                page: 1,
            }]
        );
    }

    #[tokio::test]
    async fn the_page_the_caller_asked_for_reaches_the_repository() {
        let repository = FakeFarmRepository::new();
        let use_case = ListFarmsUseCase::new(Arc::new(repository.clone()));

        let _ = use_case
            .execute(&auth_context(), &Pagination::new(4, 20))
            .await;

        assert!(
            repository
                .calls()
                .contains(&RepositoryCall::FindAllByOwner {
                    owner: OWNER.to_string(),
                    page: 4,
                })
        );
    }
}
