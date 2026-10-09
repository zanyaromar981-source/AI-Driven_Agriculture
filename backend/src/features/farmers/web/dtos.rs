use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use validator::Validate;

use crate::{
    app::{AppError as GlobalAppError, Pagination},
    features::farmers::{
        app::{
            AppError, FarmerRecord,
            use_cases::{
                EditFarmerInput, EditProfileInput, ListFarmersInput, RegisterFarmerInput,
                RequestSignInCodeInput, SignInCodeRequested, SignedIn, VerifySignInCodeInput,
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

#[derive(Deserialize, Debug, Clone, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct DashboardFarmersQuery {
    /// Only the farmer with exactly this phone, for example
    /// `+9647501234567`.
    pub phone: Option<String>,
}

impl DashboardFarmersQuery {
    pub fn into_input(self, pagination: Pagination) -> Result<ListFarmersInput, AppError> {
        Ok(ListFarmersInput {
            phone: self
                .phone
                .filter(|phone| !phone.is_empty())
                .map(Phone::new)
                .transpose()?,
            pagination,
        })
    }
}

#[derive(Serialize, Deserialize, Validate, Debug, Clone, ToSchema)]
pub struct DashboardCreateFarmerParams {
    /// E.164 Iraqi mobile number, for example `+9647501234567`.
    pub phone: String,
    pub name: Option<String>,
    pub lang: Language,
}

impl DashboardCreateFarmerParams {
    pub fn into_input(self) -> Result<RegisterFarmerInput, AppError> {
        Ok(RegisterFarmerInput {
            phone: Phone::new(self.phone)?,
            name: self.name.map(FarmerName::new).transpose()?,
            language: self.lang.into(),
        })
    }
}

/// There is no phone here: the phone is the account and cannot change.
#[derive(Serialize, Deserialize, Validate, Debug, Clone, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct DashboardUpdateFarmerParams {
    /// Send `null`, or leave it out, to clear it.
    pub name: Option<String>,
    pub lang: Language,
}

impl DashboardUpdateFarmerParams {
    pub fn into_input(self) -> Result<EditFarmerInput, AppError> {
        Ok(EditFarmerInput {
            name: self.name.map(FarmerName::new).transpose()?,
            language: self.lang.into(),
        })
    }
}

/// A farmer as staff holding a `farmers` permission see one, phone included.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct DashboardFarmerResponse {
    pub id: String,
    pub phone: String,
    pub name: Option<String>,
    pub lang: Language,
    pub farms_count: u64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TryFrom<&FarmerRecord> for DashboardFarmerResponse {
    type Error = AppError;

    fn try_from(record: &FarmerRecord) -> Result<Self, Self::Error> {
        let farmer = &record.farmer;
        let id = farmer.id().ok_or_else(|| {
            AppError::GlobalAppError(GlobalAppError::MissingValue(
                "Farmer is missing its id".to_string(),
            ))
        })?;

        Ok(Self {
            id: id.to_string(),
            phone: farmer.phone().into(),
            name: farmer.name().as_ref().map(Into::into),
            lang: (*farmer.language()).into(),
            farms_count: record.farms_count,
            created_at: *farmer.created_at(),
            updated_at: *farmer.updated_at(),
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct DashboardOneFarmerResponse {
    pub farmer: DashboardFarmerResponse,
}

impl TryFrom<&FarmerRecord> for DashboardOneFarmerResponse {
    type Error = AppError;

    fn try_from(record: &FarmerRecord) -> Result<Self, Self::Error> {
        Ok(Self {
            farmer: DashboardFarmerResponse::try_from(record)?,
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct DashboardFarmersResponse {
    pub farmers: Vec<DashboardFarmerResponse>,
    /// How many farmers match in all, on every page.
    pub count: u64,
    pub page: u64,
    pub rows_per_page: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::farmers::app::testing::{PHONE, a_farmer};

    #[test]
    fn a_farmer_travels_with_a_string_id_and_the_farms_count() {
        let body = serde_json::to_value(
            DashboardFarmerResponse::try_from(&FarmerRecord {
                farmer: a_farmer(),
                farms_count: 2,
            })
            .expect("farmer"),
        )
        .expect("json");

        assert_eq!(body["id"], "1");
        assert_eq!(body["phone"], PHONE);
        assert_eq!(body["lang"], "ku");
        assert_eq!(body["farms_count"], 2);
        assert!(body["name"].is_null());
    }

    #[test]
    fn an_update_that_tries_to_change_the_phone_is_refused() {
        let with_phone = serde_json::from_value::<DashboardUpdateFarmerParams>(
            serde_json::json!({"name": null, "lang": "en", "phone": "+9647507654321"}),
        );

        assert!(
            with_phone.is_err(),
            "silently ignoring the phone would look like it had been changed"
        );
        assert!(
            serde_json::from_value::<DashboardUpdateFarmerParams>(
                serde_json::json!({"lang": "en"})
            )
            .is_ok()
        );
    }

    #[test]
    fn an_empty_phone_filter_is_no_filter_and_a_bad_one_is_refused() {
        let query = |phone: &str| DashboardFarmersQuery {
            phone: Some(phone.to_string()),
        };

        assert!(
            query("")
                .into_input(Pagination::new(1, 20))
                .expect("input")
                .phone
                .is_none()
        );
        assert!(query("0750").into_input(Pagination::new(1, 20)).is_err());
    }
}
