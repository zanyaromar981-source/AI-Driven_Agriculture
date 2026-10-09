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

        let created = self
            .roles
            .create(&role, *actor.staff_id())
            .await
            .inspect_err(
                |error| tracing::info!(%error, by_staff_id = *actor.staff_id(), "role not created"),
            )?;

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
            app::testing::{Call, DAM_OFFICER_ROLE_ID, Fakes, OWNER_ID, a_staff_member, actor},
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

    /// A staff member holding the dam officer role: dams read and update.
    fn with_a_dam_officer() -> Fakes {
        Fakes::new().with_staff(a_staff_member(
            2,
            "dams@example.org",
            true,
            &[DAM_OFFICER_ROLE_ID],
        ))
    }

    #[tokio::test]
    async fn a_role_holding_only_what_the_actor_holds_can_be_created() {
        let fakes = with_a_dam_officer();
        let use_case = CreateRoleUseCase::new(Arc::new(fakes.clone()));

        let role = use_case
            .execute(
                &actor(2),
                CreateRoleInput {
                    name: RoleName::new("Dam reader".to_string()).expect("name"),
                    description: None,
                    permissions: vec![Permission::new(Resource::Dams, Action::Read)],
                },
            )
            .await
            .expect("role");

        assert_eq!(role.permissions().len(), 1);
    }

    #[tokio::test]
    async fn a_role_holding_what_the_actor_lacks_is_refused_and_not_stored() {
        let fakes = with_a_dam_officer();
        let use_case = CreateRoleUseCase::new(Arc::new(fakes.clone()));

        let result = use_case.execute(&actor(2), input("Fire officer")).await;

        assert!(matches!(
            result,
            Err(AppError::Staff(StaffError::CannotGrant))
        ));
        assert!(
            fakes.stored_role(3).is_none(),
            "otherwise anyone who may create roles could make themselves anything"
        );
    }
}
