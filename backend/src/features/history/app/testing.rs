use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use chrono::{DateTime, TimeZone, Utc};

use crate::{
    app::{AuthContext, Permission, StaffContext, User},
    features::history::{
        app::{AppError, HistoryFarmDirectory, HistoryFarmOwnership, HistoryRepository},
        domain::{
            FarmSite, HistorySource, HistoryWindow, Metric, MetricCoverage, Month, MonthlyPoint,
            Series, SeriesUpload,
        },
    },
    shared::Phone,
};

pub const OWNER: &str = "+9647501234567";

/// The farm the fakes say `OWNER` owns.
pub const FARM_ID: i32 = 7;

pub const STAFF_ID: i32 = 4;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Call {
    IsOwnedBy {
        farm_id: i32,
        phone: String,
    },
    AllSites,
    Exists {
        farm_id: i32,
    },
    FindSeries {
        farm_id: i32,
        metrics: Vec<Metric>,
        from: String,
        to: String,
    },
    FindAllCoverage,
    Store {
        farm_id: i32,
        metric: Metric,
        points: usize,
    },
    DeleteSeries {
        farm_id: i32,
        metric: Metric,
    },
}

/// One stored series of one farm, with every month it has.
#[derive(Clone, Debug)]
struct Stored {
    farm_id: i32,
    series: Series,
}

#[derive(Debug, Default)]
struct Script {
    owned_farm: Option<i32>,
    sites: Vec<FarmSite>,
    stored: Vec<Stored>,
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

    pub fn with_stored(self, farm_id: i32, series: Series) -> Self {
        self.script
            .lock()
            .expect("script lock")
            .stored
            .push(Stored { farm_id, series });
        self
    }

    pub fn failing(self) -> Self {
        self.script
            .lock()
            .expect("script lock")
            .fail_with_database_error = true;
        self
    }

    /// The series as the fake holds it now, after the writes of a test.
    pub fn stored(&self, farm_id: i32, metric: Metric) -> Option<Series> {
        self.script
            .lock()
            .expect("script lock")
            .stored
            .iter()
            .find(|stored| stored.farm_id == farm_id && *stored.series.metric() == metric)
            .map(|stored| stored.series.clone())
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
impl HistoryFarmOwnership for Fakes {
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
impl HistoryFarmDirectory for Fakes {
    async fn all_sites(&self) -> Result<Vec<FarmSite>, AppError> {
        self.record(Call::AllSites);
        self.guard()?;

        Ok(self.script.lock().expect("script lock").sites.clone())
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
}

#[async_trait]
impl HistoryRepository for Fakes {
    async fn find_series(
        &self,
        farm_id: i32,
        metrics: &[Metric],
        window: &HistoryWindow,
    ) -> Result<Vec<Series>, AppError> {
        self.record(Call::FindSeries {
            farm_id,
            metrics: metrics.to_vec(),
            from: window.from().to_string(),
            to: window.to().to_string(),
        });
        self.guard()?;

        let script = self.script.lock().expect("script lock");

        // Latest first, so a use case that forgets to sort is caught.
        Ok(script
            .stored
            .iter()
            .rev()
            .filter(|stored| stored.farm_id == farm_id && metrics.contains(stored.series.metric()))
            .filter_map(|stored| {
                let points: Vec<MonthlyPoint> = stored
                    .series
                    .points()
                    .iter()
                    .filter(|point| (window.from()..=window.to()).contains(point.month()))
                    .copied()
                    .collect();

                (!points.is_empty()).then(|| {
                    Series::rehydrate(
                        *stored.series.metric(),
                        stored.series.source().clone(),
                        *stored.series.as_of(),
                        points,
                    )
                })
            })
            .collect())
    }

    async fn find_all_coverage(&self) -> Result<Vec<MetricCoverage>, AppError> {
        self.record(Call::FindAllCoverage);
        self.guard()?;

        let script = self.script.lock().expect("script lock");

        Ok(script
            .stored
            .iter()
            .filter_map(|stored| {
                let points = stored.series.points();

                Some(MetricCoverage::rehydrate(
                    stored.farm_id,
                    *stored.series.metric(),
                    *points.first()?.month(),
                    *points.last()?.month(),
                    points.len() as u32,
                    *stored.series.as_of(),
                ))
            })
            .collect())
    }

    async fn store(&self, upload: &SeriesUpload) -> Result<bool, AppError> {
        self.record(Call::Store {
            farm_id: *upload.farm_id(),
            metric: *upload.metric(),
            points: upload.points().len(),
        });
        self.guard()?;

        let mut script = self.script.lock().expect("script lock");

        let before = script
            .stored
            .iter()
            .find(|stored| {
                stored.farm_id == *upload.farm_id() && stored.series.metric() == upload.metric()
            })
            .map(|stored| stored.series.clone());

        let newest = before
            .as_ref()
            .is_none_or(|series| upload.as_of() >= series.as_of());
        let named = |points: &[MonthlyPoint], month: &Month| {
            points.iter().any(|point| point.month() == month)
        };

        let mut points = before
            .as_ref()
            .map(|series| series.points().clone())
            .unwrap_or_default();

        if newest {
            points.retain(|kept| !named(upload.points(), kept.month()));
            points.extend(upload.points().iter().copied());
        } else {
            let missing: Vec<MonthlyPoint> = upload
                .points()
                .iter()
                .filter(|pushed| !named(&points, pushed.month()))
                .copied()
                .collect();
            points.extend(missing);
        }

        let (source, as_of) = match before {
            Some(series) if !newest => (series.source().clone(), *series.as_of()),
            _ => (upload.source().clone(), *upload.as_of()),
        };

        script.stored.retain(|stored| {
            stored.farm_id != *upload.farm_id() || stored.series.metric() != upload.metric()
        });
        script.stored.push(Stored {
            farm_id: *upload.farm_id(),
            series: Series::rehydrate(*upload.metric(), source, as_of, points),
        });

        Ok(newest)
    }

    async fn delete_series(&self, farm_id: i32, metric: Metric) -> Result<(), AppError> {
        self.record(Call::DeleteSeries { farm_id, metric });
        self.guard()?;

        self.script
            .lock()
            .expect("script lock")
            .stored
            .retain(|stored| stored.farm_id != farm_id || *stored.series.metric() != metric);

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

pub fn phone() -> Phone {
    Phone::new(OWNER.to_string()).expect("phone")
}

pub fn auth_context() -> AuthContext {
    AuthContext::new(User::new(phone()), "token".to_string())
}

/// The moment the tests run at: the last full month is September 2026.
pub fn now() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 10, 9, 12, 0, 0).unwrap()
}

pub fn a_month(raw: &str) -> Month {
    Month::parse(raw).expect("month")
}

pub fn a_source() -> HistorySource {
    HistorySource::new("ERA5-Land reanalysis via Open-Meteo, about 9 km".to_string())
        .expect("source")
}

pub fn a_site(farm_id: i32) -> FarmSite {
    FarmSite::rehydrate(farm_id, 35.56, 45.43, now())
}

/// A stored series with the given months.
pub fn a_series(metric: Metric, points: &[(&str, f64)]) -> Series {
    Series::rehydrate(
        metric,
        a_source(),
        now(),
        points
            .iter()
            .map(|(raw, value)| MonthlyPoint::rehydrate(a_month(raw), *value))
            .collect(),
    )
}
