use std::sync::Arc;

use crate::{
    app::StaffContext,
    features::staff::app::{AppError, RoleRepository},
};

pub struct DeleteRoleUseCase {
    roles: Arc<dyn RoleRepository>,
}

impl DeleteRoleUseCase {
    pub fn new(roles: Arc<dyn RoleRepository>) -> Self {
        Self { roles }
    }

    /// Nothing is read first: whether the role is a system role or still in
    /// use is decided by the delete itself, so a role assigned at the same
    /// moment cannot slip through.
    pub async fn execute(&self, actor: &StaffContext, id: i32) -> Result<(), AppError> {
        self.roles.delete(id).await?;

        tracing::info!(
            role_id = id,
            by_staff_id = *actor.staff_id(),
            "role deleted"
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        app::AppError as GlobalAppError,
        features::staff::{
            app::testing::{
                Call, DAM_OFFICER_ROLE_ID, Fakes, OWNER_ID, OWNER_ROLE_ID, a_staff_member, actor,
            },
            domain::StaffError,
        },
    };

    #[tokio::test]
    async fn deletes_a_custom_role_nobody_holds_in_one_call() {
        let fakes = Fakes::new();
        let use_case = DeleteRoleUseCase::new(Arc::new(fakes.clone()));

        use_case
            .execute(&actor(OWNER_ID), DAM_OFFICER_ROLE_ID)
            .await
            .expect("deleted");

        assert!(fakes.stored_role(DAM_OFFICER_ROLE_ID).is_none());
        assert_eq!(
            fakes.calls(),
            vec![Call::DeleteRole {
                id: DAM_OFFICER_ROLE_ID
            }],
            "a count read beforehand could be stale by the time of the delete"
        );
    }

    #[tokio::test]
    async fn the_owner_role_cannot_be_deleted() {
        let fakes = Fakes::new();
        let use_case = DeleteRoleUseCase::new(Arc::new(fakes.clone()));

        let result = use_case.execute(&actor(OWNER_ID), OWNER_ROLE_ID).await;

        assert!(matches!(
            result,
            Err(AppError::Staff(StaffError::SystemRole))
        ));
        assert!(fakes.stored_role(OWNER_ROLE_ID).is_some());
    }

    #[tokio::test]
    async fn a_role_someone_still_holds_cannot_be_deleted() {
        let fakes = Fakes::new().with_staff(a_staff_member(
            2,
            "dams@example.org",
            true,
            &[DAM_OFFICER_ROLE_ID],
        ));
        let use_case = DeleteRoleUseCase::new(Arc::new(fakes.clone()));

        let result = use_case
            .execute(&actor(OWNER_ID), DAM_OFFICER_ROLE_ID)
            .await;

        assert!(matches!(
            result,
            Err(AppError::Staff(StaffError::RoleInUse))
        ));
        assert!(fakes.stored_role(DAM_OFFICER_ROLE_ID).is_some());
    }

    #[tokio::test]
    async fn a_missing_role_is_not_found() {
        let use_case = DeleteRoleUseCase::new(Arc::new(Fakes::new()));

        assert!(matches!(
            use_case.execute(&actor(OWNER_ID), 99).await,
            Err(AppError::GlobalAppError(GlobalAppError::NotFound))
        ));
    }
}
