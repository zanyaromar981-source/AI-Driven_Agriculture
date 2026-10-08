use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use chrono::NaiveDate;

use crate::{
    app::{AuthContext, User},
    features::insights::{
        app::{AppError, FarmDirectory, FarmOwnership, InsightRepository},
        domain::{
            Confidence, FarmInsight, FarmSite, InsightSource, Measure, MeasureCode, ReadingStamp,
            Topic,
        },
    },
    shared::Phone,
};

pub const OWNER: &str = "+9647501234567";

/// The farm the fakes say `OWNER` owns.
pub const FARM_ID: i32 = 7;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Call {
    IsOwnedBy { farm_id: i32, phone: String },
    AllSites,
    FindAllByFarm { farm_id: i32 },
    FindAllStamps,
    Upsert { farm_id: i32, topic: Topic },
}

#[derive(Debug, Default)]
struct Script {
    owned_farm: Option<i32>,
    sites: Vec<FarmSite>,
    stored: Vec<FarmInsight>,
    fail_with_database_error: bool,
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

    /// `OWNER` owns `FARM_ID`, and the farms feature lists it.
    pub fn with_owned_farm(self) -> Self {
        {
            let mut script = self.script.lock().expect("script lock");
            script.owned_farm = Some(FARM_ID);
            script.sites.push(a_site(FARM_ID));
        }
        self
    }

    pub fn with_site(self, farm_id: i32) -> Self {
        self.script
            .lock()
            .expect("script lock")
            .sites
            .push(a_site(farm_id));
        self
    }

    pub fn with_stored(self, insight: FarmInsight) -> Self {
        self.script
            .lock()
            .expect("script lock")
            .stored
            .push(insight);
        self
    }

    pub fn failing(self) -> Self {
        self.script
            .lock()
            .expect("script lock")
            .fail_with_database_error = true;
        self
    }

    pub fn calls(&self) -> Vec<Call> {
        self.calls.lock().expect("calls lock").clone()
    }

    fn record(&self, call: Call) {
        self.calls.lock().expect("calls lock").push(call);
    }

    fn guard(&self) -> Result<(), AppError> {
        if self
            .script
            .lock()
            .expect("script lock")
            .fail_with_database_error
        {
            return Err(crate::app::AppError::DatabaseError("fake".to_string()).into());
        }

        Ok(())
    }
}

#[async_trait]
impl FarmOwnership for Fakes {
    async fn is_owned_by(&self, farm_id: i32, phone: &Phone) -> Result<bool, AppError> {
        self.record(Call::IsOwnedBy {
            farm_id,
            phone: String::from(phone),
        });
        self.guard()?;

        let owned = self.script.lock().expect("script lock").owned_farm;

        Ok(owned == Some(farm_id) && phone.as_str() == OWNER)
    }
}

#[async_trait]
impl FarmDirectory for Fakes {
    async fn all_sites(&self) -> Result<Vec<FarmSite>, AppError> {
        self.record(Call::AllSites);
        self.guard()?;

        Ok(self.script.lock().expect("script lock").sites.clone())
    }
}

#[async_trait]
impl InsightRepository for Fakes {
    async fn find_all_by_farm(&self, farm_id: i32) -> Result<Vec<FarmInsight>, AppError> {
        self.record(Call::FindAllByFarm { farm_id });
        self.guard()?;

        Ok(self
            .script
            .lock()
            .expect("script lock")
            .stored
            .iter()
            .filter(|insight| *insight.farm_id() == farm_id)
            .cloned()
            .collect())
    }

    async fn find_all_stamps(&self) -> Result<Vec<ReadingStamp>, AppError> {
        self.record(Call::FindAllStamps);
        self.guard()?;

        Ok(self
            .script
            .lock()
            .expect("script lock")
            .stored
            .iter()
            .map(|insight| {
                ReadingStamp::rehydrate(*insight.farm_id(), *insight.topic(), *insight.as_of())
            })
            .collect())
    }

    async fn upsert(&self, entity: &FarmInsight) -> Result<FarmInsight, AppError> {
        self.record(Call::Upsert {
            farm_id: *entity.farm_id(),
            topic: *entity.topic(),
        });
        self.guard()?;

        let stored = persisted(entity, 1);

        let mut script = self.script.lock().expect("script lock");
        script
            .stored
            .retain(|other| other.farm_id() != entity.farm_id() || other.topic() != entity.topic());
        script.stored.push(stored.clone());

        Ok(stored)
    }
}

fn persisted(entity: &FarmInsight, id: i32) -> FarmInsight {
    FarmInsight::rehydrate(
        id,
        *entity.farm_id(),
        *entity.topic(),
        *entity.as_of(),
        entity.source().clone(),
        *entity.confidence(),
        entity.summary_en().clone(),
        entity.summary_ku().clone(),
        entity.measures().clone(),
        *entity.updated_at(),
    )
}

pub fn phone() -> Phone {
    Phone::new(OWNER.to_string()).expect("phone")
}

pub fn auth_context() -> AuthContext {
    AuthContext::new(User::new(phone()), "token".to_string())
}

pub fn a_day(day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, 10, day).expect("date")
}

pub fn a_source() -> InsightSource {
    InsightSource::new("Sentinel-2".to_string()).expect("source")
}

pub fn a_measure(code: &str) -> Measure {
    Measure::new(
        MeasureCode::new(code.to_string()).expect("code"),
        54.2,
        "%".to_string(),
        "Dam level".to_string(),
        None,
    )
    .expect("measure")
}

pub fn a_site(farm_id: i32) -> FarmSite {
    FarmSite::rehydrate(farm_id, 36.0305, 44.6005, 4.0)
}

/// A persisted reading with one measure.
pub fn an_insight(farm_id: i32, topic: Topic, day: u32) -> FarmInsight {
    let insight = FarmInsight::new(
        farm_id,
        topic,
        a_day(day),
        a_source(),
        Confidence::Likely,
        None,
        None,
        vec![a_measure("level_pct")],
    )
    .expect("insight");

    persisted(&insight, 1)
}
