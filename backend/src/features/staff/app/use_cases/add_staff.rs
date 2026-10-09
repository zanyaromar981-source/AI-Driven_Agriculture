use std::sync::Arc;

use chrono::Utc;

use crate::{
    app::StaffContext,
    features::staff::{
        app::{AppError, PasswordHasher, StaffRepository},
        domain::{Password, RoleSelection, Staff, StaffEmail, StaffName},
    },
};

pub struct AddStaffInput {
    pub email: StaffEmail,
    pub name: StaffName,
    pub password: Password,
    pub roles: RoleSelection,
}

pub struct AddStaffUseCase {
    staff: Arc<dyn StaffRepository>,
    hasher: Arc<dyn PasswordHasher>,
}

impl AddStaffUseCase {
    pub fn new(staff: Arc<dyn StaffRepository>, hasher: Arc<dyn PasswordHasher>) -> Self {
        Self { staff, hasher }
    }

    /// Neither the email nor the roles are looked up first: the unique
    /// index and the foreign key answer when the account is stored.
    pub async fn execute(
        &self,
        actor: &StaffContext,
        input: AddStaffInput,
    ) -> Result<Staff, AppError> {
        let password_hash = self.hasher.hash(&input.password).await?;

        let staff = Staff::new(input.email, input.name, password_hash, Utc::now());

        let created = self
            .staff
            .create(&staff, &input.roles, *actor.staff_id())
            .await
            .inspect_err(|error| tracing::info!(%error, by_staff_id = *actor.staff_id(), "staff member not added"))?;

        tracing::info!(
            staff_id = created.id().unwrap_or_default(),
            roles = created.roles().len(),
            by_staff_id = *actor.staff_id(),
            "staff member added"
        );

        Ok(created)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::staff::{
        app::testing::{
            Call, DAM_OFFICER_ROLE_ID, Fakes, OWNER_EMAIL, OWNER_ID, OWNER_ROLE_ID, PASSWORD,
            a_staff_member, actor, hash_of,
        },
        domain::StaffError,
    };

    fn use_case(fakes: &Fakes) -> AddStaffUseCase {
        AddStaffUseCase::new(Arc::new(fakes.clone()), Arc::new(fakes.clone()))
    }

    fn input(email: &str, role_ids: Vec<i32>) -> AddStaffInput {
        AddStaffInput {
            email: StaffEmail::new(email.to_string()).expect("email"),
            name: StaffName::new("Dilan A.".to_string()).expect("name"),
            password: Password::new(PASSWORD.to_string()).expect("password"),
            roles: RoleSelection::new(role_ids).expect("roles"),
        }
    }

    #[tokio::test]
    async fn stores_an_active_account_with_its_roles() {
        let fakes = Fakes::new();

        let staff = use_case(&fakes)
            .execute(
                &actor(OWNER_ID),
                input("dilan@example.org", vec![DAM_OFFICER_ROLE_ID]),
            )
            .await
            .expect("staff");

        assert!(*staff.active());
        assert_eq!(staff.roles().len(), 1);
        assert_eq!(
            fakes.calls(),
            vec![
                Call::HashPassword,
                Call::CreateStaff {
                    email: "dilan@example.org".to_string(),
                    role_ids: vec![DAM_OFFICER_ROLE_ID],
                }
            ]
        );
    }

    #[tokio::test]
    async fn the_password_itself_is_never_what_is_stored() {
        let fakes = Fakes::new();

        let staff = use_case(&fakes)
            .execute(&actor(OWNER_ID), input("dilan@example.org", Vec::new()))
            .await
            .expect("staff");

        assert_eq!(*staff.password_hash(), hash_of(PASSWORD));
        assert_ne!(staff.password_hash().as_str(), PASSWORD);
    }

    #[tokio::test]
    async fn an_email_another_account_has_is_refused() {
        let result = use_case(&Fakes::new())
            .execute(&actor(OWNER_ID), input(OWNER_EMAIL, Vec::new()))
            .await;

        assert!(matches!(
            result,
            Err(AppError::Staff(StaffError::EmailTaken))
        ));
    }

    #[tokio::test]
    async fn a_role_that_does_not_exist_cannot_be_assigned() {
        let result = use_case(&Fakes::new())
            .execute(&actor(OWNER_ID), input("dilan@example.org", vec![99]))
            .await;

        assert!(matches!(
            result,
            Err(AppError::Staff(StaffError::UnknownRole))
        ));
    }

    fn with_a_dam_officer() -> Fakes {
        Fakes::new().with_staff(a_staff_member(
            2,
            "dams@example.org",
            true,
            &[DAM_OFFICER_ROLE_ID],
        ))
    }

    #[tokio::test]
    async fn a_role_granting_only_what_the_actor_holds_can_be_handed_out() {
        let staff = use_case(&with_a_dam_officer())
            .execute(
                &actor(2),
                input("dilan@example.org", vec![DAM_OFFICER_ROLE_ID]),
            )
            .await
            .expect("staff");

        assert_eq!(staff.roles().len(), 1);
    }

    #[tokio::test]
    async fn the_owner_role_cannot_be_handed_out_by_someone_who_holds_less() {
        let fakes = with_a_dam_officer();

        let result = use_case(&fakes)
            .execute(&actor(2), input("dilan@example.org", vec![OWNER_ROLE_ID]))
            .await;

        assert!(matches!(
            result,
            Err(AppError::Staff(StaffError::CannotGrant))
        ));
        assert!(fakes.stored_staff(3).is_none());
    }
}
