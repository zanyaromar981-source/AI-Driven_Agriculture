use std::sync::{Arc, Mutex};

use async_trait::async_trait;

use crate::{
    app::{Action, AuthContext, Pagination, Permission, Resource, StaffContext, User},
    features::farmers::{
        app::{
            AppError, FarmCounter, FarmHoldings, FarmRemover, FarmerFilter, FarmerRepository,
            LetterIssuers, LetterRecord, LetterRepository, SignInChallengeRepository,
            SignInCodeGenerator, SignInCodeHasher, SignInCodeSender, TokenIssuer,
        },
        domain::{
            CropHolding, FarmHolding, Farmer, FarmerChange, FarmerDetails, Language, Letter,
            LetterLanguage, LetterNumber, LetterPurpose, SignInChallenge, SignInCode,
        },
    },
    shared::Phone,
};

pub const PHONE: &str = "+9647501234567";

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Call {
    FindFarmer { phone: String },
    CreateFarmerIfAbsent { phone: String },
    UpdateFarmer,
    FindChallenge { phone: String },
    SaveChallenge { phone: String },
    RecordAttempt { phone: String },
    ConsumeChallenge { phone: String },
    SendCode { phone: String, code: String },
    IssueToken,
    CountFarms { phone: String },
    FindFarmerById { id: i32 },
    FindFarmersPage { phone: Option<String>, page: u64 },
    CreateFarmer { phone: String },
    UpdateFarmerById { id: i32 },
    DeleteFarmerWithChallenge { id: i32 },
    RemoveFarms { phone: String },
    FarmHoldings { phone: String },
    LetterIssuerName { staff_id: i32 },
    IssueLetter { farmer_id: i32, staff_id: i32 },
    FindLetter { number: String },
}

#[derive(Debug, Default)]
struct Script {
    farmer: Option<Farmer>,
    challenge: Option<SignInChallenge>,
    fail_to_send: bool,
    lose_the_race_to_consume: bool,
    fail_to_remove_farms: bool,
    fail_to_read_farmers: bool,
    fail_to_read_farms: bool,
    letters: Vec<Letter>,
}

/// One fake standing in for every port of the feature, so a test can read
/// the calls of a whole use case in the order they happened.
#[derive(Debug, Clone, Default)]
pub struct Fakes {
    script: Arc<Mutex<Script>>,
    calls: Arc<Mutex<Vec<Call>>>,
}

impl Fakes {
    pub fn new() -> Self {
        Self::default()
    }

    /// A phone that asked for a code a moment ago and has not used it yet.
    pub fn with_open_challenge(code: &str) -> Self {
        let fake = Self::new();
        let code = SignInCode::new(code.to_string()).expect("code");

        fake.script.lock().expect("script lock").challenge = Some(SignInChallenge::new(
            phone(),
            fake.hash(&phone(), &code),
            Language::Sorani,
            chrono::Utc::now(),
            chrono::Duration::minutes(10),
        ));
        fake
    }

    pub fn with_farmer(self) -> Self {
        self.script.lock().expect("script lock").farmer = Some(a_farmer());
        self
    }

    /// The phone's farmer has been blocked by staff.
    pub fn with_blocked_farmer(self) -> Self {
        self.script.lock().expect("script lock").farmer = Some(with_state(&a_farmer(), None, true));
        self
    }

    /// The farms feature cannot list the farmer's farms.
    pub fn failing_to_read_farms(self) -> Self {
        self.script.lock().expect("script lock").fail_to_read_farms = true;
        self
    }

    pub fn stored_letters(&self) -> Vec<Letter> {
        self.script.lock().expect("script lock").letters.clone()
    }

    /// Another request with the same code consumes the challenge first.
    pub fn losing_the_race_to_consume(self) -> Self {
        self.script
            .lock()
            .expect("script lock")
            .lose_the_race_to_consume = true;
        self
    }

    /// The farms feature cannot remove the farmer's farms.
    pub fn failing_to_remove_farms(self) -> Self {
        self.script
            .lock()
            .expect("script lock")
            .fail_to_remove_farms = true;
        self
    }

    /// The database cannot be read when a farmer is looked up by phone.
    pub fn failing_to_read_farmers(self) -> Self {
        self.script
            .lock()
            .expect("script lock")
            .fail_to_read_farmers = true;
        self
    }

    pub fn has_farmer(&self) -> bool {
        self.script.lock().expect("script lock").farmer.is_some()
    }

    pub fn farmer_created_at(&self) -> Option<chrono::DateTime<chrono::Utc>> {
        self.script
            .lock()
            .expect("script lock")
            .farmer
            .as_ref()
            .map(|farmer| *farmer.created_at())
    }

    /// The open code already signed someone in, this long ago.
    pub fn used(self, ago: chrono::Duration) -> Self {
        let mut script = self.script.lock().expect("script lock");

        if let Some(open) = script.challenge.clone() {
            script.challenge = Some(SignInChallenge::rehydrate(
                open.phone().clone(),
                open.code_hash().clone(),
                *open.language(),
                *open.attempts(),
                *open.sent_at(),
                *open.expires_at(),
                Some(chrono::Utc::now() - ago),
            ));
        }

        drop(script);
        self
    }

    pub fn failing_to_send() -> Self {
        let fake = Self::new();
        fake.script.lock().expect("script lock").fail_to_send = true;
        fake
    }

    pub fn calls(&self) -> Vec<Call> {
        self.calls.lock().expect("calls lock").clone()
    }

    pub fn stored_challenge(&self) -> Option<SignInChallenge> {
        self.script.lock().expect("script lock").challenge.clone()
    }

    fn record(&self, call: Call) {
        self.calls.lock().expect("calls lock").push(call);
    }
}

#[async_trait]
impl FarmerRepository for Fakes {
    async fn find_by_phone(&self, phone: &Phone) -> Result<Option<Farmer>, AppError> {
        self.record(Call::FindFarmer {
            phone: String::from(phone),
        });

        let script = self.script.lock().expect("script lock");

        if script.fail_to_read_farmers {
            return Err(
                crate::app::AppError::DatabaseError("connection refused".to_string()).into(),
            );
        }

        Ok(script.farmer.clone())
    }

    async fn create_if_absent(&self, entity: &Farmer) -> Result<(), AppError> {
        self.record(Call::CreateFarmerIfAbsent {
            phone: String::from(entity.phone()),
        });

        let mut script = self.script.lock().expect("script lock");

        if script.farmer.is_none() {
            script.farmer = Some(Farmer::rehydrate(
                1,
                entity.phone().clone(),
                entity.name().clone(),
                *entity.language(),
                entity.details().clone(),
                *entity.blocked(),
                *entity.created_at(),
                *entity.updated_at(),
            ));
        }

        Ok(())
    }

    async fn update(&self, entity: &Farmer) -> Result<Farmer, AppError> {
        self.record(Call::UpdateFarmer);

        Ok(entity.clone())
    }

    async fn find_by_id(&self, id: i32) -> Result<Option<Farmer>, AppError> {
        self.record(Call::FindFarmerById { id });

        let script = self.script.lock().expect("script lock");

        Ok(script
            .farmer
            .clone()
            .filter(|farmer| *farmer.id() == Some(id)))
    }

    async fn find_page(
        &self,
        filter: &FarmerFilter,
        pagination: &Pagination,
    ) -> Result<(Vec<Farmer>, u64), AppError> {
        let phone = filter.phone.as_ref();

        self.record(Call::FindFarmersPage {
            phone: phone.map(String::from),
            page: *pagination.page(),
        });

        let script = self.script.lock().expect("script lock");

        let farmers: Vec<Farmer> = script
            .farmer
            .iter()
            .filter(|farmer| phone.is_none_or(|phone| farmer.phone() == phone))
            .filter(|farmer| {
                filter
                    .blocked
                    .is_none_or(|blocked| *farmer.blocked() == blocked)
            })
            .cloned()
            .collect();
        let count = farmers.len() as u64;

        Ok((farmers, count))
    }

    async fn create(&self, entity: &Farmer) -> Result<Option<Farmer>, AppError> {
        self.record(Call::CreateFarmer {
            phone: String::from(entity.phone()),
        });

        let mut script = self.script.lock().expect("script lock");

        if script.farmer.is_some() {
            return Ok(None);
        }

        let created = Farmer::rehydrate(
            1,
            entity.phone().clone(),
            entity.name().clone(),
            *entity.language(),
            entity.details().clone(),
            *entity.blocked(),
            *entity.created_at(),
            *entity.updated_at(),
        );
        script.farmer = Some(created.clone());

        Ok(Some(created))
    }

    async fn update_by_id(
        &self,
        id: i32,
        change: &FarmerChange,
        now: chrono::DateTime<chrono::Utc>,
    ) -> Result<Option<Farmer>, AppError> {
        self.record(Call::UpdateFarmerById { id });

        let mut script = self.script.lock().expect("script lock");

        let Some(stored) = script
            .farmer
            .clone()
            .filter(|farmer| *farmer.id() == Some(id))
        else {
            return Ok(None);
        };

        let updated = Farmer::rehydrate(
            id,
            stored.phone().clone(),
            change.name.clone(),
            change.language,
            change.details.clone(),
            change.blocked.unwrap_or(*stored.blocked()),
            *stored.created_at(),
            now,
        );
        script.farmer = Some(updated.clone());

        Ok(Some(updated))
    }

    async fn delete_with_challenge(&self, id: i32) -> Result<bool, AppError> {
        self.record(Call::DeleteFarmerWithChallenge { id });

        let mut script = self.script.lock().expect("script lock");

        if script
            .farmer
            .as_ref()
            .is_none_or(|farmer| *farmer.id() != Some(id))
        {
            return Ok(false);
        }

        script.farmer = None;
        script.challenge = None;

        Ok(true)
    }
}

#[async_trait]
impl LetterRepository for Fakes {
    async fn issue(
        &self,
        farmer_id: i32,
        staff_id: i32,
        purpose: &LetterPurpose,
        language: LetterLanguage,
        now: chrono::DateTime<chrono::Utc>,
    ) -> Result<Option<(Letter, Farmer)>, AppError> {
        self.record(Call::IssueLetter {
            farmer_id,
            staff_id,
        });

        let mut script = self.script.lock().expect("script lock");

        let Some(farmer) = script
            .farmer
            .clone()
            .filter(|farmer| *farmer.id() == Some(farmer_id))
        else {
            return Ok(None);
        };

        let earlier = script
            .letters
            .iter()
            .filter(|letter| *letter.farmer_id() == farmer_id)
            .count() as u64;

        let issued = Letter::issue(
            farmer_id,
            earlier + 1,
            staff_id,
            purpose.clone(),
            language,
            now,
        );
        let stored = Letter::rehydrate(
            script.letters.len() as i32 + 1,
            issued.number().clone(),
            farmer_id,
            staff_id,
            purpose.clone(),
            language,
            now,
        );
        script.letters.push(stored.clone());

        Ok(Some((stored, farmer)))
    }

    async fn find_by_number(
        &self,
        number: &LetterNumber,
    ) -> Result<Option<LetterRecord>, AppError> {
        self.record(Call::FindLetter {
            number: number.as_str().to_string(),
        });

        let script = self.script.lock().expect("script lock");

        Ok(script
            .letters
            .iter()
            .find(|letter| letter.number() == number)
            .map(|letter| LetterRecord {
                letter: letter.clone(),
                farmer_name: script
                    .farmer
                    .as_ref()
                    .filter(|farmer| farmer.id() == &Some(*letter.farmer_id()))
                    .and_then(|farmer| farmer.name().clone()),
            }))
    }
}

#[async_trait]
impl FarmHoldings for Fakes {
    async fn of(&self, phone: &Phone) -> Result<Vec<FarmHolding>, AppError> {
        self.record(Call::FarmHoldings {
            phone: String::from(phone),
        });

        if self.script.lock().expect("script lock").fail_to_read_farms {
            return Err(crate::app::AppError::InternalServerError.into());
        }

        Ok(vec![
            a_holding(1, 10.0, &[("wheat", 6.0)]),
            a_holding(2, 5.0, &[("wheat", 1.0), ("barley", 4.0)]),
        ])
    }
}

#[async_trait]
impl LetterIssuers for Fakes {
    async fn name_of(&self, staff_id: i32) -> Result<Option<String>, AppError> {
        self.record(Call::LetterIssuerName { staff_id });

        Ok((staff_id == STAFF_ID).then(|| STAFF_NAME.to_string()))
    }
}

#[async_trait]
impl FarmRemover for Fakes {
    async fn remove_all_for(&self, phone: &Phone) -> Result<u64, AppError> {
        if self
            .script
            .lock()
            .expect("script lock")
            .fail_to_remove_farms
        {
            return Err(crate::app::AppError::InternalServerError.into());
        }

        self.record(Call::RemoveFarms {
            phone: String::from(phone),
        });

        Ok(2)
    }
}

#[async_trait]
impl SignInChallengeRepository for Fakes {
    async fn find_by_phone(&self, phone: &Phone) -> Result<Option<SignInChallenge>, AppError> {
        self.record(Call::FindChallenge {
            phone: String::from(phone),
        });

        Ok(self.script.lock().expect("script lock").challenge.clone())
    }

    async fn save_if_due(
        &self,
        challenge: &SignInChallenge,
        sent_before: chrono::DateTime<chrono::Utc>,
    ) -> Result<bool, AppError> {
        let mut script = self.script.lock().expect("script lock");

        if script
            .challenge
            .as_ref()
            .is_some_and(|open| *open.sent_at() > sent_before)
        {
            return Ok(false);
        }

        script.challenge = Some(challenge.clone());
        drop(script);

        self.record(Call::SaveChallenge {
            phone: String::from(challenge.phone()),
        });

        Ok(true)
    }

    async fn record_attempt(
        &self,
        phone: &Phone,
        max_attempts: u32,
    ) -> Result<Option<SignInChallenge>, AppError> {
        self.record(Call::RecordAttempt {
            phone: String::from(phone),
        });

        let mut script = self.script.lock().expect("script lock");

        let Some(open) = script.challenge.clone() else {
            return Ok(None);
        };

        if *open.attempts() >= max_attempts {
            return Ok(None);
        }

        let counted = SignInChallenge::rehydrate(
            open.phone().clone(),
            open.code_hash().clone(),
            *open.language(),
            *open.attempts() + 1,
            *open.sent_at(),
            *open.expires_at(),
            *open.used_at(),
        );
        script.challenge = Some(counted.clone());

        Ok(Some(counted))
    }

    async fn consume(
        &self,
        phone: &Phone,
        code_hash: &str,
        now: chrono::DateTime<chrono::Utc>,
        reusable_since: chrono::DateTime<chrono::Utc>,
    ) -> Result<bool, AppError> {
        self.record(Call::ConsumeChallenge {
            phone: String::from(phone),
        });

        let mut script = self.script.lock().expect("script lock");

        if script.lose_the_race_to_consume {
            script.challenge = None;

            return Ok(false);
        }

        let Some(open) = script.challenge.clone() else {
            return Ok(false);
        };

        let usable = open.code_hash() == code_hash
            && open
                .used_at()
                .is_none_or(|used_at| used_at > reusable_since);

        if usable {
            script.challenge = Some(SignInChallenge::rehydrate(
                open.phone().clone(),
                open.code_hash().clone(),
                *open.language(),
                *open.attempts(),
                *open.sent_at(),
                *open.expires_at(),
                Some(open.used_at().unwrap_or(now)),
            ));
        }

        Ok(usable)
    }
}

#[async_trait]
impl SignInCodeSender for Fakes {
    async fn send(
        &self,
        phone: &Phone,
        code: &SignInCode,
        _language: Language,
    ) -> Result<(), AppError> {
        if self.script.lock().expect("script lock").fail_to_send {
            return Err(AppError::GlobalAppError(
                crate::app::IntegrationError::TemporarilyUnavailable.into(),
            ));
        }

        self.record(Call::SendCode {
            phone: String::from(phone),
            code: code.as_str().to_string(),
        });

        Ok(())
    }
}

impl SignInCodeGenerator for Fakes {
    fn generate(&self) -> SignInCode {
        SignInCode::new("123456".to_string()).expect("code")
    }
}

impl SignInCodeHasher for Fakes {
    fn hash(&self, phone: &Phone, code: &SignInCode) -> String {
        format!("hash-of-{}-{}", phone.as_str(), code.as_str())
    }
}

impl TokenIssuer for Fakes {
    fn issue(&self, phone: &Phone) -> Result<String, AppError> {
        self.record(Call::IssueToken);

        Ok(format!("token-for-{}", phone.as_str()))
    }
}

#[async_trait]
impl FarmCounter for Fakes {
    async fn count_for(&self, phone: &Phone) -> Result<u64, AppError> {
        self.record(Call::CountFarms {
            phone: String::from(phone),
        });

        Ok(2)
    }
}

pub fn phone() -> Phone {
    Phone::new(PHONE.to_string()).expect("phone")
}

pub fn auth_context() -> AuthContext {
    AuthContext::new(User::new(phone()), "token".to_string())
}

pub fn a_farmer() -> Farmer {
    Farmer::rehydrate(
        1,
        phone(),
        None,
        Language::Sorani,
        FarmerDetails::default(),
        false,
        chrono::Utc::now(),
        chrono::Utc::now(),
    )
}

/// The same farmer with other details or another blocked state.
pub fn with_state(farmer: &Farmer, details: Option<FarmerDetails>, blocked: bool) -> Farmer {
    Farmer::rehydrate(
        farmer.id().unwrap_or_default(),
        farmer.phone().clone(),
        farmer.name().clone(),
        *farmer.language(),
        details.unwrap_or_else(|| farmer.details().clone()),
        blocked,
        *farmer.created_at(),
        *farmer.updated_at(),
    )
}

pub fn a_holding(id: i32, area_dunam: f64, crops: &[(&str, f64)]) -> FarmHolding {
    FarmHolding {
        id,
        name: format!("Farm {id}"),
        governorate: None,
        zone_slug: None,
        sub_zone_slug: None,
        area_dunam,
        crops: crops
            .iter()
            .map(|(crop, dunam)| CropHolding {
                crop: crop.to_string(),
                dunam: *dunam,
            })
            .collect(),
    }
}

pub const STAFF_ID: i32 = 3;
pub const STAFF_NAME: &str = "Dilan A.";

/// A staff member holding every permission on farmers.
pub fn staff_context() -> StaffContext {
    StaffContext::new(
        STAFF_ID,
        "officer@example.org".to_string(),
        Action::ALL
            .into_iter()
            .map(|action| Permission::new(Resource::Farmers, action))
            .collect(),
    )
}
