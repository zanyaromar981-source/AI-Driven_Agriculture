use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, TimeZone, Utc};

use crate::{
    app::{AuthContext, User},
    features::plans::{
        app::{AppError, PlanFarms, PlanRepository},
        domain::{
            AlertLevel, AlertType, DailyValues, DecisionCode, FarmPlan, PlanAlert, PlanDecision,
            PlanSite, PlanSource, PlanStamp, PlanText,
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
    Exists { farm_id: i32 },
    AllSites,
    FindByFarm { farm_id: i32 },
    FindAllStamps,
    Upsert { farm_id: i32 },
}

#[derive(Debug, Default)]
struct Script {
    owned_farm: Option<i32>,
    sites: Vec<PlanSite>,
    stored: Vec<FarmPlan>,
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

    pub fn with_stored(self, plan: FarmPlan) -> Self {
        self.script.lock().expect("script lock").stored.push(plan);
        self
    }

    pub fn failing(self) -> Self {
        self.script
            .lock()
            .expect("script lock")
            .fail_with_database_error = true;
        self
    }

    /// The plan as the fake holds it now, after the writes of a test.
    pub fn stored(&self, farm_id: i32) -> Option<FarmPlan> {
        self.script
            .lock()
            .expect("script lock")
            .stored
            .iter()
            .find(|plan| *plan.farm_id() == farm_id)
            .cloned()
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
impl PlanFarms for Fakes {
    async fn is_owned_by(&self, farm_id: i32, phone: &Phone) -> Result<bool, AppError> {
        self.record(Call::IsOwnedBy {
            farm_id,
            phone: String::from(phone),
        });
        self.guard()?;

        let owned = self.script.lock().expect("script lock").owned_farm;

        Ok(owned == Some(farm_id) && phone.as_str() == OWNER)
    }

    async fn exists(&self, farm_id: i32) -> Result<bool, AppError> {
        self.record(Call::Exists { farm_id });
        self.guard()?;

        Ok(self
            .script
            .lock()
            .expect("script lock")
            .sites
            .iter()
            .any(|site| *site.farm_id() == farm_id))
    }

    async fn all_sites(&self) -> Result<Vec<PlanSite>, AppError> {
        self.record(Call::AllSites);
        self.guard()?;

        Ok(self.script.lock().expect("script lock").sites.clone())
    }
}

#[async_trait]
impl PlanRepository for Fakes {
    async fn find_by_farm(&self, farm_id: i32) -> Result<Option<FarmPlan>, AppError> {
        self.record(Call::FindByFarm { farm_id });
        self.guard()?;

        Ok(self.stored(farm_id))
    }

    async fn find_all_stamps(&self) -> Result<Vec<PlanStamp>, AppError> {
        self.record(Call::FindAllStamps);
        self.guard()?;

        Ok(self
            .script
            .lock()
            .expect("script lock")
            .stored
            .iter()
            .map(|plan| PlanStamp::rehydrate(*plan.farm_id(), *plan.issued()))
            .collect())
    }

    /// Like the real one, an older `issued` does not replace a newer one.
    async fn upsert(&self, entity: &FarmPlan) -> Result<FarmPlan, AppError> {
        self.record(Call::Upsert {
            farm_id: *entity.farm_id(),
        });
        self.guard()?;

        let mut script = self.script.lock().expect("script lock");

        if let Some(kept) = script
            .stored
            .iter()
            .find(|plan| plan.farm_id() == entity.farm_id() && plan.issued() > entity.issued())
        {
            return Ok(kept.clone());
        }

        let stored = persisted(entity);
        script
            .stored
            .retain(|plan| plan.farm_id() != entity.farm_id());
        script.stored.push(stored.clone());

        Ok(stored)
    }
}

fn persisted(entity: &FarmPlan) -> FarmPlan {
    FarmPlan::rehydrate(
        1,
        *entity.farm_id(),
        *entity.from(),
        *entity.issued(),
        entity.daily().clone(),
        entity.alerts().clone(),
        entity.decisions().clone(),
        entity.source().clone(),
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

/// `hour` o'clock UTC on a day of October 2026.
pub fn a_time(day: u32, hour: u32) -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 10, day, hour, 0, 0)
        .single()
        .expect("time")
}

pub fn a_site(farm_id: i32) -> PlanSite {
    PlanSite::rehydrate(farm_id, 36.0305, 44.6005)
}

pub fn a_text(value: &str) -> PlanText {
    PlanText::new(value.to_string()).expect("text")
}

pub fn a_source() -> PlanSource {
    PlanSource::new("Open-Meteo (ECMWF/GraphCast family)".to_string()).expect("source")
}

pub fn ten_days() -> DailyValues {
    let numbers: Vec<Option<f64>> = (0..10).map(|day| Some(f64::from(day))).collect();

    DailyValues::new(numbers.clone(), numbers.clone(), numbers, None).expect("values")
}

pub fn an_alert(day: u32) -> PlanAlert {
    PlanAlert::new(
        AlertType::Frost,
        a_day(day),
        Some(-3.0),
        AlertLevel::Alarm,
        a_text("Frost -3 °C Sun night"),
        a_text("Frost -3 °C Sun night"),
    )
    .expect("alert")
}

pub fn a_decision() -> PlanDecision {
    PlanDecision::new(
        DecisionCode::FrostCheck,
        a_text("Hard frost on Sun."),
        a_text("Hard frost on Sun."),
    )
}

/// A persisted ten-day plan that starts on `day` of October 2026 and was
/// issued at 06:00 UTC that day, with one alert three days in.
pub fn a_plan(farm_id: i32, day: u32) -> FarmPlan {
    let plan = FarmPlan::new(
        farm_id,
        a_day(day),
        a_time(day, 6),
        ten_days(),
        vec![an_alert(day + 3)],
        vec![a_decision()],
        a_source(),
        a_time(day, 6),
    )
    .expect("plan");

    persisted(&plan)
}
