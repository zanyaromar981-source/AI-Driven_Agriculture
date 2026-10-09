use std::sync::{Arc, Mutex};

use async_trait::async_trait;

use crate::{
    app::StaffContext,
    features::water::{
        app::{AppError, WaterPlanRepository},
        domain::{DamSlug, Need, Season, WaterPlanEntry, ZoneSlug},
    },
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RepositoryCall {
    FindLatestSeason,
    FindBySeason { season: String },
    Upsert { season: String, zone_slug: String },
    Delete { season: String, zone_slug: String },
    FindSeasons,
    Create { season: String, zone_slug: String },
    Update { season: String, zone_slug: String },
}

#[derive(Debug, Default)]
struct Script {
    entries: Vec<WaterPlanEntry>,
    fail_with_database_error: bool,
}

#[derive(Debug, Clone, Default)]
pub struct FakeWaterPlanRepository {
    script: Arc<Mutex<Script>>,
    calls: Arc<Mutex<Vec<RepositoryCall>>>,
}

impl FakeWaterPlanRepository {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn holding(entries: Vec<WaterPlanEntry>) -> Self {
        let fake = Self::new();
        fake.script.lock().expect("script lock").entries = entries;
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
impl WaterPlanRepository for FakeWaterPlanRepository {
    async fn find_latest_season(&self) -> Result<Option<Season>, AppError> {
        self.record(RepositoryCall::FindLatestSeason);
        self.guard()?;

        let script = self.script.lock().expect("script lock");

        Ok(script
            .entries
            .iter()
            .map(|entry| entry.season().clone())
            .max())
    }

    async fn find_by_season(&self, season: &Season) -> Result<Vec<WaterPlanEntry>, AppError> {
        self.record(RepositoryCall::FindBySeason {
            season: String::from(season),
        });
        self.guard()?;

        let script = self.script.lock().expect("script lock");

        Ok(script
            .entries
            .iter()
            .filter(|entry| entry.season() == season)
            .cloned()
            .collect())
    }

    async fn upsert(&self, entry: &WaterPlanEntry) -> Result<WaterPlanEntry, AppError> {
        self.record(RepositoryCall::Upsert {
            season: String::from(entry.season()),
            zone_slug: String::from(entry.zone_slug()),
        });
        self.guard()?;

        Ok(WaterPlanEntry::rehydrate(
            1,
            entry.season().clone(),
            entry.zone_slug().clone(),
            *entry.need(),
            entry.dam_slug().clone(),
            *entry.send_million_m3(),
            *entry.urgent(),
            entry.note_en().clone(),
            entry.note_ku().clone(),
            *entry.updated_at(),
        ))
    }

    async fn delete(&self, season: &Season, zone_slug: &ZoneSlug) -> Result<bool, AppError> {
        self.record(RepositoryCall::Delete {
            season: String::from(season),
            zone_slug: String::from(zone_slug),
        });
        self.guard()?;

        let mut script = self.script.lock().expect("script lock");
        let before = script.entries.len();

        script
            .entries
            .retain(|entry| !(entry.season() == season && entry.zone_slug() == zone_slug));

        Ok(script.entries.len() < before)
    }

    async fn find_seasons(&self) -> Result<Vec<Season>, AppError> {
        self.record(RepositoryCall::FindSeasons);
        self.guard()?;

        let script = self.script.lock().expect("script lock");

        let mut seasons: Vec<Season> = script
            .entries
            .iter()
            .map(|entry| entry.season().clone())
            .collect();
        seasons.sort_by(|first, second| second.cmp(first));
        seasons.dedup();

        Ok(seasons)
    }

    async fn create(&self, entry: &WaterPlanEntry) -> Result<Option<WaterPlanEntry>, AppError> {
        self.record(RepositoryCall::Create {
            season: String::from(entry.season()),
            zone_slug: String::from(entry.zone_slug()),
        });
        self.guard()?;

        let mut script = self.script.lock().expect("script lock");

        if script.entries.iter().any(|stored| {
            stored.season() == entry.season() && stored.zone_slug() == entry.zone_slug()
        }) {
            return Ok(None);
        }

        let stored = persisted(entry);
        script.entries.push(stored.clone());

        Ok(Some(stored))
    }

    async fn update(&self, entry: &WaterPlanEntry) -> Result<Option<WaterPlanEntry>, AppError> {
        self.record(RepositoryCall::Update {
            season: String::from(entry.season()),
            zone_slug: String::from(entry.zone_slug()),
        });
        self.guard()?;

        let mut script = self.script.lock().expect("script lock");

        let Some(stored) = script.entries.iter_mut().find(|stored| {
            stored.season() == entry.season() && stored.zone_slug() == entry.zone_slug()
        }) else {
            return Ok(None);
        };

        *stored = persisted(entry);

        Ok(Some(stored.clone()))
    }
}

fn persisted(entry: &WaterPlanEntry) -> WaterPlanEntry {
    WaterPlanEntry::rehydrate(
        1,
        entry.season().clone(),
        entry.zone_slug().clone(),
        *entry.need(),
        entry.dam_slug().clone(),
        *entry.send_million_m3(),
        *entry.urgent(),
        entry.note_en().clone(),
        entry.note_ku().clone(),
        *entry.updated_at(),
    )
}

/// Staff member 7, holding no permission: the routes check those, the use
/// cases only record who acted.
pub fn staff() -> StaffContext {
    StaffContext::new(
        7,
        "officer@example.org".to_string(),
        std::collections::HashSet::new(),
    )
}

pub fn season(value: &str) -> Season {
    Season::new(value.to_string()).expect("season")
}

pub fn zone_slug(value: &str) -> ZoneSlug {
    ZoneSlug::new(value.to_string()).expect("zone")
}

pub fn dam_slug(value: &str) -> DamSlug {
    DamSlug::new(value.to_string()).expect("dam")
}

/// A persisted entry without notes.
pub fn an_entry(
    for_season: &str,
    zone: &str,
    need: f64,
    dam: Option<&str>,
    send_million_m3: Option<f64>,
    urgent: bool,
) -> WaterPlanEntry {
    WaterPlanEntry::rehydrate(
        1,
        season(for_season),
        zone_slug(zone),
        Need::new(need).expect("need"),
        dam.map(dam_slug),
        send_million_m3,
        urgent,
        None,
        None,
        chrono::Utc::now(),
    )
}
