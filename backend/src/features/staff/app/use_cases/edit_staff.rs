use std::sync::Arc;

use crate::{
    app::StaffContext,
    features::staff::{
        app::{AppError, PasswordHasher, StaffRepository},
        domain::{JobTitle, Password, RoleSelection, Staff, StaffChange, StaffName},
    },
    shared::Phone,
};

pub struct EditStaffInput {
    pub name: StaffName,
    /// `None` clears it.
    pub phone: Option<Phone>,
    /// `None` clears it.
    pub job_title: Option<JobTitle>,
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

        let change = StaffChange::new(input.name, input.active, input.roles, password_hash)
            .with_details(input.phone, input.job_title);

        change
            .ensure_not_deactivating_own_account(*actor.staff_id(), id)
            .inspect_err(|_| tracing::info!(staff_id = id, "staff edit refused: own account"))?;

        let updated = self
            .staff
            .update(id, &change, *actor.staff_id())
            .await
            .inspect_err(|error| tracing::info!(%error, staff_id = id, by_staff_id = *actor.staff_id(), "staff member not edited"))?;

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
            phone: None,
            job_title: None,
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
    async fn sets_the_phone_and_the_job_title_and_clears_them_when_left_out() {
        let fakes = with_a_dam_officer();
        let use_case = use_case(&fakes);

        let staff = use_case
            .execute(
                &actor(OWNER_ID),
                OTHER_ID,
                EditStaffInput {
                    phone: Some(Phone::new("+9647501234567".to_string()).expect("phone")),
                    job_title: Some(JobTitle::new("Dam engineer".to_string()).expect("title")),
                    ..input(true, vec![DAM_OFFICER_ROLE_ID])
                },
            )
            .await
            .expect("staff");

        assert_eq!(
            staff.phone().as_ref().map(Phone::as_str),
            Some("+9647501234567")
        );
        assert_eq!(
            staff.job_title().as_ref().map(JobTitle::as_str),
            Some("Dam engineer")
        );

        let cleared = use_case
            .execute(
                &actor(OWNER_ID),
                OTHER_ID,
                input(true, vec![DAM_OFFICER_ROLE_ID]),
            )
            .await
            .expect("staff");

        assert!(cleared.phone().is_none() && cleared.job_title().is_none());
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

    fn a_new_password(role_ids: Vec<i32>) -> EditStaffInput {
        EditStaffInput {
            password: Some(Password::new("a brand new password".to_string()).expect("ok")),
            ..input(true, role_ids)
        }
    }

    #[tokio::test]
    async fn nobody_can_give_themselves_a_role_that_grants_more_than_they_hold() {
        let fakes = with_a_dam_officer();

        let result = use_case(&fakes)
            .execute(
                &actor(OTHER_ID),
                OTHER_ID,
                input(true, vec![OWNER_ROLE_ID, DAM_OFFICER_ROLE_ID]),
            )
            .await;

        assert!(matches!(
            result,
            Err(AppError::Staff(StaffError::CannotGrant))
        ));
        assert!(
            !fakes
                .stored_staff(OTHER_ID)
                .expect("stored")
                .is_active_owner()
        );
    }

    #[tokio::test]
    async fn a_role_already_held_stays_without_the_actor_holding_it() {
        // The dam officer renames the owner and leaves their roles alone.
        let staff = use_case(&with_a_dam_officer())
            .execute(&actor(OTHER_ID), OWNER_ID, input(true, vec![OWNER_ROLE_ID]))
            .await
            .expect("staff");

        assert!(staff.is_active_owner());
    }

    #[tokio::test]
    async fn removing_a_role_needs_no_permission_of_ones_own() {
        let fakes = with_a_second_owner().with_staff(a_staff_member(
            3,
            "dams@example.org",
            true,
            &[DAM_OFFICER_ROLE_ID],
        ));

        let staff = use_case(&fakes)
            .execute(&actor(3), OTHER_ID, input(true, Vec::new()))
            .await
            .expect("staff");

        assert!(staff.roles().is_empty());
    }

    #[tokio::test]
    async fn the_password_of_someone_who_can_do_more_cannot_be_reset() {
        let fakes = with_a_dam_officer();

        let result = use_case(&fakes)
            .execute(
                &actor(OTHER_ID),
                OWNER_ID,
                a_new_password(vec![OWNER_ROLE_ID]),
            )
            .await;

        assert!(matches!(
            result,
            Err(AppError::Staff(StaffError::CannotGrant))
        ));
        assert_eq!(
            *fakes
                .stored_staff(OWNER_ID)
                .expect("stored")
                .password_hash(),
            hash_of(PASSWORD),
            "a new password for the owner would be the owner's account"
        );
    }

    #[tokio::test]
    async fn ones_own_password_can_always_be_changed() {
        let staff = use_case(&with_a_dam_officer())
            .execute(
                &actor(OTHER_ID),
                OTHER_ID,
                a_new_password(vec![DAM_OFFICER_ROLE_ID]),
            )
            .await
            .expect("staff");

        assert_eq!(*staff.password_hash(), hash_of("a brand new password"));
    }
}
