use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use chrono::NaiveDate;

use crate::{
    app::{Pagination, StaffContext},
    features::dams::{
        app::{AppError, DamRepository},
        domain::{Dam, DamReading, DamSlug, PercentFull, ReadingSource},
    },
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RepositoryCall {
    FindAll,
    FindBySlug {
        slug: String,
    },
    FindLatestReading {
        dam_id: i32,
    },
    FindReadingsBetween {
        dam_id: i32,
        from: NaiveDate,
        to: NaiveDate,
    },
    UpsertReading {
        dam_id: i32,
        day: NaiveDate,
    },
    FindReadingsPage {
        dam_id: i32,
        from: Option<NaiveDate>,
        to: Option<NaiveDate>,
        page: u64,
        rows_per_page: u64,
    },
    CreateReading {
        dam_id: i32,
        day: NaiveDate,
    },
    UpdateReading {
        dam_id: i32,
        day: NaiveDate,
    },
    DeleteReading {
        dam_id: i32,
        day: NaiveDate,
    },
}

#[derive(Debug, Default)]
struct Script {
    dams: Vec<Dam>,
    readings: Vec<DamReading>,
    fail_with_database_error: bool,
}

#[derive(Debug, Clone, Default)]
pub struct FakeDamRepository {
    script: Arc<Mutex<Script>>,
    calls: Arc<Mutex<Vec<RepositoryCall>>>,
}

impl FakeDamRepository {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn holding(dams: Vec<Dam>, readings: Vec<DamReading>) -> Self {
        let fake = Self::new();
        {
            let mut script = fake.script.lock().expect("script lock");
            script.dams = dams;
            script.readings = readings;
        }
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
impl DamRepository for FakeDamRepository {
    async fn find_all(&self) -> Result<Vec<Dam>, AppError> {
        self.record(RepositoryCall::FindAll);
        self.guard()?;

        Ok(self.script.lock().expect("script lock").dams.clone())
    }

    async fn find_by_slug(&self, slug: &DamSlug) -> Result<Option<Dam>, AppError> {
        self.record(RepositoryCall::FindBySlug {
            slug: String::from(slug),
        });
        self.guard()?;

        let script = self.script.lock().expect("script lock");

        Ok(script.dams.iter().find(|dam| dam.slug() == slug).cloned())
    }

    async fn find_latest_reading(&self, dam_id: i32) -> Result<Option<DamReading>, AppError> {
        self.record(RepositoryCall::FindLatestReading { dam_id });
        self.guard()?;

        let script = self.script.lock().expect("script lock");

        Ok(script
            .readings
            .iter()
            .filter(|reading| *reading.dam_id() == dam_id)
            .max_by_key(|reading| *reading.day())
            .cloned())
    }

    async fn find_readings_between(
        &self,
        dam_id: i32,
        from: NaiveDate,
        to: NaiveDate,
    ) -> Result<Vec<DamReading>, AppError> {
        self.record(RepositoryCall::FindReadingsBetween { dam_id, from, to });
        self.guard()?;

        let script = self.script.lock().expect("script lock");

        let mut readings: Vec<DamReading> = script
            .readings
            .iter()
            .filter(|reading| {
                *reading.dam_id() == dam_id && *reading.day() >= from && *reading.day() <= to
            })
            .cloned()
            .collect();
        readings.sort_by_key(|reading| *reading.day());

        Ok(readings)
    }

    async fn upsert_reading(&self, reading: &DamReading) -> Result<DamReading, AppError> {
        self.record(RepositoryCall::UpsertReading {
            dam_id: *reading.dam_id(),
            day: *reading.day(),
        });
        self.guard()?;

        Ok(DamReading::rehydrate(
            1,
            *reading.dam_id(),
            *reading.day(),
            *reading.pct_full(),
            *reading.volume_bn_m3(),
            *reading.lake_area_km2(),
            *reading.farm_supply_bn_m3(),
            reading.source().clone(),
            *reading.updated_at(),
        ))
    }

    async fn find_readings_page(
        &self,
        dam_id: i32,
        from: Option<NaiveDate>,
        to: Option<NaiveDate>,
        pagination: &Pagination,
    ) -> Result<(Vec<DamReading>, u64), AppError> {
        self.record(RepositoryCall::FindReadingsPage {
            dam_id,
            from,
            to,
            page: *pagination.page(),
            rows_per_page: *pagination.rows_per_page(),
        });
        self.guard()?;

        let script = self.script.lock().expect("script lock");

        let mut readings: Vec<DamReading> = script
            .readings
            .iter()
            .filter(|reading| {
                *reading.dam_id() == dam_id
                    && from.is_none_or(|from| *reading.day() >= from)
                    && to.is_none_or(|to| *reading.day() <= to)
            })
            .cloned()
            .collect();
        readings.sort_by_key(|reading| std::cmp::Reverse(*reading.day()));

        let count = readings.len() as u64;

        Ok((
            readings
                .into_iter()
                .skip(pagination.skip() as usize)
                .take(*pagination.rows_per_page() as usize)
                .collect(),
            count,
        ))
    }

    async fn create_reading(&self, reading: &DamReading) -> Result<Option<DamReading>, AppError> {
        self.record(RepositoryCall::CreateReading {
            dam_id: *reading.dam_id(),
            day: *reading.day(),
        });
        self.guard()?;

        let mut script = self.script.lock().expect("script lock");

        if script
            .readings
            .iter()
            .any(|stored| stored.dam_id() == reading.dam_id() && stored.day() == reading.day())
        {
            return Ok(None);
        }

        let stored = persisted(reading);
        script.readings.push(stored.clone());

        Ok(Some(stored))
    }

    async fn update_reading(&self, reading: &DamReading) -> Result<Option<DamReading>, AppError> {
        self.record(RepositoryCall::UpdateReading {
            dam_id: *reading.dam_id(),
            day: *reading.day(),
        });
        self.guard()?;

        let mut script = self.script.lock().expect("script lock");

        let Some(stored) = script
            .readings
            .iter_mut()
            .find(|stored| stored.dam_id() == reading.dam_id() && stored.day() == reading.day())
        else {
            return Ok(None);
        };

        *stored = persisted(reading);

        Ok(Some(stored.clone()))
    }

    async fn delete_reading(&self, dam_id: i32, day: NaiveDate) -> Result<bool, AppError> {
        self.record(RepositoryCall::DeleteReading { dam_id, day });
        self.guard()?;

        let mut script = self.script.lock().expect("script lock");
        let before = script.readings.len();

        script
            .readings
            .retain(|reading| !(*reading.dam_id() == dam_id && *reading.day() == day));

        Ok(script.readings.len() < before)
    }
}

fn persisted(reading: &DamReading) -> DamReading {
    DamReading::rehydrate(
        1,
        *reading.dam_id(),
        *reading.day(),
        *reading.pct_full(),
        *reading.volume_bn_m3(),
        *reading.lake_area_km2(),
        *reading.farm_supply_bn_m3(),
        reading.source().clone(),
        *reading.updated_at(),
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

pub fn day(year: i32, month: u32, day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(year, month, day).expect("day")
}

pub fn slug(value: &str) -> DamSlug {
    DamSlug::new(value.to_string()).expect("slug")
}

/// Dukan, id 1, 6.97 billion m3.
pub fn dukan() -> Dam {
    Dam::rehydrate(
        1,
        slug("dukan"),
        "Dukan".to_string(),
        "دووکان".to_string(),
        6.97,
    )
}

/// Darbandikhan, id 2, 3.0 billion m3.
pub fn darbandikhan() -> Dam {
    Dam::rehydrate(
        2,
        slug("darbandikhan"),
        "Darbandikhan".to_string(),
        "دەربەندیخان".to_string(),
        3.0,
    )
}

/// A persisted reading for the dam on the day.
pub fn a_reading(dam: &Dam, on: NaiveDate, pct_full: f64) -> DamReading {
    DamReading::rehydrate(
        1,
        *dam.id(),
        on,
        PercentFull::new(pct_full).expect("percent"),
        None,
        None,
        None,
        ReadingSource::new("test".to_string()).expect("source"),
        chrono::Utc::now(),
    )
}
