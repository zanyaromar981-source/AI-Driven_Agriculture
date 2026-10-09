use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    app::AppError as GlobalAppError,
    features::{
        farmers::{app::FarmerRepository, domain::Farmer},
        messages::{
            app::{AppError, SenderDirectory},
            domain::{FarmerContact, SearchText},
        },
    },
    shared::Phone,
};

/// A search by name or phone looks among this many matching farmers at
/// most, newest first. A text so short that more farmers match it is not
/// narrowing anything down.
const MAX_MATCHED_FARMERS: u64 = 2_000;

/// Finds the farmers behind messages by asking the farmers feature through
/// its own repository port. Only the id, the name and the phone cross over.
#[derive(Debug)]
pub struct FarmersFeatureSenderDirectory {
    farmers: Arc<dyn FarmerRepository>,
}

impl FarmersFeatureSenderDirectory {
    pub fn new(farmers: Arc<dyn FarmerRepository>) -> Self {
        Self { farmers }
    }
}

fn failed(error: impl std::fmt::Display, doing: &str) -> AppError {
    tracing::error!(%error, "{doing} for messages failed");

    GlobalAppError::InternalServerError.into()
}

fn contact_of(farmer: &Farmer) -> Option<FarmerContact> {
    Some(FarmerContact::rehydrate(
        (*farmer.id())?,
        farmer.name().as_ref().map(Into::into),
        farmer.phone().clone(),
    ))
}

#[async_trait]
impl SenderDirectory for FarmersFeatureSenderDirectory {
    async fn farmer_id_of(&self, phone: &Phone) -> Result<Option<i32>, AppError> {
        let farmer = self
            .farmers
            .find_by_phone(phone)
            .await
            .map_err(|error| failed(error, "looking up a farmer by phone"))?;

        Ok(farmer.and_then(|farmer| *farmer.id()))
    }

    async fn contacts_of(&self, farmer_ids: &[i32]) -> Result<Vec<FarmerContact>, AppError> {
        if farmer_ids.is_empty() {
            return Ok(Vec::new());
        }

        let farmers = self
            .farmers
            .find_many(Some(farmer_ids), None, farmer_ids.len() as u64)
            .await
            .map_err(|error| failed(error, "reading farmers' names and phones"))?;

        Ok(farmers.iter().filter_map(contact_of).collect())
    }

    async fn ids_matching(&self, search: &SearchText) -> Result<Vec<i32>, AppError> {
        let farmers = self
            .farmers
            .find_many(None, Some(search.as_str()), MAX_MATCHED_FARMERS)
            .await
            .map_err(|error| failed(error, "searching farmers by name or phone"))?;

        Ok(farmers.iter().filter_map(|farmer| *farmer.id()).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::farmers::app::testing::{Call, Fakes, PHONE, phone};

    fn search(text: &str) -> SearchText {
        SearchText::new(text.to_string()).expect("search")
    }

    #[tokio::test]
    async fn a_phone_the_farmers_feature_knows_has_an_id() {
        let farmers = Fakes::new().with_farmer();
        let directory = FarmersFeatureSenderDirectory::new(Arc::new(farmers.clone()));

        assert_eq!(directory.farmer_id_of(&phone()).await.expect("id"), Some(1));
        assert_eq!(
            farmers.calls(),
            vec![Call::FindFarmer {
                phone: PHONE.to_string()
            }]
        );
    }

    #[tokio::test]
    async fn a_phone_the_farmers_feature_does_not_know_has_none() {
        let directory = FarmersFeatureSenderDirectory::new(Arc::new(Fakes::new()));

        assert_eq!(directory.farmer_id_of(&phone()).await.expect("id"), None);
    }

    #[tokio::test]
    async fn the_contact_of_a_farmer_is_their_id_name_and_phone() {
        let farmers = Fakes::new().with_farmer();
        let directory = FarmersFeatureSenderDirectory::new(Arc::new(farmers.clone()));

        let contacts = directory.contacts_of(&[1, 2]).await.expect("contacts");

        assert_eq!(
            contacts,
            vec![FarmerContact::rehydrate(1, None, phone())],
            "the farmer with id 2 is gone and is simply absent"
        );
        assert_eq!(
            farmers.calls(),
            vec![Call::FindManyFarmers {
                ids: Some(vec![1, 2]),
                matching: None
            }]
        );
    }

    #[tokio::test]
    async fn no_ids_means_no_call() {
        let farmers = Fakes::new().with_farmer();
        let directory = FarmersFeatureSenderDirectory::new(Arc::new(farmers.clone()));

        assert!(directory.contacts_of(&[]).await.expect("none").is_empty());
        assert!(farmers.calls().is_empty());
    }

    #[tokio::test]
    async fn a_search_returns_the_ids_of_the_farmers_it_matched() {
        let farmers = Fakes::new().with_farmer();
        let directory = FarmersFeatureSenderDirectory::new(Arc::new(farmers.clone()));

        assert_eq!(
            directory.ids_matching(&search("7501")).await.expect("ids"),
            vec![1]
        );
        assert!(
            directory
                .ids_matching(&search("nobody"))
                .await
                .expect("ids")
                .is_empty()
        );
        assert_eq!(
            farmers.calls()[0],
            Call::FindManyFarmers {
                ids: None,
                matching: Some("7501".to_string())
            }
        );
    }

    #[tokio::test]
    async fn a_failure_in_the_farmers_feature_is_an_error_not_an_empty_answer() {
        let directory =
            FarmersFeatureSenderDirectory::new(Arc::new(Fakes::new().failing_to_read_farmers()));

        assert!(directory.farmer_id_of(&phone()).await.is_err());
        assert!(directory.contacts_of(&[1]).await.is_err());
        assert!(directory.ids_matching(&search("a")).await.is_err());
    }
}
