use std::collections::{BTreeSet, HashMap};

use crate::features::messages::{
    app::{AppError, MessageFarmDirectory, SenderDirectory},
    domain::{FarmCard, FarmerContact, Message},
};

/// A message as Ministry staff see one: the stored message, who sent it and
/// which farm it is about. The farmer comes from the farmers feature and the
/// farm from the farms feature, as they are now: `None` when staff removed
/// the farmer, or when the farm is gone or was never named.
#[derive(Clone, Debug)]
pub struct MessageRecord {
    pub message: Message,
    pub farmer: Option<FarmerContact>,
    pub farm: Option<FarmCard>,
}

/// Looks up the farmers and farms of a page of messages, each with one call
/// for the whole page.
pub async fn describe(
    farmers: &dyn SenderDirectory,
    farms: &dyn MessageFarmDirectory,
    messages: Vec<Message>,
) -> Result<Vec<MessageRecord>, AppError> {
    if messages.is_empty() {
        return Ok(Vec::new());
    }

    let farmer_ids: Vec<i32> = messages
        .iter()
        .map(|message| *message.farmer_id())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    let farm_ids: Vec<i32> = messages
        .iter()
        .filter_map(|message| *message.farm_id())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();

    let contacts: HashMap<i32, FarmerContact> = farmers
        .contacts_of(&farmer_ids)
        .await?
        .into_iter()
        .map(|contact| (*contact.id(), contact))
        .collect();

    let cards: HashMap<i32, FarmCard> = if farm_ids.is_empty() {
        HashMap::new()
    } else {
        farms
            .cards_of(&farm_ids)
            .await?
            .into_iter()
            .map(|card| (*card.id(), card))
            .collect()
    };

    Ok(messages
        .into_iter()
        .map(|message| MessageRecord {
            farmer: contacts.get(message.farmer_id()).cloned(),
            farm: message.farm_id().and_then(|id| cards.get(&id).cloned()),
            message,
        })
        .collect())
}
