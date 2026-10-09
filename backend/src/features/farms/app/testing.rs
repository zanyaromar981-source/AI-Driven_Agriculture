use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use async_trait::async_trait;

use crate::{
    app::{Action, AuthContext, Pagination, Permission, Resource, StaffContext, User},
    features::farms::{
        app::{
            AppError, AreaDirectory, CropDirectory, FarmRepository, FarmerDirectory, PlaceLocator,
            PublicTotalsSwitch,
        },
        domain::{
            ActiveCrops, AreaCount, AreaCropSum, AreaLevel, AreaNames, Cell, Crop, Farm,
            FarmFilter, FarmLocation, FarmName, FarmOrder, FarmPlace, FarmSummary, GovernorateName,
            GridCell, IdempotencyKey, Outline, OwnedFarmSummary, PaintedCell, Point, SubZoneName,
            UnplacedFarm, ZoneName,
        },
    },
    shared::Phone,
};

pub const OWNER: &str = "+9647501234567";
pub const MAX_CELLS: usize = 1_000;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RepositoryCall {
    FindAllByOwner {
        owner: String,
    },
    FindByIdempotencyKeyAndOwner {
        key: String,
        owner: String,
    },
    FindByIdAndOwner {
        id: i32,
        owner: String,
    },
    CountByOwner {
        owner: String,
    },
    FindAllLocations,
    Exists {
        id: i32,
    },
    Create,
    Update,
    Replace {
        id: i32,
        owner: String,
    },
    Delete {
        id: i32,
        owner: String,
    },
    FindPage {
        filter: FarmFilter,
        order: FarmOrder,
        page: u64,
    },
    SumByArea {
        filter: FarmFilter,
        deepest: AreaLevel,
    },
    FindUnplaced {
        after_id: i32,
        limit: u64,
    },
    FillPlace {
        id: i32,
        place: Option<FarmPlace>,
    },
    FindById {
        id: i32,
    },
    Rename {
        id: i32,
        name: String,
    },
    DeleteById {
        id: i32,
    },
    DeleteAllByOwner {
        owner: String,
    },
    FindSummariesByIds {
        ids: Vec<i32>,
    },
    IsCropPainted {
        crop: Crop,
    },
}

#[derive(Debug, Default)]
struct Script {
    owned_count: u64,
    existing: Option<Farm>,
    fail_with_database_error: bool,
    nothing_to_delete: bool,
    gone_before_the_write: bool,
    unplaced: Vec<UnplacedFarm>,
    written_while_filling: Vec<i32>,
    counts: Vec<AreaCount>,
    crop_sums: Vec<AreaCropSum>,
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

    /// The farm is there when it is looked up and deleted before the write.
    pub fn holding_one_that_vanishes(existing: Farm) -> Self {
        let fake = Self::holding(existing);
        fake.script
            .lock()
            .expect("script lock")
            .gone_before_the_write = true;
        fake
    }

    /// Holds these farms with no place and no area stored.
    pub fn holding_unplaced(unplaced: Vec<UnplacedFarm>) -> Self {
        let fake = Self::new();
        fake.script.lock().expect("script lock").unplaced = unplaced;
        fake
    }

    /// These farms are written by someone else between being read and
    /// being given their place.
    pub fn written_while_filling(self, ids: Vec<i32>) -> Self {
        self.script
            .lock()
            .expect("script lock")
            .written_while_filling = ids;
        self
    }

    /// What `sum_by_area` answers.
    pub fn summing(counts: Vec<AreaCount>, crop_sums: Vec<AreaCropSum>) -> Self {
        let fake = Self::new();

        {
            let mut script = fake.script.lock().expect("script lock");
            script.counts = counts;
            script.crop_sums = crop_sums;
        }

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

    async fn replace(&self, entity: &Farm) -> Result<Farm, AppError> {
        let id = entity.id().unwrap_or_default();

        self.record(RepositoryCall::Replace {
            id,
            owner: String::from(entity.owner()),
        });
        self.guard()?;

        let mut script = self.script.lock().expect("script lock");

        if script.gone_before_the_write {
            return Err(crate::app::AppError::NotFound.into());
        }

        let replaced = persisted(entity, id);
        script.existing = Some(replaced.clone());

        Ok(replaced)
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
        filter: &FarmFilter,
        order: FarmOrder,
        pagination: &Pagination,
    ) -> Result<(Vec<OwnedFarmSummary>, u64), AppError> {
        self.record(RepositoryCall::FindPage {
            filter: filter.clone(),
            order,
            page: *pagination.page(),
        });
        self.guard()?;

        let script = self.script.lock().expect("script lock");
        let owner = filter.owner.as_ref();

        let farms: Vec<OwnedFarmSummary> = script
            .existing
            .iter()
            .filter(|one| owner.is_none_or(|owner| one.is_owned_by(owner)))
            .map(|one| OwnedFarmSummary::new(one.owner().clone(), summary_of(one)))
            .collect();
        let count = farms.len() as u64;

        Ok((farms, count))
    }

    async fn sum_by_area(
        &self,
        filter: &FarmFilter,
        deepest: AreaLevel,
    ) -> Result<(Vec<AreaCount>, Vec<AreaCropSum>), AppError> {
        self.record(RepositoryCall::SumByArea {
            filter: filter.clone(),
            deepest,
        });
        self.guard()?;

        let script = self.script.lock().expect("script lock");

        Ok((script.counts.clone(), script.crop_sums.clone()))
    }

    async fn find_unplaced(
        &self,
        after_id: i32,
        limit: u64,
    ) -> Result<Vec<UnplacedFarm>, AppError> {
        self.record(RepositoryCall::FindUnplaced { after_id, limit });
        self.guard()?;

        let script = self.script.lock().expect("script lock");

        Ok(script
            .unplaced
            .iter()
            .filter(|farm| *farm.id() > after_id)
            .take(limit as usize)
            .cloned()
            .collect())
    }

    async fn fill_place(
        &self,
        farm: &UnplacedFarm,
        place: Option<&FarmPlace>,
    ) -> Result<bool, AppError> {
        self.record(RepositoryCall::FillPlace {
            id: *farm.id(),
            place: place.cloned(),
        });
        self.guard()?;

        let script = self.script.lock().expect("script lock");

        Ok(!script.written_while_filling.contains(farm.id()))
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
            let mut renamed = Farm::rehydrate(
                id,
                name.clone(),
                one.owner().clone(),
                one.outline().clone(),
                one.cells().clone(),
                one.idempotency_key().clone(),
                *one.created_offline_at(),
                *one.created_at(),
                now,
            );
            renamed.place_at(one.place().clone());

            renamed
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

    async fn find_summaries_by_ids(&self, ids: &[i32]) -> Result<Vec<FarmSummary>, AppError> {
        self.record(RepositoryCall::FindSummariesByIds { ids: ids.to_vec() });
        self.guard()?;

        let script = self.script.lock().expect("script lock");

        Ok(script
            .existing
            .iter()
            .filter(|one| one.id().is_some_and(|id| ids.contains(&id)))
            .map(summary_of)
            .collect())
    }

    async fn is_crop_painted(&self, crop: Crop) -> Result<bool, AppError> {
        self.record(RepositoryCall::IsCropPainted { crop });
        self.guard()?;

        let script = self.script.lock().expect("script lock");

        Ok(script
            .existing
            .iter()
            .any(|farm| farm.cells().iter().any(|cell| cell.crop() == crop)))
    }
}

/// Stands in for the crops feature: the crops staff have switched on.
#[derive(Debug, Clone, Default)]
pub struct FakeCropDirectory {
    active: Vec<Crop>,
    failing: bool,
    asked: Arc<Mutex<u32>>,
}

impl FakeCropDirectory {
    /// The farm crops that were fixed in code before staff kept the list.
    pub fn seeded() -> Self {
        Self::with(&[
            "wheat",
            "barley",
            "tomato",
            "cucumber",
            "potato",
            "onion",
            "watermelon",
            "grape",
            "olive",
            "sunflower",
            "chickpea",
        ])
    }

    pub fn with(codes: &[&str]) -> Self {
        Self {
            active: codes.iter().map(|code| Crop::of(code)).collect(),
            ..Self::default()
        }
    }

    pub fn failing() -> Self {
        Self {
            failing: true,
            ..Self::default()
        }
    }

    /// How many times the list was asked for.
    pub fn asked(&self) -> u32 {
        *self.asked.lock().expect("asked lock")
    }
}

#[async_trait]
impl CropDirectory for FakeCropDirectory {
    async fn active(&self) -> Result<ActiveCrops, AppError> {
        *self.asked.lock().expect("asked lock") += 1;

        if self.failing {
            return Err(crate::app::AppError::InternalServerError.into());
        }

        Ok(ActiveCrops::new(self.active.iter().copied()))
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

/// Stands in for the zones feature's map. Every point north of latitude
/// 36.025 is in `the_place()` and every point south of it is in no place,
/// so `an_outline()` and `another_outline()` are inside and
/// `an_outline_outside()` is not.
#[derive(Debug, Clone, Default)]
pub struct FakePlaceLocator {
    failing: bool,
    asked: Arc<Mutex<Vec<(f64, f64)>>>,
}

pub const PLACE_SOUTH_EDGE: f64 = 36.025;

impl FakePlaceLocator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn failing() -> Self {
        Self {
            failing: true,
            asked: Arc::default(),
        }
    }

    /// The points it was asked about as `(lat, lon)`, in order.
    pub fn asked(&self) -> Vec<(f64, f64)> {
        self.asked.lock().expect("asked lock").clone()
    }
}

#[async_trait]
impl PlaceLocator for FakePlaceLocator {
    async fn locate(&self, lat: f64, lon: f64) -> Result<Option<FarmPlace>, AppError> {
        self.asked.lock().expect("asked lock").push((lat, lon));

        if self.failing {
            return Err(crate::app::AppError::InternalServerError.into());
        }

        Ok((lat > PLACE_SOUTH_EDGE).then(the_place))
    }
}

pub fn the_place() -> FarmPlace {
    FarmPlace::new(
        "Sulaymaniyah".to_string(),
        "chamchamal".to_string(),
        "sangaw".to_string(),
    )
    .expect("place")
}

/// Stands in for the zones feature's names.
#[derive(Debug, Clone, Default)]
pub struct FakeAreaDirectory {
    failing: bool,
    asked: Arc<Mutex<u32>>,
}

impl FakeAreaDirectory {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn failing() -> Self {
        Self {
            failing: true,
            asked: Arc::default(),
        }
    }

    /// How many times the names were asked for.
    pub fn asked(&self) -> u32 {
        *self.asked.lock().expect("asked lock")
    }
}

#[async_trait]
impl AreaDirectory for FakeAreaDirectory {
    async fn names(&self) -> Result<AreaNames, AppError> {
        *self.asked.lock().expect("asked lock") += 1;

        if self.failing {
            return Err(crate::app::AppError::InternalServerError.into());
        }

        Ok(area_names())
    }
}

/// Stands in for the app settings' public totals switch.
#[derive(Debug, Clone, Copy)]
pub struct FakeTotalsSwitch(pub bool);

#[async_trait]
impl PublicTotalsSwitch for FakeTotalsSwitch {
    async fn is_on(&self) -> Result<bool, AppError> {
        Ok(self.0)
    }
}

/// Sulaymaniyah with Chamchamal and its sub-zone Sangaw.
pub fn area_names() -> AreaNames {
    AreaNames {
        governorates: vec![GovernorateName {
            name_en: "Sulaymaniyah".to_string(),
            name_ku: "سلێمانی".to_string(),
        }],
        zones: vec![ZoneName {
            slug: "chamchamal".to_string(),
            name_en: "Chamchamal".to_string(),
            name_ku: "چەمچەماڵ".to_string(),
        }],
        sub_zones: vec![SubZoneName {
            zone_slug: "chamchamal".to_string(),
            slug: "sangaw".to_string(),
            name_en: "Sangaw".to_string(),
            name_ku: "سەنگاو".to_string(),
        }],
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
        .map(|(cell, cell_id)| {
            Cell::rehydrate(cell_id, cell.position(), cell.crop(), cell.inside_pct())
        })
        .collect();

    let mut stored = Farm::rehydrate(
        id,
        entity.name().clone(),
        entity.owner().clone(),
        entity.outline().clone(),
        cells,
        entity.idempotency_key().clone(),
        *entity.created_offline_at(),
        *entity.created_at(),
        *entity.updated_at(),
    );
    stored.place_at(entity.place().clone());

    stored
}

fn summary_of(farm: &Farm) -> FarmSummary {
    let mut inside: HashMap<Crop, f64> = HashMap::new();

    for cell in farm.cells() {
        *inside.entry(cell.crop()).or_default() += cell.inside_pct();
    }

    let inside_per_crop = inside.into_iter().collect();

    FarmSummary::rehydrate(
        farm.id().unwrap_or_default(),
        farm.name().clone(),
        farm.outline(),
        farm.place().clone(),
        inside_per_crop,
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
    an_outline().cells(MAX_CELLS).expect("cells")[0].position()
}

/// A smaller field inside `an_outline`, for an edit that moves the border.
pub fn another_outline() -> Outline {
    Outline::new(vec![
        Point::new(36.0302, 44.6002, None, None).expect("point"),
        Point::new(36.0302, 44.6008, None, None).expect("point"),
        Point::new(36.0307, 44.6008, None, None).expect("point"),
        Point::new(36.0307, 44.6002, None, None).expect("point"),
    ])
    .expect("outline")
}

/// A field south of `an_outline`, where `FakePlaceLocator` knows no place.
pub fn an_outline_outside() -> Outline {
    Outline::new(vec![
        Point::new(36.0200, 44.6000, None, None).expect("point"),
        Point::new(36.0200, 44.6010, None, None).expect("point"),
        Point::new(36.0210, 44.6010, None, None).expect("point"),
        Point::new(36.0210, 44.6000, None, None).expect("point"),
    ])
    .expect("outline")
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
        vec![PaintedCell::new(a_cell_inside(), Crop::of("wheat"))],
        None,
        None,
        MAX_CELLS,
    )
    .expect("farm");

    persisted(&farm, 7)
}
