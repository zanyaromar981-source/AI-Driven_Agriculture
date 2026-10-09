use std::sync::Arc;

use crate::{
    app::Pagination,
    features::farmers::app::{AppError, FarmCounter, FarmerFilter, FarmerRecord, FarmerRepository},
};

pub struct ListFarmersInput {
    pub filter: FarmerFilter,
    pub pagination: Pagination,
}

pub struct ListFarmersUseCase {
    farmers: Arc<dyn FarmerRepository>,
    farms: Arc<dyn FarmCounter>,
}

impl ListFarmersUseCase {
    pub fn new(farmers: Arc<dyn FarmerRepository>, farms: Arc<dyn FarmCounter>) -> Self {
        Self { farmers, farms }
    }

    /// Returns the page, in the filter's order, and how many farmers match
    /// in all.
    pub async fn execute(
        &self,
        input: ListFarmersInput,
    ) -> Result<(Vec<FarmerRecord>, u64), AppError> {
        let (farmers, count) = self
            .farmers
            .find_page(&input.filter, &input.pagination)
            .await?;

        // A page holds at most 100 farmers, so this is a bounded number of
        // small counts through the farms feature's port.
        let mut records = Vec::with_capacity(farmers.len());

        for farmer in farmers {
            let farms_count = self.farms.count_for(farmer.phone()).await?;

            records.push(FarmerRecord {
                farmer,
                farms_count,
            });
        }

        tracing::debug!(returned = records.len(), count, "farmers listed for staff");

        Ok((records, count))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::farmers::app::testing::{Call, Fakes, PHONE, phone};

    fn use_case(fakes: &Fakes) -> ListFarmersUseCase {
        ListFarmersUseCase::new(Arc::new(fakes.clone()), Arc::new(fakes.clone()))
    }

    #[tokio::test]
    async fn lists_farmers_with_how_many_farms_each_has() {
        let fakes = Fakes::new().with_farmer();

        let (records, count) = use_case(&fakes)
            .execute(ListFarmersInput {
                filter: FarmerFilter::default(),
                pagination: Pagination::new(2, 20),
            })
            .await
            .expect("listing");

        assert_eq!(count, 1);
        assert_eq!(records[0].farms_count, 2);
        assert_eq!(
            fakes.calls(),
            vec![
                Call::FindFarmersPage {
                    phone: None,
                    page: 2,
                },
                Call::CountFarms {
                    phone: PHONE.to_string()
                },
            ]
        );
    }

    #[tokio::test]
    async fn a_phone_narrows_the_listing_to_that_farmer() {
        let fakes = Fakes::new().with_farmer();

        use_case(&fakes)
            .execute(ListFarmersInput {
                filter: FarmerFilter {
                    phone: Some(phone()),
                    ..FarmerFilter::default()
                },
                pagination: Pagination::new(1, 20),
            })
            .await
            .expect("listing");

        assert_eq!(
            fakes.calls()[0],
            Call::FindFarmersPage {
                phone: Some(PHONE.to_string()),
                page: 1,
            }
        );
    }

    #[tokio::test]
    async fn an_empty_page_counts_no_farms() {
        let fakes = Fakes::new();

        let (records, count) = use_case(&fakes)
            .execute(ListFarmersInput {
                filter: FarmerFilter::default(),
                pagination: Pagination::new(1, 20),
            })
            .await
            .expect("listing");

        assert!(records.is_empty());
        assert_eq!(count, 0);
        assert_eq!(fakes.calls().len(), 1);
    }
}
