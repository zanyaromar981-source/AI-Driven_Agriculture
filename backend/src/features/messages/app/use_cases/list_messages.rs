use std::sync::Arc;

use crate::{
    app::Pagination,
    features::messages::{
        app::{
            AppError, MessageFarmDirectory, MessageFilter, MessageRecord, MessageRepository,
            MessageSearch, SenderDirectory, describe,
        },
        domain::{MessageKind, MessageState, SearchText},
    },
};

pub struct ListMessagesInput {
    pub state: Option<MessageState>,
    pub kind: Option<MessageKind>,
    pub governorate: Option<String>,
    pub zone_slug: Option<String>,
    pub search: Option<SearchText>,
    pub pagination: Pagination,
}

pub struct ListMessagesUseCase {
    repository: Arc<dyn MessageRepository>,
    farmers: Arc<dyn SenderDirectory>,
    farms: Arc<dyn MessageFarmDirectory>,
}

impl ListMessagesUseCase {
    pub fn new(
        repository: Arc<dyn MessageRepository>,
        farmers: Arc<dyn SenderDirectory>,
        farms: Arc<dyn MessageFarmDirectory>,
    ) -> Self {
        Self {
            repository,
            farmers,
            farms,
        }
    }

    /// Returns one page of every farmer's messages, newest first, each with
    /// its farmer and farm, and how many match in all.
    pub async fn execute(
        &self,
        input: ListMessagesInput,
    ) -> Result<(Vec<MessageRecord>, u64), AppError> {
        // Where a farm is, only the farms feature knows: it names the farms
        // in the place, and the messages are then filtered by farm.
        let farm_ids = if input.governorate.is_some() || input.zone_slug.is_some() {
            let farm_ids = self
                .farms
                .ids_in_place(input.governorate.as_deref(), input.zone_slug.as_deref())
                .await?;

            if farm_ids.is_empty() {
                tracing::debug!("no farm is known in that place: no message can match");

                return Ok((Vec::new(), 0));
            }

            Some(farm_ids)
        } else {
            None
        };

        // Likewise for names and phones: the farmers feature names the
        // farmers the text matches.
        let search = match input.search {
            Some(text) => Some(MessageSearch {
                farmer_ids: self.farmers.ids_matching(&text).await?,
                text,
            }),
            None => None,
        };

        let filter = MessageFilter {
            state: input.state,
            kind: input.kind,
            farm_ids,
            search,
        };

        let (messages, count) = self
            .repository
            .find_page(&filter, &input.pagination)
            .await?;

        let records = describe(self.farmers.as_ref(), self.farms.as_ref(), messages).await?;

        tracing::debug!(
            page = *input.pagination.page(),
            listed = records.len(),
            count,
            "messages listed for staff"
        );

        Ok((records, count))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::messages::app::testing::{
        Call, FARM_ID, FARMER_ID, Fakes, a_card, a_contact, a_message, a_message_of,
    };

    fn use_case(fakes: &Fakes) -> ListMessagesUseCase {
        ListMessagesUseCase::new(
            Arc::new(fakes.clone()),
            Arc::new(fakes.clone()),
            Arc::new(fakes.clone()),
        )
    }

    fn input() -> ListMessagesInput {
        ListMessagesInput {
            state: None,
            kind: None,
            governorate: None,
            zone_slug: None,
            search: None,
            pagination: Pagination::new(1, 20),
        }
    }

    #[tokio::test]
    async fn every_message_comes_with_its_farmer_and_its_farm() {
        let fakes = Fakes::new()
            .with_stored(a_message(1))
            .with_contact(a_contact())
            .with_card(a_card());

        let (records, count) = use_case(&fakes).execute(input()).await.expect("page");

        assert_eq!(count, 1);
        assert_eq!(records[0].farmer, Some(a_contact()));
        assert_eq!(records[0].farm, Some(a_card()));
        assert_eq!(
            fakes.calls(),
            vec![
                Call::FindPage {
                    filter: MessageFilter::default(),
                    page: 1
                },
                Call::ContactsOf {
                    farmer_ids: vec![FARMER_ID]
                },
                Call::CardsOf {
                    farm_ids: vec![FARM_ID]
                },
            ],
            "one call per feature for the whole page"
        );
    }

    #[tokio::test]
    async fn a_removed_farmer_and_a_removed_farm_are_absent_not_an_error() {
        let fakes = Fakes::new().with_stored(a_message(1));

        let (records, _) = use_case(&fakes).execute(input()).await.expect("page");

        assert_eq!(records.len(), 1, "the message stays for the record");
        assert!(records[0].farmer.is_none());
        assert!(records[0].farm.is_none());
    }

    #[tokio::test]
    async fn several_messages_of_one_farmer_ask_for_that_farmer_once() {
        let fakes = Fakes::new()
            .with_stored(a_message(1))
            .with_stored(a_message(2));

        use_case(&fakes).execute(input()).await.expect("page");

        assert!(fakes.calls().contains(&Call::ContactsOf {
            farmer_ids: vec![FARMER_ID]
        }));
    }

    #[tokio::test]
    async fn the_state_and_kind_filters_reach_the_repository() {
        let fakes = Fakes::new()
            .with_stored(a_message(1))
            .with_stored(a_message_of(2, FARMER_ID, MessageState::Closed));

        let (records, count) = use_case(&fakes)
            .execute(ListMessagesInput {
                state: Some(MessageState::Closed),
                kind: Some(MessageKind::Report),
                ..input()
            })
            .await
            .expect("page");

        assert_eq!(count, 1);
        assert_eq!(*records[0].message.id(), Some(2));
    }

    #[tokio::test]
    async fn a_search_also_looks_among_the_farmers_the_farmers_feature_matched() {
        let fakes = Fakes::new().with_farmers_matching(vec![FARMER_ID, 8]);
        let text = SearchText::new("azad".to_string()).expect("search");

        use_case(&fakes)
            .execute(ListMessagesInput {
                search: Some(text.clone()),
                ..input()
            })
            .await
            .expect("page");

        assert_eq!(
            fakes.calls(),
            vec![
                Call::IdsMatching {
                    text: "azad".to_string()
                },
                Call::FindPage {
                    filter: MessageFilter {
                        search: Some(MessageSearch {
                            text,
                            farmer_ids: vec![FARMER_ID, 8]
                        }),
                        ..MessageFilter::default()
                    },
                    page: 1
                },
            ]
        );
    }

    #[tokio::test]
    async fn a_place_filter_becomes_the_farms_the_farms_feature_names() {
        let fakes = Fakes::new().with_farms_in_place(vec![FARM_ID]);

        use_case(&fakes)
            .execute(ListMessagesInput {
                governorate: Some("sulaymaniyah".to_string()),
                zone_slug: Some("chamchamal".to_string()),
                ..input()
            })
            .await
            .expect("page");

        assert_eq!(
            fakes.calls(),
            vec![
                Call::IdsInPlace {
                    governorate: Some("sulaymaniyah".to_string()),
                    zone_slug: Some("chamchamal".to_string())
                },
                Call::FindPage {
                    filter: MessageFilter {
                        farm_ids: Some(vec![FARM_ID]),
                        ..MessageFilter::default()
                    },
                    page: 1
                },
            ]
        );
    }

    #[tokio::test]
    async fn a_place_with_no_known_farm_matches_no_message_and_reads_none() {
        let fakes = Fakes::new().with_stored(a_message(1));

        let (records, count) = use_case(&fakes)
            .execute(ListMessagesInput {
                zone_slug: Some("chamchamal".to_string()),
                ..input()
            })
            .await
            .expect("page");

        assert!(records.is_empty());
        assert_eq!(count, 0);
        assert!(
            !fakes
                .calls()
                .iter()
                .any(|call| matches!(call, Call::FindPage { .. })),
            "a filter that cannot match must not fall back to listing everything"
        );
    }

    #[tokio::test]
    async fn a_repository_failure_surfaces() {
        assert!(
            use_case(&Fakes::new().failing())
                .execute(input())
                .await
                .is_err()
        );
    }
}
