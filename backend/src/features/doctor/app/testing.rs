use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use chrono::NaiveDate;

use crate::{
    app::{AuthContext, User},
    features::doctor::{
        app::{AppError, Doctor, FarmBriefs, FarmHistory},
        domain::{
            Consultation, DoctorReply, Enquiry, FarmBrief, HistoryMeasure, HistoryTopic, Language,
            Photo, PhotoType, TappedCell,
        },
    },
    shared::Phone,
};

pub const OWNER: &str = "+9647501234567";

/// The farm the fakes say `OWNER` owns.
pub const FARM_ID: i32 = 5;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Call {
    BriefFor { farm_id: i32, owner: String },
    HistoryOf { farm_id: i32 },
    Ask { farm_id: i32 },
}

/// What the fake Doctor service does when asked.
#[derive(Clone, Debug)]
enum Behaviour {
    Answers(DoctorReply),
    NotReady,
    Fails,
}

#[derive(Debug)]
struct Script {
    owned: Option<FarmBrief>,
    history: Vec<HistoryTopic>,
    doctor: Behaviour,
    consultations: Vec<Consultation>,
}

impl Default for Script {
    fn default() -> Self {
        Self {
            owned: None,
            history: Vec::new(),
            doctor: Behaviour::Answers(a_reply()),
            consultations: Vec::new(),
        }
    }
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

    /// `OWNER` owns `FARM_ID`, described by `a_brief()`.
    pub fn with_owned_farm(self) -> Self {
        self.script.lock().expect("script lock").owned = Some(a_brief());
        self
    }

    pub fn with_history(self, topic: HistoryTopic) -> Self {
        self.script.lock().expect("script lock").history.push(topic);
        self
    }

    pub fn answering(self, reply: DoctorReply) -> Self {
        self.script.lock().expect("script lock").doctor = Behaviour::Answers(reply);
        self
    }

    /// The Doctor service is up but has no AI key yet.
    pub fn with_doctor_not_ready(self) -> Self {
        self.script.lock().expect("script lock").doctor = Behaviour::NotReady;
        self
    }

    pub fn with_doctor_failing(self) -> Self {
        self.script.lock().expect("script lock").doctor = Behaviour::Fails;
        self
    }

    pub fn calls(&self) -> Vec<Call> {
        self.calls.lock().expect("calls lock").clone()
    }

    /// Everything the Doctor was given, one entry per question.
    pub fn consultations(&self) -> Vec<Consultation> {
        self.script
            .lock()
            .expect("script lock")
            .consultations
            .clone()
    }

    fn record(&self, call: Call) {
        self.calls.lock().expect("calls lock").push(call);
    }
}

#[async_trait]
impl FarmBriefs for Fakes {
    async fn brief_for(&self, farm_id: i32, owner: &Phone) -> Result<Option<FarmBrief>, AppError> {
        self.record(Call::BriefFor {
            farm_id,
            owner: String::from(owner),
        });

        let script = self.script.lock().expect("script lock");

        Ok(script
            .owned
            .clone()
            .filter(|brief| *brief.id() == farm_id && owner.as_str() == OWNER))
    }
}

#[async_trait]
impl FarmHistory for Fakes {
    async fn history_of(&self, farm_id: i32) -> Result<Vec<HistoryTopic>, AppError> {
        self.record(Call::HistoryOf { farm_id });

        Ok(self.script.lock().expect("script lock").history.clone())
    }
}

#[async_trait]
impl Doctor for Fakes {
    async fn ask(&self, consultation: &Consultation) -> Result<DoctorReply, AppError> {
        self.record(Call::Ask {
            farm_id: *consultation.farm().id(),
        });

        let mut script = self.script.lock().expect("script lock");
        script.consultations.push(consultation.clone());

        match &script.doctor {
            Behaviour::Answers(reply) => Ok(reply.clone()),
            Behaviour::NotReady => Err(AppError::DoctorNotReady),
            Behaviour::Fails => Err(AppError::DoctorFailed),
        }
    }
}

pub fn auth_context() -> AuthContext {
    AuthContext::new(
        User::new(Phone::new(OWNER.to_string()).expect("phone")),
        "token".to_string(),
    )
}

pub fn another_farmer() -> AuthContext {
    AuthContext::new(
        User::new(Phone::new("+9647709876543".to_string()).expect("phone")),
        "token".to_string(),
    )
}

pub fn a_brief() -> FarmBrief {
    FarmBrief::rehydrate(
        FARM_ID,
        "Upper field".to_string(),
        36.0305,
        44.6005,
        19_451.0,
        vec!["wheat".to_string()],
    )
}

pub fn a_topic() -> HistoryTopic {
    HistoryTopic::rehydrate(
        "rain".to_string(),
        NaiveDate::from_ymd_opt(2026, 10, 8).expect("date"),
        "CHIRPS daily rain".to_string(),
        "likely".to_string(),
        Some("A dry month".to_string()),
        None,
        vec![HistoryMeasure::rehydrate(
            "rain_mm_30d".to_string(),
            3.5,
            "mm".to_string(),
            "Rain, last 30 days".to_string(),
            None,
        )],
    )
}

/// A tiny JPEG: its signature followed by `marker`, so two photos can be
/// told apart.
pub fn a_photo(marker: u8) -> Photo {
    let mut bytes = PhotoType::Jpeg.signature().to_vec();
    bytes.push(marker);

    Photo::new(PhotoType::Jpeg, bytes).expect("photo")
}

pub fn an_enquiry() -> Enquiry {
    Enquiry::new(
        Some("Why are the leaves yellow?".to_string()),
        vec![a_photo(1), a_photo(2)],
        Some(TappedCell::new(46_415, 398_748)),
        Language::English,
    )
    .expect("enquiry")
}

pub fn a_reply() -> DoctorReply {
    DoctorReply {
        likely: Some("Yellow rust".to_string()),
        confidence: Some("likely".to_string()),
        why: Some(vec!["weather -> cool and wet".to_string()]),
        actions_this_week: Some(vec!["Walk the field".to_string()]),
        cannot_tell: Some(vec![]),
        refer_to_officer: Some(false),
        ku: Some("ژەنگی زەرد".to_string()),
        en: Some("Probably yellow rust.".to_string()),
        inputs_used: Some(vec!["weather".to_string()]),
    }
}
