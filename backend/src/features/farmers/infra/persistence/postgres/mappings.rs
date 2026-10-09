use sea_orm::ActiveValue::{NotSet, Set};

use crate::{
    features::farmers::{
        app::AppError,
        domain::{
            BirthYear, Farmer, FarmerDetails, FarmerName, FarmerNotes, Gender, Language, Letter,
            LetterLanguage, LetterNumber, LetterPurpose, PlaceSlug, SignInChallenge, Village,
        },
        infra::persistence::postgres::entities::{farmers, letters, sign_in_challenges},
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
            FarmerDetails {
                gender: model.gender.as_deref().map(Gender::try_from).transpose()?,
                birth_year: model.birth_year.map(BirthYear::rehydrate),
                village: model.village.map(Village::new).transpose()?,
                governorate: model.governorate.map(PlaceSlug::new).transpose()?,
                zone_slug: model.zone_slug.map(PlaceSlug::new).transpose()?,
                sub_zone_slug: model.sub_zone_slug.map(PlaceSlug::new).transpose()?,
                notes: model.notes.map(FarmerNotes::new).transpose()?,
            },
            model.blocked,
            model.created_at.and_utc(),
            model.updated_at.and_utc(),
        ))
    }
}

/// The details staff record and the blocked state are never written from
/// here. A new farmer takes the column defaults (nothing known, not
/// blocked), and the farmer's own profile edit leaves them alone: writing
/// them back from a copy read a moment earlier would undo a block that staff
/// set in between. Staff write them through `update_by_id`.
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
            gender: NotSet,
            birth_year: NotSet,
            village: NotSet,
            governorate: NotSet,
            zone_slug: NotSet,
            sub_zone_slug: NotSet,
            notes: NotSet,
            blocked: NotSet,
        }
    }
}

impl TryFrom<letters::Model> for Letter {
    type Error = AppError;

    fn try_from(model: letters::Model) -> Result<Self, Self::Error> {
        Ok(Letter::rehydrate(
            model.id,
            LetterNumber::new(model.number)?,
            model.farmer_id,
            model.staff_id,
            LetterPurpose::new(model.purpose)?,
            LetterLanguage::try_from(model.lang.as_str())?,
            model.created_at.and_utc(),
        ))
    }
}

impl From<&Letter> for letters::ActiveModel {
    fn from(letter: &Letter) -> Self {
        letters::ActiveModel {
            id: match *letter.id() {
                Some(id) => Set(id),
                None => NotSet,
            },
            number: Set(letter.number().into()),
            farmer_id: Set(*letter.farmer_id()),
            staff_id: Set(*letter.staff_id()),
            purpose: Set(letter.purpose().into()),
            lang: Set((*letter.language()).into()),
            created_at: Set(letter.created_at().naive_utc()),
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
            model.used_at.map(|at| at.and_utc()),
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
            used_at: Set(challenge.used_at().map(|at| at.naive_utc())),
        }
    }
}
