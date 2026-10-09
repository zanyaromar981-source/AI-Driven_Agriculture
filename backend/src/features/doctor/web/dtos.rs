use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::features::doctor::{
    domain::{self, DoctorAnswer, Enquiry, Language, Photo, PhotoType, TappedCell},
    web::errors::WebError,
};

/// The form of `POST /v1/farms/{id}/ask`, sent as `multipart/form-data`.
/// At least a question or one photo.
#[derive(Default, ToSchema)]
pub struct DoctorAskForm {
    /// What the farmer asks, up to 1000 characters.
    pub question: Option<String>,
    /// 0 to 6 photos, each `image/jpeg` or `image/png` (the part's
    /// Content-Type) and 4 MB at most. One part per photo, named `photos`
    /// or `photos[]`.
    #[schema(value_type = Vec<String>, format = Binary, required = false)]
    pub photos: Vec<DoctorPhotoPart>,
    /// The 10 m cell the farmer tapped, as JSON text: `{"e": 46415, "n": 398748}`.
    pub cell: Option<String>,
    /// `ku` (the default) or `en`.
    pub lang: Option<String>,
}

/// One photo part as it arrived, before its bytes are checked.
pub struct DoctorPhotoPart {
    pub kind: PhotoType,
    pub bytes: Vec<u8>,
}

#[derive(Deserialize)]
struct DoctorCellParams {
    e: i32,
    n: i32,
}

impl DoctorAskForm {
    pub fn into_input(self) -> Result<Enquiry, WebError> {
        let cell = self
            .cell
            .map(|raw| serde_json::from_str::<DoctorCellParams>(&raw))
            .transpose()
            .map_err(|_| {
                WebError::BadField(r#"cell must be JSON like {"e": 1, "n": 2}"#.to_string())
            })?
            .map(|cell| TappedCell::new(cell.e, cell.n));

        let language = match self.lang.as_deref().map(str::trim) {
            None | Some("") => Language::default(),
            Some(code) => Language::try_from(code)?,
        };

        let photos = self
            .photos
            .into_iter()
            .map(|part| Photo::new(part.kind, part.bytes))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Enquiry::new(self.question, photos, cell, language)?)
    }
}

/// How sure the Doctor is of its answer.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum DoctorConfidence {
    Sure,
    Likely,
    Unsure,
}

impl From<DoctorConfidence> for domain::Confidence {
    fn from(value: DoctorConfidence) -> Self {
        match value {
            DoctorConfidence::Sure => domain::Confidence::Sure,
            DoctorConfidence::Likely => domain::Confidence::Likely,
            DoctorConfidence::Unsure => domain::Confidence::Unsure,
        }
    }
}

impl From<domain::Confidence> for DoctorConfidence {
    fn from(value: domain::Confidence) -> Self {
        match value {
            domain::Confidence::Sure => DoctorConfidence::Sure,
            domain::Confidence::Likely => DoctorConfidence::Likely,
            domain::Confidence::Unsure => DoctorConfidence::Unsure,
        }
    }
}

/// The Doctor's answer, as the Doctor service gave it once it passed the
/// rules: `actions_this_week` holds 3 at most, and a list the service left
/// out is empty.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct DoctorAnswerResponse {
    /// The likely cause, in the language asked for.
    pub likely: String,
    pub confidence: DoctorConfidence,
    /// Each reason as `input -> conclusion`.
    pub why: Vec<String>,
    pub actions_this_week: Vec<String>,
    pub cannot_tell: Vec<String>,
    /// The Doctor wants an agriculture officer to look at the farm.
    pub refer_to_officer: bool,
    /// The whole answer in Sorani.
    pub ku: String,
    /// The whole answer in English.
    pub en: String,
    /// Which sources the Doctor read, for example `weather` or `field_eye`.
    pub inputs_used: Vec<String>,
}

impl From<&DoctorAnswer> for DoctorAnswerResponse {
    fn from(answer: &DoctorAnswer) -> Self {
        Self {
            likely: answer.likely().clone(),
            confidence: (*answer.confidence()).into(),
            why: answer.why().clone(),
            actions_this_week: answer.actions_this_week().clone(),
            cannot_tell: answer.cannot_tell().clone(),
            refer_to_officer: *answer.refer_to_officer(),
            ku: answer.ku().clone(),
            en: answer.en().clone(),
            inputs_used: answer.inputs_used().clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::doctor::domain::DoctorError;

    fn a_jpeg_part() -> DoctorPhotoPart {
        DoctorPhotoPart {
            kind: PhotoType::Jpeg,
            bytes: PhotoType::Jpeg.signature().to_vec(),
        }
    }

    fn a_form() -> DoctorAskForm {
        DoctorAskForm {
            question: Some("Why yellow?".to_string()),
            photos: vec![a_jpeg_part()],
            cell: Some(r#"{"e": 46415, "n": 398748}"#.to_string()),
            lang: Some("en".to_string()),
        }
    }

    #[test]
    fn a_full_form_becomes_an_enquiry() {
        let enquiry = a_form().into_input().expect("enquiry");

        assert_eq!(enquiry.photos().len(), 1);
        assert_eq!(
            enquiry.cell().map(|cell| (cell.e(), cell.n())),
            Some((46_415, 398_748))
        );
        assert_eq!(*enquiry.language(), Language::English);
    }

    #[test]
    fn a_missing_or_blank_language_is_sorani() {
        for lang in [None, Some(String::new())] {
            let enquiry = DoctorAskForm { lang, ..a_form() }
                .into_input()
                .expect("enquiry");

            assert_eq!(*enquiry.language(), Language::Sorani);
        }
    }

    #[test]
    fn a_cell_that_is_not_the_agreed_json_cannot_be_read() {
        for cell in ["46415,398748", r#"{"e": "x", "n": 1}"#, r#"{"e": 1}"#] {
            let result = DoctorAskForm {
                cell: Some(cell.to_string()),
                ..a_form()
            }
            .into_input();

            assert!(
                matches!(result, Err(WebError::BadField(_))),
                "{cell} must be refused"
            );
        }
    }

    #[test]
    fn an_empty_form_is_an_empty_question() {
        let result = DoctorAskForm::default().into_input();

        assert!(matches!(
            result,
            Err(WebError::AppError(
                crate::features::doctor::app::AppError::Doctor(DoctorError::EmptyQuestion)
            ))
        ));
    }

    #[test]
    fn every_confidence_converts_both_ways() {
        for confidence in domain::Confidence::ALL {
            assert_eq!(
                domain::Confidence::from(DoctorConfidence::from(confidence)),
                confidence
            );
        }
    }
}
