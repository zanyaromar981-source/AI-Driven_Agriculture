use chrono::{DateTime, Duration, Utc};
use getset::Getters;

use crate::{
    features::farmers::domain::{FarmerError, FarmerName, Language},
    shared::Phone,
};

#[derive(Clone, Debug, Getters)]
#[getset(get = "pub")]
pub struct Farmer {
    /// None = new (not yet persisted), Some = existing (persisted)
    id: Option<i32>,
    phone: Phone,
    name: Option<FarmerName>,
    language: Language,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl Farmer {
    /// A farmer exists from the first time a phone proves itself with a code.
    pub fn new(phone: Phone, language: Language) -> Self {
        let now = Utc::now();

        Self {
            id: None,
            phone,
            name: None,
            language,
            created_at: now,
            updated_at: now,
        }
    }

    /// Reconstruct from persisted state.
    pub fn rehydrate(
        id: i32,
        phone: Phone,
        name: Option<FarmerName>,
        language: Language,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id: Some(id),
            phone,
            name,
            language,
            created_at,
            updated_at,
        }
    }

    pub fn update(&mut self, name: Option<FarmerName>, language: Language) {
        self.name = name;
        self.language = language;
        self.updated_at = Utc::now();
    }
}

/// One code sent to one phone. A phone has at most one open challenge: asking
/// for a new code replaces the old one.
#[derive(Clone, Debug, Getters)]
#[getset(get = "pub")]
pub struct SignInChallenge {
    phone: Phone,
    /// The code is never stored, only its hash.
    code_hash: String,
    language: Language,
    attempts: u32,
    sent_at: DateTime<Utc>,
    expires_at: DateTime<Utc>,
}

impl SignInChallenge {
    pub fn new(
        phone: Phone,
        code_hash: String,
        language: Language,
        now: DateTime<Utc>,
        valid_for: Duration,
    ) -> Self {
        Self {
            phone,
            code_hash,
            language,
            attempts: 0,
            sent_at: now,
            expires_at: now + valid_for,
        }
    }

    /// Reconstruct from persisted state.
    pub fn rehydrate(
        phone: Phone,
        code_hash: String,
        language: Language,
        attempts: u32,
        sent_at: DateTime<Utc>,
        expires_at: DateTime<Utc>,
    ) -> Self {
        Self {
            phone,
            code_hash,
            language,
            attempts,
            sent_at,
            expires_at,
        }
    }

    /// A new code may only be asked for once `resend_after` has passed since
    /// the last one, so a phone cannot be flooded with messages.
    pub fn ensure_can_resend(
        &self,
        now: DateTime<Utc>,
        resend_after: Duration,
    ) -> Result<(), FarmerError> {
        let wait = (self.sent_at + resend_after - now).num_seconds();

        if wait > 0 {
            return Err(FarmerError::CodeRequestedTooSoon(wait as u64));
        }

        Ok(())
    }

    /// Checks a presented code against the stored hash. It does not count
    /// the attempt: the repository does that in one step with the limit, so
    /// that guesses sent at the same moment cannot slip past it.
    pub fn check(&self, presented_hash: &str, now: DateTime<Utc>) -> Result<(), FarmerError> {
        if now >= self.expires_at {
            return Err(FarmerError::CodeExpired);
        }

        if presented_hash != self.code_hash {
            return Err(FarmerError::WrongCode);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn phone() -> Phone {
        Phone::new("+9647501234567".to_string()).expect("phone")
    }

    fn challenge(now: DateTime<Utc>) -> SignInChallenge {
        SignInChallenge::new(
            phone(),
            "right".to_string(),
            Language::Sorani,
            now,
            Duration::minutes(10),
        )
    }

    #[test]
    fn the_right_code_passes() {
        let now = Utc::now();

        assert!(challenge(now).check("right", now).is_ok());
    }

    #[test]
    fn a_wrong_code_fails() {
        let now = Utc::now();

        assert!(matches!(
            challenge(now).check("wrong", now),
            Err(FarmerError::WrongCode)
        ));
    }

    #[test]
    fn the_code_stops_working_the_moment_it_expires() {
        let now = Utc::now();

        assert!(matches!(
            challenge(now).check("right", now + Duration::minutes(10)),
            Err(FarmerError::CodeExpired)
        ));
        assert!(
            challenge(now)
                .check("right", now + Duration::minutes(10) - Duration::seconds(1))
                .is_ok()
        );
    }

    #[test]
    fn a_second_code_cannot_be_asked_for_straight_away() {
        let now = Utc::now();
        let challenge = challenge(now);

        assert!(matches!(
            challenge.ensure_can_resend(now + Duration::seconds(1), Duration::seconds(60)),
            Err(FarmerError::CodeRequestedTooSoon(59))
        ));
        assert!(
            challenge
                .ensure_can_resend(now + Duration::seconds(60), Duration::seconds(60))
                .is_ok()
        );
    }

    #[test]
    fn editing_a_profile_can_clear_the_name() {
        let mut farmer = Farmer::new(phone(), Language::Sorani);

        farmer.update(
            Some(FarmerName::new("Hiwa K.".to_string()).expect("name")),
            Language::English,
        );
        assert_eq!(*farmer.language(), Language::English);

        farmer.update(None, Language::English);
        assert!(farmer.name().is_none());
    }
}
