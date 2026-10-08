use std::sync::Arc;

use crate::{
    app::AppError as GlobalAppError,
    features::staff::{
        app::{AppError, RoleRepository},
        domain::Role,
    },
};

pub struct ViewRoleUseCase {
    roles: Arc<dyn RoleRepository>,
}

impl ViewRoleUseCase {
    pub fn new(roles: Arc<dyn RoleRepository>) -> Self {
        Self { roles }
    }

    pub async fn execute(&self, id: i32) -> Result<Role, AppError> {
        self.roles
            .find_by_id(id)
            .await?
            .ok_or_else(|| GlobalAppError::NotFound.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::staff::app::testing::{Call, DAM_OFFICER_ROLE_ID, Fakes};

    #[tokio::test]
    async fn returns_the_role_with_its_permissions() {
        let fakes = Fakes::new();
        let use_case = ViewRoleUseCase::new(Arc::new(fakes.clone()));

        let role = use_case.execute(DAM_OFFICER_ROLE_ID).await.expect("role");

        assert_eq!(role.name().as_str(), "Dam officer");
        assert_eq!(role.permissions().len(), 2);
        assert_eq!(
            fakes.calls(),
            vec![Call::FindRole {
                id: DAM_OFFICER_ROLE_ID
            }]
        );
    }

    #[tokio::test]
    async fn a_missing_role_is_not_found() {
        let use_case = ViewRoleUseCase::new(Arc::new(Fakes::new()));

        assert!(matches!(
            use_case.execute(99).await,
            Err(AppError::GlobalAppError(GlobalAppError::NotFound))
        ));
    }
}
