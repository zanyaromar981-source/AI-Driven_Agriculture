use std::{collections::HashMap, sync::Mutex};

use chrono::{DateTime, Duration, Utc};

use crate::features::app_config::domain::AppVersion;

/// What the version check keeps in memory so that it does not ask the
/// database on every request of every farmer: the oldest version allowed,
/// and which farmer was already noted on which version this hour.
///
/// It is per process. A second server learns of a new `min_version` within
/// `MIN_VERSION_TTL_SECONDS`; the database statement, not this map, is what
/// keeps a sighting to one write an hour across servers.
#[derive(Debug, Default)]
pub struct VersionGate {
    min_version: Mutex<Option<(AppVersion, DateTime<Utc>)>>,
    noted: Mutex<HashMap<(String, AppVersion), DateTime<Utc>>>,
}

impl VersionGate {
    /// How long the oldest version allowed is trusted before it is read
    /// again.
    pub const MIN_VERSION_TTL_SECONDS: i64 = 30;

    /// A farmer on a version is written down once in this many minutes.
    pub const SIGHTING_GAP_MINUTES: i64 = 60;

    /// Above this many remembered sightings, the ones older than the gap
    /// are dropped. They would be written again anyway.
    const MAX_NOTED: usize = 50_000;

    pub fn new() -> Self {
        Self::default()
    }

    /// The oldest version allowed, when it was read recently enough.
    pub fn fresh_min_version(&self, now: DateTime<Utc>) -> Option<AppVersion> {
        let cached = *self.min_version.lock().ok()?;

        cached
            .filter(|(_, read_at)| {
                let age = now - *read_at;

                // A clock that stepped back makes the age negative: read
                // again rather than trust the value for longer than meant.
                age >= Duration::zero() && age < Duration::seconds(Self::MIN_VERSION_TTL_SECONDS)
            })
            .map(|(version, _)| version)
    }

    /// The oldest version allowed as last read, however long ago.
    pub fn last_min_version(&self) -> Option<AppVersion> {
        let cached = *self.min_version.lock().ok()?;

        cached.map(|(version, _)| version)
    }

    pub fn remember_min_version(&self, version: AppVersion, now: DateTime<Utc>) {
        if let Ok(mut cached) = self.min_version.lock() {
            *cached = Some((version, now));
        }
    }

    /// Whether this farmer on this version is due to be written down, and
    /// if so remembers that it is being done now. Of many requests at the
    /// same moment exactly one is told yes.
    pub fn claim_sighting(&self, phone: &str, version: AppVersion, now: DateTime<Utc>) -> bool {
        let Ok(mut noted) = self.noted.lock() else {
            return false;
        };

        let gap = Duration::minutes(Self::SIGHTING_GAP_MINUTES);
        let key = (phone.to_string(), version);

        if noted
            .get(&key)
            .is_some_and(|at| now - *at < gap && now >= *at)
        {
            return false;
        }

        if noted.len() >= Self::MAX_NOTED {
            noted.retain(|_, at| now - *at < gap);
        }

        noted.insert(key, now);

        true
    }

    /// Gives a claim back when the write it stood for failed, so the next
    /// request tries again.
    pub fn release_sighting(&self, phone: &str, version: AppVersion) {
        if let Ok(mut noted) = self.noted.lock() {
            noted.remove(&(phone.to_string(), version));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    const PHONE: &str = "+9647501234567";

    fn now() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 10, 9, 12, 0, 0)
            .single()
            .expect("time")
    }

    fn version(value: &str) -> AppVersion {
        AppVersion::new(value).expect("version")
    }

    #[test]
    fn nothing_is_known_before_the_first_read() {
        let gate = VersionGate::new();

        assert_eq!(gate.fresh_min_version(now()), None);
        assert_eq!(gate.last_min_version(), None);
    }

    #[test]
    fn a_remembered_version_is_fresh_for_thirty_seconds() {
        let gate = VersionGate::new();
        gate.remember_min_version(version("1.2.0"), now());

        assert_eq!(
            gate.fresh_min_version(now() + Duration::seconds(29)),
            Some(version("1.2.0"))
        );
        assert_eq!(gate.fresh_min_version(now() + Duration::seconds(30)), None);
        assert_eq!(
            gate.last_min_version(),
            Some(version("1.2.0")),
            "a stale value is still there to fall back on"
        );
    }

    #[test]
    fn a_clock_that_stepped_back_does_not_extend_the_trust() {
        let gate = VersionGate::new();
        gate.remember_min_version(version("1.2.0"), now());

        assert_eq!(gate.fresh_min_version(now() - Duration::minutes(5)), None);
    }

    #[test]
    fn a_farmer_on_a_version_is_claimed_once_an_hour() {
        let gate = VersionGate::new();

        assert!(gate.claim_sighting(PHONE, version("1.0.3"), now()));
        assert!(!gate.claim_sighting(PHONE, version("1.0.3"), now()));
        assert!(!gate.claim_sighting(PHONE, version("1.0.3"), now() + Duration::minutes(59)));
        assert!(gate.claim_sighting(PHONE, version("1.0.3"), now() + Duration::minutes(60)));
    }

    #[test]
    fn another_version_or_another_farmer_is_claimed_on_its_own() {
        let gate = VersionGate::new();

        assert!(gate.claim_sighting(PHONE, version("1.0.3"), now()));
        assert!(gate.claim_sighting(PHONE, version("1.1.0"), now()));
        assert!(gate.claim_sighting("+9647509999999", version("1.0.3"), now()));
    }

    #[test]
    fn a_released_claim_can_be_taken_again_at_once() {
        let gate = VersionGate::new();

        assert!(gate.claim_sighting(PHONE, version("1.0.3"), now()));
        gate.release_sighting(PHONE, version("1.0.3"));

        assert!(gate.claim_sighting(PHONE, version("1.0.3"), now()));
    }
}
