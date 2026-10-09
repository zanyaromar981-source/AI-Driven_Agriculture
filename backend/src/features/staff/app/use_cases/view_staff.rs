use std::sync::Arc;

use crate::{
    app::AppError as GlobalAppError,
    features::staff::{
        app::{AppError, StaffRepository},
        domain::Staff,
    },
};

pub struct ViewStaffUseCase {
    staff: Arc<dyn StaffRepository>,
}

impl ViewStaffUseCase {
    pub fn new(staff: Arc<dyn StaffRepository>) -> Self {
        Self { staff }
    }

    pub async fn execute(&self, id: i32) -> Result<Staff, AppError> {
        self.staff
            .find_by_id(id)
            .await?
            .ok_or_else(|| GlobalAppError::NotFound.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::staff::app::testing::{Call, Fakes, OWNER_EMAIL, OWNER_ID};

    #[tokio::test]
    async fn returns_the_staff_member_with_their_roles() {
        let fakes = Fakes::new();
        let use_case = ViewStaffUseCase::new(Arc::new(fakes.clone()));

        let staff = use_case.execute(OWNER_ID).await.expect("staff");

        assert_eq!(staff.email().as_str(), OWNER_EMAIL);
        assert_eq!(staff.roles().len(), 1);
        assert_eq!(fakes.calls(), vec![Call::FindStaff { id: OWNER_ID }]);
    }

    #[tokio::test]
    async fn a_missing_staff_member_is_not_found() {
        let use_case = ViewStaffUseCase::new(Arc::new(Fakes::new()));

        assert!(matches!(
            use_case.execute(99).await,
            Err(AppError::GlobalAppError(GlobalAppError::NotFound))
        ));
    }
}
