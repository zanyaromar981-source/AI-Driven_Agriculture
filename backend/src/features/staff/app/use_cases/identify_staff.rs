use std::sync::Arc;

use crate::{
    app::AppError as GlobalAppError,
    features::staff::{
        app::{AppError, StaffRepository},
        domain::StaffAccess,
    },
};

/// Answers, on every dashboard request, who a token belongs to and what
/// they may do right now.
pub struct IdentifyStaffUseCase {
    staff: Arc<dyn StaffRepository>,
}

impl IdentifyStaffUseCase {
    pub fn new(staff: Arc<dyn StaffRepository>) -> Self {
        Self { staff }
    }

    /// A token outlives a deleted or deactivated account, so the account is
    /// read again each time: either one makes the token useless at once.
    pub async fn execute(&self, staff_id: i32) -> Result<StaffAccess, AppError> {
        let Some(staff) = self
            .staff
            .find_by_id(staff_id)
            .await?
            .filter(|staff| *staff.active())
        else {
            tracing::info!(staff_id, "dashboard token refused: no active account");

            return Err(GlobalAppError::Unauthorized(
                "The staff member is gone or inactive".to_string(),
            )
            .into());
        };

        let granted = self.staff.permissions_granted_to(staff_id).await?;

        Ok(StaffAccess::new(staff, granted))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        app::{Action, Permission, Resource},
        features::staff::app::testing::{
            Call, DAM_OFFICER_ROLE_ID, Fakes, OWNER_ID, OWNER_ROLE_ID, a_staff_member,
        },
    };

    #[tokio::test]
    async fn an_active_staff_member_gets_the_permissions_of_their_roles() {
        let fakes = Fakes::new().with_staff(a_staff_member(
            2,
            "dams@example.org",
            true,
            &[DAM_OFFICER_ROLE_ID],
        ));
        let use_case = IdentifyStaffUseCase::new(Arc::new(fakes.clone()));

        let access = use_case.execute(2).await.expect("access");

        assert_eq!(
            *access.permissions(),
            vec![
                Permission::new(Resource::Dams, Action::Read),
                Permission::new(Resource::Dams, Action::Update),
            ]
        );
        assert_eq!(
            fakes.calls(),
            vec![
                Call::FindStaff { id: 2 },
                Call::PermissionsGrantedTo { staff_id: 2 }
            ]
        );
    }

    #[tokio::test]
    async fn holding_two_roles_gives_the_union_with_nothing_twice() {
        let fakes = Fakes::new().with_staff(a_staff_member(
            2,
            "both@example.org",
            true,
            &[OWNER_ROLE_ID, DAM_OFFICER_ROLE_ID],
        ));
        let use_case = IdentifyStaffUseCase::new(Arc::new(fakes));

        let access = use_case.execute(2).await.expect("access");

        assert_eq!(*access.permissions(), Permission::all());
    }

    #[tokio::test]
    async fn an_inactive_staff_member_is_refused_before_any_permission_is_read() {
        let fakes = Fakes::new().with_staff(a_staff_member(
            2,
            "gone@example.org",
            false,
            &[OWNER_ROLE_ID],
        ));
        let use_case = IdentifyStaffUseCase::new(Arc::new(fakes.clone()));

        let result = use_case.execute(2).await;

        assert!(matches!(
            result,
            Err(AppError::GlobalAppError(GlobalAppError::Unauthorized(_)))
        ));
        assert_eq!(fakes.calls(), vec![Call::FindStaff { id: 2 }]);
    }

    #[tokio::test]
    async fn a_token_for_a_deleted_account_is_refused() {
        let use_case = IdentifyStaffUseCase::new(Arc::new(Fakes::new()));

        assert!(matches!(
            use_case.execute(99).await,
            Err(AppError::GlobalAppError(GlobalAppError::Unauthorized(_)))
        ));
        assert!(use_case.execute(OWNER_ID).await.is_ok());
    }
}
