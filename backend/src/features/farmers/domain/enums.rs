use crate::{features::farmers::domain::FarmerError, shared::DomainError};

/// The language a farmer reads the app and its messages in.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Language {
    /// Sorani (Central Kurdish), the app's default.
    #[default]
    Sorani,
    /// Kurmanji (Northern Kurdish).
    Kurmanji,
    Arabic,
    English,
}

impl Language {
    pub const ALL: [Language; 4] = [
        Language::Sorani,
        Language::Kurmanji,
        Language::Arabic,
        Language::English,
    ];
}

impl From<Language> for String {
    fn from(value: Language) -> Self {
        match value {
            Language::Sorani => "ku".to_string(),
            Language::Kurmanji => "kmr".to_string(),
            Language::Arabic => "ar".to_string(),
            Language::English => "en".to_string(),
        }
    }
}

impl TryFrom<&str> for Language {
    type Error = FarmerError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "ku" => Ok(Language::Sorani),
            "kmr" => Ok(Language::Kurmanji),
            "ar" => Ok(Language::Arabic),
            "en" => Ok(Language::English),
            _ => Err(DomainError::InvalidValue(format!("Invalid language: {value}")).into()),
        }
    }
}

/// A farmer's gender as a support letter prints it. Unknown is the absence
/// of a value, not a variant.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Gender {
    Male,
    Female,
}

impl Gender {
    pub const ALL: [Gender; 2] = [Gender::Male, Gender::Female];
}

impl From<Gender> for String {
    fn from(value: Gender) -> Self {
        match value {
            Gender::Male => "male".to_string(),
            Gender::Female => "female".to_string(),
        }
    }
}

impl TryFrom<&str> for Gender {
    type Error = FarmerError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "male" => Ok(Gender::Male),
            "female" => Ok(Gender::Female),
            _ => Err(DomainError::InvalidValue(format!("Invalid gender: {value}")).into()),
        }
    }
}

/// The language a support letter is printed in. The Ministry prints in
/// Sorani and English only, so this is narrower than [`Language`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LetterLanguage {
    Sorani,
    English,
}

impl LetterLanguage {
    pub const ALL: [LetterLanguage; 2] = [LetterLanguage::Sorani, LetterLanguage::English];
}

impl From<LetterLanguage> for String {
    fn from(value: LetterLanguage) -> Self {
        match value {
            LetterLanguage::Sorani => "ku".to_string(),
            LetterLanguage::English => "en".to_string(),
        }
    }
}

impl TryFrom<&str> for LetterLanguage {
    type Error = FarmerError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "ku" => Ok(LetterLanguage::Sorani),
            "en" => Ok(LetterLanguage::English),
            _ => Err(DomainError::InvalidValue(format!("Invalid letter language: {value}")).into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every variant must survive a trip to the database and back. A mismatch
    /// between the two directions corrupts rows silently rather than failing.
    #[test]
    fn every_language_round_trips() {
        for language in Language::ALL {
            let stored = String::from(language);
            let parsed = Language::try_from(stored.as_str())
                .unwrap_or_else(|err| panic!("{language:?} stored as {stored:?} but {err:?}"));

            assert_eq!(parsed, language, "{stored:?} did not round trip");
        }
    }

    #[test]
    fn sorani_is_the_code_the_app_already_sends() {
        assert_eq!(String::from(Language::Sorani), "ku");
        assert_eq!(Language::default(), Language::Sorani);
    }

    #[test]
    fn an_unknown_stored_value_is_rejected_rather_than_defaulted() {
        assert!(Language::try_from("fr").is_err());
        assert!(Language::try_from("").is_err());
        assert!(Language::try_from("KU").is_err());
    }

    #[test]
    fn every_gender_round_trips() {
        for gender in Gender::ALL {
            let stored = String::from(gender);

            assert_eq!(Gender::try_from(stored.as_str()).expect("gender"), gender);
        }

        assert!(Gender::try_from("other").is_err());
        assert!(Gender::try_from("Male").is_err());
    }

    #[test]
    fn every_letter_language_round_trips_and_only_sorani_and_english_exist() {
        for language in LetterLanguage::ALL {
            let stored = String::from(language);

            assert_eq!(
                LetterLanguage::try_from(stored.as_str()).expect("language"),
                language
            );
        }

        assert!(LetterLanguage::try_from("ar").is_err());
        assert!(LetterLanguage::try_from("kmr").is_err());
    }
}
