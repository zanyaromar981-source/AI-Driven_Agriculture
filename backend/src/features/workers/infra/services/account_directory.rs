use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    app::{AppError as GlobalAppError, Pagination},
    features::{
        farmers::app::{FarmerFilter, FarmerRepository},
        workers::app::{AccountDirectory, AppError},
    },
    shared::Phone,
};

const PAGE_SIZE: u64 = 100;

/// Finds the blocked accounts by asking the farmers feature through its own
/// repository port. Only the phones cross over.
#[derive(Debug)]
pub struct FarmersFeatureAccountDirectory {
    farmers: Arc<dyn FarmerRepository>,
}

impl FarmersFeatureAccountDirectory {
    pub fn new(farmers: Arc<dyn FarmerRepository>) -> Self {
        Self { farmers }
    }
}

#[async_trait]
impl AccountDirectory for FarmersFeatureAccountDirectory {
    async fn blocked_phones(&self) -> Result<Vec<Phone>, AppError> {
        let filter = FarmerFilter {
            blocked: Some(true),
            ..FarmerFilter::default()
        };
        let mut phones = Vec::new();
        let mut page = 1;

        // Blocked accounts are few, so this is one read almost always. It
        // goes on to the last page all the same: a blocked account left out
        // here would have its phone shown.
        loop {
            let (farmers, count) = self
                .farmers
                .find_page(&filter, &Pagination::new(page, PAGE_SIZE))
                .await
                .map_err(|error| {
                    tracing::error!(%error, "reading blocked accounts for workers failed");

                    AppError::from(GlobalAppError::InternalServerError)
                })?;

            let read = farmers.len() as u64;
            phones.extend(farmers.iter().map(|farmer| farmer.phone().clone()));

            if read < PAGE_SIZE || phones.len() as u64 >= count {
                return Ok(phones);
            }

            page += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::farmers::app::testing::{Fakes, phone};

    #[tokio::test]
    async fn a_blocked_farmer_is_named_by_their_phone() {
        let directory =
            FarmersFeatureAccountDirectory::new(Arc::new(Fakes::new().with_blocked_farmer()));

        assert_eq!(
            directory.blocked_phones().await.expect("phones"),
            vec![phone()]
        );
    }

    #[tokio::test]
    async fn a_farmer_who_is_not_blocked_is_not_named() {
        let directory = FarmersFeatureAccountDirectory::new(Arc::new(Fakes::new().with_farmer()));

        assert!(directory.blocked_phones().await.expect("phones").is_empty());
    }
}
