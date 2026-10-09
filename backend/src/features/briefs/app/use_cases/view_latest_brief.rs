use std::sync::Arc;

use crate::{
    app::AppError as GlobalAppError,
    features::briefs::{
        app::{AppError, BriefRepository},
        domain::{BriefScope, DailyBrief},
    },
};

pub struct ViewLatestBriefUseCase {
    repository: Arc<dyn BriefRepository>,
}

impl ViewLatestBriefUseCase {
    pub fn new(repository: Arc<dyn BriefRepository>) -> Self {
        Self { repository }
    }

    /// Returns the brief of the newest day stored for the scope. A scope
    /// with no brief at all is not found: nothing is made up in its place.
    pub async fn execute(&self, scope: &BriefScope) -> Result<DailyBrief, AppError> {
        let Some(brief) = self.repository.find_latest(scope).await? else {
            return Err(GlobalAppError::NotFound.into());
        };

        tracing::debug!(scope = scope.as_str(), day = %brief.day(), "latest brief viewed");

        Ok(brief)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::briefs::app::testing::{Call, Fakes, a_brief, a_day, a_scope};

    #[tokio::test]
    async fn returns_the_newest_day_of_the_scope_asked_for() {
        let fakes = Fakes::new()
            .with_stored(a_brief(7, "region"))
            .with_stored(a_brief(9, "region"))
            .with_stored(a_brief(8, "region"))
            .with_stored(a_brief(10, "kalar"));
        let use_case = ViewLatestBriefUseCase::new(Arc::new(fakes.clone()));

        let brief = use_case.execute(&a_scope("region")).await.expect("brief");

        assert_eq!(
            *brief.day(),
            a_day(9),
            "a newer brief of a district is not the region's"
        );
        assert_eq!(
            fakes.calls(),
            vec![Call::FindLatest {
                scope: "region".to_string()
            }]
        );
    }

    #[tokio::test]
    async fn a_scope_with_no_brief_is_not_found() {
        let fakes = Fakes::new().with_stored(a_brief(9, "region"));
        let use_case = ViewLatestBriefUseCase::new(Arc::new(fakes));

        let result = use_case.execute(&a_scope("kalar")).await;

        assert!(matches!(
            result,
            Err(AppError::GlobalAppError(GlobalAppError::NotFound))
        ));
    }

    #[tokio::test]
    async fn a_repository_failure_surfaces() {
        let use_case = ViewLatestBriefUseCase::new(Arc::new(Fakes::new().failing()));

        assert!(use_case.execute(&a_scope("region")).await.is_err());
    }
}
