use std::sync::Arc;

use crate::features::staff::{
    app::{AppError, StaffRepository},
    domain::Staff,
};

pub struct ListStaffUseCase {
    staff: Arc<dyn StaffRepository>,
}

impl ListStaffUseCase {
    pub fn new(staff: Arc<dyn StaffRepository>) -> Self {
        Self { staff }
    }

    pub async fn execute(&self) -> Result<Vec<Staff>, AppError> {
        let staff = self.staff.find_all().await?;

        tracing::debug!(count = staff.len(), "listed staff");

        Ok(staff)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::staff::app::testing::{Call, Fakes};

    #[tokio::test]
    async fn returns_every_staff_member() {
        let fakes = Fakes::new();
        let use_case = ListStaffUseCase::new(Arc::new(fakes.clone()));

        let staff = use_case.execute().await.expect("staff");

        assert_eq!(staff.len(), 1);
        assert_eq!(fakes.calls(), vec![Call::FindAllStaff]);
    }

    #[tokio::test]
    async fn a_repository_failure_surfaces() {
        let use_case = ListStaffUseCase::new(Arc::new(Fakes::failing()));

        assert!(use_case.execute().await.is_err());
    }
}
