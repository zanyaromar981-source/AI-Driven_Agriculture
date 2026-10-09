use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use chrono::{DateTime, TimeZone, Utc};

use crate::{
    app::{Action, Permission, Resource, StaffContext},
    features::app_config::{
        app::{AppConfigRepository, AppError, AppFarmers},
        domain::{AppConfig, AppSettings, AppVersion, Features, Limits},
    },
    shared::Phone,
};

pub const PHONE: &str = "+9647501234567";
pub const FARMER_ID: i32 = 3;
pub const STAFF_ID: i32 = 9;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Call {
    Find,
    Replace {
        min_version: String,
        updated_by: Option<i32>,
    },
    RecordSighting {
        farmer_id: i32,
        version: String,
        unless_after: DateTime<Utc>,
    },
    CountFarmersByVersion {
        since: DateTime<Utc>,
    },
    FarmerIdOf {
        phone: String,
    },
}

#[derive(Debug)]
struct Script {
    config: AppConfig,
    counts: Vec<(AppVersion, u64)>,
    no_farmer: bool,
    fail_with_database_error: bool,
}

/// One fake standing in for every port of the feature, so a test can read
/// the calls of a whole use case in the order they happened.
#[derive(Debug, Clone)]
pub struct Fakes {
    script: Arc<Mutex<Script>>,
    calls: Arc<Mutex<Vec<Call>>>,
}

impl Fakes {
    /// Holds the config as the migration seeds it, with `min_version` 1.0.0.
    pub fn new() -> Self {
        Self {
            script: Arc::new(Mutex::new(Script {
                config: AppConfig::rehydrate(settings(), None, a_time()),
                counts: Vec::new(),
                no_farmer: false,
                fail_with_database_error: false,
            })),
            calls: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn with_min_version(self, min_version: &str) -> Self {
        {
            let mut script = self.script.lock().expect("script lock");

            script.config = AppConfig::rehydrate(
                AppSettings {
                    min_version: version(min_version),
                    latest_version: version("9.0.0"),
                    ..settings()
                },
                None,
                a_time(),
            );
        }

        self
    }

    pub fn with_public_farm_totals(self, on: bool) -> Self {
        {
            let mut script = self.script.lock().expect("script lock");

            script.config = AppConfig::rehydrate(
                AppSettings {
                    public_farm_totals: on,
                    ..settings()
                },
                None,
                a_time(),
            );
        }

        self
    }

    pub fn with_counts(self, counts: Vec<(AppVersion, u64)>) -> Self {
        self.script.lock().expect("script lock").counts = counts;
        self
    }

    /// The phone on the token has no farmer any more.
    pub fn without_farmer(self) -> Self {
        self.script.lock().expect("script lock").no_farmer = true;
        self
    }

    pub fn failing(self) -> Self {
        self.set_failing(true);
        self
    }

    pub fn set_failing(&self, failing: bool) {
        self.script
            .lock()
            .expect("script lock")
            .fail_with_database_error = failing;
    }

    pub fn calls(&self) -> Vec<Call> {
        self.calls.lock().expect("calls lock").clone()
    }

    pub fn stored(&self) -> AppConfig {
        self.script.lock().expect("script lock").config.clone()
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

impl Default for Fakes {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AppConfigRepository for Fakes {
    async fn find(&self) -> Result<AppConfig, AppError> {
        self.record(Call::Find);
        self.guard()?;

        Ok(self.script.lock().expect("script lock").config.clone())
    }

    async fn replace(&self, config: &AppConfig) -> Result<AppConfig, AppError> {
        self.record(Call::Replace {
            min_version: config.settings().min_version.to_string(),
            updated_by: *config.updated_by(),
        });
        self.guard()?;

        self.script.lock().expect("script lock").config = config.clone();

        Ok(config.clone())
    }

    async fn record_sighting(
        &self,
        farmer_id: i32,
        version: &AppVersion,
        _now: DateTime<Utc>,
        unless_after: DateTime<Utc>,
    ) -> Result<(), AppError> {
        self.record(Call::RecordSighting {
            farmer_id,
            version: version.to_string(),
            unless_after,
        });
        self.guard()
    }

    async fn count_farmers_by_version(
        &self,
        since: DateTime<Utc>,
    ) -> Result<Vec<(AppVersion, u64)>, AppError> {
        self.record(Call::CountFarmersByVersion { since });
        self.guard()?;

        Ok(self.script.lock().expect("script lock").counts.clone())
    }
}

#[async_trait]
impl AppFarmers for Fakes {
    async fn farmer_id_of(&self, phone: &Phone) -> Result<Option<i32>, AppError> {
        self.record(Call::FarmerIdOf {
            phone: phone.as_str().to_string(),
        });
        self.guard()?;

        let script = self.script.lock().expect("script lock");

        Ok((!script.no_farmer).then_some(FARMER_ID))
    }
}

pub fn phone() -> Phone {
    Phone::new(PHONE.to_string()).expect("phone")
}

pub fn a_time() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 10, 9, 8, 0, 0)
        .single()
        .expect("time")
}

pub fn version(value: &str) -> AppVersion {
    AppVersion::new(value).expect("version")
}

/// The settings the migration seeds.
pub fn settings() -> AppSettings {
    AppSettings {
        latest_version: version("1.0.0"),
        min_version: version("1.0.0"),
        update_message_ku: None,
        update_message_en: None,
        maintenance: false,
        maintenance_message_ku: None,
        maintenance_message_en: None,
        maintenance_from: None,
        maintenance_until: None,
        announcement_on: false,
        announcement_ku: None,
        announcement_en: None,
        features: Features {
            add_farm: true,
            walk_mode: true,
            satellite: true,
            doctor: true,
            reports: false,
            alwa: true,
            plan: false,
            push: false,
        },
        limits: Limits::new(20, 2_000, 3, 50, 10).expect("limits"),
        help_phone: None,
        public_farm_totals: true,
    }
}

/// A staff member holding every permission on the app config.
pub fn staff_context() -> StaffContext {
    StaffContext::new(
        STAFF_ID,
        "officer@example.org".to_string(),
        Action::ALL
            .into_iter()
            .map(|action| Permission::new(Resource::App, action))
            .collect(),
    )
}
