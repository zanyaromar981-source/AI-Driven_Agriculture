use std::sync::Arc;

use crate::{
    app::StaffContext,
    features::staff::{
        app::{AppError, StaffRepository},
        domain::ensure_not_own_account,
    },
};

pub struct RemoveStaffUseCase {
    staff: Arc<dyn StaffRepository>,
}

impl RemoveStaffUseCase {
    pub fn new(staff: Arc<dyn StaffRepository>) -> Self {
        Self { staff }
    }

    /// Whether the staff member is the last owner is checked by the
    /// repository inside the transaction that deletes them.
    pub async fn execute(&self, actor: &StaffContext, id: i32) -> Result<(), AppError> {
        ensure_not_own_account(*actor.staff_id(), id)
            .inspect_err(|_| tracing::info!(staff_id = id, "staff removal refused: own account"))?;

        self.staff.delete(id).await?;

        tracing::info!(
            staff_id = id,
            by_staff_id = *actor.staff_id(),
            "staff member removed"
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

    const OTHER_ID: i32 = 2;

    #[tokio::test]
    async fn removes_another_staff_member() {
        let fakes = Fakes::new().with_staff(a_staff_member(
            OTHER_ID,
            "dams@example.org",
            true,
            &[DAM_OFFICER_ROLE_ID],
        ));
        let use_case = RemoveStaffUseCase::new(Arc::new(fakes.clone()));

        use_case
            .execute(&actor(OWNER_ID), OTHER_ID)
            .await
            .expect("removed");

        assert!(fakes.stored_staff(OTHER_ID).is_none());
        assert_eq!(fakes.calls(), vec![Call::DeleteStaff { id: OTHER_ID }]);
    }

    #[tokio::test]
    async fn nobody_can_delete_their_own_account() {
        let fakes = Fakes::new();
        let use_case = RemoveStaffUseCase::new(Arc::new(fakes.clone()));

        let result = use_case.execute(&actor(OWNER_ID), OWNER_ID).await;

        assert!(matches!(
            result,
            Err(AppError::Staff(StaffError::OwnAccount))
        ));
        assert!(fakes.calls().is_empty());
    }

    #[tokio::test]
    async fn the_last_active_owner_cannot_be_deleted() {
        let fakes = Fakes::new().with_staff(a_staff_member(
            OTHER_ID,
            "dams@example.org",
            true,
            &[DAM_OFFICER_ROLE_ID],
        ));
        let use_case = RemoveStaffUseCase::new(Arc::new(fakes.clone()));

        let result = use_case.execute(&actor(OTHER_ID), OWNER_ID).await;

        assert!(matches!(
            result,
            Err(AppError::Staff(StaffError::LastOwner))
        ));
        assert!(fakes.stored_staff(OWNER_ID).is_some());
    }

    #[tokio::test]
    async fn an_owner_can_be_deleted_while_another_active_owner_remains() {
        let fakes = Fakes::new().with_staff(a_staff_member(
            OTHER_ID,
            "second@example.org",
            true,
            &[OWNER_ROLE_ID],
        ));
        let use_case = RemoveStaffUseCase::new(Arc::new(fakes.clone()));

        assert!(use_case.execute(&actor(OWNER_ID), OTHER_ID).await.is_ok());
    }

    #[tokio::test]
    async fn an_inactive_owner_does_not_count_as_the_other_owner() {
        let fakes = Fakes::new()
            .with_staff(a_staff_member(
                OTHER_ID,
                "asleep@example.org",
                false,
                &[OWNER_ROLE_ID],
            ))
            .with_staff(a_staff_member(
                3,
                "dams@example.org",
                true,
                &[DAM_OFFICER_ROLE_ID],
            ));
        let use_case = RemoveStaffUseCase::new(Arc::new(fakes));

        let result = use_case.execute(&actor(3), OWNER_ID).await;

        assert!(
            matches!(result, Err(AppError::Staff(StaffError::LastOwner))),
            "an owner who cannot sign in cannot manage anything"
        );
    }

    #[tokio::test]
    async fn a_missing_staff_member_is_not_found() {
        let use_case = RemoveStaffUseCase::new(Arc::new(Fakes::new()));

        assert!(matches!(
            use_case.execute(&actor(OWNER_ID), 99).await,
            Err(AppError::GlobalAppError(GlobalAppError::NotFound))
        ));
    }
}
