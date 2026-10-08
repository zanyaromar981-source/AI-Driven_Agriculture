use sea_orm::ActiveValue::{NotSet, Set};

use crate::{
    features::farmers::{
        app::AppError,
        domain::{Farmer, FarmerName, Language, SignInChallenge},
        infra::persistence::postgres::entities::{farmers, sign_in_challenges},
    },
    shared::Phone,
};

impl TryFrom<farmers::Model> for Farmer {
    type Error = AppError;

    fn try_from(model: farmers::Model) -> Result<Self, Self::Error> {
        Ok(Farmer::rehydrate(
            model.id,
            Phone::new(model.phone)?,
            model.name.map(FarmerName::new).transpose()?,
            Language::try_from(model.language.as_str())?,
            model.created_at.and_utc(),
            model.updated_at.and_utc(),
        ))
    }
}

impl From<&Farmer> for farmers::ActiveModel {
    fn from(farmer: &Farmer) -> Self {
        farmers::ActiveModel {
            id: match *farmer.id() {
                Some(id) => Set(id),
                None => NotSet,
            },
            phone: Set(farmer.phone().into()),
            name: Set(farmer.name().as_ref().map(Into::into)),
            language: Set((*farmer.language()).into()),
            created_at: Set(farmer.created_at().naive_utc()),
            updated_at: Set(farmer.updated_at().naive_utc()),
        }
    }
}

impl TryFrom<sign_in_challenges::Model> for SignInChallenge {
    type Error = AppError;

    fn try_from(model: sign_in_challenges::Model) -> Result<Self, Self::Error> {
        Ok(SignInChallenge::rehydrate(
            Phone::new(model.phone)?,
            model.code_hash,
            Language::try_from(model.language.as_str())?,
            u32::try_from(model.attempts).unwrap_or(u32::MAX),
            model.sent_at.and_utc(),
            model.expires_at.and_utc(),
        ))
    }
}

impl From<&SignInChallenge> for sign_in_challenges::ActiveModel {
    fn from(challenge: &SignInChallenge) -> Self {
        sign_in_challenges::ActiveModel {
            phone: Set(challenge.phone().into()),
            code_hash: Set(challenge.code_hash().clone()),
            language: Set((*challenge.language()).into()),
            attempts: Set(i32::try_from(*challenge.attempts()).unwrap_or(i32::MAX)),
            sent_at: Set(challenge.sent_at().naive_utc()),
            expires_at: Set(challenge.expires_at().naive_utc()),
        }
    }
}
