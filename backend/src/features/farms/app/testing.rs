use std::sync::{Arc, Mutex};

use async_trait::async_trait;

use crate::{
    app::{Action, AuthContext, Pagination, Permission, Resource, StaffContext, User},
    features::farms::{
        app::{AppError, FarmRepository, FarmerDirectory},
        domain::{
            Cell, Crop, Farm, FarmLocation, FarmName, FarmSummary, GridCell, IdempotencyKey,
            Outline, OwnedFarmSummary, PaintedCell, Point,
        },
    },
    shared::Phone,
};

pub const OWNER: &str = "+9647501234567";
pub const MAX_CELLS: usize = 1_000;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RepositoryCall {
    FindAllByOwner { owner: String },
    FindByIdempotencyKeyAndOwner { key: String, owner: String },
    FindByIdAndOwner { id: i32, owner: String },
    CountByOwner { owner: String },
    FindAllLocations,
    Exists { id: i32 },
    Create,
    Update,
    Delete { id: i32, owner: String },
    FindPage { owner: Option<String>, page: u64 },
    FindById { id: i32 },
    Rename { id: i32, name: String },
    DeleteById { id: i32 },
    DeleteAllByOwner { owner: String },
}

#[derive(Debug, Default)]
struct Script {
    owned_count: u64,
    existing: Option<Farm>,
    fail_with_database_error: bool,
    nothing_to_delete: bool,
}

#[derive(Debug, Clone, Default)]
pub struct FakeFarmRepository {
    script: Arc<Mutex<Script>>,
    calls: Arc<Mutex<Vec<RepositoryCall>>>,
}

impl FakeFarmRepository {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn owning(count: u64) -> Self {
        let fake = Self::new();
        fake.script.lock().expect("script lock").owned_count = count;
        fake
    }

    pub fn holding(existing: Farm) -> Self {
        let fake = Self::new();
        fake.script.lock().expect("script lock").existing = Some(existing);
        fake
    }

    /// The farm a delete asks for is not there (or belongs to someone else).
    pub fn holding_nothing_to_delete() -> Self {
        let fake = Self::new();
        fake.script.lock().expect("script lock").nothing_to_delete = true;
        fake
    }

    pub fn failing() -> Self {
        let fake = Self::new();
        fake.script
            .lock()
            .expect("script lock")
            .fail_with_database_error = true;
        fake
    }

    pub fn calls(&self) -> Vec<RepositoryCall> {
        self.calls.lock().expect("calls lock").clone()
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
impl FarmRepository for FakeFarmRepository {
    async fn find_all_by_owner(&self, owner: &Phone) -> Result<Vec<FarmSummary>, AppError> {
        self.record(RepositoryCall::FindAllByOwner {
            owner: String::from(owner),
        });
        self.guard()?;

        let script = self.script.lock().expect("script lock");

        Ok(script
            .existing
            .as_ref()
            .map(|one| vec![summary_of(one)])
            .unwrap_or_default())
    }

    async fn find_by_idempotency_key_and_owner(
        &self,
        key: &IdempotencyKey,
        owner: &Phone,
    ) -> Result<Option<Farm>, AppError> {
        self.record(RepositoryCall::FindByIdempotencyKeyAndOwner {
            key: key.as_str().to_string(),
            owner: String::from(owner),
        });
        self.guard()?;

        Ok(self.script.lock().expect("script lock").existing.clone())
    }

    async fn find_by_id_and_owner(&self, id: i32, owner: &Phone) -> Result<Option<Farm>, AppError> {
        self.record(RepositoryCall::FindByIdAndOwner {
            id,
            owner: String::from(owner),
        });
        self.guard()?;

        Ok(self.script.lock().expect("script lock").existing.clone())
    }

    async fn count_by_owner(&self, owner: &Phone) -> Result<u64, AppError> {
        self.record(RepositoryCall::CountByOwner {
            owner: String::from(owner),
        });
        self.guard()?;

        Ok(self.script.lock().expect("script lock").owned_count)
    }

    async fn exists(&self, id: i32) -> Result<bool, AppError> {
        self.record(RepositoryCall::Exists { id });
        self.guard()?;

        let script = self.script.lock().expect("script lock");

        Ok(script
            .existing
            .as_ref()
            .is_some_and(|farm| *farm.id() == Some(id)))
    }

    async fn find_all_locations(&self) -> Result<Vec<FarmLocation>, AppError> {
        self.record(RepositoryCall::FindAllLocations);
        self.guard()?;

        let script = self.script.lock().expect("script lock");

        Ok(script
            .existing
            .as_ref()
            .map(|one| {
                vec![FarmLocation::rehydrate(
                    one.id().unwrap_or_default(),
                    one.outline(),
                )]
            })
            .unwrap_or_default())
    }

    async fn create(&self, entity: &Farm) -> Result<Farm, AppError> {
        self.record(RepositoryCall::Create);
        self.guard()?;

        Ok(persisted(entity, 1))
    }

    async fn update(&self, entity: &Farm) -> Result<Farm, AppError> {
        self.record(RepositoryCall::Update);
        self.guard()?;

        Ok(entity.clone())
    }

    async fn delete(&self, id: i32, owner: &Phone) -> Result<(), AppError> {
        self.record(RepositoryCall::Delete {
            id,
            owner: String::from(owner),
        });
        self.guard()?;

        if self.script.lock().expect("script lock").nothing_to_delete {
            return Err(crate::app::AppError::NotFound.into());
        }

        Ok(())
    }

    async fn find_page(
        &self,
        owner: Option<&Phone>,
        pagination: &Pagination,
    ) -> Result<(Vec<OwnedFarmSummary>, u64), AppError> {
        self.record(RepositoryCall::FindPage {
            owner: owner.map(String::from),
            page: *pagination.page(),
        });
        self.guard()?;

        let script = self.script.lock().expect("script lock");

        let farms: Vec<OwnedFarmSummary> = script
            .existing
            .iter()
            .filter(|one| owner.is_none_or(|owner| one.is_owned_by(owner)))
            .map(|one| OwnedFarmSummary::new(one.owner().clone(), summary_of(one)))
            .collect();
        let count = farms.len() as u64;

        Ok((farms, count))
    }

    async fn find_by_id(&self, id: i32) -> Result<Option<Farm>, AppError> {
        self.record(RepositoryCall::FindById { id });
        self.guard()?;

        Ok(self.script.lock().expect("script lock").existing.clone())
    }

    async fn rename(
        &self,
        id: i32,
        name: &FarmName,
        now: chrono::DateTime<chrono::Utc>,
    ) -> Result<Option<Farm>, AppError> {
        self.record(RepositoryCall::Rename {
            id,
            name: name.as_str().to_string(),
        });
        self.guard()?;

        let script = self.script.lock().expect("script lock");

        Ok(script.existing.as_ref().map(|one| {
            Farm::rehydrate(
                id,
                name.clone(),
                one.owner().clone(),
                one.outline().clone(),
                one.cells().clone(),
                one.idempotency_key().clone(),
                *one.created_offline_at(),
                *one.created_at(),
                now,
            )
        }))
    }

    async fn delete_by_id(&self, id: i32) -> Result<bool, AppError> {
        self.record(RepositoryCall::DeleteById { id });
        self.guard()?;

        Ok(!self.script.lock().expect("script lock").nothing_to_delete)
    }

    async fn delete_all_by_owner(&self, owner: &Phone) -> Result<u64, AppError> {
        self.record(RepositoryCall::DeleteAllByOwner {
            owner: String::from(owner),
        });
        self.guard()?;

        Ok(self.script.lock().expect("script lock").owned_count)
    }
}

/// Stands in for the farmers feature: every phone is registered, or none is.
#[derive(Debug, Clone)]
pub struct FakeFarmerDirectory {
    registered: bool,
    asked: Arc<Mutex<Vec<String>>>,
}

impl FakeFarmerDirectory {
    pub fn knowing_everyone() -> Self {
        Self {
            registered: true,
            asked: Arc::default(),
        }
    }

    pub fn knowing_no_one() -> Self {
        Self {
            registered: false,
            asked: Arc::default(),
        }
    }

    /// The phones it was asked about, in order.
    pub fn asked(&self) -> Vec<String> {
        self.asked.lock().expect("asked lock").clone()
    }
}

#[async_trait]
impl FarmerDirectory for FakeFarmerDirectory {
    async fn is_registered(&self, phone: &Phone) -> Result<bool, AppError> {
        self.asked
            .lock()
            .expect("asked lock")
            .push(String::from(phone));

        Ok(self.registered)
    }
}

pub const STAFF_ID: i32 = 3;

/// A staff member holding every permission on farms.
pub fn staff_context() -> StaffContext {
    StaffContext::new(
        STAFF_ID,
        "officer@example.org".to_string(),
        Action::ALL
            .into_iter()
            .map(|action| Permission::new(Resource::Farms, action))
            .collect(),
    )
}

fn persisted(entity: &Farm, id: i32) -> Farm {
    let cells = entity
        .cells()
        .iter()
        .zip(1..)
        .map(|(cell, cell_id)| Cell::rehydrate(cell_id, cell.position(), cell.crop()))
        .collect();

    Farm::rehydrate(
        id,
        entity.name().clone(),
        entity.owner().clone(),
        entity.outline().clone(),
        cells,
        entity.idempotency_key().clone(),
        *entity.created_offline_at(),
        *entity.created_at(),
        *entity.updated_at(),
    )
}

fn summary_of(farm: &Farm) -> FarmSummary {
    let cells_per_crop = Crop::ALL
        .into_iter()
        .map(|crop| {
            let cells = farm.cells().iter().filter(|cell| cell.crop() == crop);

            (crop, cells.count())
        })
        .collect();

    FarmSummary::rehydrate(
        farm.id().unwrap_or_default(),
        farm.name().clone(),
        farm.outline(),
        cells_per_crop,
        *farm.created_at(),
    )
}

pub fn auth_context() -> AuthContext {
    AuthContext::new(
        User::new(Phone::new(OWNER.to_string()).expect("phone")),
        "token".to_string(),
    )
}

/// Roughly 90 m east to west and 111 m south to north.
pub fn an_outline() -> Outline {
    Outline::new(vec![
        Point::new(36.0300, 44.6000, None, None).expect("point"),
        Point::new(36.0300, 44.6010, None, None).expect("point"),
        Point::new(36.0310, 44.6010, None, None).expect("point"),
        Point::new(36.0310, 44.6000, None, None).expect("point"),
    ])
    .expect("outline")
}

pub fn a_cell_inside() -> GridCell {
    an_outline().cells(MAX_CELLS).expect("cells")[0]
}

pub fn a_cell_outside() -> GridCell {
    GridCell::new(1, 1)
}

/// A persisted farm with id 7 and one cell of wheat.
pub fn a_farm() -> Farm {
    let (farm, _) = Farm::new(
        FarmName::new("Upper field".to_string()).expect("name"),
        Phone::new(OWNER.to_string()).expect("phone"),
        an_outline(),
        vec![PaintedCell::new(a_cell_inside(), Crop::Wheat)],
        None,
        None,
        MAX_CELLS,
    )
    .expect("farm");

    persisted(&farm, 7)
}
