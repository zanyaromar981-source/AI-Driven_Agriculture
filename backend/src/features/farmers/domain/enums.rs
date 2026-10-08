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
}
