use std::str::FromStr;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use validator::Validate;

use crate::{
    app::{AppError as GlobalAppError, Pagination},
    features::workers::{
        app::{
            AppError,
            use_cases::{BrowseWorkersInput, ListAllWorkersInput},
        },
        domain::{
            self, CostIqd, GeoPoint, ListedWorker, Worker, WorkerDetails, WorkerName, WorkerNote,
            WorkerSearch, ZoneSlug,
        },
    },
    shared::DomainError,
};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum WorkerCostPer {
    Day,
    Hour,
}

impl From<WorkerCostPer> for domain::CostPer {
    fn from(value: WorkerCostPer) -> Self {
        match value {
            WorkerCostPer::Day => domain::CostPer::Day,
            WorkerCostPer::Hour => domain::CostPer::Hour,
        }
    }
}

impl From<domain::CostPer> for WorkerCostPer {
    fn from(value: domain::CostPer) -> Self {
        match value {
            domain::CostPer::Day => WorkerCostPer::Day,
            domain::CostPer::Hour => WorkerCostPer::Hour,
        }
    }
}

/// What a person writes on their card. There is no phone here: the card
/// carries the phone the caller signed in with.
#[derive(Serialize, Deserialize, Validate, Debug, Clone, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct PutWorkerParams {
    /// 1 to 80 characters.
    pub name: String,
    /// Whole Iraqi dinars, 1,000 to 10,000,000.
    pub cost_iqd: i64,
    /// What the cost pays for. `day` when left out.
    pub cost_per: Option<WorkerCostPer>,
    /// What work the person does, up to 200 characters.
    pub note: Option<String>,
    /// The district the person is in, by its slug.
    pub zone_slug: Option<String>,
    /// Where the person is, WGS84, inside the Kurdistan Region. Given
    /// together with `lon` or not at all.
    pub lat: Option<f64>,
    pub lon: Option<f64>,
    /// `false` takes the card off the list without deleting it. `true`
    /// when left out.
    pub available: Option<bool>,
}

impl PutWorkerParams {
    pub fn into_details(self) -> Result<WorkerDetails, AppError> {
        Ok(WorkerDetails {
            name: WorkerName::new(self.name)?,
            cost: CostIqd::new(self.cost_iqd)?,
            cost_per: self.cost_per.map(Into::into).unwrap_or_default(),
            note: self.note.map(WorkerNote::new).transpose()?.flatten(),
            zone_slug: given(self.zone_slug).map(ZoneSlug::new).transpose()?,
            point: GeoPoint::from_pair(self.lat, self.lon, GeoPoint::in_region)?,
            available: self.available.unwrap_or(true),
        })
    }
}

fn id_of(worker: &Worker) -> Result<String, AppError> {
    worker.id().map(|id| id.to_string()).ok_or_else(|| {
        AppError::GlobalAppError(GlobalAppError::MissingValue(
            "Worker is missing its id".to_string(),
        ))
    })
}

/// A card as its owner reads it.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct WorkerResponse {
    pub id: String,
    pub name: String,
    pub phone: String,
    pub cost_iqd: i32,
    pub cost_per: WorkerCostPer,
    pub note: Option<String>,
    pub zone_slug: Option<String>,
    pub lat: Option<f64>,
    pub lon: Option<f64>,
    pub available: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TryFrom<&Worker> for WorkerResponse {
    type Error = AppError;

    fn try_from(worker: &Worker) -> Result<Self, Self::Error> {
        Ok(Self {
            id: id_of(worker)?,
            name: worker.name().into(),
            phone: worker.phone().into(),
            cost_iqd: worker.cost().value(),
            cost_per: (*worker.cost_per()).into(),
            note: worker.note().as_ref().map(Into::into),
            zone_slug: worker.zone_slug().as_ref().map(Into::into),
            lat: worker.point().map(|point| point.lat()),
            lon: worker.point().map(|point| point.lon()),
            available: *worker.available(),
            created_at: *worker.created_at(),
            updated_at: *worker.updated_at(),
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct OneWorkerResponse {
    pub worker: WorkerResponse,
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct MyWorkerResponse {
    /// `null` when the caller has no card.
    pub worker: Option<WorkerResponse>,
}

/// A card as a farmer reads it on the list. The phone is the number to
/// call; this answer is only ever given to a signed-in caller.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct WorkerCardResponse {
    pub id: String,
    pub name: String,
    pub phone: String,
    pub cost_iqd: i32,
    pub cost_per: WorkerCostPer,
    pub note: Option<String>,
    pub zone_slug: Option<String>,
    pub lat: Option<f64>,
    pub lon: Option<f64>,
    /// From the `lat` and `lon` of the request to the card's point. `null`
    /// when either is missing.
    pub distance_km: Option<f64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TryFrom<&ListedWorker> for WorkerCardResponse {
    type Error = AppError;

    fn try_from(listed: &ListedWorker) -> Result<Self, Self::Error> {
        let card = WorkerResponse::try_from(listed.worker())?;

        Ok(Self {
            id: card.id,
            name: card.name,
            phone: card.phone,
            cost_iqd: card.cost_iqd,
            cost_per: card.cost_per,
            note: card.note,
            zone_slug: card.zone_slug,
            lat: card.lat,
            lon: card.lon,
            distance_km: *listed.distance_km(),
            created_at: card.created_at,
            updated_at: card.updated_at,
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct WorkersResponse {
    pub workers: Vec<WorkerCardResponse>,
    /// How many workers match in all.
    pub count: u64,
    pub page: u64,
    pub rows_per_page: u64,
}

/// A card as Ministry staff read it: the list card plus whether it is on
/// the list.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct DashboardWorkerResponse {
    pub id: String,
    pub name: String,
    pub phone: String,
    pub cost_iqd: i32,
    pub cost_per: WorkerCostPer,
    pub note: Option<String>,
    pub zone_slug: Option<String>,
    pub lat: Option<f64>,
    pub lon: Option<f64>,
    pub available: bool,
    pub distance_km: Option<f64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TryFrom<&ListedWorker> for DashboardWorkerResponse {
    type Error = AppError;

    fn try_from(listed: &ListedWorker) -> Result<Self, Self::Error> {
        let card = WorkerResponse::try_from(listed.worker())?;

        Ok(Self {
            id: card.id,
            name: card.name,
            phone: card.phone,
            cost_iqd: card.cost_iqd,
            cost_per: card.cost_per,
            note: card.note,
            zone_slug: card.zone_slug,
            lat: card.lat,
            lon: card.lon,
            available: card.available,
            distance_km: *listed.distance_km(),
            created_at: card.created_at,
            updated_at: card.updated_at,
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct DashboardWorkersResponse {
    pub workers: Vec<DashboardWorkerResponse>,
    /// How many cards match in all.
    pub count: u64,
    pub page: u64,
    pub rows_per_page: u64,
}

/// A filter sent empty, as in `?zone=&q=`, is not set.
fn given(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn parsed<T: FromStr>(name: &str, value: Option<String>) -> Result<Option<T>, AppError> {
    given(value)
        .map(|value| {
            value.parse::<T>().map_err(|_| {
                AppError::from(DomainError::InvalidValue(format!("{name} is not valid")))
            })
        })
        .transpose()
}

/// The filters both lists share, read from the query as text.
struct Filters {
    zone: Option<ZoneSlug>,
    search: Option<WorkerSearch>,
    max_cost: Option<CostIqd>,
    cost_per: Option<domain::CostPer>,
    near: Option<GeoPoint>,
}

impl Filters {
    fn read(
        zone: Option<String>,
        q: Option<String>,
        lat: Option<String>,
        lon: Option<String>,
        max_cost_iqd: Option<String>,
        cost_per: Option<String>,
    ) -> Result<Self, AppError> {
        Ok(Self {
            zone: given(zone).map(ZoneSlug::new).transpose()?,
            search: given(q).map(WorkerSearch::new).transpose()?,
            max_cost: parsed::<i64>("max_cost_iqd", max_cost_iqd)?
                .map(CostIqd::new)
                .transpose()?,
            cost_per: given(cost_per)
                .map(|cost_per| domain::CostPer::try_from(cost_per.as_str()))
                .transpose()?,
            // The reader may stand anywhere: only the cards must be inside
            // the region.
            near: GeoPoint::from_pair(
                parsed("lat", lat)?,
                parsed("lon", lon)?,
                GeoPoint::anywhere,
            )?,
        })
    }
}

#[derive(Deserialize, Debug, Clone, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct WorkersQuery {
    /// Only workers in this district (its slug).
    pub zone: Option<String>,
    /// Text to look for in the name or the note, whatever the case.
    pub q: Option<String>,
    /// Where the reader stands, WGS84. With `lon`, the workers come nearest
    /// first, each with its `distance_km`.
    pub lat: Option<String>,
    /// Given together with `lat` or not at all.
    pub lon: Option<String>,
    /// Only workers who cost this much or less, 1,000 to 10,000,000.
    pub max_cost_iqd: Option<String>,
    /// Only costs of this kind: `day` or `hour`.
    pub cost_per: Option<String>,
}

impl WorkersQuery {
    pub fn into_input(self, pagination: Pagination) -> Result<BrowseWorkersInput, AppError> {
        let filters = Filters::read(
            self.zone,
            self.q,
            self.lat,
            self.lon,
            self.max_cost_iqd,
            self.cost_per,
        )?;

        Ok(BrowseWorkersInput {
            zone: filters.zone,
            search: filters.search,
            max_cost: filters.max_cost,
            cost_per: filters.cost_per,
            near: filters.near,
            pagination,
        })
    }
}

#[derive(Deserialize, Debug, Clone, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct DashboardWorkersQuery {
    /// Only workers in this district (its slug).
    pub zone: Option<String>,
    /// Text to look for in the name or the note, whatever the case.
    pub q: Option<String>,
    /// With `lon`, the cards come nearest to this point first.
    pub lat: Option<String>,
    /// Given together with `lat` or not at all.
    pub lon: Option<String>,
    /// Only workers who cost this much or less, 1,000 to 10,000,000.
    pub max_cost_iqd: Option<String>,
    /// Only costs of this kind: `day` or `hour`.
    pub cost_per: Option<String>,
    /// `true` for the cards on the farmers' list, `false` for the ones
    /// taken off it. Both when left out.
    pub available: Option<String>,
}

impl DashboardWorkersQuery {
    pub fn into_input(self, pagination: Pagination) -> Result<ListAllWorkersInput, AppError> {
        let filters = Filters::read(
            self.zone,
            self.q,
            self.lat,
            self.lon,
            self.max_cost_iqd,
            self.cost_per,
        )?;

        Ok(ListAllWorkersInput {
            zone: filters.zone,
            search: filters.search,
            max_cost: filters.max_cost,
            cost_per: filters.cost_per,
            available: parsed("available", self.available)?,
            near: filters.near,
            pagination,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::workers::app::testing::{PHONE, a_worker, unavailable};

    fn params(json: serde_json::Value) -> PutWorkerParams {
        serde_json::from_value(json).expect("params")
    }

    fn query() -> WorkersQuery {
        WorkersQuery {
            zone: None,
            q: None,
            lat: None,
            lon: None,
            max_cost_iqd: None,
            cost_per: None,
        }
    }

    fn text(value: &str) -> Option<String> {
        Some(value.to_string())
    }

    #[test]
    fn a_name_and_a_cost_are_enough_and_the_rest_has_defaults() {
        let details = params(serde_json::json!({ "name": "Azad", "cost_iqd": 25000 }))
            .into_details()
            .expect("details");

        assert_eq!(details.cost_per, domain::CostPer::Day);
        assert!(details.available);
        assert_eq!(details.note, None);
        assert_eq!(details.point, None);
    }

    #[test]
    fn a_phone_cannot_be_sent_with_a_card() {
        let result = serde_json::from_value::<PutWorkerParams>(serde_json::json!({
            "name": "Azad", "cost_iqd": 25000, "phone": "+9647701112233"
        }));

        assert!(
            result.is_err(),
            "the phone is the signed-in one, so a typed phone is refused, not ignored"
        );
    }

    #[test]
    fn each_broken_rule_of_a_card_is_refused() {
        let broken = [
            serde_json::json!({ "name": "", "cost_iqd": 25000 }),
            serde_json::json!({ "name": "Azad", "cost_iqd": 0 }),
            serde_json::json!({ "name": "Azad", "cost_iqd": 25000, "lat": 35.5 }),
            serde_json::json!({ "name": "Azad", "cost_iqd": 25000, "lat": 30.0, "lon": 44.6 }),
            serde_json::json!({ "name": "Azad", "cost_iqd": 25000, "zone_slug": "Zone 1" }),
            serde_json::json!({ "name": "Azad", "cost_iqd": 25000, "note": "a".repeat(201) }),
        ];

        for body in broken {
            assert!(params(body.clone()).into_details().is_err(), "{body}");
        }

        assert!(
            serde_json::from_value::<PutWorkerParams>(
                serde_json::json!({ "name": "Azad", "cost_iqd": 25000, "cost_per": "week" })
            )
            .is_err()
        );
    }

    #[test]
    fn the_owner_and_staff_see_whether_the_card_is_on_the_list_and_farmers_do_not() {
        let worker = unavailable(&a_worker(4, PHONE));

        let mine =
            serde_json::to_value(WorkerResponse::try_from(&worker).expect("mine")).expect("json");
        assert_eq!(mine["id"], "4");
        assert_eq!(mine["phone"], PHONE);
        assert_eq!(mine["available"], false);
        assert_eq!(mine["cost_per"], "day");

        let listed = worker.seen_from(None);
        let card = serde_json::to_value(WorkerCardResponse::try_from(&listed).expect("card"))
            .expect("json");
        assert!(card.get("available").is_none());
        assert!(card["distance_km"].is_null());

        let staff =
            serde_json::to_value(DashboardWorkerResponse::try_from(&listed).expect("staff"))
                .expect("json");
        assert_eq!(staff["available"], false);
    }

    #[test]
    fn blank_filters_are_no_filters() {
        let input = WorkersQuery {
            zone: text(""),
            q: text("  "),
            lat: text(""),
            lon: text(""),
            max_cost_iqd: text(""),
            cost_per: text(""),
        }
        .into_input(Pagination::new(1, 20))
        .expect("input");

        assert!(input.zone.is_none() && input.search.is_none() && input.near.is_none());
        assert!(input.max_cost.is_none() && input.cost_per.is_none());
    }

    #[test]
    fn a_readers_point_needs_both_numbers_and_may_be_outside_the_region() {
        let both = WorkersQuery {
            lat: text("51.5"),
            lon: text("-0.12"),
            ..query()
        };
        assert!(
            both.into_input(Pagination::new(1, 20))
                .expect("input")
                .near
                .is_some()
        );

        let one = WorkersQuery {
            lat: text("35.5"),
            ..query()
        };
        assert!(one.into_input(Pagination::new(1, 20)).is_err());

        let words = WorkersQuery {
            lat: text("north"),
            lon: text("45"),
            ..query()
        };
        assert!(words.into_input(Pagination::new(1, 20)).is_err());
    }

    #[test]
    fn an_unknown_cost_per_or_a_cost_out_of_range_is_invalid() {
        let week = WorkersQuery {
            cost_per: text("week"),
            ..query()
        };
        assert!(week.into_input(Pagination::new(1, 20)).is_err());

        let zero = WorkersQuery {
            max_cost_iqd: text("0"),
            ..query()
        };
        assert!(zero.into_input(Pagination::new(1, 20)).is_err());
    }

    #[test]
    fn staff_may_filter_by_availability_with_true_or_false_only() {
        let staff = |available: &str| DashboardWorkersQuery {
            zone: None,
            q: None,
            lat: None,
            lon: None,
            max_cost_iqd: None,
            cost_per: None,
            available: text(available),
        };

        let input = |available: &str| staff(available).into_input(Pagination::new(1, 20));

        assert_eq!(input("false").expect("input").available, Some(false));
        assert_eq!(input("").expect("input").available, None);
        assert!(input("maybe").is_err());
    }

    #[test]
    fn every_cost_per_converts_both_ways() {
        for cost_per in domain::CostPer::ALL {
            assert_eq!(
                domain::CostPer::from(WorkerCostPer::from(cost_per)),
                cost_per
            );
        }
    }
}
