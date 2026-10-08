use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};

use crate::features::fires::{
    app::{AppError, FireRepository},
    domain::{ExternalId, Fire, FireLocation, FireSource, FireStatus, WindDirection, ZoneSlug},
};

#[derive(Clone, Debug, PartialEq)]
pub enum RepositoryCall {
    FindDetectedSince { since: DateTime<Utc> },
    Upsert { external_id: String },
}

#[derive(Debug, Default)]
struct Script {
    stored: Vec<Fire>,
    fail_with_database_error: bool,
}

#[derive(Debug, Clone, Default)]
pub struct FakeFireRepository {
    script: Arc<Mutex<Script>>,
    calls: Arc<Mutex<Vec<RepositoryCall>>>,
}

impl FakeFireRepository {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn holding(stored: Vec<Fire>) -> Self {
        let fake = Self::new();
        fake.script.lock().expect("script lock").stored = stored;
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
impl FireRepository for FakeFireRepository {
    async fn find_detected_since(&self, since: DateTime<Utc>) -> Result<Vec<Fire>, AppError> {
        self.record(RepositoryCall::FindDetectedSince { since });
        self.guard()?;

        let mut fires: Vec<Fire> = self
            .script
            .lock()
            .expect("script lock")
            .stored
            .iter()
            .filter(|fire| *fire.detected_at() >= since)
            .cloned()
            .collect();

        fires.sort_by(|first, second| second.detected_at().cmp(first.detected_at()));

        Ok(fires)
    }

    async fn upsert(&self, entity: &Fire) -> Result<Fire, AppError> {
        self.record(RepositoryCall::Upsert {
            external_id: String::from(entity.external_id()),
        });
        self.guard()?;

        Ok(persisted(entity, 1))
    }
}

fn persisted(entity: &Fire, id: i32) -> Fire {
    Fire::rehydrate(
        id,
        entity.external_id().clone(),
        *entity.location(),
        entity.zone_slug().clone(),
        entity.place_en().clone(),
        entity.place_ku().clone(),
        *entity.detected_at(),
        *entity.area_ha(),
        *entity.wind_kmh(),
        *entity.wind_direction(),
        *entity.status(),
        *entity.farms_within_5km(),
        *entity.farmers_alerted(),
        entity.source().clone(),
        *entity.updated_at(),
    )
}

pub fn an_external_id(value: &str) -> ExternalId {
    ExternalId::new(value.to_string()).expect("external id")
}

pub fn a_location() -> FireLocation {
    FireLocation::new(35.53, 44.83).expect("location")
}

pub fn a_source() -> FireSource {
    FireSource::new("NASA FIRMS".to_string()).expect("source")
}

/// A persisted fire in the zone `chamchamal`, detected that many hours ago.
pub fn a_fire(id: i32, status: FireStatus, hours_ago: i64) -> Fire {
    let fire = Fire::new(
        an_external_id(&format!("firms-{id}")),
        a_location(),
        Some(ZoneSlug::new("chamchamal".to_string()).expect("zone")),
        None,
        None,
        Utc::now() - Duration::hours(hours_ago),
        Some(10.0),
        Some(20.0),
        Some(WindDirection::Ne),
        status,
        Some(3),
        Some(5),
        a_source(),
    )
    .expect("fire");

    persisted(&fire, id)
}
