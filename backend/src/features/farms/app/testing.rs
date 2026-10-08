use std::sync::{Arc, Mutex};

use async_trait::async_trait;

use crate::{
    app::{AuthContext, Pagination, User},
    features::farms::{
        app::{AppError, FarmRepository},
        domain::{Cell, Crop, Farm, FarmName, FarmSummary, GridCell, Outline, PaintedCell, Point},
    },
    shared::Phone,
};

pub const OWNER: &str = "+9647501234567";
pub const MAX_CELLS: usize = 1_000;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RepositoryCall {
    FindAllByOwner { owner: String, page: u64 },
    FindByIdAndOwner { id: i32, owner: String },
    CountByOwner { owner: String },
    Create,
    Update,
    Delete { id: i32, owner: String },
}

#[derive(Debug, Default)]
struct Script {
    owned_count: u64,
    existing: Option<Farm>,
    fail_with_database_error: bool,
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
    async fn find_all_by_owner(
        &self,
        owner: &Phone,
        pagination: &Pagination,
    ) -> Result<(Vec<FarmSummary>, Option<u64>), AppError> {
        self.record(RepositoryCall::FindAllByOwner {
            owner: String::from(owner),
            page: *pagination.page(),
        });
        self.guard()?;

        let script = self.script.lock().expect("script lock");
        let rows = script
            .existing
            .as_ref()
            .map(|one| vec![summary_of(one)])
            .unwrap_or_default();
        let total = pagination.is_first_page().then_some(script.owned_count);

        Ok((rows, total))
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

        Ok(())
    }
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
        MAX_CELLS,
    )
    .expect("farm");

    persisted(&farm, 7)
}
