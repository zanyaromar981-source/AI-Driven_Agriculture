use std::sync::Arc;

use crate::{
    app::{AppError as GlobalAppError, StaffContext},
    features::briefs::{
        app::{AppError, BriefRepository, use_cases::RecordBriefInput},
        domain::DailyBrief,
    },
};

pub struct CorrectBriefUseCase {
    repository: Arc<dyn BriefRepository>,
}

impl CorrectBriefUseCase {
    pub fn new(repository: Arc<dyn BriefRepository>) -> Self {
        Self { repository }
    }

    /// Replaces the brief stored for a day and scope with what a staff
    /// member entered. Nothing is read first: whether there is such a brief
    /// is decided by the update itself. A day and scope with no brief is not
    /// found, because only the nightly job writes new briefs.
    pub async fn execute(
        &self,
        actor: &StaffContext,
        input: RecordBriefInput,
    ) -> Result<DailyBrief, AppError> {
        let brief = input.into_brief()?;

        let Some(stored) = self.repository.update(&brief).await? else {
            tracing::info!(
                staff_id = *actor.staff_id(),
                day = %brief.day(),
                scope = brief.scope().as_str(),
                "brief not corrected: there is none for this day and scope"
            );

            return Err(GlobalAppError::NotFound.into());
        };

        tracing::info!(
            staff_id = *actor.staff_id(),
            day = %stored.day(),
            scope = stored.scope().as_str(),
            author = stored.author().as_str(),
            "brief corrected by staff"
        );

        Ok(stored)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::briefs::{
        app::testing::{Call, Fakes, a_brief, a_day, a_source, actor, an_input},
        domain::{BriefError, MAX_SOURCES},
    };

    #[tokio::test]
    async fn replaces_the_stored_brief_in_one_call() {
        let fakes = Fakes::new().with_stored(a_brief(9, "region"));
        let use_case = CorrectBriefUseCase::new(Arc::new(fakes.clone()));

        let stored = use_case
            .execute(&actor(), an_input(9, "region", "Corrected headline"))
            .await
            .expect("corrected");

        assert_eq!(stored.headline_en().as_str(), "Corrected headline");
        assert_eq!(
            fakes.calls(),
            vec![Call::Update {
                day: a_day(9),
                scope: "region".to_string(),
            }],
            "the brief is not looked up before the write"
        );
    }

    #[tokio::test]
    async fn a_day_and_scope_with_no_brief_is_not_found_and_is_not_created() {
        let fakes = Fakes::new().with_stored(a_brief(9, "region"));
        let use_case = CorrectBriefUseCase::new(Arc::new(fakes.clone()));

        let result = use_case
            .execute(&actor(), an_input(9, "kalar", "Corrected headline"))
            .await;

        assert!(matches!(
            result,
            Err(AppError::GlobalAppError(GlobalAppError::NotFound))
        ));
        assert!(
            fakes.stored(a_day(9), "kalar").is_none(),
            "staff correct briefs, they do not write new ones"
        );
    }

    #[tokio::test]
    async fn a_correction_that_breaks_a_rule_is_not_written() {
        let fakes = Fakes::new().with_stored(a_brief(9, "region"));
        let use_case = CorrectBriefUseCase::new(Arc::new(fakes.clone()));

        let result = use_case
            .execute(
                &actor(),
                RecordBriefInput {
                    sources: vec![a_source(); MAX_SOURCES + 1],
                    ..an_input(9, "region", "Corrected headline")
                },
            )
            .await;

        assert!(matches!(
            result,
            Err(AppError::Brief(BriefError::TooManySources { .. }))
        ));
        assert!(fakes.calls().is_empty());
    }

    #[tokio::test]
    async fn a_repository_failure_surfaces() {
        let use_case = CorrectBriefUseCase::new(Arc::new(Fakes::new().failing()));

        assert!(
            use_case
                .execute(&actor(), an_input(9, "region", "Corrected headline"))
                .await
                .is_err()
        );
    }
}
