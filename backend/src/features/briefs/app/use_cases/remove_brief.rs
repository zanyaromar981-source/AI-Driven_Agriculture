use std::sync::Arc;

use chrono::NaiveDate;

use crate::{
    app::StaffContext,
    features::briefs::{
        app::{AppError, BriefRepository},
        domain::BriefScope,
    },
};

pub struct RemoveBriefUseCase {
    repository: Arc<dyn BriefRepository>,
}

impl RemoveBriefUseCase {
    pub fn new(repository: Arc<dyn BriefRepository>) -> Self {
        Self { repository }
    }

    /// Removes the brief of a day and scope on a staff member's word. One
    /// that is already gone is a success, so a repeated delete gets the same
    /// answer as the first.
    pub async fn execute(
        &self,
        actor: &StaffContext,
        day: NaiveDate,
        scope: &BriefScope,
    ) -> Result<(), AppError> {
        self.repository.delete(day, scope).await?;

        tracing::info!(
            staff_id = *actor.staff_id(),
            %day,
            scope = scope.as_str(),
            "brief removed by staff"
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::briefs::app::testing::{Call, Fakes, a_brief, a_day, a_scope, actor};

    #[tokio::test]
    async fn removes_only_that_day_of_that_scope_in_one_call() {
        let fakes = Fakes::new()
            .with_stored(a_brief(9, "kalar"))
            .with_stored(a_brief(9, "region"));
        let use_case = RemoveBriefUseCase::new(Arc::new(fakes.clone()));

        use_case
            .execute(&actor(), a_day(9), &a_scope("kalar"))
            .await
            .expect("removed");

        assert!(fakes.stored(a_day(9), "kalar").is_none());
        assert!(fakes.stored(a_day(9), "region").is_some());
        assert_eq!(
            fakes.calls(),
            vec![Call::Delete {
                day: a_day(9),
                scope: "kalar".to_string(),
            }]
        );
    }

    #[tokio::test]
    async fn removing_a_brief_that_is_already_gone_succeeds_again() {
        let use_case = RemoveBriefUseCase::new(Arc::new(Fakes::new()));

        use_case
            .execute(&actor(), a_day(9), &a_scope("kalar"))
            .await
            .expect("first");
        use_case
            .execute(&actor(), a_day(9), &a_scope("kalar"))
            .await
            .expect("repeat");
    }

    #[tokio::test]
    async fn a_repository_failure_surfaces() {
        let use_case = RemoveBriefUseCase::new(Arc::new(Fakes::new().failing()));

        assert!(
            use_case
                .execute(&actor(), a_day(9), &a_scope("kalar"))
                .await
                .is_err()
        );
    }
}
