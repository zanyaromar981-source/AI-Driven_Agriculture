use chrono::{DateTime, Datelike, Utc};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use validator::Validate;

use crate::{
    app::{AppError as GlobalAppError, Pagination},
    features::farmers::{
        app::{
            AppError, FarmerFilter, FarmerRecord, FarmerSort, IssuedLetter, LetterRecord,
            use_cases::{
                EditProfileInput, IssueLetterInput, ListFarmersInput, RegisterFarmerInput,
                RequestSignInCodeInput, SignInCodeRequested, SignedIn, VerifySignInCodeInput,
            },
        },
        domain::{
            self, BirthYear, CropHolding, FarmHolding, Farmer, FarmerChange, FarmerDetails,
            FarmerName, FarmerNotes, FarmerSearch, LetterPurpose, LetterTotals, PlaceSlug,
            SignInCode, Village,
        },
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

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum FarmerGender {
    Male,
    Female,
}

impl From<FarmerGender> for domain::Gender {
    fn from(value: FarmerGender) -> Self {
        match value {
            FarmerGender::Male => domain::Gender::Male,
            FarmerGender::Female => domain::Gender::Female,
        }
    }
}

impl From<domain::Gender> for FarmerGender {
    fn from(value: domain::Gender) -> Self {
        match value {
            domain::Gender::Male => FarmerGender::Male,
            domain::Gender::Female => FarmerGender::Female,
        }
    }
}

/// The language a support letter is printed in.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, ToSchema)]
pub enum FarmerLetterLanguage {
    /// Sorani
    #[serde(rename = "ku")]
    Sorani,
    #[serde(rename = "en")]
    English,
}

impl From<FarmerLetterLanguage> for domain::LetterLanguage {
    fn from(value: FarmerLetterLanguage) -> Self {
        match value {
            FarmerLetterLanguage::Sorani => domain::LetterLanguage::Sorani,
            FarmerLetterLanguage::English => domain::LetterLanguage::English,
        }
    }
}

impl From<domain::LetterLanguage> for FarmerLetterLanguage {
    fn from(value: domain::LetterLanguage) -> Self {
        match value {
            domain::LetterLanguage::Sorani => FarmerLetterLanguage::Sorani,
            domain::LetterLanguage::English => FarmerLetterLanguage::English,
        }
    }
}

#[derive(Deserialize, Debug, Clone, Copy, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum DashboardFarmerSort {
    CreatedAt,
    Name,
}

#[derive(Deserialize, Debug, Clone, Copy, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum DashboardFarmerOrder {
    Asc,
    Desc,
}

#[derive(Deserialize, Debug, Clone, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct DashboardFarmersQuery {
    /// Only the farmer with exactly this phone, for example
    /// `+9647501234567`.
    pub phone: Option<String>,
    /// Only farmers whose name or phone contains this text, whatever the
    /// case. Up to 80 characters.
    pub q: Option<String>,
    /// Only farmers whose home governorate has this slug.
    pub governorate: Option<String>,
    /// Only farmers whose home zone has this slug.
    pub zone: Option<String>,
    /// Only blocked farmers (`true`) or only the others (`false`).
    pub blocked: Option<bool>,
    /// `created_at` when left out.
    #[param(inline)]
    pub sort: Option<DashboardFarmerSort>,
    /// When left out: `desc` for `created_at` (newest first), `asc` for
    /// `name`. Farmers without a name always come last.
    #[param(inline)]
    pub order: Option<DashboardFarmerOrder>,
}

impl DashboardFarmersQuery {
    pub fn into_input(self, pagination: Pagination) -> Result<ListFarmersInput, AppError> {
        Ok(ListFarmersInput {
            filter: FarmerFilter {
                phone: self
                    .phone
                    .filter(|phone| !phone.is_empty())
                    .map(Phone::new)
                    .transpose()?,
                search: FarmerSearch::optional(self.q)?,
                governorate: PlaceSlug::optional(self.governorate)?,
                zone: PlaceSlug::optional(self.zone)?,
                blocked: self.blocked,
                sort: match self.sort {
                    Some(DashboardFarmerSort::Name) => FarmerSort::Name,
                    Some(DashboardFarmerSort::CreatedAt) | None => FarmerSort::CreatedAt,
                },
                descending: self
                    .order
                    .map(|order| matches!(order, DashboardFarmerOrder::Desc)),
            },
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
/// Every field but `lang` and `blocked` is replaced: one that is `null`,
/// empty or left out is cleared.
#[derive(Serialize, Deserialize, Validate, Debug, Clone, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct DashboardUpdateFarmerParams {
    /// Send `null`, or leave it out, to clear it.
    pub name: Option<String>,
    pub lang: Language,
    pub gender: Option<FarmerGender>,
    /// From 1900 to the current year.
    pub birth_year: Option<i32>,
    /// Up to 80 characters.
    pub village: Option<String>,
    /// The slug of the governorate the farmer lives in.
    pub governorate: Option<String>,
    /// The slug of the zone the farmer lives in.
    pub zone_slug: Option<String>,
    /// The slug of the sub-zone the farmer lives in.
    pub sub_zone_slug: Option<String>,
    /// For staff only, up to 1,000 characters. The farmer never sees it.
    pub notes: Option<String>,
    /// `true` signs the farmer out at once and refuses sign-in; `false`
    /// lets them in again. Left out, the farmer stays as they are.
    pub blocked: Option<bool>,
}

impl DashboardUpdateFarmerParams {
    pub fn into_input(self) -> Result<FarmerChange, AppError> {
        let current_year = Utc::now().year();

        Ok(FarmerChange {
            name: self.name.map(FarmerName::new).transpose()?,
            language: self.lang.into(),
            details: FarmerDetails {
                gender: self.gender.map(Into::into),
                birth_year: self
                    .birth_year
                    .map(|year| BirthYear::new(year, current_year))
                    .transpose()?,
                village: Village::optional(self.village)?,
                governorate: PlaceSlug::optional(self.governorate)?,
                zone_slug: PlaceSlug::optional(self.zone_slug)?,
                sub_zone_slug: PlaceSlug::optional(self.sub_zone_slug)?,
                notes: FarmerNotes::optional(self.notes)?,
            },
            blocked: self.blocked,
        })
    }
}

/// A farmer as staff holding a `farmers` permission see one, phone and
/// staff notes included.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct DashboardFarmerResponse {
    pub id: String,
    pub phone: String,
    pub name: Option<String>,
    pub lang: Language,
    pub farms_count: u64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub gender: Option<FarmerGender>,
    pub birth_year: Option<i32>,
    pub village: Option<String>,
    pub governorate: Option<String>,
    pub zone_slug: Option<String>,
    pub sub_zone_slug: Option<String>,
    pub notes: Option<String>,
    pub blocked: bool,
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
            gender: farmer.details().gender.map(Into::into),
            birth_year: farmer.details().birth_year.map(|year| year.value()),
            village: farmer.details().village.as_ref().map(Into::into),
            governorate: farmer.details().governorate.as_ref().map(Into::into),
            zone_slug: farmer.details().zone_slug.as_ref().map(Into::into),
            sub_zone_slug: farmer.details().sub_zone_slug.as_ref().map(Into::into),
            notes: farmer.details().notes.as_ref().map(Into::into),
            blocked: *farmer.blocked(),
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

#[derive(Serialize, Deserialize, Validate, Debug, Clone, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct DashboardIssueLetterParams {
    /// What the letter is for, 3 to 300 characters.
    pub purpose: String,
    pub lang: FarmerLetterLanguage,
}

impl DashboardIssueLetterParams {
    pub fn into_input(self) -> Result<IssueLetterInput, AppError> {
        Ok(IssueLetterInput {
            purpose: LetterPurpose::new(self.purpose)?,
            language: self.lang.into(),
        })
    }
}

/// The staff member who issued a letter.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct DashboardLetterIssuerResponse {
    pub id: String,
    pub name: String,
}

/// The farmer as a letter names them. Staff notes are not part of a letter.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct DashboardLetterFarmerResponse {
    pub id: String,
    pub name: Option<String>,
    pub phone: String,
    pub gender: Option<FarmerGender>,
    pub birth_year: Option<i32>,
    pub village: Option<String>,
    pub governorate: Option<String>,
    pub zone_slug: Option<String>,
    pub sub_zone_slug: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct DashboardLetterCropResponse {
    /// The crop's code, as the farms routes spell it.
    pub crop: String,
    pub dunam: f64,
}

impl From<&CropHolding> for DashboardLetterCropResponse {
    fn from(holding: &CropHolding) -> Self {
        Self {
            crop: holding.crop.clone(),
            dunam: holding.dunam,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct DashboardLetterFarmResponse {
    pub id: String,
    pub name: String,
    /// Where the farm is. `null` while the farm has no place.
    pub governorate: Option<String>,
    pub zone_slug: Option<String>,
    pub sub_zone_slug: Option<String>,
    pub area_dunam: f64,
    /// Largest area first. Unpainted land is not a crop.
    pub crops: Vec<DashboardLetterCropResponse>,
}

impl From<&FarmHolding> for DashboardLetterFarmResponse {
    fn from(farm: &FarmHolding) -> Self {
        Self {
            id: farm.id.to_string(),
            name: farm.name.clone(),
            governorate: farm.governorate.clone(),
            zone_slug: farm.zone_slug.clone(),
            sub_zone_slug: farm.sub_zone_slug.clone(),
            area_dunam: farm.area_dunam,
            crops: farm.crops.iter().map(Into::into).collect(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct DashboardLetterTotalsResponse {
    pub farms: u64,
    pub dunam: f64,
    /// Each crop over all farms, largest area first.
    pub crops: Vec<DashboardLetterCropResponse>,
}

impl From<&LetterTotals> for DashboardLetterTotalsResponse {
    fn from(totals: &LetterTotals) -> Self {
        Self {
            farms: totals.farms,
            dunam: totals.dunam,
            crops: totals.crops.iter().map(Into::into).collect(),
        }
    }
}

/// Everything a printed support letter shows.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct DashboardIssuedLetterResponse {
    /// `JTY-<yyyymm>-<farmer id>-<n>`: `n` counts this farmer's letters.
    pub number: String,
    pub purpose: String,
    pub lang: FarmerLetterLanguage,
    pub created_at: DateTime<Utc>,
    pub issued_by: DashboardLetterIssuerResponse,
    pub farmer: DashboardLetterFarmerResponse,
    pub farms: Vec<DashboardLetterFarmResponse>,
    pub totals: DashboardLetterTotalsResponse,
}

impl TryFrom<&IssuedLetter> for DashboardIssuedLetterResponse {
    type Error = AppError;

    fn try_from(issued: &IssuedLetter) -> Result<Self, Self::Error> {
        let farmer = &issued.farmer;
        let farmer_id = farmer.id().ok_or_else(|| {
            AppError::GlobalAppError(GlobalAppError::MissingValue(
                "Farmer is missing its id".to_string(),
            ))
        })?;

        Ok(Self {
            number: issued.letter.number().into(),
            purpose: issued.letter.purpose().into(),
            lang: (*issued.letter.language()).into(),
            created_at: *issued.letter.created_at(),
            issued_by: DashboardLetterIssuerResponse {
                id: issued.letter.staff_id().to_string(),
                name: issued.issued_by_name.clone(),
            },
            farmer: DashboardLetterFarmerResponse {
                id: farmer_id.to_string(),
                name: farmer.name().as_ref().map(Into::into),
                phone: farmer.phone().into(),
                gender: farmer.details().gender.map(Into::into),
                birth_year: farmer.details().birth_year.map(|year| year.value()),
                village: farmer.details().village.as_ref().map(Into::into),
                governorate: farmer.details().governorate.as_ref().map(Into::into),
                zone_slug: farmer.details().zone_slug.as_ref().map(Into::into),
                sub_zone_slug: farmer.details().sub_zone_slug.as_ref().map(Into::into),
            },
            farms: issued.farms.iter().map(Into::into).collect(),
            totals: (&issued.totals).into(),
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct DashboardOneIssuedLetterResponse {
    pub letter: DashboardIssuedLetterResponse,
}

/// A stored letter, for checking one later.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct DashboardLetterRecordResponse {
    pub number: String,
    pub farmer_id: String,
    /// The farmer's name as it is now. `null` when the farmer has no name
    /// or has been removed since.
    pub farmer_name: Option<String>,
    pub staff_id: String,
    pub purpose: String,
    pub lang: FarmerLetterLanguage,
    pub created_at: DateTime<Utc>,
}

impl From<&LetterRecord> for DashboardLetterRecordResponse {
    fn from(record: &LetterRecord) -> Self {
        Self {
            number: record.letter.number().into(),
            farmer_id: record.letter.farmer_id().to_string(),
            farmer_name: record.farmer_name.as_ref().map(Into::into),
            staff_id: record.letter.staff_id().to_string(),
            purpose: record.letter.purpose().into(),
            lang: (*record.letter.language()).into(),
            created_at: *record.letter.created_at(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct DashboardOneLetterRecordResponse {
    pub letter: DashboardLetterRecordResponse,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::farmers::app::testing::{PHONE, a_farmer, with_state};

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

    fn no_filters() -> DashboardFarmersQuery {
        DashboardFarmersQuery {
            phone: None,
            q: None,
            governorate: None,
            zone: None,
            blocked: None,
            sort: None,
            order: None,
        }
    }

    fn details_with_notes() -> FarmerDetails {
        FarmerDetails {
            gender: Some(domain::Gender::Female),
            birth_year: Some(BirthYear::rehydrate(1980)),
            village: Some(Village::new("Sangaw".to_string()).expect("village")),
            notes: Some(FarmerNotes::new("Came to the office twice".to_string()).expect("notes")),
            ..FarmerDetails::default()
        }
    }

    #[test]
    fn the_farmers_own_profile_never_carries_what_staff_recorded() {
        let farmer = with_state(&a_farmer(), Some(details_with_notes()), false);

        let body = serde_json::to_value(ProfileResponse::from(&farmer)).expect("json");
        let mut keys: Vec<&str> = body
            .as_object()
            .expect("object")
            .keys()
            .map(String::as_str)
            .collect();
        keys.sort_unstable();

        assert_eq!(
            keys,
            ["created_at", "lang", "name", "phone"],
            "notes, gender, birth year, home place and the blocked state are for staff"
        );
    }

    #[test]
    fn staff_see_the_details_the_notes_and_the_blocked_state() {
        let body = serde_json::to_value(
            DashboardFarmerResponse::try_from(&FarmerRecord {
                farmer: with_state(&a_farmer(), Some(details_with_notes()), true),
                farms_count: 0,
            })
            .expect("farmer"),
        )
        .expect("json");

        assert_eq!(body["gender"], "female");
        assert_eq!(body["birth_year"], 1980);
        assert_eq!(body["village"], "Sangaw");
        assert_eq!(body["notes"], "Came to the office twice");
        assert_eq!(body["blocked"], true);
        assert!(body["governorate"].is_null());
    }

    #[test]
    fn an_update_validates_every_detail() {
        let params = |extra: serde_json::Value| {
            let mut body = serde_json::json!({"lang": "ku"});
            body.as_object_mut()
                .expect("object")
                .extend(extra.as_object().expect("object").clone());

            serde_json::from_value::<DashboardUpdateFarmerParams>(body)
        };

        for bad in [
            serde_json::json!({"birth_year": 1899}),
            serde_json::json!({"birth_year": 3000}),
            serde_json::json!({"village": "x".repeat(81)}),
            serde_json::json!({"governorate": "Sulaymaniyah"}),
            serde_json::json!({"zone_slug": "zone 1"}),
            serde_json::json!({"sub_zone_slug": "a".repeat(41)}),
            serde_json::json!({"notes": "x".repeat(1001)}),
        ] {
            assert!(
                params(bad.clone()).expect("shape").into_input().is_err(),
                "{bad}"
            );
        }

        assert!(params(serde_json::json!({"gender": "other"})).is_err());

        let change = params(serde_json::json!({
            "gender": "male", "birth_year": 1975, "village": " Sangaw ",
            "governorate": "sulaymaniyah", "zone_slug": "chamchamal",
            "sub_zone_slug": "sangaw", "notes": "", "blocked": true
        }))
        .expect("shape")
        .into_input()
        .expect("valid");

        assert_eq!(change.details.gender, Some(domain::Gender::Male));
        assert_eq!(change.details.village.expect("village").as_str(), "Sangaw");
        assert!(change.details.notes.is_none(), "empty notes are no notes");
        assert_eq!(change.blocked, Some(true));
    }

    #[test]
    fn an_update_that_does_not_mention_blocking_leaves_it_alone() {
        let change = serde_json::from_value::<DashboardUpdateFarmerParams>(
            serde_json::json!({"lang": "ku"}),
        )
        .expect("shape")
        .into_input()
        .expect("valid");

        assert_eq!(change.blocked, None);
    }

    #[test]
    fn the_listing_filters_are_validated_and_empty_ones_ignored() {
        let input = DashboardFarmersQuery {
            q: Some(" hiwa ".to_string()),
            governorate: Some(String::new()),
            zone: Some("chamchamal".to_string()),
            blocked: Some(true),
            sort: Some(DashboardFarmerSort::Name),
            order: None,
            ..no_filters()
        }
        .into_input(Pagination::new(1, 20))
        .expect("input");

        assert_eq!(input.filter.search.expect("search").as_str(), "hiwa");
        assert!(input.filter.governorate.is_none());
        assert_eq!(input.filter.zone.expect("zone").as_str(), "chamchamal");
        assert_eq!(input.filter.blocked, Some(true));
        assert_eq!(input.filter.sort, FarmerSort::Name);

        for bad in [
            DashboardFarmersQuery {
                zone: Some("Zone 1".to_string()),
                ..no_filters()
            },
            DashboardFarmersQuery {
                q: Some("x".repeat(81)),
                ..no_filters()
            },
        ] {
            assert!(bad.into_input(Pagination::new(1, 20)).is_err());
        }
    }

    #[test]
    fn a_letter_needs_a_purpose_and_sorani_or_english() {
        let params =
            |body: serde_json::Value| serde_json::from_value::<DashboardIssueLetterParams>(body);

        assert!(
            params(serde_json::json!({"purpose": "Bank loan", "lang": "ku"}))
                .expect("shape")
                .into_input()
                .is_ok()
        );
        assert!(params(serde_json::json!({"purpose": "Bank loan", "lang": "ar"})).is_err());
        assert!(
            params(serde_json::json!({"purpose": "ab", "lang": "en"}))
                .expect("shape")
                .into_input()
                .is_err()
        );
    }

    #[test]
    fn an_empty_phone_filter_is_no_filter_and_a_bad_one_is_refused() {
        let query = |phone: &str| DashboardFarmersQuery {
            phone: Some(phone.to_string()),
            ..no_filters()
        };

        assert!(
            query("")
                .into_input(Pagination::new(1, 20))
                .expect("input")
                .filter
                .phone
                .is_none()
        );
        assert!(query("0750").into_input(Pagination::new(1, 20)).is_err());
    }
}
