use crate::{features::doctor::domain::DoctorError, shared::DomainError};

/// The language the farmer wants the answer in. The Doctor answers in both,
/// but leads with this one.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Language {
    /// Sorani (Central Kurdish), the app's default.
    #[default]
    Sorani,
    English,
}

impl Language {
    pub const ALL: [Language; 2] = [Language::Sorani, Language::English];
}

impl From<Language> for String {
    fn from(value: Language) -> Self {
        match value {
            Language::Sorani => "ku".to_string(),
            Language::English => "en".to_string(),
        }
    }
}

impl TryFrom<&str> for Language {
    type Error = DoctorError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "ku" => Ok(Language::Sorani),
            "en" => Ok(Language::English),
            _ => Err(DomainError::InvalidValue(format!("Invalid language: {value}")).into()),
        }
    }
}

/// How sure the Doctor is of its answer. The app words the answer
/// differently for each, so a guess is never shown as a fact.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Confidence {
    Sure,
    Likely,
    Unsure,
}

impl Confidence {
    pub const ALL: [Confidence; 3] = [Confidence::Sure, Confidence::Likely, Confidence::Unsure];
}

impl From<Confidence> for String {
    fn from(value: Confidence) -> Self {
        match value {
            Confidence::Sure => "sure".to_string(),
            Confidence::Likely => "likely".to_string(),
            Confidence::Unsure => "unsure".to_string(),
        }
    }
}

impl TryFrom<&str> for Confidence {
    type Error = DoctorError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "sure" => Ok(Confidence::Sure),
            "likely" => Ok(Confidence::Likely),
            "unsure" => Ok(Confidence::Unsure),
            _ => Err(DomainError::InvalidValue(format!("Invalid confidence: {value}")).into()),
        }
    }
}

/// The image formats the Doctor's model reads. The string form is the
/// media type.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PhotoType {
    Jpeg,
    Png,
}

impl PhotoType {
    pub const ALL: [PhotoType; 2] = [PhotoType::Jpeg, PhotoType::Png];

    /// The type a form part declares. Media types are case-insensitive and
    /// may carry parameters (`image/jpeg; name=a.jpg`), so only the bare
    /// type is compared.
    pub fn from_declared(media_type: &str) -> Result<Self, DoctorError> {
        let bare = media_type
            .split(';')
            .next()
            .unwrap_or_default()
            .trim()
            .to_ascii_lowercase();

        Self::try_from(bare.as_str()).map_err(|_| DoctorError::PhotoNotAnImage)
    }

    /// The bytes every file of this type starts with.
    pub fn signature(self) -> &'static [u8] {
        match self {
            PhotoType::Jpeg => &[0xFF, 0xD8, 0xFF],
            PhotoType::Png => &[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A],
        }
    }
}

impl From<PhotoType> for String {
    fn from(value: PhotoType) -> Self {
        match value {
            PhotoType::Jpeg => "image/jpeg".to_string(),
            PhotoType::Png => "image/png".to_string(),
        }
    }
}

impl TryFrom<&str> for PhotoType {
    type Error = DoctorError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "image/jpeg" => Ok(PhotoType::Jpeg),
            "image/png" => Ok(PhotoType::Png),
            _ => Err(DomainError::InvalidValue(format!("Invalid photo type: {value}")).into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every variant must survive a trip to its string form and back. A
    /// mismatch between the two directions turns a valid value into an
    /// error, or into another value, without anyone noticing.
    #[test]
    fn every_language_round_trips() {
        for language in Language::ALL {
            let text = String::from(language);
            let parsed = Language::try_from(text.as_str())
                .unwrap_or_else(|err| panic!("{language:?} written as {text:?} but {err:?}"));

            assert_eq!(parsed, language, "{text:?} did not round trip");
        }
    }

    #[test]
    fn sorani_is_the_default_and_the_code_the_app_sends() {
        assert_eq!(Language::default(), Language::Sorani);
        assert_eq!(String::from(Language::Sorani), "ku");
        assert_eq!(String::from(Language::English), "en");
    }

    #[test]
    fn a_language_the_doctor_does_not_answer_in_is_rejected() {
        assert!(
            Language::try_from("ar").is_err(),
            "the Doctor answers in Sorani and English only"
        );
        assert!(Language::try_from("KU").is_err());
        assert!(Language::try_from("").is_err());
    }

    #[test]
    fn every_confidence_round_trips() {
        for confidence in Confidence::ALL {
            let text = String::from(confidence);
            let parsed = Confidence::try_from(text.as_str())
                .unwrap_or_else(|err| panic!("{confidence:?} written as {text:?} but {err:?}"));

            assert_eq!(parsed, confidence, "{text:?} did not round trip");
        }
    }

    #[test]
    fn an_unknown_confidence_is_rejected_rather_than_defaulted() {
        assert!(Confidence::try_from("certain").is_err());
        assert!(Confidence::try_from("Sure").is_err());
        assert!(Confidence::try_from("").is_err());
    }

    #[test]
    fn every_photo_type_round_trips() {
        for kind in PhotoType::ALL {
            let text = String::from(kind);
            let parsed = PhotoType::try_from(text.as_str())
                .unwrap_or_else(|err| panic!("{kind:?} written as {text:?} but {err:?}"));

            assert_eq!(parsed, kind, "{text:?} did not round trip");
        }
    }

    #[test]
    fn a_declared_type_is_read_without_case_or_parameters() {
        assert_eq!(
            PhotoType::from_declared("IMAGE/JPEG").expect("jpeg"),
            PhotoType::Jpeg
        );
        assert_eq!(
            PhotoType::from_declared("image/png; name=leaf.png").expect("png"),
            PhotoType::Png
        );
    }

    #[test]
    fn a_declared_type_that_is_not_jpeg_or_png_is_not_a_photo() {
        for declared in ["image/heic", "image/gif", "application/octet-stream", ""] {
            assert!(
                matches!(
                    PhotoType::from_declared(declared),
                    Err(DoctorError::PhotoNotAnImage)
                ),
                "{declared:?} must be refused as a photo"
            );
        }
    }
}
