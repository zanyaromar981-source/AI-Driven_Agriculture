use chrono::NaiveDate;
use getset::Getters;

use crate::features::doctor::domain::{
    Confidence, DoctorError, Language, Photo, Question, TappedCell,
};

/// What the farmer asks: words, photos or both, about the whole farm or one
/// cell of it. Nothing of it is stored.
#[derive(Clone, Debug, Getters)]
#[getset(get = "pub")]
pub struct Enquiry {
    question: Option<Question>,
    photos: Vec<Photo>,
    cell: Option<TappedCell>,
    language: Language,
}

impl Enquiry {
    pub const MAX_PHOTOS: usize = 6;

    /// A blank question counts as no question, so a photo with an empty
    /// text box is still a valid enquiry.
    pub fn new(
        question: Option<String>,
        photos: Vec<Photo>,
        cell: Option<TappedCell>,
        language: Language,
    ) -> Result<Self, DoctorError> {
        if photos.len() > Self::MAX_PHOTOS {
            return Err(DoctorError::TooManyPhotos(Self::MAX_PHOTOS));
        }

        let question = question
            .filter(|text| !text.trim().is_empty())
            .map(Question::new)
            .transpose()?;

        if question.is_none() && photos.is_empty() {
            return Err(DoctorError::EmptyQuestion);
        }

        Ok(Self {
            question,
            photos,
            cell,
            language,
        })
    }
}

/// The farm as the Doctor is told about it, read from the farms feature for
/// each question.
#[derive(Clone, Debug, PartialEq, Getters)]
#[getset(get = "pub")]
pub struct FarmBrief {
    id: i32,
    name: String,
    /// The centre of the walked outline.
    lat: f64,
    lon: f64,
    /// The exact area inside the outline.
    area_m2: f64,
    /// Crop codes, largest area first, without `empty`.
    crops: Vec<String>,
}

impl FarmBrief {
    /// Reconstruct from the farms feature's state.
    pub fn rehydrate(
        id: i32,
        name: String,
        lat: f64,
        lon: f64,
        area_m2: f64,
        crops: Vec<String>,
    ) -> Self {
        Self {
            id,
            name,
            lat,
            lon,
            area_m2,
            crops,
        }
    }
}

/// One topic of what the data jobs already know about the farm, as the
/// insights feature serves it to the app.
#[derive(Clone, Debug, PartialEq, Getters)]
#[getset(get = "pub")]
pub struct HistoryTopic {
    topic: String,
    as_of: NaiveDate,
    source: String,
    confidence: String,
    summary_en: Option<String>,
    summary_ku: Option<String>,
    measures: Vec<HistoryMeasure>,
}

impl HistoryTopic {
    /// Reconstruct from the insights feature's state.
    pub fn rehydrate(
        topic: String,
        as_of: NaiveDate,
        source: String,
        confidence: String,
        summary_en: Option<String>,
        summary_ku: Option<String>,
        measures: Vec<HistoryMeasure>,
    ) -> Self {
        Self {
            topic,
            as_of,
            source,
            confidence,
            summary_en,
            summary_ku,
            measures,
        }
    }
}

/// One named number of a history topic.
#[derive(Clone, Debug, PartialEq, Getters)]
#[getset(get = "pub")]
pub struct HistoryMeasure {
    code: String,
    value: f64,
    unit: String,
    label_en: String,
    label_ku: Option<String>,
}

impl HistoryMeasure {
    /// Reconstruct from the insights feature's state.
    pub fn rehydrate(
        code: String,
        value: f64,
        unit: String,
        label_en: String,
        label_ku: Option<String>,
    ) -> Self {
        Self {
            code,
            value,
            unit,
            label_en,
            label_ku,
        }
    }
}

/// Everything the Doctor is given for one question: the farm, what is known
/// about it, and what the farmer asked.
#[derive(Clone, Debug, Getters)]
#[getset(get = "pub")]
pub struct Consultation {
    farm: FarmBrief,
    /// `None` when no data job has told us anything about the farm yet.
    history: Option<Vec<HistoryTopic>>,
    enquiry: Enquiry,
}

impl Consultation {
    /// An empty history is passed on as none, so the Doctor reads "nothing
    /// known yet" and never an empty reading.
    pub fn new(farm: FarmBrief, history: Vec<HistoryTopic>, enquiry: Enquiry) -> Self {
        Self {
            farm,
            history: (!history.is_empty()).then_some(history),
            enquiry,
        }
    }
}

/// The Doctor service's answer as it arrived, before any rule is applied.
/// Every field is optional because the service is outside this backend:
/// `DoctorAnswer::check` decides what is usable.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DoctorReply {
    pub likely: Option<String>,
    pub confidence: Option<String>,
    pub why: Option<Vec<String>>,
    pub actions_this_week: Option<Vec<String>>,
    pub cannot_tell: Option<Vec<String>>,
    pub refer_to_officer: Option<bool>,
    pub ku: Option<String>,
    pub en: Option<String>,
    pub inputs_used: Option<Vec<String>>,
}

/// The Doctor's answer once it has passed the rules, as the farmer sees it.
#[derive(Clone, Debug, PartialEq, Getters)]
#[getset(get = "pub")]
pub struct DoctorAnswer {
    likely: String,
    confidence: Confidence,
    why: Vec<String>,
    actions_this_week: Vec<String>,
    cannot_tell: Vec<String>,
    refer_to_officer: bool,
    ku: String,
    en: String,
    /// Which sources the Doctor read, for example `weather` or `field_eye`.
    inputs_used: Vec<String>,
}

impl DoctorAnswer {
    /// A farmer acts on a short list; more than three things to do in one
    /// week gets none of them done.
    pub const MAX_ACTIONS: usize = 3;

    /// The cause, how sure, the referral and both texts must be there: an
    /// answer without them would have to be guessed at, so it is refused.
    /// A missing list only means the Doctor had nothing to put in it.
    pub fn check(reply: DoctorReply) -> Result<Self, DoctorError> {
        let likely = reply
            .likely
            .filter(|text| !text.trim().is_empty())
            .ok_or_else(|| bad("the likely cause is missing"))?;

        let confidence = reply
            .confidence
            .ok_or_else(|| bad("the confidence is missing"))?;
        let confidence = Confidence::try_from(confidence.as_str())
            .map_err(|_| bad(format!("unknown confidence {confidence:?}")))?;

        let refer_to_officer = reply
            .refer_to_officer
            .ok_or_else(|| bad("refer_to_officer is missing"))?;

        let mut actions_this_week = reply.actions_this_week.unwrap_or_default();
        actions_this_week.truncate(Self::MAX_ACTIONS);

        Ok(Self {
            likely,
            confidence,
            why: reply.why.unwrap_or_default(),
            actions_this_week,
            cannot_tell: reply.cannot_tell.unwrap_or_default(),
            refer_to_officer,
            ku: reply.ku.ok_or_else(|| bad("the Sorani text is missing"))?,
            en: reply.en.ok_or_else(|| bad("the English text is missing"))?,
            inputs_used: reply.inputs_used.unwrap_or_default(),
        })
    }
}

fn bad(reason: impl Into<String>) -> DoctorError {
    DoctorError::BadAnswer(reason.into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::doctor::domain::PhotoType;

    fn a_photo() -> Photo {
        Photo::new(PhotoType::Jpeg, PhotoType::Jpeg.signature().to_vec()).expect("photo")
    }

    fn a_brief() -> FarmBrief {
        FarmBrief::rehydrate(
            5,
            "Upper field".to_string(),
            36.01,
            44.62,
            19_451.0,
            vec!["wheat".to_string()],
        )
    }

    fn a_topic() -> HistoryTopic {
        HistoryTopic::rehydrate(
            "rain".to_string(),
            NaiveDate::from_ymd_opt(2026, 10, 8).expect("date"),
            "CHIRPS".to_string(),
            "likely".to_string(),
            None,
            None,
            vec![HistoryMeasure::rehydrate(
                "rain_mm_30d".to_string(),
                12.5,
                "mm".to_string(),
                "Rain, last 30 days".to_string(),
                None,
            )],
        )
    }

    fn a_full_reply() -> DoctorReply {
        DoctorReply {
            likely: Some("Yellow rust".to_string()),
            confidence: Some("likely".to_string()),
            why: Some(vec!["weather -> cool and wet".to_string()]),
            actions_this_week: Some(vec!["Walk the field".to_string()]),
            cannot_tell: Some(vec!["How far it has spread".to_string()]),
            refer_to_officer: Some(true),
            ku: Some("ژەنگی زەرد".to_string()),
            en: Some("Probably yellow rust.".to_string()),
            inputs_used: Some(vec!["weather".to_string(), "field_eye".to_string()]),
        }
    }

    #[test]
    fn a_question_alone_is_enough() {
        let enquiry = Enquiry::new(
            Some("Why yellow?".to_string()),
            vec![],
            None,
            Language::Sorani,
        )
        .expect("enquiry");

        assert_eq!(
            enquiry.question().as_ref().map(Question::as_str),
            Some("Why yellow?")
        );
    }

    #[test]
    fn a_photo_alone_is_enough() {
        let enquiry = Enquiry::new(None, vec![a_photo()], None, Language::Sorani).expect("enquiry");

        assert!(enquiry.question().is_none());
        assert_eq!(enquiry.photos().len(), 1);
    }

    #[test]
    fn neither_a_question_nor_a_photo_is_an_empty_question() {
        assert!(matches!(
            Enquiry::new(None, vec![], None, Language::Sorani),
            Err(DoctorError::EmptyQuestion)
        ));
    }

    #[test]
    fn a_blank_question_without_a_photo_is_an_empty_question() {
        assert!(
            matches!(
                Enquiry::new(Some("  \n ".to_string()), vec![], None, Language::Sorani),
                Err(DoctorError::EmptyQuestion)
            ),
            "an empty text box is no question, not an invalid one"
        );
    }

    #[test]
    fn a_blank_question_with_a_photo_is_a_photo_without_a_question() {
        let enquiry = Enquiry::new(
            Some(" ".to_string()),
            vec![a_photo()],
            None,
            Language::English,
        )
        .expect("enquiry");

        assert!(enquiry.question().is_none());
    }

    #[test]
    fn six_photos_are_allowed_and_seven_are_too_many() {
        assert!(Enquiry::new(None, vec![a_photo(); 6], None, Language::Sorani).is_ok());
        assert!(matches!(
            Enquiry::new(None, vec![a_photo(); 7], None, Language::Sorani),
            Err(DoctorError::TooManyPhotos(6))
        ));
    }

    #[test]
    fn a_question_over_the_limit_is_refused_even_with_photos() {
        assert!(matches!(
            Enquiry::new(
                Some("a".repeat(Question::MAX_LENGTH + 1)),
                vec![a_photo()],
                None,
                Language::Sorani
            ),
            Err(DoctorError::QuestionTooLong(_))
        ));
    }

    #[test]
    fn a_farm_with_no_history_is_sent_with_none() {
        let enquiry = Enquiry::new(None, vec![a_photo()], None, Language::Sorani).expect("enquiry");

        let consultation = Consultation::new(a_brief(), vec![], enquiry);

        assert!(consultation.history().is_none());
    }

    #[test]
    fn a_farm_with_history_is_sent_with_all_of_it() {
        let enquiry = Enquiry::new(None, vec![a_photo()], None, Language::Sorani).expect("enquiry");

        let consultation = Consultation::new(a_brief(), vec![a_topic()], enquiry);

        assert_eq!(consultation.history().as_deref(), Some(&[a_topic()][..]));
    }

    #[test]
    fn a_full_reply_passes_unchanged() {
        let answer = DoctorAnswer::check(a_full_reply()).expect("answer");

        assert_eq!(answer.likely(), "Yellow rust");
        assert_eq!(*answer.confidence(), Confidence::Likely);
        assert_eq!(answer.why(), &vec!["weather -> cool and wet".to_string()]);
        assert!(*answer.refer_to_officer());
        assert_eq!(answer.ku(), "ژەنگی زەرد");
        assert_eq!(answer.en(), "Probably yellow rust.");
        assert_eq!(answer.inputs_used().len(), 2);
    }

    #[test]
    fn every_confidence_the_contract_names_is_accepted() {
        for confidence in ["sure", "likely", "unsure"] {
            let reply = DoctorReply {
                confidence: Some(confidence.to_string()),
                ..a_full_reply()
            };

            assert!(DoctorAnswer::check(reply).is_ok(), "{confidence} refused");
        }
    }

    #[test]
    fn a_confidence_outside_the_three_is_refused() {
        let reply = DoctorReply {
            confidence: Some("certain".to_string()),
            ..a_full_reply()
        };

        assert!(matches!(
            DoctorAnswer::check(reply),
            Err(DoctorError::BadAnswer(_))
        ));
    }

    #[test]
    fn only_the_first_three_actions_are_kept() {
        let reply = DoctorReply {
            actions_this_week: Some((1..=5).map(|step| format!("Step {step}")).collect()),
            ..a_full_reply()
        };

        let answer = DoctorAnswer::check(reply).expect("answer");

        assert_eq!(
            answer.actions_this_week(),
            &vec![
                "Step 1".to_string(),
                "Step 2".to_string(),
                "Step 3".to_string()
            ],
            "extra actions are cut, the answer is not refused"
        );
    }

    #[test]
    fn missing_lists_become_empty() {
        let reply = DoctorReply {
            why: None,
            actions_this_week: None,
            cannot_tell: None,
            inputs_used: None,
            ..a_full_reply()
        };

        let answer = DoctorAnswer::check(reply).expect("answer");

        assert!(answer.why().is_empty());
        assert!(answer.actions_this_week().is_empty());
        assert!(answer.cannot_tell().is_empty());
        assert!(answer.inputs_used().is_empty());
    }

    #[test]
    fn an_answer_missing_what_the_farmer_must_read_is_refused() {
        let without: [fn(&mut DoctorReply); 6] = [
            |reply| reply.likely = None,
            |reply| reply.likely = Some("  ".to_string()),
            |reply| reply.confidence = None,
            |reply| reply.refer_to_officer = None,
            |reply| reply.ku = None,
            |reply| reply.en = None,
        ];

        for (case, remove) in without.iter().enumerate() {
            let mut reply = a_full_reply();
            remove(&mut reply);

            assert!(
                matches!(DoctorAnswer::check(reply), Err(DoctorError::BadAnswer(_))),
                "case {case} must be refused, not filled in"
            );
        }
    }
}
