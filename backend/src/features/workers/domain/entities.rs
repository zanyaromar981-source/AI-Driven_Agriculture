use chrono::{DateTime, Utc};
use getset::Getters;

use crate::shared::Phone;

use super::{CostIqd, CostPer, GeoPoint, WorkerName, WorkerNote, ZoneSlug};

/// Everything a person writes on their card. The phone is not here: it is
/// the phone they signed in with, never something they type.
#[derive(Clone, Debug, PartialEq)]
pub struct WorkerDetails {
    pub name: WorkerName,
    pub cost: CostIqd,
    pub cost_per: CostPer,
    pub note: Option<WorkerNote>,
    pub zone_slug: Option<ZoneSlug>,
    pub point: Option<GeoPoint>,
    /// `false` takes the card off the list without deleting it.
    pub available: bool,
}

/// A person who can be hired for farm work: one card per phone. A farmer
/// reads the card and calls the number; there is no booking here.
#[derive(Clone, Debug, PartialEq, Getters)]
#[getset(get = "pub")]
pub struct Worker {
    id: Option<i32>,
    phone: Phone,
    name: WorkerName,
    cost: CostIqd,
    cost_per: CostPer,
    note: Option<WorkerNote>,
    zone_slug: Option<ZoneSlug>,
    point: Option<GeoPoint>,
    available: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl Worker {
    /// The card of the person signed in with `phone`.
    pub fn new(phone: Phone, details: WorkerDetails, now: DateTime<Utc>) -> Self {
        Self {
            id: None,
            phone,
            name: details.name,
            cost: details.cost,
            cost_per: details.cost_per,
            note: details.note,
            zone_slug: details.zone_slug,
            point: details.point,
            available: details.available,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn rehydrate(
        id: i32,
        phone: Phone,
        details: WorkerDetails,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id: Some(id),
            created_at,
            updated_at,
            ..Self::new(phone, details, updated_at)
        }
    }

    /// The card as someone standing at `from` reads it, when they said
    /// where they stand.
    pub fn seen_from(self, from: Option<&GeoPoint>) -> ListedWorker {
        let distance_km = match (from, &self.point) {
            (Some(from), Some(point)) => Some(from.km_to(point)),
            _ => None,
        };

        ListedWorker {
            worker: self,
            distance_km,
        }
    }
}

/// A card on a list, with how far away the worker is. The distance is
/// `None` when the reader gave no point or the card has none: it is never
/// guessed from the district.
#[derive(Clone, Debug, PartialEq, Getters)]
#[getset(get = "pub")]
pub struct ListedWorker {
    worker: Worker,
    distance_km: Option<f64>,
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use super::*;

    fn details(point: Option<GeoPoint>) -> WorkerDetails {
        WorkerDetails {
            name: WorkerName::new("Azad".to_string()).expect("name"),
            cost: CostIqd::new(25_000).expect("cost"),
            cost_per: CostPer::Day,
            note: None,
            zone_slug: None,
            point,
            available: true,
        }
    }

    fn phone() -> Phone {
        Phone::new("+9647501234567".to_string()).expect("phone")
    }

    fn now() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 10, 10, 8, 0, 0)
            .single()
            .expect("time")
    }

    #[test]
    fn a_new_card_is_not_stored_yet_and_carries_the_signed_in_phone() {
        let worker = Worker::new(phone(), details(None), now());

        assert_eq!(*worker.id(), None);
        assert_eq!(worker.phone(), &phone());
        assert_eq!(worker.created_at(), worker.updated_at());
    }

    #[test]
    fn a_stored_card_keeps_its_id_and_both_times() {
        let later = now() + chrono::Duration::hours(2);
        let worker = Worker::rehydrate(4, phone(), details(None), now(), later);

        assert_eq!(*worker.id(), Some(4));
        assert_eq!(*worker.created_at(), now());
        assert_eq!(*worker.updated_at(), later);
    }

    #[test]
    fn the_distance_is_given_only_when_both_points_are_known() {
        let sulaymaniyah = GeoPoint::in_region(35.5572, 45.4356).expect("point");
        let erbil = GeoPoint::in_region(36.1911, 44.0092).expect("point");

        let near = Worker::new(phone(), details(Some(erbil)), now()).seen_from(Some(&sulaymaniyah));
        let km = near.distance_km().expect("distance");
        assert!((145.0..150.0).contains(&km), "got {km}");

        let no_reader = Worker::new(phone(), details(Some(erbil)), now()).seen_from(None);
        assert_eq!(*no_reader.distance_km(), None);

        let no_point = Worker::new(phone(), details(None), now()).seen_from(Some(&sulaymaniyah));
        assert_eq!(
            *no_point.distance_km(),
            None,
            "a card without a point has no distance, never a guessed one"
        );
    }
}
