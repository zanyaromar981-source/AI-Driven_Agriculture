use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

use crate::{
    features::farmers::{
        app::{
            AppError,
            use_cases::{
                EditProfileInput, RequestSignInCodeInput, SignInCodeRequested, SignedIn,
                VerifySignInCodeInput,
            },
        },
        domain::{self, Farmer, FarmerName, SignInCode},
    },
    shared::Phone,
};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Default, ToSchema)]
pub enum Language {
    /// Sorani
    #[default]
    #[serde(rename = "ku")]
    Sorani,
    /// Kurmanji
    #[serde(rename = "kmr")]
    Kurmanji,
    #[serde(rename = "ar")]
    Arabic,
    #[serde(rename = "en")]
    English,
}

impl From<Language> for domain::Language {
    fn from(value: Language) -> Self {
        match value {
            Language::Sorani => domain::Language::Sorani,
            Language::Kurmanji => domain::Language::Kurmanji,
            Language::Arabic => domain::Language::Arabic,
            Language::English => domain::Language::English,
        }
    }
}

impl From<domain::Language> for Language {
    fn from(value: domain::Language) -> Self {
        match value {
            domain::Language::Sorani => Language::Sorani,
            domain::Language::Kurmanji => Language::Kurmanji,
            domain::Language::Arabic => Language::Arabic,
            domain::Language::English => Language::English,
        }
    }
}

#[derive(Serialize, Deserialize, Validate, Debug, Clone, ToSchema)]
pub struct SendSignInCodeParams {
    /// E.164 Iraqi mobile number, for example `+9647501234567`.
    pub phone: String,
    #[serde(default)]
    pub lang: Language,
}

impl SendSignInCodeParams {
    pub fn into_input(self) -> Result<RequestSignInCodeInput, AppError> {
        Ok(RequestSignInCodeInput {
            phone: Phone::new(self.phone)?,
            language: self.lang.into(),
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct SignInCodeSentResponse {
    pub sent: bool,
    pub retry_after_s: u64,
}

impl From<SignInCodeRequested> for SignInCodeSentResponse {
    fn from(requested: SignInCodeRequested) -> Self {
        Self {
            sent: true,
            retry_after_s: requested.retry_after_s,
        }
    }
}

#[derive(Serialize, Deserialize, Validate, Debug, Clone, ToSchema)]
pub struct VerifySignInCodeParams {
    pub phone: String,
    pub code: String,
}

impl VerifySignInCodeParams {
    pub fn into_input(self) -> Result<VerifySignInCodeInput, AppError> {
        Ok(VerifySignInCodeInput {
            phone: Phone::new(self.phone)?,
            code: SignInCode::new(self.code)?,
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct SignedInResponse {
    pub token: String,
    pub farms_count: u64,
}

impl From<SignedIn> for SignedInResponse {
    fn from(signed_in: SignedIn) -> Self {
        Self {
            token: signed_in.token,
            farms_count: signed_in.farms_count,
        }
    }
}

#[derive(Serialize, Deserialize, Validate, Debug, Clone, ToSchema)]
pub struct EditProfileParams {
    /// Shown to buyers. Send `null` to clear it.
    pub name: Option<String>,
    pub lang: Language,
}

impl EditProfileParams {
    pub fn into_input(self) -> Result<EditProfileInput, AppError> {
        Ok(EditProfileInput {
            name: self.name.map(FarmerName::new).transpose()?,
            language: self.lang.into(),
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct ProfileResponse {
    pub phone: String,
    pub name: Option<String>,
    pub lang: Language,
    pub created_at: DateTime<Utc>,
}

impl From<&Farmer> for ProfileResponse {
    fn from(farmer: &Farmer) -> Self {
        Self {
            phone: farmer.phone().into(),
            name: farmer.name().as_ref().map(Into::into),
            lang: (*farmer.language()).into(),
            created_at: *farmer.created_at(),
        }
    }
}
