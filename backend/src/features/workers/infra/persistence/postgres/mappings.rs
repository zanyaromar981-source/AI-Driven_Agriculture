use sea_orm::ActiveValue::{NotSet, Set};

use crate::{
    features::workers::{
        app::AppError,
        domain::{
            CostIqd, CostPer, GeoPoint, Worker, WorkerDetails, WorkerName, WorkerNote, ZoneSlug,
        },
        infra::persistence::postgres::entities::workers,
    },
    shared::Phone,
};

impl TryFrom<workers::Model> for Worker {
    type Error = AppError;

    fn try_from(model: workers::Model) -> Result<Self, Self::Error> {
        let details = WorkerDetails {
            name: WorkerName::new(model.name)?,
            cost: CostIqd::new(i64::from(model.cost_iqd))?,
            cost_per: CostPer::try_from(model.cost_per.as_str())?,
            note: model.note.map(WorkerNote::new).transpose()?.flatten(),
            zone_slug: model.zone_slug.map(ZoneSlug::new).transpose()?,
            point: GeoPoint::from_pair(model.lat, model.lon, GeoPoint::in_region)?,
            available: model.available,
        };

        Ok(Worker::rehydrate(
            model.id,
            Phone::new(model.phone)?,
            details,
            model.created_at.and_utc(),
            model.updated_at.and_utc(),
        ))
    }
}

impl From<&Worker> for workers::ActiveModel {
    fn from(worker: &Worker) -> Self {
        workers::ActiveModel {
            id: match *worker.id() {
                Some(id) => Set(id),
                None => NotSet,
            },
            phone: Set(worker.phone().into()),
            name: Set(worker.name().into()),
            cost_iqd: Set(worker.cost().value()),
            cost_per: Set((*worker.cost_per()).into()),
            note: Set(worker.note().as_ref().map(Into::into)),
            zone_slug: Set(worker.zone_slug().as_ref().map(Into::into)),
            lat: Set(worker.point().map(|point| point.lat())),
            lon: Set(worker.point().map(|point| point.lon())),
            available: Set(*worker.available()),
            created_at: Set(worker.created_at().naive_utc()),
            updated_at: Set(worker.updated_at().naive_utc()),
        }
    }
}
