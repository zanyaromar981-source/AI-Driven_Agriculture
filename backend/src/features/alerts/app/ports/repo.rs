use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};

use crate::{
    features::alerts::{
        app::AppError,
        domain::{Alert, Device, PushToken},
    },
    shared::Phone,
};

#[async_trait]
pub trait AlertRepository: Send + Sync + std::fmt::Debug {
    /// Stores the alert under its farm and key in one statement. When the
    /// farm already has an alert with that key, its type, day, level,
    /// confidence, texts and source are replaced; `done`, `pushed` and their
    /// times are kept as they are. Returns the stored alert.
    async fn upsert(&self, entity: &Alert) -> Result<Alert, AppError>;

    /// Returns the alerts of the given farms for `first_day` and later,
    /// newest day first.
    async fn find_by_farms(
        &self,
        farm_ids: &[i32],
        first_day: NaiveDate,
    ) -> Result<Vec<Alert>, AppError>;

    /// Returns every alert of one farm, newest day first.
    async fn find_all_by_farm(&self, farm_id: i32) -> Result<Vec<Alert>, AppError>;

    async fn find_by_id(&self, id: i32) -> Result<Option<Alert>, AppError>;

    /// Ticks the alert in one statement, keeping the first time it was
    /// ticked. Returns whether there is such an alert.
    async fn mark_done(&self, id: i32, now: DateTime<Utc>) -> Result<bool, AppError>;

    /// Marks the alert as pushed in one statement, keeping the first time.
    /// Returns whether there is such an alert.
    async fn mark_pushed(&self, id: i32, now: DateTime<Utc>) -> Result<bool, AppError>;

    /// Returns the alarms not yet pushed or ticked, for `first_day` and
    /// later.
    async fn find_unpushed_alarms(&self, first_day: NaiveDate) -> Result<Vec<Alert>, AppError>;

    /// Returns the farms that had an alert pushed at `since` or later.
    async fn find_farms_pushed_since(&self, since: DateTime<Utc>) -> Result<Vec<i32>, AppError>;

    /// Removes one alert. Returns whether it was there.
    async fn delete(&self, id: i32) -> Result<bool, AppError>;

    /// Removes every alert of the given farms. Returns how many.
    async fn delete_by_farms(&self, farm_ids: &[i32]) -> Result<u64, AppError>;
}

#[async_trait]
pub trait DeviceRepository: Send + Sync + std::fmt::Debug {
    /// Stores the device under its token in one statement. A token already
    /// stored, under this phone or another, now belongs to this phone with
    /// these settings.
    async fn upsert(&self, entity: &Device) -> Result<(), AppError>;

    /// Removes the token if it belongs to the phone. Removing one that is
    /// not there, or that belongs to another phone, changes nothing.
    async fn delete(&self, push_token: &PushToken, phone: &Phone) -> Result<(), AppError>;

    /// Removes the token whoever it belongs to, for the push sender when
    /// the push service says the token is dead. Removing one that is not
    /// there changes nothing.
    async fn delete_by_token(&self, push_token: &PushToken) -> Result<(), AppError>;

    /// Returns the phone's devices that want red alerts, oldest first.
    async fn find_alert_devices(&self, phone: &Phone) -> Result<Vec<Device>, AppError>;

    /// Removes every device of the phone. Returns how many.
    async fn delete_all_by_phone(&self, phone: &Phone) -> Result<u64, AppError>;
}
