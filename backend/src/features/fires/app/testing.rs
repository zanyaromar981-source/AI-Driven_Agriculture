use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};

use crate::{
    app::{Pagination, Permission, StaffContext},
    features::fires::{
        app::{AppError, FireFilter, FireRepository},
        domain::{
            ExternalId, Fire, FireCorrection, FireLocation, FireSource, FireStatus, WindDirection,
            ZoneSlug,
        },
    },
};

pub const STAFF_ID: i32 = 4;

#[derive(Clone, Debug, PartialEq)]
pub enum RepositoryCall {
    FindDetectedSince {
        since: DateTime<Utc>,
    },
    Upsert {
        external_id: String,
    },
    FindPage {
        filter: FireFilter,
        page: u64,
        rows_per_page: u64,
    },
    FindById {
        id: i32,
    },
    Create {
        external_id: String,
    },
    Update {
        id: i32,
    },
    Delete {
        id: i32,
    },
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

    /// The fire as the fake holds it now, after the writes of a test.
    pub fn stored(&self, id: i32) -> Option<Fire> {
        self.script
            .lock()
            .expect("script lock")
            .stored
            .iter()
            .find(|fire| *fire.id() == Some(id))
            .cloned()
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

    async fn find_page(
        &self,
        filter: &FireFilter,
        pagination: &Pagination,
    ) -> Result<(Vec<Fire>, u64), AppError> {
        self.record(RepositoryCall::FindPage {
            filter: filter.clone(),
            page: *pagination.page(),
            rows_per_page: *pagination.rows_per_page(),
        });
        self.guard()?;

        let mut fires: Vec<Fire> = self
            .script
            .lock()
            .expect("script lock")
            .stored
            .iter()
            .filter(|fire| filter.span.contains(*fire.detected_at()))
            .filter(|fire| filter.status.is_none_or(|status| *fire.status() == status))
            .filter(|fire| {
                filter
                    .zone_slug
                    .as_ref()
                    .is_none_or(|zone| fire.zone_slug().as_ref() == Some(zone))
            })
            .cloned()
            .collect();

        fires.sort_by(|first, second| second.detected_at().cmp(first.detected_at()));

        let count = fires.len() as u64;
        let page = fires
            .into_iter()
            .skip(pagination.skip() as usize)
            .take(*pagination.rows_per_page() as usize)
            .collect();

        Ok((page, count))
    }

    async fn find_by_id(&self, id: i32) -> Result<Option<Fire>, AppError> {
        self.record(RepositoryCall::FindById { id });
        self.guard()?;

        Ok(self.stored(id))
    }

    async fn create(&self, entity: &Fire) -> Result<Option<Fire>, AppError> {
        self.record(RepositoryCall::Create {
            external_id: String::from(entity.external_id()),
        });
        self.guard()?;

        let mut script = self.script.lock().expect("script lock");

        if script
            .stored
            .iter()
            .any(|fire| fire.external_id() == entity.external_id())
        {
            return Ok(None);
        }

        let next_id = script
            .stored
            .iter()
            .filter_map(|fire| *fire.id())
            .max()
            .unwrap_or(0)
            + 1;
        let stored = persisted(entity, next_id);
        script.stored.push(stored.clone());

        Ok(Some(stored))
    }

    async fn update(&self, id: i32, correction: &FireCorrection) -> Result<Option<Fire>, AppError> {
        self.record(RepositoryCall::Update { id });
        self.guard()?;

        let mut script = self.script.lock().expect("script lock");

        let Some(slot) = script.stored.iter_mut().find(|fire| *fire.id() == Some(id)) else {
            return Ok(None);
        };

        *slot = Fire::rehydrate(
            id,
            slot.external_id().clone(),
            *correction.location(),
            correction.zone_slug().clone(),
            correction.place_en().clone(),
            correction.place_ku().clone(),
            *correction.detected_at(),
            *correction.area_ha(),
            *correction.wind_kmh(),
            *correction.wind_direction(),
            *correction.status(),
            *correction.farms_within_5km(),
            *correction.farmers_alerted(),
            correction.source().clone(),
            *correction.updated_at(),
        );

        Ok(Some(slot.clone()))
    }

    async fn delete(&self, id: i32) -> Result<(), AppError> {
        self.record(RepositoryCall::Delete { id });
        self.guard()?;

        self.script
            .lock()
            .expect("script lock")
            .stored
            .retain(|fire| *fire.id() != Some(id));

        Ok(())
    }
}

/// The signed-in staff member a dashboard use case is acting for.
pub fn actor() -> StaffContext {
    StaffContext::new(
        STAFF_ID,
        "officer@example.org".to_string(),
        Permission::all().into_iter().collect(),
    )
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
