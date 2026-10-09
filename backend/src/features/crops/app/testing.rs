use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use chrono::{DateTime, TimeZone, Utc};

use crate::{
    app::StaffContext,
    features::crops::{
        app::{AppError, CropRepository, CropUsage},
        domain::{
            Crop, CropCategory, CropCode, CropColor, CropDetails, CropName, CropSeason, SortOrder,
        },
    },
};

#[derive(Clone, Debug, PartialEq)]
pub enum RepositoryCall {
    FindAll { only_active: bool },
    Create { code: String },
    Update { code: String, details: CropDetails },
    Delete { code: String },
}

#[derive(Debug, Default)]
struct Script {
    crops: Vec<Crop>,
    fail_with_database_error: bool,
}

#[derive(Debug, Clone, Default)]
pub struct FakeCropRepository {
    script: Arc<Mutex<Script>>,
    calls: Arc<Mutex<Vec<RepositoryCall>>>,
}

impl FakeCropRepository {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn holding(crops: Vec<Crop>) -> Self {
        let fake = Self::new();
        fake.script.lock().expect("script lock").crops = crops;
        fake
    }

    pub fn failing() -> Self {
        let fake = Self::new();
        fake.script
            .lock()
            .expect("script lock")
            .fail_with_database_error = true;
        fake
    }

    pub fn calls(&self) -> Vec<RepositoryCall> {
        self.calls.lock().expect("calls lock").clone()
    }

    pub fn stored(&self, code: &str) -> Option<Crop> {
        self.script
            .lock()
            .expect("script lock")
            .crops
            .iter()
            .find(|crop| crop.code().as_str() == code)
            .cloned()
    }

    fn record(&self, call: RepositoryCall) {
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
impl CropRepository for FakeCropRepository {
    async fn find_all(&self, only_active: bool) -> Result<Vec<Crop>, AppError> {
        self.record(RepositoryCall::FindAll { only_active });
        self.guard()?;

        let script = self.script.lock().expect("script lock");

        let mut crops: Vec<Crop> = script
            .crops
            .iter()
            .filter(|crop| !only_active || crop.details().active)
            .cloned()
            .collect();
        crops.sort_by(|first, second| {
            (first.details().sort_order, first.code().as_str())
                .cmp(&(second.details().sort_order, second.code().as_str()))
        });

        Ok(crops)
    }

    async fn create(&self, crop: &Crop) -> Result<Option<Crop>, AppError> {
        self.record(RepositoryCall::Create {
            code: crop.code().into(),
        });
        self.guard()?;

        let mut script = self.script.lock().expect("script lock");

        if script.crops.iter().any(|known| known.code() == crop.code()) {
            return Ok(None);
        }

        script.crops.push(crop.clone());

        Ok(Some(crop.clone()))
    }

    async fn update(
        &self,
        code: &CropCode,
        details: &CropDetails,
        now: DateTime<Utc>,
    ) -> Result<Option<Crop>, AppError> {
        self.record(RepositoryCall::Update {
            code: code.into(),
            details: details.clone(),
        });
        self.guard()?;

        let mut script = self.script.lock().expect("script lock");

        let Some(position) = script.crops.iter().position(|crop| crop.code() == code) else {
            return Ok(None);
        };

        let updated = Crop::rehydrate(
            code.clone(),
            details.clone(),
            *script.crops[position].created_at(),
            now,
        );
        script.crops[position] = updated.clone();

        Ok(Some(updated))
    }

    async fn delete(&self, code: &CropCode) -> Result<bool, AppError> {
        self.record(RepositoryCall::Delete { code: code.into() });
        self.guard()?;

        let mut script = self.script.lock().expect("script lock");
        let before = script.crops.len();
        script.crops.retain(|crop| crop.code() != code);

        Ok(script.crops.len() < before)
    }
}

/// Stands for one feature that stores crop codes: it says the listed codes
/// are in use, and records every code it was asked about.
#[derive(Debug, Clone, Default)]
pub struct FakeCropUsage {
    used: Arc<Mutex<Vec<String>>>,
    asked: Arc<Mutex<Vec<String>>>,
    fails: bool,
}

impl FakeCropUsage {
    pub fn using(codes: &[&str]) -> Self {
        Self {
            used: Arc::new(Mutex::new(
                codes.iter().map(|code| code.to_string()).collect(),
            )),
            ..Self::default()
        }
    }

    pub fn failing() -> Self {
        Self {
            fails: true,
            ..Self::default()
        }
    }

    pub fn asked(&self) -> Vec<String> {
        self.asked.lock().expect("asked lock").clone()
    }
}

#[async_trait]
impl CropUsage for FakeCropUsage {
    async fn is_used(&self, code: &CropCode) -> Result<bool, AppError> {
        self.asked.lock().expect("asked lock").push(code.into());

        if self.fails {
            return Err(crate::app::AppError::InternalServerError.into());
        }

        Ok(self
            .used
            .lock()
            .expect("used lock")
            .iter()
            .any(|used| used == code.as_str()))
    }
}

pub fn staff() -> StaffContext {
    StaffContext::new(
        7,
        "officer@example.org".to_string(),
        std::collections::HashSet::new(),
    )
}

pub fn code(value: &str) -> CropCode {
    CropCode::new(value.to_string()).expect("code")
}

pub fn at(hour: u32) -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 10, 9, hour, 0, 0).unwrap()
}

pub fn details(name_en: &str, sort_order: i32, active: bool) -> CropDetails {
    CropDetails {
        name_en: CropName::new(name_en.to_string()).expect("name"),
        name_ku: None,
        color: CropColor::new("#e0b13a".to_string()).expect("colour"),
        category: CropCategory::Cereal,
        season: CropSeason::Winter,
        yield_kg_per_dunam: None,
        active,
        sort_order: SortOrder::new(sort_order).expect("order"),
    }
}

pub fn a_crop(code_value: &str, sort_order: i32, active: bool) -> Crop {
    Crop::rehydrate(
        code(code_value),
        details(code_value, sort_order, active),
        at(8),
        at(8),
    )
}
