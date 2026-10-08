use std::sync::{Arc, Mutex};

use async_trait::async_trait;

use crate::features::zones::{
    app::{AppError, ZoneRepository},
    domain::{Dryness, Month, ReadingSource, SubZone, SubZoneReading, Zone, ZoneReading, ZoneSlug},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RepositoryCall {
    FindAllZones,
    FindZoneBySlug {
        slug: String,
    },
    FindSubZonesByZone {
        zone_id: i32,
    },
    FindSubZoneBySlug {
        zone_id: i32,
        slug: String,
    },
    FindReadingMonths,
    FindReadingsInMonths {
        months: Vec<String>,
    },
    FindReadingsByZone {
        zone_id: i32,
    },
    FindSubZoneReadingsInMonth {
        sub_zone_ids: Vec<i32>,
        month: String,
    },
    UpsertReading {
        zone_id: i32,
        month: String,
    },
    UpsertSubZoneReading {
        sub_zone_id: i32,
        month: String,
    },
}

#[derive(Debug, Default)]
struct Script {
    zones: Vec<Zone>,
    sub_zones: Vec<SubZone>,
    readings: Vec<ZoneReading>,
    sub_zone_readings: Vec<SubZoneReading>,
    fail_with_database_error: bool,
}

#[derive(Debug, Clone, Default)]
pub struct FakeZoneRepository {
    script: Arc<Mutex<Script>>,
    calls: Arc<Mutex<Vec<RepositoryCall>>>,
}

impl FakeZoneRepository {
    /// Holds the three zones of `zones()` and the sub-zones of
    /// `sub_zones()`, with no readings.
    pub fn seeded() -> Self {
        let fake = Self::default();

        {
            let mut script = fake.script.lock().expect("script lock");
            script.zones = zones();
            script.sub_zones = sub_zones();
        }

        fake
    }

    pub fn with_readings(self, readings: Vec<ZoneReading>) -> Self {
        self.script.lock().expect("script lock").readings = readings;
        self
    }

    pub fn with_sub_zone_readings(self, readings: Vec<SubZoneReading>) -> Self {
        self.script.lock().expect("script lock").sub_zone_readings = readings;
        self
    }

    pub fn failing() -> Self {
        let fake = Self::seeded();
        fake.script
            .lock()
            .expect("script lock")
            .fail_with_database_error = true;
        fake
    }

    pub fn calls(&self) -> Vec<RepositoryCall> {
        self.calls.lock().expect("calls lock").clone()
    }

    pub fn wrote(&self) -> bool {
        self.calls().iter().any(|call| {
            matches!(
                call,
                RepositoryCall::UpsertReading { .. } | RepositoryCall::UpsertSubZoneReading { .. }
            )
        })
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
impl ZoneRepository for FakeZoneRepository {
    async fn find_all_zones(&self) -> Result<Vec<Zone>, AppError> {
        self.record(RepositoryCall::FindAllZones);
        self.guard()?;

        Ok(self.script.lock().expect("script lock").zones.clone())
    }

    async fn find_zone_by_slug(&self, slug: &ZoneSlug) -> Result<Option<Zone>, AppError> {
        self.record(RepositoryCall::FindZoneBySlug {
            slug: String::from(slug),
        });
        self.guard()?;

        let script = self.script.lock().expect("script lock");

        Ok(script
            .zones
            .iter()
            .find(|zone| zone.slug() == slug)
            .cloned())
    }

    async fn find_sub_zones_by_zone(&self, zone_id: i32) -> Result<Vec<SubZone>, AppError> {
        self.record(RepositoryCall::FindSubZonesByZone { zone_id });
        self.guard()?;

        let script = self.script.lock().expect("script lock");

        Ok(script
            .sub_zones
            .iter()
            .filter(|sub_zone| *sub_zone.zone_id() == zone_id)
            .cloned()
            .collect())
    }

    async fn find_sub_zone_by_slug(
        &self,
        zone_id: i32,
        slug: &ZoneSlug,
    ) -> Result<Option<SubZone>, AppError> {
        self.record(RepositoryCall::FindSubZoneBySlug {
            zone_id,
            slug: String::from(slug),
        });
        self.guard()?;

        let script = self.script.lock().expect("script lock");

        Ok(script
            .sub_zones
            .iter()
            .find(|sub_zone| *sub_zone.zone_id() == zone_id && sub_zone.slug() == slug)
            .cloned())
    }

    async fn find_reading_months(&self) -> Result<Vec<Month>, AppError> {
        self.record(RepositoryCall::FindReadingMonths);
        self.guard()?;

        let script = self.script.lock().expect("script lock");

        let mut months: Vec<Month> = script
            .readings
            .iter()
            .map(|reading| *reading.month())
            .collect();
        months.sort();
        months.dedup();

        Ok(months)
    }

    async fn find_readings_in_months(
        &self,
        months: &[Month],
    ) -> Result<Vec<ZoneReading>, AppError> {
        self.record(RepositoryCall::FindReadingsInMonths {
            months: months.iter().map(String::from).collect(),
        });
        self.guard()?;

        let script = self.script.lock().expect("script lock");

        Ok(script
            .readings
            .iter()
            .filter(|reading| months.contains(reading.month()))
            .cloned()
            .collect())
    }

    async fn find_readings_by_zone(&self, zone_id: i32) -> Result<Vec<ZoneReading>, AppError> {
        self.record(RepositoryCall::FindReadingsByZone { zone_id });
        self.guard()?;

        let script = self.script.lock().expect("script lock");

        Ok(script
            .readings
            .iter()
            .filter(|reading| *reading.zone_id() == zone_id)
            .cloned()
            .collect())
    }

    async fn find_sub_zone_readings_in_month(
        &self,
        sub_zone_ids: &[i32],
        month: Month,
    ) -> Result<Vec<SubZoneReading>, AppError> {
        self.record(RepositoryCall::FindSubZoneReadingsInMonth {
            sub_zone_ids: sub_zone_ids.to_vec(),
            month: String::from(month),
        });
        self.guard()?;

        let script = self.script.lock().expect("script lock");

        Ok(script
            .sub_zone_readings
            .iter()
            .filter(|reading| {
                sub_zone_ids.contains(reading.sub_zone_id()) && *reading.month() == month
            })
            .cloned()
            .collect())
    }

    async fn upsert_reading(&self, entity: &ZoneReading) -> Result<ZoneReading, AppError> {
        self.record(RepositoryCall::UpsertReading {
            zone_id: *entity.zone_id(),
            month: String::from(entity.month()),
        });
        self.guard()?;

        Ok(ZoneReading::rehydrate(
            1,
            *entity.zone_id(),
            *entity.month(),
            *entity.dryness(),
            *entity.rain_pct_of_normal(),
            *entity.greenness_pct_vs_normal(),
            *entity.water_need(),
            *entity.nitrogen_hold(),
            entity.best_crops().clone(),
            entity.source().clone(),
            *entity.updated_at(),
        ))
    }

    async fn upsert_sub_zone_reading(
        &self,
        entity: &SubZoneReading,
    ) -> Result<SubZoneReading, AppError> {
        self.record(RepositoryCall::UpsertSubZoneReading {
            sub_zone_id: *entity.sub_zone_id(),
            month: String::from(entity.month()),
        });
        self.guard()?;

        Ok(SubZoneReading::rehydrate(
            1,
            *entity.sub_zone_id(),
            *entity.month(),
            *entity.dryness(),
            *entity.updated_at(),
        ))
    }
}

pub fn a_slug(value: &str) -> ZoneSlug {
    ZoneSlug::new(value.to_string()).expect("slug")
}

pub fn a_month(value: &str) -> Month {
    Month::parse(value).expect("month")
}

pub fn a_dryness(value: i32) -> Dryness {
    Dryness::new(value).expect("dryness")
}

pub fn a_source() -> ReadingSource {
    ReadingSource::new("chirps+modis".to_string()).expect("source")
}

/// Chamchamal (id 1), Kalar (id 2) and Qushtapa (id 3).
pub fn zones() -> Vec<Zone> {
    vec![
        Zone::rehydrate(
            1,
            a_slug("chamchamal"),
            "Chamchamal".to_string(),
            "چەمچەماڵ".to_string(),
            "Sulaymaniyah".to_string(),
        ),
        Zone::rehydrate(
            2,
            a_slug("kalar"),
            "Kalar".to_string(),
            "کەلار".to_string(),
            "Sulaymaniyah".to_string(),
        ),
        Zone::rehydrate(
            3,
            a_slug("qushtapa"),
            "Qushtapa".to_string(),
            "دەشتی هەولێر".to_string(),
            "Erbil".to_string(),
        ),
    ]
}

/// Chamchamal's first three sub-zones (ids 1 to 3) and Kalar's first (id 4).
pub fn sub_zones() -> Vec<SubZone> {
    vec![
        SubZone::rehydrate(
            1,
            1,
            a_slug("markaz-chamchamal"),
            "Markaz Chamchamal".to_string(),
            "ناوەندی چەمچەماڵ".to_string(),
        ),
        SubZone::rehydrate(
            2,
            1,
            a_slug("aghjalar"),
            "Aghjalar".to_string(),
            "ئاغجەلەر".to_string(),
        ),
        SubZone::rehydrate(
            3,
            1,
            a_slug("sangaw"),
            "Sangaw".to_string(),
            "سەنگاو".to_string(),
        ),
        SubZone::rehydrate(
            4,
            2,
            a_slug("markaz-kalar"),
            "Markaz Kalar".to_string(),
            "ناوەندی کەلار".to_string(),
        ),
    ]
}

/// A reading that carries only a dryness.
pub fn a_reading(zone_id: i32, month: &str, dryness: i32) -> ZoneReading {
    ZoneReading::new(
        zone_id,
        a_month(month),
        a_dryness(dryness),
        None,
        None,
        None,
        false,
        vec![],
        a_source(),
    )
    .expect("reading")
}

pub fn a_sub_zone_reading(sub_zone_id: i32, month: &str, dryness: i32) -> SubZoneReading {
    SubZoneReading::new(sub_zone_id, a_month(month), a_dryness(dryness))
}
