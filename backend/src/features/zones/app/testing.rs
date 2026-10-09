use std::sync::{Arc, Mutex};

use async_trait::async_trait;

use crate::{
    app::{Pagination, Permission, StaffContext},
    features::zones::{
        app::{AppError, ZoneRepository},
        domain::{
            Dryness, Month, MonthRange, ReadingSource, Shape, SubZone, SubZoneReading, Zone,
            ZoneReading, ZoneSlug,
        },
    },
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
    FindAllSubZones,
    FindReadingsByZoneInRange {
        zone_id: i32,
        from: String,
        to: String,
        page: u64,
        rows_per_page: u64,
    },
    InsertReading {
        zone_id: i32,
        month: String,
    },
    UpdateReading {
        zone_id: i32,
        month: String,
    },
    DeleteReading {
        zone_id: i32,
        month: String,
    },
    FindSubZoneReadingsInRange {
        sub_zone_id: i32,
        from: String,
        to: String,
        page: u64,
        rows_per_page: u64,
    },
    InsertSubZoneReading {
        sub_zone_id: i32,
        month: String,
    },
    UpdateSubZoneReading {
        sub_zone_id: i32,
        month: String,
    },
    DeleteSubZoneReading {
        sub_zone_id: i32,
        month: String,
    },
    FindAllSubZoneShapes,
}

#[derive(Debug, Default)]
struct Script {
    zones: Vec<Zone>,
    sub_zones: Vec<SubZone>,
    readings: Vec<ZoneReading>,
    sub_zone_readings: Vec<SubZoneReading>,
    shapes: Vec<(SubZone, Shape)>,
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

    pub fn with_shapes(self, shapes: Vec<(SubZone, Shape)>) -> Self {
        self.script.lock().expect("script lock").shapes = shapes;
        self
    }

    /// Fails every call from now on, or succeeds again.
    pub fn fail(&self, failing: bool) {
        self.script
            .lock()
            .expect("script lock")
            .fail_with_database_error = failing;
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
                RepositoryCall::UpsertReading { .. }
                    | RepositoryCall::UpsertSubZoneReading { .. }
                    | RepositoryCall::InsertReading { .. }
                    | RepositoryCall::UpdateReading { .. }
                    | RepositoryCall::DeleteReading { .. }
                    | RepositoryCall::InsertSubZoneReading { .. }
                    | RepositoryCall::UpdateSubZoneReading { .. }
                    | RepositoryCall::DeleteSubZoneReading { .. }
            )
        })
    }

    /// The zone readings now held, in the order they were stored.
    pub fn stored_readings(&self) -> Vec<ZoneReading> {
        self.script.lock().expect("script lock").readings.clone()
    }

    /// The sub-zone readings now held, in the order they were stored.
    pub fn stored_sub_zone_readings(&self) -> Vec<SubZoneReading> {
        self.script
            .lock()
            .expect("script lock")
            .sub_zone_readings
            .clone()
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

    async fn find_all_sub_zones(&self) -> Result<Vec<SubZone>, AppError> {
        self.record(RepositoryCall::FindAllSubZones);
        self.guard()?;

        Ok(self.script.lock().expect("script lock").sub_zones.clone())
    }

    async fn find_readings_by_zone_in_range(
        &self,
        zone_id: i32,
        range: MonthRange,
        pagination: &Pagination,
    ) -> Result<(Vec<ZoneReading>, u64), AppError> {
        self.record(RepositoryCall::FindReadingsByZoneInRange {
            zone_id,
            from: String::from(range.from()),
            to: String::from(range.to()),
            page: *pagination.page(),
            rows_per_page: *pagination.rows_per_page(),
        });
        self.guard()?;

        let script = self.script.lock().expect("script lock");

        let mut matching: Vec<ZoneReading> = script
            .readings
            .iter()
            .filter(|reading| *reading.zone_id() == zone_id && range.contains(*reading.month()))
            .cloned()
            .collect();
        matching.sort_by(|a, b| b.month().cmp(a.month()));

        let count = matching.len() as u64;

        Ok((
            matching
                .into_iter()
                .skip(pagination.skip() as usize)
                .take(*pagination.rows_per_page() as usize)
                .collect(),
            count,
        ))
    }

    async fn insert_reading(&self, entity: &ZoneReading) -> Result<Option<ZoneReading>, AppError> {
        self.record(RepositoryCall::InsertReading {
            zone_id: *entity.zone_id(),
            month: String::from(entity.month()),
        });
        self.guard()?;

        let mut script = self.script.lock().expect("script lock");

        if script
            .readings
            .iter()
            .any(|kept| kept.zone_id() == entity.zone_id() && kept.month() == entity.month())
        {
            return Ok(None);
        }

        let stored = stored_reading(script.readings.len() as i32 + 1, entity);
        script.readings.push(stored.clone());

        Ok(Some(stored))
    }

    async fn update_reading(&self, entity: &ZoneReading) -> Result<Option<ZoneReading>, AppError> {
        self.record(RepositoryCall::UpdateReading {
            zone_id: *entity.zone_id(),
            month: String::from(entity.month()),
        });
        self.guard()?;

        let mut script = self.script.lock().expect("script lock");

        let Some(kept) = script
            .readings
            .iter_mut()
            .find(|kept| kept.zone_id() == entity.zone_id() && kept.month() == entity.month())
        else {
            return Ok(None);
        };

        *kept = stored_reading(kept.id().unwrap_or(1), entity);

        Ok(Some(kept.clone()))
    }

    async fn delete_reading(&self, zone_id: i32, month: Month) -> Result<bool, AppError> {
        self.record(RepositoryCall::DeleteReading {
            zone_id,
            month: String::from(month),
        });
        self.guard()?;

        let mut script = self.script.lock().expect("script lock");
        let before = script.readings.len();
        script
            .readings
            .retain(|kept| !(*kept.zone_id() == zone_id && *kept.month() == month));

        Ok(script.readings.len() < before)
    }

    async fn find_sub_zone_readings_in_range(
        &self,
        sub_zone_id: i32,
        range: MonthRange,
        pagination: &Pagination,
    ) -> Result<(Vec<SubZoneReading>, u64), AppError> {
        self.record(RepositoryCall::FindSubZoneReadingsInRange {
            sub_zone_id,
            from: String::from(range.from()),
            to: String::from(range.to()),
            page: *pagination.page(),
            rows_per_page: *pagination.rows_per_page(),
        });
        self.guard()?;

        let script = self.script.lock().expect("script lock");

        let mut matching: Vec<SubZoneReading> = script
            .sub_zone_readings
            .iter()
            .filter(|reading| {
                *reading.sub_zone_id() == sub_zone_id && range.contains(*reading.month())
            })
            .cloned()
            .collect();
        matching.sort_by(|a, b| b.month().cmp(a.month()));

        let count = matching.len() as u64;

        Ok((
            matching
                .into_iter()
                .skip(pagination.skip() as usize)
                .take(*pagination.rows_per_page() as usize)
                .collect(),
            count,
        ))
    }

    async fn insert_sub_zone_reading(
        &self,
        entity: &SubZoneReading,
    ) -> Result<Option<SubZoneReading>, AppError> {
        self.record(RepositoryCall::InsertSubZoneReading {
            sub_zone_id: *entity.sub_zone_id(),
            month: String::from(entity.month()),
        });
        self.guard()?;

        let mut script = self.script.lock().expect("script lock");

        if script.sub_zone_readings.iter().any(|kept| {
            kept.sub_zone_id() == entity.sub_zone_id() && kept.month() == entity.month()
        }) {
            return Ok(None);
        }

        let stored = stored_sub_zone_reading(script.sub_zone_readings.len() as i32 + 1, entity);
        script.sub_zone_readings.push(stored.clone());

        Ok(Some(stored))
    }

    async fn update_sub_zone_reading(
        &self,
        entity: &SubZoneReading,
    ) -> Result<Option<SubZoneReading>, AppError> {
        self.record(RepositoryCall::UpdateSubZoneReading {
            sub_zone_id: *entity.sub_zone_id(),
            month: String::from(entity.month()),
        });
        self.guard()?;

        let mut script = self.script.lock().expect("script lock");

        let Some(kept) = script.sub_zone_readings.iter_mut().find(|kept| {
            kept.sub_zone_id() == entity.sub_zone_id() && kept.month() == entity.month()
        }) else {
            return Ok(None);
        };

        *kept = stored_sub_zone_reading(kept.id().unwrap_or(1), entity);

        Ok(Some(kept.clone()))
    }

    async fn delete_sub_zone_reading(
        &self,
        sub_zone_id: i32,
        month: Month,
    ) -> Result<bool, AppError> {
        self.record(RepositoryCall::DeleteSubZoneReading {
            sub_zone_id,
            month: String::from(month),
        });
        self.guard()?;

        let mut script = self.script.lock().expect("script lock");
        let before = script.sub_zone_readings.len();
        script
            .sub_zone_readings
            .retain(|kept| !(*kept.sub_zone_id() == sub_zone_id && *kept.month() == month));

        Ok(script.sub_zone_readings.len() < before)
    }

    async fn find_all_sub_zone_shapes(&self) -> Result<Vec<(SubZone, Shape)>, AppError> {
        self.record(RepositoryCall::FindAllSubZoneShapes);
        self.guard()?;

        Ok(self.script.lock().expect("script lock").shapes.clone())
    }
}

fn stored_reading(id: i32, entity: &ZoneReading) -> ZoneReading {
    ZoneReading::rehydrate(
        id,
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
    )
}

fn stored_sub_zone_reading(id: i32, entity: &SubZoneReading) -> SubZoneReading {
    SubZoneReading::rehydrate(
        id,
        *entity.sub_zone_id(),
        *entity.month(),
        *entity.dryness(),
        *entity.updated_at(),
    )
}

/// A signed-in staff member, for the use cases that log who acted.
pub fn an_actor(staff_id: i32) -> StaffContext {
    StaffContext::new(
        staff_id,
        "officer@example.org".to_string(),
        Permission::all().into_iter().collect(),
    )
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

/// A square one degree wide with its south-west corner at the given point.
pub fn a_square(west: f64, south: f64) -> Shape {
    Shape::new(vec![vec![
        (west, south),
        (west + 1.0, south),
        (west + 1.0, south + 1.0),
        (west, south + 1.0),
    ]])
    .expect("shape")
}

/// A shape for each of `sub_zones()`: Chamchamal's three side by side from
/// 44 to 47 degrees east between 35 and 36 north, and Kalar's south of the
/// first of them.
pub fn shapes() -> Vec<(SubZone, Shape)> {
    let squares = [
        a_square(44.0, 35.0),
        a_square(45.0, 35.0),
        a_square(46.0, 35.0),
        a_square(44.0, 34.0),
    ];

    sub_zones().into_iter().zip(squares).collect()
}
