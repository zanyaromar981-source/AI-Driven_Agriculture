use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    app::AppError as GlobalAppError,
    features::{
        farmers::app::{AppError, LetterIssuers},
        staff::app::StaffRepository,
    },
};

/// Finds the name of the staff member issuing a letter by asking the staff
/// feature through its own repository port.
#[derive(Debug)]
pub struct StaffFeatureLetterIssuers {
    staff: Arc<dyn StaffRepository>,
}

impl StaffFeatureLetterIssuers {
    pub fn new(staff: Arc<dyn StaffRepository>) -> Self {
        Self { staff }
    }
}

#[async_trait]
impl LetterIssuers for StaffFeatureLetterIssuers {
    async fn name_of(&self, staff_id: i32) -> Result<Option<String>, AppError> {
        let staff = self.staff.find_by_id(staff_id).await.map_err(|error| {
            tracing::error!(%error, "looking up the staff member issuing a letter failed");

            AppError::from(GlobalAppError::InternalServerError)
        })?;

        Ok(staff.map(|staff| staff.name().into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::staff::app::testing::{Fakes, OWNER_ID};

    #[tokio::test]
    async fn gives_the_name_of_a_staff_member_and_none_for_a_missing_one() {
        let issuers = StaffFeatureLetterIssuers::new(Arc::new(Fakes::new()));

        assert_eq!(
            issuers.name_of(OWNER_ID).await.expect("name").as_deref(),
            Some("Hiwa K.")
        );
        assert!(issuers.name_of(99).await.expect("none").is_none());
    }

    #[tokio::test]
    async fn a_failure_in_the_staff_feature_is_an_error() {
        let issuers = StaffFeatureLetterIssuers::new(Arc::new(Fakes::failing()));

        assert!(issuers.name_of(OWNER_ID).await.is_err());
    }
}
