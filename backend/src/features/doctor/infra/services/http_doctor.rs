use std::time::Duration;

use async_trait::async_trait;
use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::features::doctor::{
    app::{AppError, Doctor},
    domain::{Consultation, DoctorReply, HistoryMeasure, HistoryTopic, Photo},
};

/// The Doctor's model takes 10 to 25 s for an answer. This leaves room for a
/// slow one without holding the farmer's request open forever.
const TIMEOUT: Duration = Duration::from_secs(90);

/// The Farm Doctor service on this machine, reached over plain HTTP:
/// `POST {base}/ask` with the consultation as JSON.
#[derive(Debug)]
pub struct HttpDoctor {
    client: reqwest::Client,
    ask_url: String,
}

impl HttpDoctor {
    pub fn new(base_url: &str) -> Result<Self, reqwest::Error> {
        Ok(Self {
            client: reqwest::Client::builder().timeout(TIMEOUT).build()?,
            ask_url: format!("{}/ask", base_url.trim_end_matches('/')),
        })
    }
}

#[async_trait]
impl Doctor for HttpDoctor {
    async fn ask(&self, consultation: &Consultation) -> Result<DoctorReply, AppError> {
        let farm_id = *consultation.farm().id();

        let response = self
            .client
            .post(&self.ask_url)
            .json(&AskBody::from(consultation))
            .send()
            .await
            .map_err(|error| {
                tracing::error!(
                    %error,
                    farm_id,
                    url = %self.ask_url,
                    timed_out = error.is_timeout(),
                    "the Doctor service could not be reached"
                );

                AppError::DoctorFailed
            })?;

        let status = response.status().as_u16();

        let body = response.bytes().await.map_err(|error| {
            tracing::error!(%error, farm_id, status, "the Doctor service's answer was cut off");

            AppError::DoctorFailed
        })?;

        reply_from(status, &body).inspect_err(|_| {
            tracing::error!(
                farm_id,
                status,
                body = %String::from_utf8_lossy(&body[..body.len().min(500)]),
                "the Doctor service gave no usable answer"
            );
        })
    }
}

/// Reads the service's answer. `503 doctor_not_ready` is the one failure the
/// app is told apart, because it means "try later", not "broken".
fn reply_from(status: u16, body: &[u8]) -> Result<DoctorReply, AppError> {
    match status {
        200 => serde_json::from_slice::<ReplyBody>(body)
            .map(Into::into)
            .map_err(|error| {
                tracing::error!(%error, "the Doctor service's answer is not the agreed JSON");

                AppError::DoctorFailed
            }),
        503 if error_code(body).as_deref() == Some("doctor_not_ready") => {
            Err(AppError::DoctorNotReady)
        }
        _ => Err(AppError::DoctorFailed),
    }
}

fn error_code(body: &[u8]) -> Option<String> {
    #[derive(Deserialize)]
    struct ErrorBody {
        error: String,
    }

    serde_json::from_slice::<ErrorBody>(body)
        .ok()
        .map(|body| body.error)
}

#[derive(Serialize)]
struct AskBody<'a> {
    farm: FarmBody<'a>,
    history: Option<HistoryBody<'a>>,
    question: Option<&'a str>,
    cell: Option<CellBody>,
    lang: String,
    photos: Vec<PhotoBody>,
}

impl<'a> From<&'a Consultation> for AskBody<'a> {
    fn from(consultation: &'a Consultation) -> Self {
        let farm = consultation.farm();
        let enquiry = consultation.enquiry();

        Self {
            farm: FarmBody {
                id: farm.id().to_string(),
                name: farm.name(),
                lat: *farm.lat(),
                lon: *farm.lon(),
                area_m2: *farm.area_m2(),
                crops: farm.crops(),
            },
            history: consultation.history().as_ref().map(|topics| HistoryBody {
                topics: topics.iter().map(TopicBody::from).collect(),
            }),
            question: enquiry
                .question()
                .as_ref()
                .map(|question| question.as_str()),
            cell: enquiry.cell().map(|cell| CellBody {
                e: cell.e(),
                n: cell.n(),
            }),
            lang: String::from(*enquiry.language()),
            photos: enquiry.photos().iter().map(PhotoBody::from).collect(),
        }
    }
}

#[derive(Serialize)]
struct FarmBody<'a> {
    id: String,
    name: &'a str,
    lat: f64,
    lon: f64,
    area_m2: f64,
    crops: &'a [String],
}

/// The shape `GET /v1/farms/{id}/insights` answers with, without the farm
/// id, which `farm` already carries.
#[derive(Serialize)]
struct HistoryBody<'a> {
    topics: Vec<TopicBody<'a>>,
}

#[derive(Serialize)]
struct TopicBody<'a> {
    topic: &'a str,
    as_of: NaiveDate,
    source: &'a str,
    confidence: &'a str,
    summary_en: Option<&'a str>,
    summary_ku: Option<&'a str>,
    measures: Vec<MeasureBody<'a>>,
}

impl<'a> From<&'a HistoryTopic> for TopicBody<'a> {
    fn from(topic: &'a HistoryTopic) -> Self {
        Self {
            topic: topic.topic(),
            as_of: *topic.as_of(),
            source: topic.source(),
            confidence: topic.confidence(),
            summary_en: topic.summary_en().as_deref(),
            summary_ku: topic.summary_ku().as_deref(),
            measures: topic.measures().iter().map(MeasureBody::from).collect(),
        }
    }
}

#[derive(Serialize)]
struct MeasureBody<'a> {
    code: &'a str,
    value: f64,
    unit: &'a str,
    label_en: &'a str,
    label_ku: Option<&'a str>,
}

impl<'a> From<&'a HistoryMeasure> for MeasureBody<'a> {
    fn from(measure: &'a HistoryMeasure) -> Self {
        Self {
            code: measure.code(),
            value: *measure.value(),
            unit: measure.unit(),
            label_en: measure.label_en(),
            label_ku: measure.label_ku().as_deref(),
        }
    }
}

#[derive(Serialize)]
struct CellBody {
    e: i32,
    n: i32,
}

#[derive(Serialize)]
struct PhotoBody {
    mime: String,
    data: String,
}

impl From<&Photo> for PhotoBody {
    fn from(photo: &Photo) -> Self {
        Self {
            mime: photo.kind().into(),
            data: BASE64.encode(photo.bytes()),
        }
    }
}

/// Everything optional: which fields an answer must have is the domain's
/// rule (`DoctorAnswer::check`), not the parser's.
#[derive(Deserialize)]
struct ReplyBody {
    likely: Option<String>,
    confidence: Option<String>,
    why: Option<Vec<String>>,
    actions_this_week: Option<Vec<String>>,
    cannot_tell: Option<Vec<String>>,
    refer_to_officer: Option<bool>,
    ku: Option<String>,
    en: Option<String>,
    inputs_used: Option<Vec<String>>,
}

impl From<ReplyBody> for DoctorReply {
    fn from(body: ReplyBody) -> Self {
        Self {
            likely: body.likely,
            confidence: body.confidence,
            why: body.why,
            actions_this_week: body.actions_this_week,
            cannot_tell: body.cannot_tell,
            refer_to_officer: body.refer_to_officer,
            ku: body.ku,
            en: body.en,
            inputs_used: body.inputs_used,
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::features::doctor::{
        app::testing::{a_brief, a_photo, a_topic, an_enquiry},
        domain::{Enquiry, Language},
    };

    #[test]
    fn the_request_has_the_shape_the_doctor_service_reads() {
        let consultation = Consultation::new(a_brief(), vec![a_topic()], an_enquiry());

        let body = serde_json::to_value(AskBody::from(&consultation)).expect("json");

        assert_eq!(
            body,
            json!({
                "farm": {
                    "id": "5",
                    "name": "Upper field",
                    "lat": 36.0305,
                    "lon": 44.6005,
                    "area_m2": 19451.0,
                    "crops": ["wheat"]
                },
                "history": {"topics": [{
                    "topic": "rain",
                    "as_of": "2026-10-08",
                    "source": "CHIRPS daily rain",
                    "confidence": "likely",
                    "summary_en": "A dry month",
                    "summary_ku": null,
                    "measures": [{
                        "code": "rain_mm_30d",
                        "value": 3.5,
                        "unit": "mm",
                        "label_en": "Rain, last 30 days",
                        "label_ku": null
                    }]
                }]},
                "question": "Why are the leaves yellow?",
                "cell": {"e": 46415, "n": 398748},
                "lang": "en",
                "photos": [
                    {"mime": "image/jpeg", "data": BASE64.encode([0xFF, 0xD8, 0xFF, 1])},
                    {"mime": "image/jpeg", "data": BASE64.encode([0xFF, 0xD8, 0xFF, 2])}
                ]
            })
        );
    }

    #[test]
    fn what_was_not_given_is_sent_as_null_not_left_out() {
        let enquiry =
            Enquiry::new(None, vec![a_photo(1)], None, Language::Sorani).expect("enquiry");
        let consultation = Consultation::new(a_brief(), vec![], enquiry);

        let body = serde_json::to_value(AskBody::from(&consultation)).expect("json");

        assert_eq!(body["history"], json!(null));
        assert_eq!(body["question"], json!(null));
        assert_eq!(body["cell"], json!(null));
        assert_eq!(body["lang"], json!("ku"));
    }

    #[test]
    fn a_full_answer_is_read_field_by_field() {
        let body = json!({
            "likely": "Yellow rust", "confidence": "likely", "why": ["weather -> wet"],
            "actions_this_week": ["Walk the field"], "cannot_tell": [],
            "refer_to_officer": true, "ku": "ژەنگ", "en": "Rust",
            "inputs_used": ["weather", "field_eye"]
        });

        let reply = reply_from(200, body.to_string().as_bytes()).expect("reply");

        assert_eq!(reply.likely.as_deref(), Some("Yellow rust"));
        assert_eq!(reply.refer_to_officer, Some(true));
        assert_eq!(
            reply.inputs_used,
            Some(vec!["weather".to_string(), "field_eye".to_string()])
        );
    }

    #[test]
    fn missing_fields_are_left_for_the_domain_to_judge() {
        let reply = reply_from(200, br#"{"likely": "Rust"}"#).expect("reply");

        assert_eq!(reply.likely.as_deref(), Some("Rust"));
        assert_eq!(reply.why, None);
    }

    #[test]
    fn a_not_ready_service_is_doctor_not_ready() {
        assert!(matches!(
            reply_from(503, br#"{"error": "doctor_not_ready"}"#),
            Err(AppError::DoctorNotReady)
        ));
    }

    #[test]
    fn every_other_failure_is_doctor_failed() {
        let failures: [(u16, &[u8]); 6] = [
            (502, br#"{"error": "doctor_failed"}"#),
            (503, b"Service Unavailable"),
            (500, b""),
            (404, b"not found"),
            (200, b"<html>not json</html>"),
            (200, br#"{"likely": "Rust", "why": "not a list"}"#),
        ];

        for (status, body) in failures {
            assert!(
                matches!(reply_from(status, body), Err(AppError::DoctorFailed)),
                "{status} {:?} must be doctor_failed",
                String::from_utf8_lossy(body)
            );
        }
    }

    #[tokio::test]
    async fn a_service_that_is_not_running_is_doctor_failed() {
        // Port 9 (discard) has no listener on a developer machine, so the
        // connection is refused at once.
        let doctor = HttpDoctor::new("http://127.0.0.1:9/").expect("client");
        let consultation = Consultation::new(a_brief(), vec![], an_enquiry());

        assert!(matches!(
            doctor.ask(&consultation).await,
            Err(AppError::DoctorFailed)
        ));
    }

    #[test]
    fn the_ask_path_is_added_once_whatever_the_base_ends_with() {
        for base in ["http://127.0.0.1:8090", "http://127.0.0.1:8090/"] {
            assert_eq!(
                HttpDoctor::new(base).expect("client").ask_url,
                "http://127.0.0.1:8090/ask"
            );
        }
    }
}
