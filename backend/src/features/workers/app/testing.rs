use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use chrono::{DateTime, TimeZone, Utc};

use crate::{
    app::{
        Action, AppError as GlobalAppError, AuthContext, Pagination, Permission, Resource,
        StaffContext, User,
    },
    features::workers::{
        app::{AccountDirectory, AppError, WorkerFilter, WorkerRepository},
        domain::{CostIqd, CostPer, GeoPoint, Worker, WorkerDetails, WorkerName},
    },
    shared::Phone,
};

pub const PHONE: &str = "+9647501234567";
pub const OTHER_PHONE: &str = "+9647701112233";
pub const STAFF_ID: i32 = 9;

#[derive(Clone, Debug, PartialEq)]
pub enum Call {
    BlockedPhones,
    Save {
        phone: String,
    },
    FindByPhone {
        phone: String,
    },
    DeleteByPhone {
        phone: String,
    },
    FindPage {
        filter: WorkerFilter,
        near: Option<GeoPoint>,
        page: u64,
    },
    Delete {
        id: i32,
    },
}

#[derive(Debug, Default)]
struct Script {
    fail: bool,
    blocked: Vec<Phone>,
}

/// One fake for the repository and the account directory, so a test reads
/// the calls to both in the order they were made.
#[derive(Clone, Debug, Default)]
pub struct Fakes {
    script: Arc<Mutex<Script>>,
    stored: Arc<Mutex<Vec<Worker>>>,
    calls: Arc<Mutex<Vec<Call>>>,
}

impl Fakes {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_stored(self, worker: Worker) -> Self {
        self.stored.lock().expect("stored lock").push(worker);
        self
    }

    pub fn with_blocked(self, phone: &str) -> Self {
        self.script
            .lock()
            .expect("script lock")
            .blocked
            .push(a_phone(phone));
        self
    }

    pub fn failing(self) -> Self {
        self.script.lock().expect("script lock").fail = true;
        self
    }

    pub fn stored(&self) -> Vec<Worker> {
        self.stored.lock().expect("stored lock").clone()
    }

    pub fn calls(&self) -> Vec<Call> {
        self.calls.lock().expect("calls lock").clone()
    }

    fn record(&self, call: Call) -> Result<(), AppError> {
        self.calls.lock().expect("calls lock").push(call);

        if self.script.lock().expect("script lock").fail {
            return Err(GlobalAppError::InternalServerError.into());
        }

        Ok(())
    }
}

#[async_trait]
impl AccountDirectory for Fakes {
    async fn blocked_phones(&self) -> Result<Vec<Phone>, AppError> {
        self.record(Call::BlockedPhones)?;

        Ok(self.script.lock().expect("script lock").blocked.clone())
    }
}

#[async_trait]
impl WorkerRepository for Fakes {
    async fn save(&self, entity: &Worker) -> Result<Worker, AppError> {
        self.record(Call::Save {
            phone: entity.phone().as_str().to_string(),
        })?;

        let mut stored = self.stored.lock().expect("stored lock");
        let existing = stored
            .iter()
            .position(|worker| worker.phone() == entity.phone());

        let (id, created_at) = match existing {
            Some(index) => {
                let old = stored.remove(index);
                (old.id().unwrap_or_default(), *old.created_at())
            }
            None => (stored.len() as i32 + 1, *entity.created_at()),
        };

        let saved = Worker::rehydrate(
            id,
            entity.phone().clone(),
            details_of(entity),
            created_at,
            *entity.updated_at(),
        );
        stored.push(saved.clone());

        Ok(saved)
    }

    async fn find_by_phone(&self, phone: &Phone) -> Result<Option<Worker>, AppError> {
        self.record(Call::FindByPhone {
            phone: phone.as_str().to_string(),
        })?;

        Ok(self
            .stored()
            .into_iter()
            .find(|worker| worker.phone() == phone))
    }

    async fn delete_by_phone(&self, phone: &Phone) -> Result<bool, AppError> {
        self.record(Call::DeleteByPhone {
            phone: phone.as_str().to_string(),
        })?;

        let mut stored = self.stored.lock().expect("stored lock");
        let before = stored.len();
        stored.retain(|worker| worker.phone() != phone);

        Ok(stored.len() < before)
    }

    /// Applies only the parts of the filter the use cases decide on, the
    /// availability and the phones left out. The rest is SQL, proven by a
    /// run against a real database.
    async fn find_page(
        &self,
        filter: &WorkerFilter,
        near: Option<&GeoPoint>,
        pagination: &Pagination,
    ) -> Result<(Vec<Worker>, u64), AppError> {
        self.record(Call::FindPage {
            filter: filter.clone(),
            near: near.copied(),
            page: *pagination.page(),
        })?;

        let found: Vec<Worker> = self
            .stored()
            .into_iter()
            .filter(|worker| {
                filter
                    .available
                    .is_none_or(|available| *worker.available() == available)
                    && !filter.excluding.contains(worker.phone())
            })
            .collect();
        let count = found.len() as u64;

        Ok((found, count))
    }

    async fn delete(&self, id: i32) -> Result<bool, AppError> {
        self.record(Call::Delete { id })?;

        let mut stored = self.stored.lock().expect("stored lock");
        let before = stored.len();
        stored.retain(|worker| *worker.id() != Some(id));

        Ok(stored.len() < before)
    }
}

pub fn a_phone(value: &str) -> Phone {
    Phone::new(value.to_string()).expect("phone")
}

pub fn auth_context() -> AuthContext {
    AuthContext::new(User::new(a_phone(PHONE)), "token".to_string())
}

/// A staff member holding every permission on farmers, the resource the
/// cards are guarded by.
pub fn staff_context() -> StaffContext {
    StaffContext::new(
        STAFF_ID,
        "officer@example.org".to_string(),
        Action::ALL
            .into_iter()
            .map(|action| Permission::new(Resource::Farmers, action))
            .collect(),
    )
}

pub fn a_time() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 10, 10, 8, 0, 0)
        .single()
        .expect("time")
}

pub fn details(name: &str, cost_iqd: i64) -> WorkerDetails {
    WorkerDetails {
        name: WorkerName::new(name.to_string()).expect("name"),
        cost: CostIqd::new(cost_iqd).expect("cost"),
        cost_per: CostPer::Day,
        note: None,
        zone_slug: None,
        point: None,
        available: true,
    }
}

pub fn details_of(worker: &Worker) -> WorkerDetails {
    WorkerDetails {
        name: worker.name().clone(),
        cost: *worker.cost(),
        cost_per: *worker.cost_per(),
        note: worker.note().clone(),
        zone_slug: worker.zone_slug().clone(),
        point: *worker.point(),
        available: *worker.available(),
    }
}

/// A stored, available card of `phone`.
pub fn a_worker(id: i32, phone: &str) -> Worker {
    Worker::rehydrate(
        id,
        a_phone(phone),
        details("Azad", 25_000),
        a_time(),
        a_time(),
    )
}

/// The same card taken off the list by its owner.
pub fn unavailable(worker: &Worker) -> Worker {
    Worker::rehydrate(
        worker.id().unwrap_or_default(),
        worker.phone().clone(),
        WorkerDetails {
            available: false,
            ..details_of(worker)
        },
        *worker.created_at(),
        *worker.updated_at(),
    )
}
