use std::sync::Arc;

use crate::features::versions::{
    app::{AppError, VersionRepository},
    domain::DataVersion,
};

/// Who is asking decides what they may learn has changed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Audience {
    /// Anyone: the public topics only.
    Public,
    /// Signed-in staff: every topic.
    Staff,
}

pub struct ListVersionsUseCase {
    repository: Arc<dyn VersionRepository>,
}

impl ListVersionsUseCase {
    pub fn new(repository: Arc<dyn VersionRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, audience: Audience) -> Result<Vec<DataVersion>, AppError> {
        let versions = self.repository.find_all().await?;

        Ok(versions
            .into_iter()
            .filter(|version| audience == Audience::Staff || !version.topic().is_private())
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::versions::{app::testing::FakeVersionRepository, domain::Topic};

    #[tokio::test]
    async fn the_public_sees_no_private_topic() {
        let use_case = ListVersionsUseCase::new(Arc::new(FakeVersionRepository::new()));

        let versions = use_case.execute(Audience::Public).await.expect("versions");

        assert!(!versions.is_empty());
        assert!(
            versions.iter().all(|version| !version.topic().is_private()),
            "even the fact that farmers changed is not public"
        );
    }

    #[tokio::test]
    async fn staff_see_every_topic() {
        let use_case = ListVersionsUseCase::new(Arc::new(FakeVersionRepository::new()));

        let versions = use_case.execute(Audience::Staff).await.expect("versions");

        assert_eq!(versions.len(), Topic::ALL.len());
    }

    #[tokio::test]
    async fn a_repository_failure_surfaces() {
        let use_case = ListVersionsUseCase::new(Arc::new(FakeVersionRepository::failing()));

        assert!(use_case.execute(Audience::Public).await.is_err());
    }
}
