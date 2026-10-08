use std::sync::Arc;

use chrono::Utc;

use crate::{
    app::{Permission, StaffContext},
    features::staff::{
        app::{AppError, RoleRepository},
        domain::{Role, RoleDescription, RoleName},
    },
};

pub struct CreateRoleInput {
    pub name: RoleName,
    pub description: Option<RoleDescription>,
    pub permissions: Vec<Permission>,
}

pub struct CreateRoleUseCase {
    roles: Arc<dyn RoleRepository>,
}

impl CreateRoleUseCase {
    pub fn new(roles: Arc<dyn RoleRepository>) -> Self {
        Self { roles }
    }

    /// Whether the name is free is not looked up first: the unique index
    /// answers that when the role is stored.
    pub async fn execute(
        &self,
        actor: &StaffContext,
        input: CreateRoleInput,
    ) -> Result<Role, AppError> {
        let role = Role::new(input.name, input.description, input.permissions, Utc::now());

        let created = self.roles.create(&role).await?;

        tracing::info!(
            role_id = created.id().unwrap_or_default(),
            permissions = created.permissions().len(),
            by_staff_id = *actor.staff_id(),
            "role created"
        );

        Ok(created)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        app::{Action, Resource},
        features::staff::{
            app::testing::{Call, Fakes, OWNER_ID, actor},
            domain::StaffError,
        },
    };

    fn input(name: &str) -> CreateRoleInput {
        CreateRoleInput {
            name: RoleName::new(name.to_string()).expect("name"),
            description: None,
            permissions: vec![
                Permission::new(Resource::Fires, Action::Read),
                Permission::new(Resource::Fires, Action::Read),
            ],
        }
    }

    #[tokio::test]
    async fn creates_a_custom_role_with_each_permission_once() {
        let fakes = Fakes::new();
        let use_case = CreateRoleUseCase::new(Arc::new(fakes.clone()));

        let role = use_case
            .execute(&actor(OWNER_ID), input("Fire officer"))
            .await
            .expect("role");

        assert!(!*role.system(), "staff can never create a system role");
        assert_eq!(
            *role.permissions(),
            vec![Permission::new(Resource::Fires, Action::Read)]
        );
        assert_eq!(
            fakes.calls(),
            vec![Call::CreateRole {
                name: "Fire officer".to_string()
            }],
            "the name must be decided by the write, not by a read before it"
        );
    }

    #[tokio::test]
    async fn a_name_another_role_has_is_refused() {
        let use_case = CreateRoleUseCase::new(Arc::new(Fakes::new()));

        let result = use_case
            .execute(&actor(OWNER_ID), input("Dam officer"))
            .await;

        assert!(matches!(
            result,
            Err(AppError::Staff(StaffError::RoleNameTaken))
        ));
    }
}
