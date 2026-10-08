use std::sync::Arc;

use crate::{
    app::StaffContext,
    features::staff::{
        app::{AppError, PasswordHasher, StaffRepository},
        domain::{Password, RoleSelection, Staff, StaffChange, StaffName},
    },
};

pub struct EditStaffInput {
    pub name: StaffName,
    pub active: bool,
    pub roles: RoleSelection,
    /// `None` keeps the password the account has.
    pub password: Option<Password>,
}

pub struct EditStaffUseCase {
    staff: Arc<dyn StaffRepository>,
    hasher: Arc<dyn PasswordHasher>,
}

impl EditStaffUseCase {
    pub fn new(staff: Arc<dyn StaffRepository>, hasher: Arc<dyn PasswordHasher>) -> Self {
        Self { staff, hasher }
    }

    /// The rule about the last owner depends on what other requests are
    /// doing right now, so it is not checked here: the repository checks it
    /// inside the transaction that makes the change.
    pub async fn execute(
        &self,
        actor: &StaffContext,
        id: i32,
        input: EditStaffInput,
    ) -> Result<Staff, AppError> {
        let password_hash = match &input.password {
            Some(password) => Some(self.hasher.hash(password).await?),
            None => None,
        };

        let change = StaffChange::new(input.name, input.active, input.roles, password_hash);

        change
            .ensure_not_deactivating_own_account(*actor.staff_id(), id)
            .inspect_err(|_| tracing::info!(staff_id = id, "staff edit refused: own account"))?;

        let updated = self.staff.update(id, &change).await?;

        tracing::info!(
            staff_id = id,
            active = *updated.active(),
            roles = updated.roles().len(),
            password_changed = change.password_hash().is_some(),
            by_staff_id = *actor.staff_id(),
            "staff member edited"
        );

        Ok(updated)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        app::AppError as GlobalAppError,
        features::staff::{
            app::testing::{
                Call, DAM_OFFICER_ROLE_ID, Fakes, OWNER_ID, OWNER_ROLE_ID, PASSWORD,
                a_staff_member, actor, hash_of,
            },
            domain::StaffError,
        },
    };

    const OTHER_ID: i32 = 2;

    fn use_case(fakes: &Fakes) -> EditStaffUseCase {
        EditStaffUseCase::new(Arc::new(fakes.clone()), Arc::new(fakes.clone()))
    }

    fn input(active: bool, role_ids: Vec<i32>) -> EditStaffInput {
        EditStaffInput {
            name: StaffName::new("Renamed".to_string()).expect("name"),
            active,
            roles: RoleSelection::new(role_ids).expect("roles"),
            password: None,
        }
    }

    fn with_a_dam_officer() -> Fakes {
        Fakes::new().with_staff(a_staff_member(
            OTHER_ID,
            "dams@example.org",
            true,
            &[DAM_OFFICER_ROLE_ID],
        ))
    }

    fn with_a_second_owner() -> Fakes {
        Fakes::new().with_staff(a_staff_member(
            OTHER_ID,
            "second@example.org",
            true,
            &[OWNER_ROLE_ID],
        ))
    }

    #[tokio::test]
    async fn changes_the_name_the_state_and_the_roles_but_keeps_the_password() {
        let fakes = with_a_dam_officer();

        let staff = use_case(&fakes)
            .execute(&actor(OWNER_ID), OTHER_ID, input(false, Vec::new()))
            .await
            .expect("staff");

        assert_eq!(staff.name().as_str(), "Renamed");
        assert!(!*staff.active());
        assert!(staff.roles().is_empty());
        assert_eq!(*staff.password_hash(), hash_of(PASSWORD));
        assert_eq!(
            fakes.calls(),
            vec![Call::UpdateStaff {
                id: OTHER_ID,
                active: false,
                role_ids: Vec::new(),
                sets_password: false,
            }],
            "no password was sent, so nothing may be hashed or overwritten"
        );
    }

    #[tokio::test]
    async fn a_new_password_is_hashed_before_it_is_stored() {
        let fakes = with_a_dam_officer();

        let staff = use_case(&fakes)
            .execute(
                &actor(OWNER_ID),
                OTHER_ID,
                EditStaffInput {
                    password: Some(Password::new("a brand new password".to_string()).expect("ok")),
                    ..input(true, vec![DAM_OFFICER_ROLE_ID])
                },
            )
            .await
            .expect("staff");

        assert_eq!(*staff.password_hash(), hash_of("a brand new password"));
        assert_eq!(fakes.calls().first(), Some(&Call::HashPassword));
    }

    #[tokio::test]
    async fn nobody_can_deactivate_their_own_account() {
        let fakes = with_a_second_owner();

        let result = use_case(&fakes)
            .execute(
                &actor(OWNER_ID),
                OWNER_ID,
                input(false, vec![OWNER_ROLE_ID]),
            )
            .await;

        assert!(matches!(
            result,
            Err(AppError::Staff(StaffError::OwnAccount))
        ));
        assert!(
            !fakes
                .calls()
                .iter()
                .any(|call| matches!(call, Call::UpdateStaff { .. })),
            "a refused change must not reach the repository"
        );
    }

    #[tokio::test]
    async fn the_last_active_owner_cannot_be_stripped_of_the_owner_role() {
        let fakes = Fakes::new();

        let result = use_case(&fakes)
            .execute(
                &actor(OWNER_ID),
                OWNER_ID,
                input(true, vec![DAM_OFFICER_ROLE_ID]),
            )
            .await;

        assert!(matches!(
            result,
            Err(AppError::Staff(StaffError::LastOwner))
        ));
        assert!(
            fakes
                .stored_staff(OWNER_ID)
                .expect("still there")
                .is_active_owner()
        );
    }

    #[tokio::test]
    async fn the_last_active_owner_cannot_be_deactivated_by_someone_else() {
        // The other staff member holds a custom role, so the owner is alone.
        let fakes = with_a_dam_officer();

        let result = use_case(&fakes)
            .execute(
                &actor(OTHER_ID),
                OWNER_ID,
                input(false, vec![OWNER_ROLE_ID]),
            )
            .await;

        assert!(matches!(
            result,
            Err(AppError::Staff(StaffError::LastOwner))
        ));
    }

    #[tokio::test]
    async fn an_owner_can_step_down_while_another_active_owner_remains() {
        let fakes = with_a_second_owner();

        let staff = use_case(&fakes)
            .execute(
                &actor(OWNER_ID),
                OWNER_ID,
                input(true, vec![DAM_OFFICER_ROLE_ID]),
            )
            .await
            .expect("staff");

        assert!(!staff.is_active_owner());
    }

    #[tokio::test]
    async fn a_role_that_does_not_exist_cannot_be_assigned() {
        let result = use_case(&with_a_dam_officer())
            .execute(&actor(OWNER_ID), OTHER_ID, input(true, vec![99]))
            .await;

        assert!(matches!(
            result,
            Err(AppError::Staff(StaffError::UnknownRole))
        ));
    }

    #[tokio::test]
    async fn a_missing_staff_member_is_not_found() {
        let result = use_case(&Fakes::new())
            .execute(&actor(OWNER_ID), 99, input(true, Vec::new()))
            .await;

        assert!(matches!(
            result,
            Err(AppError::GlobalAppError(GlobalAppError::NotFound))
        ));
    }
}
