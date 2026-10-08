use std::sync::Arc;

use crate::features::staff::{
    app::{AppError, RoleRepository},
    domain::Role,
};

pub struct ListRolesUseCase {
    roles: Arc<dyn RoleRepository>,
}

impl ListRolesUseCase {
    pub fn new(roles: Arc<dyn RoleRepository>) -> Self {
        Self { roles }
    }

    pub async fn execute(&self) -> Result<Vec<Role>, AppError> {
        let roles = self.roles.find_all().await?;

        tracing::debug!(count = roles.len(), "listed roles");

        Ok(roles)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::staff::app::testing::{Call, Fakes};

    #[tokio::test]
    async fn returns_every_role() {
        let fakes = Fakes::new();
        let use_case = ListRolesUseCase::new(Arc::new(fakes.clone()));

        let roles = use_case.execute().await.expect("roles");

        assert_eq!(roles.len(), 2);
        assert_eq!(fakes.calls(), vec![Call::FindAllRoles]);
    }

    #[tokio::test]
    async fn a_repository_failure_surfaces() {
        let use_case = ListRolesUseCase::new(Arc::new(Fakes::failing()));

        assert!(use_case.execute().await.is_err());
    }
}
