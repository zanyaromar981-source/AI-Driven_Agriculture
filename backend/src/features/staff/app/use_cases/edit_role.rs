use std::sync::Arc;

use chrono::Utc;

use crate::{
    app::{AppError as GlobalAppError, Permission, StaffContext},
    features::staff::{
        app::{AppError, RoleRepository},
        domain::{Role, RoleDescription, RoleName},
    },
};

pub struct EditRoleInput {
    pub name: RoleName,
    pub description: Option<RoleDescription>,
    pub permissions: Vec<Permission>,
}

pub struct EditRoleUseCase {
    roles: Arc<dyn RoleRepository>,
}

impl EditRoleUseCase {
    pub fn new(roles: Arc<dyn RoleRepository>) -> Self {
        Self { roles }
    }

    pub async fn execute(
        &self,
        actor: &StaffContext,
        id: i32,
        input: EditRoleInput,
    ) -> Result<Role, AppError> {
        let Some(mut role) = self.roles.find_by_id(id).await? else {
            return Err(GlobalAppError::NotFound.into());
        };

        role.edit(input.name, input.description, input.permissions, Utc::now())
            .inspect_err(|_| tracing::info!(role_id = id, "role edit refused: system role"))?;

        let updated = self.roles.update(&role).await?;

        tracing::info!(
            role_id = id,
            permissions = updated.permissions().len(),
            by_staff_id = *actor.staff_id(),
            "role edited"
        );

        Ok(updated)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        app::{Action, Resource},
        features::staff::{
            app::testing::{Call, DAM_OFFICER_ROLE_ID, Fakes, OWNER_ID, OWNER_ROLE_ID, actor},
            domain::StaffError,
        },
    };

    fn input(name: &str) -> EditRoleInput {
        EditRoleInput {
            name: RoleName::new(name.to_string()).expect("name"),
            description: None,
            permissions: vec![Permission::new(Resource::Dams, Action::Read)],
        }
    }

    #[tokio::test]
    async fn replaces_the_name_and_the_whole_permission_set() {
        let fakes = Fakes::new();
        let use_case = EditRoleUseCase::new(Arc::new(fakes.clone()));

        let role = use_case
            .execute(&actor(OWNER_ID), DAM_OFFICER_ROLE_ID, input("Dam reader"))
            .await
            .expect("role");

        assert_eq!(role.name().as_str(), "Dam reader");
        assert_eq!(
            *fakes
                .stored_role(DAM_OFFICER_ROLE_ID)
                .expect("stored")
                .permissions(),
            vec![Permission::new(Resource::Dams, Action::Read)],
            "the update permission the role had must be gone"
        );
    }

    #[tokio::test]
    async fn the_owner_role_cannot_be_edited_and_is_not_written() {
        let fakes = Fakes::new();
        let use_case = EditRoleUseCase::new(Arc::new(fakes.clone()));

        let result = use_case
            .execute(&actor(OWNER_ID), OWNER_ROLE_ID, input("Renamed"))
            .await;

        assert!(matches!(
            result,
            Err(AppError::Staff(StaffError::SystemRole))
        ));
        assert!(
            !fakes
                .calls()
                .iter()
                .any(|call| matches!(call, Call::UpdateRole { .. }))
        );
    }

    #[tokio::test]
    async fn a_name_another_role_has_is_refused() {
        let use_case = EditRoleUseCase::new(Arc::new(Fakes::new()));

        let result = use_case
            .execute(&actor(OWNER_ID), DAM_OFFICER_ROLE_ID, input("Owner"))
            .await;

        assert!(matches!(
            result,
            Err(AppError::Staff(StaffError::RoleNameTaken))
        ));
    }

    #[tokio::test]
    async fn a_missing_role_is_not_found() {
        let use_case = EditRoleUseCase::new(Arc::new(Fakes::new()));

        assert!(matches!(
            use_case.execute(&actor(OWNER_ID), 99, input("Ghost")).await,
            Err(AppError::GlobalAppError(GlobalAppError::NotFound))
        ));
    }
}
