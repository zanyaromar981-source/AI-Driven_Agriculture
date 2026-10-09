use sea_orm::ActiveValue::{NotSet, Set};
use sha2::{Digest, Sha256};

use crate::features::alerts::{
    app::AppError,
    domain::{
        Alert, AlertConfidence, AlertKey, AlertLevel, AlertSource, AlertText, AlertType, Device,
        DeviceLanguage, Platform, PushToken,
    },
    infra::persistence::postgres::entities::{alerts, devices},
};
use crate::shared::Phone;

/// The unique key of a device row: the token itself can be too long for an
/// index entry.
pub fn token_hash(push_token: &PushToken) -> String {
    hex::encode(Sha256::digest(push_token.as_str().as_bytes()))
}

impl TryFrom<alerts::Model> for Alert {
    type Error = AppError;

    fn try_from(model: alerts::Model) -> Result<Self, Self::Error> {
        Ok(Alert::rehydrate(
            model.id,
            model.farm_id,
            AlertKey::new(model.key)?,
            AlertType::try_from(model.r#type.as_str())?,
            model.day,
            AlertLevel::try_from(model.level.as_str())?,
            AlertConfidence::try_from(model.confidence.as_str())?,
            AlertText::new(model.ku)?,
            AlertText::new(model.en)?,
            AlertText::new(model.action_ku)?,
            AlertText::new(model.action_en)?,
            model.pushed,
            model.pushed_at.map(|at| at.and_utc()),
            model.done,
            model.done_at.map(|at| at.and_utc()),
            AlertSource::new(model.source)?,
            model.created_at.and_utc(),
            model.updated_at.and_utc(),
        ))
    }
}

impl From<&Alert> for alerts::ActiveModel {
    fn from(entity: &Alert) -> Self {
        Self {
            id: entity.id().map_or(NotSet, Set),
            farm_id: Set(*entity.farm_id()),
            key: Set(entity.key().into()),
            r#type: Set(String::from(*entity.alert_type())),
            day: Set(*entity.day()),
            level: Set(String::from(*entity.level())),
            confidence: Set(String::from(*entity.confidence())),
            ku: Set(entity.ku().into()),
            en: Set(entity.en().into()),
            action_ku: Set(entity.action_ku().into()),
            action_en: Set(entity.action_en().into()),
            pushed: Set(*entity.pushed()),
            pushed_at: Set(entity.pushed_at().map(|at| at.naive_utc())),
            done: Set(*entity.done()),
            done_at: Set(entity.done_at().map(|at| at.naive_utc())),
            source: Set(entity.source().into()),
            created_at: Set(entity.created_at().naive_utc()),
            updated_at: Set(entity.updated_at().naive_utc()),
        }
    }
}

impl TryFrom<devices::Model> for Device {
    type Error = AppError;

    fn try_from(model: devices::Model) -> Result<Self, Self::Error> {
        Ok(Device::new(
            PushToken::new(model.push_token)?,
            Phone::new(model.phone)?,
            Platform::try_from(model.platform.as_str())?,
            DeviceLanguage::try_from(model.lang.as_str())?,
            model.red_alerts,
            model.weekly_plan,
            model.updated_at.and_utc(),
        ))
    }
}

impl From<&Device> for devices::ActiveModel {
    fn from(entity: &Device) -> Self {
        Self {
            id: NotSet,
            token_hash: Set(token_hash(entity.push_token())),
            push_token: Set(entity.push_token().into()),
            phone: Set(entity.phone().into()),
            platform: Set(String::from(*entity.platform())),
            lang: Set(String::from(*entity.lang())),
            red_alerts: Set(*entity.red_alerts()),
            weekly_plan: Set(*entity.weekly_plan()),
            created_at: Set(entity.updated_at().naive_utc()),
            updated_at: Set(entity.updated_at().naive_utc()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_same_token_always_hashes_to_the_same_64_characters() {
        let token = PushToken::new("a".repeat(4096)).expect("token");

        assert_eq!(token_hash(&token), token_hash(&token.clone()));
        assert_eq!(token_hash(&token).len(), 64);
        assert_ne!(
            token_hash(&token),
            token_hash(&PushToken::new("b".to_string()).expect("token"))
        );
    }
}
