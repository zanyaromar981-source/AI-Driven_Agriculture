use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};

use crate::{
    app::{AuthContext, Permission, StaffContext, User},
    features::alerts::{
        app::{
            AlertFarms, AlertRepository, AppError, DeviceRepository, use_cases::RecordAlertInput,
        },
        domain::{
            Alert, AlertConfidence, AlertKey, AlertLevel, AlertSource, AlertText, AlertType,
            Device, DeviceLanguage, Platform, PushToken, baghdad_day,
        },
    },
    shared::Phone,
};

pub const OWNER: &str = "+9647501234567";
pub const OTHER_OWNER: &str = "+9647507654321";

/// The farm the fakes say `OWNER` owns.
pub const FARM_ID: i32 = 7;

/// The farm the fakes say `OTHER_OWNER` owns.
pub const OTHER_FARM_ID: i32 = 8;

pub const STAFF_ID: i32 = 4;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Call {
    IsOwnedBy { farm_id: i32, phone: String },
    IdsOwnedBy { phone: String },
    OwnerOf { farm_id: i32 },
    Upsert { farm_id: i32, key: String },
    FindByFarms { farm_ids: Vec<i32> },
    FindAllByFarm { farm_id: i32 },
    FindById { id: i32 },
    MarkDone { id: i32 },
    MarkPushed { id: i32 },
    FindUnpushedAlarms,
    FindFarmsPushedSince,
    Delete { id: i32 },
    DeleteByFarms { farm_ids: Vec<i32> },
    UpsertDevice { phone: String },
    DeleteDevice { phone: String },
    FindAlertDevices { phone: String },
    DeleteDeviceByToken,
    DeleteDevicesByPhone { phone: String },
}

#[derive(Debug, Default)]
struct Script {
    alerts: Vec<Alert>,
    devices: Vec<Device>,
}

/// One fake standing in for every port of the feature, so a test can read
/// the calls of a whole use case in the order they happened.
#[derive(Debug, Clone, Default)]
pub struct Fakes {
    script: Arc<Mutex<Script>>,
    calls: Arc<Mutex<Vec<Call>>>,
}

fn owner_of(farm_id: i32) -> Option<&'static str> {
    match farm_id {
        FARM_ID => Some(OWNER),
        OTHER_FARM_ID => Some(OTHER_OWNER),
        _ => None,
    }
}

#[allow(clippy::too_many_arguments)]
fn with_state(
    alert: &Alert,
    id: i32,
    pushed: bool,
    pushed_at: Option<DateTime<Utc>>,
    done: bool,
    done_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
) -> Alert {
    Alert::rehydrate(
        id,
        *alert.farm_id(),
        alert.key().clone(),
        *alert.alert_type(),
        *alert.day(),
        *alert.level(),
        *alert.confidence(),
        alert.ku().clone(),
        alert.en().clone(),
        alert.action_ku().clone(),
        alert.action_en().clone(),
        pushed,
        pushed_at,
        done,
        done_at,
        alert.source().clone(),
        created_at,
        *alert.updated_at(),
    )
}

impl Fakes {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_stored(self, alert: Alert) -> Self {
        self.script.lock().expect("script lock").alerts.push(alert);
        self
    }

    /// `OWNER` has a phone registered that wants red alerts.
    pub fn with_device(self, push_token: &str) -> Self {
        self.script
            .lock()
            .expect("script lock")
            .devices
            .push(Device::new(
                token(push_token),
                phone(OWNER),
                Platform::Android,
                DeviceLanguage::Ku,
                true,
                true,
                Utc::now(),
            ));
        self
    }

    pub fn stored(&self, id: i32) -> Option<Alert> {
        self.script
            .lock()
            .expect("script lock")
            .alerts
            .iter()
            .find(|alert| *alert.id() == Some(id))
            .cloned()
    }

    pub fn stored_count(&self) -> usize {
        self.script.lock().expect("script lock").alerts.len()
    }

    pub fn devices(&self) -> Vec<Device> {
        self.script.lock().expect("script lock").devices.clone()
    }

    /// The farmer ticked the alert and the sender pushed it.
    pub fn tick_and_push(&self, id: i32) {
        self.change(id, |alert| {
            let now = Utc::now();

            with_state(
                alert,
                id,
                true,
                Some(now),
                true,
                Some(now),
                *alert.created_at(),
            )
        });
    }

    /// The sender pushed the alert a moment ago.
    pub fn push(&self, id: i32) {
        self.change(id, |alert| {
            with_state(
                alert,
                id,
                true,
                Some(Utc::now()),
                *alert.done(),
                *alert.done_at(),
                *alert.created_at(),
            )
        });
    }

    fn change(&self, id: i32, change: impl Fn(&Alert) -> Alert) -> bool {
        let mut script = self.script.lock().expect("script lock");

        match script
            .alerts
            .iter_mut()
            .find(|alert| *alert.id() == Some(id))
        {
            Some(alert) => {
                *alert = change(alert);
                true
            }
            None => false,
        }
    }

    pub fn calls(&self) -> Vec<Call> {
        self.calls.lock().expect("calls lock").clone()
    }

    fn record(&self, call: Call) {
        self.calls.lock().expect("calls lock").push(call);
    }
}

#[async_trait]
impl AlertFarms for Fakes {
    async fn is_owned_by(&self, farm_id: i32, phone: &Phone) -> Result<bool, AppError> {
        self.record(Call::IsOwnedBy {
            farm_id,
            phone: String::from(phone),
        });

        Ok(owner_of(farm_id) == Some(phone.as_str()))
    }

    async fn ids_owned_by(&self, phone: &Phone) -> Result<Vec<i32>, AppError> {
        self.record(Call::IdsOwnedBy {
            phone: String::from(phone),
        });

        Ok([FARM_ID, OTHER_FARM_ID]
            .into_iter()
            .filter(|farm_id| owner_of(*farm_id) == Some(phone.as_str()))
            .collect())
    }

    async fn owner_of(&self, farm_id: i32) -> Result<Option<Phone>, AppError> {
        self.record(Call::OwnerOf { farm_id });

        Ok(owner_of(farm_id).map(phone))
    }
}

#[async_trait]
impl AlertRepository for Fakes {
    async fn upsert(&self, entity: &Alert) -> Result<Alert, AppError> {
        self.record(Call::Upsert {
            farm_id: *entity.farm_id(),
            key: entity.key().into(),
        });

        let mut script = self.script.lock().expect("script lock");
        let next_id = script.alerts.len() as i32 + 1;

        match script
            .alerts
            .iter_mut()
            .find(|alert| alert.farm_id() == entity.farm_id() && alert.key() == entity.key())
        {
            Some(kept) => {
                *kept = with_state(
                    entity,
                    kept.id().expect("id"),
                    *kept.pushed(),
                    *kept.pushed_at(),
                    *kept.done(),
                    *kept.done_at(),
                    *kept.created_at(),
                );

                Ok(kept.clone())
            }
            None => {
                let stored = with_state(
                    entity,
                    next_id,
                    false,
                    None,
                    false,
                    None,
                    *entity.created_at(),
                );
                script.alerts.push(stored.clone());

                Ok(stored)
            }
        }
    }

    async fn find_by_farms(
        &self,
        farm_ids: &[i32],
        first_day: NaiveDate,
    ) -> Result<Vec<Alert>, AppError> {
        self.record(Call::FindByFarms {
            farm_ids: farm_ids.to_vec(),
        });

        Ok(self
            .script
            .lock()
            .expect("script lock")
            .alerts
            .iter()
            .filter(|alert| farm_ids.contains(alert.farm_id()) && *alert.day() >= first_day)
            .cloned()
            .collect())
    }

    async fn find_all_by_farm(&self, farm_id: i32) -> Result<Vec<Alert>, AppError> {
        self.record(Call::FindAllByFarm { farm_id });

        Ok(self
            .script
            .lock()
            .expect("script lock")
            .alerts
            .iter()
            .filter(|alert| *alert.farm_id() == farm_id)
            .cloned()
            .collect())
    }

    async fn find_by_id(&self, id: i32) -> Result<Option<Alert>, AppError> {
        self.record(Call::FindById { id });

        Ok(self.stored(id))
    }

    async fn mark_done(&self, id: i32, now: DateTime<Utc>) -> Result<bool, AppError> {
        self.record(Call::MarkDone { id });

        Ok(self.change(id, |alert| {
            with_state(
                alert,
                id,
                *alert.pushed(),
                *alert.pushed_at(),
                true,
                alert.done_at().or(Some(now)),
                *alert.created_at(),
            )
        }))
    }

    async fn mark_pushed(&self, id: i32, now: DateTime<Utc>) -> Result<bool, AppError> {
        self.record(Call::MarkPushed { id });

        Ok(self.change(id, |alert| {
            with_state(
                alert,
                id,
                true,
                alert.pushed_at().or(Some(now)),
                *alert.done(),
                *alert.done_at(),
                *alert.created_at(),
            )
        }))
    }

    async fn find_unpushed_alarms(&self, first_day: NaiveDate) -> Result<Vec<Alert>, AppError> {
        self.record(Call::FindUnpushedAlarms);

        Ok(self
            .script
            .lock()
            .expect("script lock")
            .alerts
            .iter()
            .filter(|alert| {
                *alert.level() == AlertLevel::Alarm
                    && !alert.pushed()
                    && !alert.done()
                    && *alert.day() >= first_day
            })
            .cloned()
            .collect())
    }

    async fn find_farms_pushed_since(&self, since: DateTime<Utc>) -> Result<Vec<i32>, AppError> {
        self.record(Call::FindFarmsPushedSince);

        Ok(self
            .script
            .lock()
            .expect("script lock")
            .alerts
            .iter()
            .filter(|alert| alert.pushed_at().is_some_and(|at| at >= since))
            .map(|alert| *alert.farm_id())
            .collect())
    }

    async fn delete(&self, id: i32) -> Result<bool, AppError> {
        self.record(Call::Delete { id });

        let mut script = self.script.lock().expect("script lock");
        let before = script.alerts.len();
        script.alerts.retain(|alert| *alert.id() != Some(id));

        Ok(script.alerts.len() < before)
    }

    async fn delete_by_farms(&self, farm_ids: &[i32]) -> Result<u64, AppError> {
        self.record(Call::DeleteByFarms {
            farm_ids: farm_ids.to_vec(),
        });

        let mut script = self.script.lock().expect("script lock");
        let before = script.alerts.len();
        script
            .alerts
            .retain(|alert| !farm_ids.contains(alert.farm_id()));

        Ok((before - script.alerts.len()) as u64)
    }
}

#[async_trait]
impl DeviceRepository for Fakes {
    async fn upsert(&self, entity: &Device) -> Result<(), AppError> {
        self.record(Call::UpsertDevice {
            phone: String::from(entity.phone()),
        });

        let mut script = self.script.lock().expect("script lock");
        script
            .devices
            .retain(|device| device.push_token() != entity.push_token());
        script.devices.push(entity.clone());

        Ok(())
    }

    async fn delete(&self, push_token: &PushToken, phone: &Phone) -> Result<(), AppError> {
        self.record(Call::DeleteDevice {
            phone: String::from(phone),
        });

        self.script
            .lock()
            .expect("script lock")
            .devices
            .retain(|device| !(device.push_token() == push_token && device.phone() == phone));

        Ok(())
    }

    async fn delete_by_token(&self, push_token: &PushToken) -> Result<(), AppError> {
        self.record(Call::DeleteDeviceByToken);

        self.script
            .lock()
            .expect("script lock")
            .devices
            .retain(|device| device.push_token() != push_token);

        Ok(())
    }

    async fn find_alert_devices(&self, phone: &Phone) -> Result<Vec<Device>, AppError> {
        self.record(Call::FindAlertDevices {
            phone: String::from(phone),
        });

        Ok(self
            .script
            .lock()
            .expect("script lock")
            .devices
            .iter()
            .filter(|device| device.phone() == phone && *device.red_alerts())
            .cloned()
            .collect())
    }

    async fn delete_all_by_phone(&self, phone: &Phone) -> Result<u64, AppError> {
        self.record(Call::DeleteDevicesByPhone {
            phone: String::from(phone),
        });

        let mut script = self.script.lock().expect("script lock");
        let before = script.devices.len();
        script.devices.retain(|device| device.phone() != phone);

        Ok((before - script.devices.len()) as u64)
    }
}

pub fn phone(value: &str) -> Phone {
    Phone::new(value.to_string()).expect("phone")
}

pub fn token(value: &str) -> PushToken {
    PushToken::new(value.to_string()).expect("token")
}

pub fn auth_context() -> AuthContext {
    other_auth_context(OWNER)
}

pub fn other_auth_context(phone_number: &str) -> AuthContext {
    AuthContext::new(User::new(phone(phone_number)), "token".to_string())
}

pub fn staff_context() -> StaffContext {
    StaffContext::new(
        STAFF_ID,
        "officer@example.org".to_string(),
        Permission::all().into_iter().collect(),
    )
}

fn text(value: &str) -> AlertText {
    AlertText::new(value.to_string()).expect("text")
}

/// What a job sends for a frost alarm on the given key.
pub fn an_input(farm_id: i32, key: &str, en: &str) -> RecordAlertInput {
    RecordAlertInput {
        farm_id,
        key: AlertKey::new(key.to_string()).expect("key"),
        alert_type: AlertType::Frost,
        day: baghdad_day(Utc::now()),
        level: AlertLevel::Alarm,
        confidence: AlertConfidence::Likely,
        ku: text("ku"),
        en: text(en),
        action_ku: text("ku"),
        action_en: text("Cover the seedlings"),
        source: AlertSource::new("test".to_string()).expect("source"),
    }
}

fn stored_alert(id: i32, farm_id: i32, level: AlertLevel) -> Alert {
    let now = Utc::now();

    Alert::rehydrate(
        id,
        farm_id,
        AlertKey::new(format!("frost:{id}")).expect("key"),
        AlertType::Frost,
        baghdad_day(now),
        level,
        AlertConfidence::Likely,
        text("ku"),
        text("Frost"),
        text("ku"),
        text("Cover the seedlings"),
        false,
        None,
        false,
        None,
        AlertSource::new("test".to_string()).expect("source"),
        now,
        now,
    )
}

/// A stored `watch` alert for today, neither pushed nor ticked.
pub fn an_alert(id: i32, farm_id: i32) -> Alert {
    stored_alert(id, farm_id, AlertLevel::Watch)
}

/// A stored `alarm` alert for today, neither pushed nor ticked.
pub fn an_alarm(id: i32, farm_id: i32) -> Alert {
    stored_alert(id, farm_id, AlertLevel::Alarm)
}
