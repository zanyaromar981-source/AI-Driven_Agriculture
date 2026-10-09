use std::collections::HashMap;

use async_trait::async_trait;
use chrono::{DateTime, NaiveDateTime, Utc};
use sea_orm::{
    AccessMode, ActiveModelTrait, ColumnTrait, Condition, ConnectionTrait, DatabaseConnection,
    DbErr, EntityTrait, FromQueryResult, IsolationLevel, JoinType, Order, PaginatorTrait,
    QueryFilter, QueryOrder, QuerySelect, RelationTrait, TransactionTrait,
    sea_query::{Expr, Func, NullOrdering, Query},
};

use crate::{
    app::{AppError as GlobalAppError, Pagination},
    features::farms::{
        app::{AppError, FarmRepository},
        domain::{
            AreaCount, AreaCropSum, AreaFilter, AreaKey, AreaLevel, Cell, Crop, Farm, FarmFilter,
            FarmLocation, FarmName, FarmOrder, FarmPlace, FarmSortKey, FarmSummary, IdempotencyKey,
            OwnedFarmSummary, SortDirection, UnplacedFarm,
        },
        infra::persistence::postgres::{
            entities::{farm_cells, farms},
            mappings::{cell_active_model, stored_outline},
        },
    },
    shared::Phone,
};

/// Postgres accepts 65,535 bind parameters per statement. A cell insert binds
/// five, an id list binds one per cell.
const CELLS_PER_INSERT: usize = 10_000;
const IDS_PER_UPDATE: usize = 30_000;

fn database_error(error: DbErr) -> AppError {
    tracing::error!(%error, "farm repository query failed");

    GlobalAppError::DatabaseError(error.to_string()).into()
}

#[derive(FromQueryResult)]
struct InsidePerCrop {
    farm_id: i32,
    crop: String,
    inside_pct: f64,
}

/// One row of the farms added up: the region when all three place columns
/// are rolled up, otherwise the area the grouped ones name.
#[derive(FromQueryResult)]
struct AreaCountRow {
    governorate: Option<String>,
    zone_slug: Option<String>,
    sub_zone_slug: Option<String>,
    rolled_up: i32,
    farms: i64,
    farmers: i64,
    area_m2: Option<f64>,
    latest_change: Option<NaiveDateTime>,
}

#[derive(FromQueryResult)]
struct AreaCropRow {
    governorate: Option<String>,
    zone_slug: Option<String>,
    sub_zone_slug: Option<String>,
    rolled_up: i32,
    crop: String,
    inside_pct: Option<f64>,
    farms: i64,
    farmers: i64,
}

const GOVERNORATE: &str = r#""farms"."governorate""#;
const ZONE_SLUG: &str = r#""farms"."zone_slug""#;
const SUB_ZONE_SLUG: &str = r#""farms"."sub_zone_slug""#;
const CELL_CROP: &str = r#""farm_cells"."crop""#;

/// The grouping sets for the region and each kind of area down to
/// `deepest`, each with `also` (a column every set groups by, or nothing).
/// One pass over the rows fills every set.
fn grouping_sets(deepest: AreaLevel, also: Option<&str>) -> String {
    let mut sets = vec![
        vec![],
        vec![GOVERNORATE],
        vec![GOVERNORATE, ZONE_SLUG],
        vec![GOVERNORATE, ZONE_SLUG, SUB_ZONE_SLUG],
    ];

    if deepest != AreaLevel::SubZone {
        sets.pop();
    }

    let sets: Vec<String> = sets
        .into_iter()
        .map(|mut columns| {
            columns.extend(also);

            format!("({})", columns.join(", "))
        })
        .collect();

    format!("GROUPING SETS ({})", sets.join(", "))
}

/// A number that tells the sets of `grouping_sets` apart: 7 for the region,
/// 3 for a governorate, 1 for a zone, 0 for a sub-zone. A null place column
/// alone cannot tell them apart, because a farm with no place has nulls
/// too. When sub-zones are not grouped the column may not be named at all,
/// so its bit is set by hand.
fn rolled_up(deepest: AreaLevel) -> String {
    if deepest == AreaLevel::SubZone {
        format!("GROUPING({GOVERNORATE}, {ZONE_SLUG}, {SUB_ZONE_SLUG})")
    } else {
        format!("GROUPING({GOVERNORATE}, {ZONE_SLUG}) * 2 + 1")
    }
}

/// The sub-zone column of a sum, or a null of its type when sub-zones are
/// not grouped and the column may not be selected.
fn sub_zone_column(deepest: AreaLevel) -> &'static str {
    if deepest == AreaLevel::SubZone {
        SUB_ZONE_SLUG
    } else {
        "CAST(NULL AS varchar)"
    }
}

fn level_of(rolled_up: i32) -> Result<AreaLevel, AppError> {
    match rolled_up {
        7 => Ok(AreaLevel::Region),
        3 => Ok(AreaLevel::Governorate),
        1 => Ok(AreaLevel::Zone),
        0 => Ok(AreaLevel::SubZone),
        other => Err(GlobalAppError::MissingValue(format!(
            "Farm sums came back grouped in a way that was not asked for: {other}"
        ))
        .into()),
    }
}

/// Makes the text match itself inside a `LIKE` pattern whose escape
/// character is the backslash, the one Postgres uses by default: `%` and `_` stand for themselves instead of
/// "anything" and "any one character".
fn like_literal(text: &str) -> String {
    let mut literal = String::with_capacity(text.len());

    for character in text.chars() {
        if matches!(character, '\\' | '%' | '_') {
            literal.push('\\');
        }

        literal.push(character);
    }

    literal
}

/// The farms a filter keeps, as a condition on the `farms` table. The crop
/// part is left out when `with_crop` is false, for a query that joins the
/// cells and narrows them itself.
fn kept_by(filter: &FarmFilter, with_crop: bool) -> Condition {
    let mut condition = Condition::all();

    if let Some(owner) = &filter.owner {
        condition = condition.add(farms::Column::Phone.eq(owner.as_str()));
    }

    if let Some(governorate) = &filter.governorate {
        condition = condition.add(match governorate {
            AreaFilter::Named(name) => {
                Expr::cust_with_values(format!("LOWER({GOVERNORATE}) = $1"), [name.as_str()])
            }
            AreaFilter::Unknown => farms::Column::Governorate.is_null(),
        });
    }

    if let Some(zone) = &filter.zone {
        condition = condition.add(match zone {
            AreaFilter::Named(slug) => farms::Column::ZoneSlug.eq(slug.as_str()),
            AreaFilter::Unknown => farms::Column::ZoneSlug.is_null(),
        });
    }

    if let Some(sub_zone) = &filter.sub_zone {
        condition = condition.add(match sub_zone {
            AreaFilter::Named(slug) => farms::Column::SubZoneSlug.eq(slug.as_str()),
            AreaFilter::Unknown => farms::Column::SubZoneSlug.is_null(),
        });
    }

    if let Some(crop) = filter.crop.filter(|_| with_crop) {
        condition = condition.add(
            farms::Column::Id.in_subquery(
                Query::select()
                    .column(farm_cells::Column::FarmId)
                    .from(farm_cells::Entity)
                    .and_where(farm_cells::Column::Crop.eq(String::from(crop.crop())))
                    .to_owned(),
            ),
        );
    }

    if let Some(search) = &filter.search {
        let pattern = format!("%{}%", like_literal(search.as_str()));
        // No ESCAPE clause: the backslash `like_literal` writes is the
        // escape character Postgres uses when none is named. The method is
        // called by its path because with the trait in scope its `contains`
        // and `matches` would take over those of every string in this file.
        let contains = |column: farms::Column| {
            sea_orm::sea_query::extension::postgres::PgExpr::ilike(
                Expr::col((farms::Entity, column)),
                pattern.clone(),
            )
        };

        condition = condition.add(
            Condition::any()
                .add(contains(farms::Column::Name))
                .add(contains(farms::Column::Phone)),
        );
    }

    condition
}

#[derive(Debug)]
pub struct FarmPostgresRepository {
    conn: DatabaseConnection,
}

impl FarmPostgresRepository {
    pub fn new(conn: DatabaseConnection) -> Self {
        Self { conn }
    }

    async fn load<C: ConnectionTrait>(conn: &C, model: farms::Model) -> Result<Farm, AppError> {
        let cells = farm_cells::Entity::find()
            .filter(farm_cells::Column::FarmId.eq(model.id))
            .order_by_asc(farm_cells::Column::Id)
            .all(conn)
            .await
            .map_err(database_error)?;

        Farm::try_from((model, cells))
    }

    /// For each of the farms, the summed `inside_pct` of the cells under
    /// each crop: the area of the crop, not a count of its cells.
    async fn inside_per_crop(
        &self,
        farm_ids: Vec<i32>,
    ) -> Result<HashMap<i32, Vec<(String, f64)>>, AppError> {
        let sums = farm_cells::Entity::find()
            .select_only()
            .column(farm_cells::Column::FarmId)
            .column(farm_cells::Column::Crop)
            .column_as(farm_cells::Column::InsidePct.sum(), "inside_pct")
            .filter(farm_cells::Column::FarmId.is_in(farm_ids))
            .group_by(farm_cells::Column::FarmId)
            .group_by(farm_cells::Column::Crop)
            .into_model::<InsidePerCrop>()
            .all(&self.conn)
            .await
            .map_err(database_error)?;

        let mut inside_per_crop: HashMap<i32, Vec<(String, f64)>> = HashMap::new();

        for sum in sums {
            inside_per_crop
                .entry(sum.farm_id)
                .or_default()
                .push((sum.crop, sum.inside_pct));
        }

        Ok(inside_per_crop)
    }

    async fn insert_cells<C: ConnectionTrait>(
        conn: &C,
        farm_id: i32,
        cells: &[Cell],
    ) -> Result<(), AppError> {
        for cells in cells.chunks(CELLS_PER_INSERT) {
            farm_cells::Entity::insert_many(
                cells.iter().map(|cell| cell_active_model(farm_id, cell)),
            )
            .exec(conn)
            .await
            .map_err(database_error)?;
        }

        Ok(())
    }
}

#[async_trait]
impl FarmRepository for FarmPostgresRepository {
    async fn find_all_by_owner(&self, owner: &Phone) -> Result<Vec<FarmSummary>, AppError> {
        let models = farms::Entity::find()
            .filter(farms::Column::Phone.eq(owner.as_str()))
            .order_by_asc(farms::Column::Id)
            .all(&self.conn)
            .await
            .map_err(database_error)?;

        let mut inside_per_crop = self
            .inside_per_crop(models.iter().map(|model| model.id).collect())
            .await?;

        models
            .into_iter()
            .map(|model| {
                let cells = inside_per_crop.remove(&model.id).unwrap_or_default();

                FarmSummary::try_from((model, cells))
            })
            .collect()
    }

    async fn find_by_idempotency_key_and_owner(
        &self,
        key: &IdempotencyKey,
        owner: &Phone,
    ) -> Result<Option<Farm>, AppError> {
        let model = farms::Entity::find()
            .filter(farms::Column::Phone.eq(owner.as_str()))
            .filter(farms::Column::IdempotencyKey.eq(key.as_str()))
            .one(&self.conn)
            .await
            .map_err(database_error)?;

        match model {
            Some(model) => Ok(Some(Self::load(&self.conn, model).await?)),
            None => Ok(None),
        }
    }

    async fn find_by_id_and_owner(&self, id: i32, owner: &Phone) -> Result<Option<Farm>, AppError> {
        let model = farms::Entity::find_by_id(id)
            .filter(farms::Column::Phone.eq(owner.as_str()))
            .one(&self.conn)
            .await
            .map_err(database_error)?;

        match model {
            Some(model) => Ok(Some(Self::load(&self.conn, model).await?)),
            None => Ok(None),
        }
    }

    async fn count_by_owner(&self, owner: &Phone) -> Result<u64, AppError> {
        farms::Entity::find()
            .filter(farms::Column::Phone.eq(owner.as_str()))
            .count(&self.conn)
            .await
            .map_err(database_error)
    }

    async fn exists(&self, id: i32) -> Result<bool, AppError> {
        farms::Entity::find_by_id(id)
            .count(&self.conn)
            .await
            .map(|count| count > 0)
            .map_err(database_error)
    }

    async fn find_all_locations(&self) -> Result<Vec<FarmLocation>, AppError> {
        let models = farms::Entity::find()
            .order_by_asc(farms::Column::Id)
            .all(&self.conn)
            .await
            .map_err(database_error)?;

        models.into_iter().map(FarmLocation::try_from).collect()
    }

    async fn create(&self, entity: &Farm) -> Result<Farm, AppError> {
        let transaction = self.conn.begin().await.map_err(database_error)?;

        let model = farms::ActiveModel::from(entity)
            .insert(&transaction)
            .await
            .map_err(database_error)?;

        Self::insert_cells(&transaction, model.id, entity.cells()).await?;

        let farm = Self::load(&transaction, model).await?;

        transaction.commit().await.map_err(database_error)?;

        Ok(farm)
    }

    async fn update(&self, entity: &Farm) -> Result<Farm, AppError> {
        let id = farm_id(entity)?;

        let transaction = self.conn.begin().await.map_err(database_error)?;

        // Only the time is written to the farm's row. The name and the
        // outline in `entity` are as old as its lookup, and writing them
        // back would undo an edit or a rename that landed since. The write
        // also takes the row's lock, so a repaint waits for an edit of the
        // same farm that is under way and never paints half of it.
        let touched = farms::Entity::update_many()
            .col_expr(
                farms::Column::UpdatedAt,
                Expr::value(entity.updated_at().naive_utc()),
            )
            .filter(farms::Column::Id.eq(id))
            .filter(farms::Column::Phone.eq(entity.owner().as_str()))
            .exec_with_returning(&transaction)
            .await
            .map_err(database_error)?;

        // No row: the farm was deleted between this request loading it and
        // writing it. It is gone, which is not a server fault.
        let Some(model) = touched.into_iter().next() else {
            return Err(GlobalAppError::NotFound.into());
        };

        // A repaint never adds or removes a cell, so it only has to move the
        // repainted cells to the crop they now carry. Cells this request did
        // not change are not written, so a repaint running at the same
        // moment on other cells is not undone. If an edit replaced the cells
        // since the lookup, these ids are gone and nothing is painted: the
        // result is the one of the repaint having come first.
        let mut ids_per_crop: HashMap<Crop, Vec<i32>> = HashMap::new();

        for cell in entity.cells().iter().filter(|cell| cell.repainted()) {
            let id = cell_id(cell)?;

            ids_per_crop.entry(cell.crop()).or_default().push(id);
        }

        for (crop, ids) in ids_per_crop {
            let crop = String::from(crop);

            for ids in ids.chunks(IDS_PER_UPDATE) {
                farm_cells::Entity::update_many()
                    .col_expr(farm_cells::Column::Crop, Expr::value(crop.clone()))
                    .filter(farm_cells::Column::FarmId.eq(model.id))
                    .filter(farm_cells::Column::Id.is_in(ids.to_vec()))
                    .filter(farm_cells::Column::Crop.ne(crop.clone()))
                    .exec(&transaction)
                    .await
                    .map_err(database_error)?;
            }
        }

        let farm = Self::load(&transaction, model).await?;

        transaction.commit().await.map_err(database_error)?;

        Ok(farm)
    }

    async fn replace(&self, entity: &Farm) -> Result<Farm, AppError> {
        let id = farm_id(entity)?;

        let transaction = self.conn.begin().await.map_err(database_error)?;

        // The lock is what makes the edit safe: a second edit or a repaint
        // of this farm waits here until this one has committed, so two sets
        // of cells are never mixed. Finding no row decides the 404, also
        // when the farm was deleted after the use case looked it up.
        let locked = farms::Entity::find_by_id(id)
            .filter(farms::Column::Phone.eq(entity.owner().as_str()))
            .lock_exclusive()
            .one(&transaction)
            .await
            .map_err(database_error)?;

        if locked.is_none() {
            return Err(GlobalAppError::NotFound.into());
        }

        // Only what an edit changes is written; the owner, the idempotency
        // key of the upload and the creation times stay as they are. The
        // place and the area follow the outline, so they go with it.
        let place = entity.place().as_ref();

        let updated = farms::Entity::update_many()
            .col_expr(farms::Column::Name, Expr::value(entity.name().as_str()))
            .col_expr(
                farms::Column::Outline,
                Expr::value(stored_outline(entity.outline())),
            )
            .col_expr(
                farms::Column::Governorate,
                Expr::value(place.map(|place| place.governorate().clone())),
            )
            .col_expr(
                farms::Column::ZoneSlug,
                Expr::value(place.map(|place| place.zone_slug().clone())),
            )
            .col_expr(
                farms::Column::SubZoneSlug,
                Expr::value(place.map(|place| place.sub_zone_slug().clone())),
            )
            .col_expr(farms::Column::AreaM2, Expr::value(entity.area_m2()))
            .col_expr(
                farms::Column::UpdatedAt,
                Expr::value(entity.updated_at().naive_utc()),
            )
            .filter(farms::Column::Id.eq(id))
            .exec_with_returning(&transaction)
            .await
            .map_err(database_error)?;

        let Some(model) = updated.into_iter().next() else {
            return Err(GlobalAppError::NotFound.into());
        };

        // The whole set is deleted and written again rather than compared
        // with the old one: a moved border changes the share of nearly
        // every edge cell and the app sends every crop anyway, so a diff
        // would rewrite most rows in three kinds of statement to save
        // little, and after a plain delete no cell of the old outline can
        // be left behind.
        farm_cells::Entity::delete_many()
            .filter(farm_cells::Column::FarmId.eq(id))
            .exec(&transaction)
            .await
            .map_err(database_error)?;

        Self::insert_cells(&transaction, id, entity.cells()).await?;

        let farm = Self::load(&transaction, model).await?;

        transaction.commit().await.map_err(database_error)?;

        Ok(farm)
    }

    async fn delete(&self, id: i32, owner: &Phone) -> Result<(), AppError> {
        let result = farms::Entity::delete_many()
            .filter(farms::Column::Id.eq(id))
            .filter(farms::Column::Phone.eq(owner.as_str()))
            .exec(&self.conn)
            .await
            .map_err(database_error)?;

        if result.rows_affected == 0 {
            return Err(GlobalAppError::NotFound.into());
        }

        Ok(())
    }

    async fn find_page(
        &self,
        filter: &FarmFilter,
        order: FarmOrder,
        pagination: &Pagination,
    ) -> Result<(Vec<OwnedFarmSummary>, u64), AppError> {
        let query = farms::Entity::find().filter(kept_by(filter, true));

        let count = query
            .clone()
            .count(&self.conn)
            .await
            .map_err(database_error)?;

        let direction = match order.direction {
            SortDirection::Ascending => Order::Asc,
            SortDirection::Descending => Order::Desc,
        };

        let query = match order.key {
            FarmSortKey::CreatedAt => query.order_by(farms::Column::CreatedAt, direction.clone()),
            // The area is a stored column so that the database can sort by
            // it. A farm that has none yet goes last either way, rather
            // than leading the largest-first list as a null would.
            FarmSortKey::AreaDunam => query.order_by_with_nulls(
                farms::Column::AreaM2,
                direction.clone(),
                NullOrdering::Last,
            ),
            FarmSortKey::Name => query.order_by(farms::Column::Name, direction.clone()),
        };

        let models = query
            .order_by(farms::Column::Id, direction)
            .offset(pagination.skip())
            .limit(*pagination.rows_per_page())
            .all(&self.conn)
            .await
            .map_err(database_error)?;

        let mut inside_per_crop = self
            .inside_per_crop(models.iter().map(|model| model.id).collect())
            .await?;

        let farms = models
            .into_iter()
            .map(|model| {
                let owner = Phone::new(model.phone.clone())?;
                let cells = inside_per_crop.remove(&model.id).unwrap_or_default();

                Ok(OwnedFarmSummary::new(
                    owner,
                    FarmSummary::try_from((model, cells))?,
                ))
            })
            .collect::<Result<Vec<_>, AppError>>()?;

        Ok((farms, count))
    }

    async fn sum_by_area(
        &self,
        filter: &FarmFilter,
        deepest: AreaLevel,
    ) -> Result<(Vec<AreaCount>, Vec<AreaCropSum>), AppError> {
        // Both sums are read from one snapshot: a farm saved between them
        // would otherwise be in the crops and not in the totals.
        let transaction = self
            .conn
            .begin_with_config(
                Some(IsolationLevel::RepeatableRead),
                Some(AccessMode::ReadOnly),
            )
            .await
            .map_err(database_error)?;

        let count_rows = farms::Entity::find()
            .select_only()
            .column(farms::Column::Governorate)
            .column(farms::Column::ZoneSlug)
            .column_as(Expr::cust(sub_zone_column(deepest)), "sub_zone_slug")
            .column_as(Expr::cust(rolled_up(deepest)), "rolled_up")
            .column_as(farms::Column::Id.count(), "farms")
            .column_as(
                Expr::expr(Func::count_distinct(Expr::col((
                    farms::Entity,
                    farms::Column::Phone,
                )))),
                "farmers",
            )
            .column_as(farms::Column::AreaM2.sum(), "area_m2")
            .column_as(farms::Column::UpdatedAt.max(), "latest_change")
            .filter(kept_by(filter, true))
            .group_by(Expr::cust(grouping_sets(deepest, None)))
            .into_model::<AreaCountRow>()
            .all(&transaction)
            .await
            .map_err(database_error)?;

        // The cells are joined to their farms and added up per crop. A crop
        // filter narrows the cells themselves, which also narrows the farms
        // to those growing it. `empty` is unpainted land, not a crop.
        let mut crops = farm_cells::Entity::find()
            .select_only()
            .join(JoinType::InnerJoin, farm_cells::Relation::Farms.def())
            .column(farms::Column::Governorate)
            .column(farms::Column::ZoneSlug)
            .column_as(Expr::cust(sub_zone_column(deepest)), "sub_zone_slug")
            .column_as(Expr::cust(rolled_up(deepest)), "rolled_up")
            .column(farm_cells::Column::Crop)
            .column_as(farm_cells::Column::InsidePct.sum(), "inside_pct")
            .column_as(
                Expr::expr(Func::count_distinct(Expr::col((
                    farms::Entity,
                    farms::Column::Id,
                )))),
                "farms",
            )
            .column_as(
                Expr::expr(Func::count_distinct(Expr::col((
                    farms::Entity,
                    farms::Column::Phone,
                )))),
                "farmers",
            )
            .filter(farm_cells::Column::Crop.ne(String::from(Crop::Empty)))
            .filter(kept_by(filter, false));

        if let Some(crop) = filter.crop {
            crops = crops.filter(farm_cells::Column::Crop.eq(String::from(crop.crop())));
        }

        let crop_rows = crops
            .group_by(Expr::cust(grouping_sets(deepest, Some(CELL_CROP))))
            .into_model::<AreaCropRow>()
            .all(&transaction)
            .await
            .map_err(database_error)?;

        transaction.commit().await.map_err(database_error)?;

        let counts = count_rows
            .into_iter()
            .map(|row| {
                Ok(AreaCount {
                    level: level_of(row.rolled_up)?,
                    key: AreaKey {
                        governorate: row.governorate,
                        zone_slug: row.zone_slug,
                        sub_zone_slug: row.sub_zone_slug,
                    },
                    farms: u64::try_from(row.farms).unwrap_or_default(),
                    farmers: u64::try_from(row.farmers).unwrap_or_default(),
                    area_m2: row.area_m2.unwrap_or_default(),
                    latest_change: row.latest_change.map(|at| at.and_utc()),
                })
            })
            .collect::<Result<Vec<_>, AppError>>()?;

        let crop_sums = crop_rows
            .into_iter()
            .map(|row| {
                Ok(AreaCropSum {
                    level: level_of(row.rolled_up)?,
                    key: AreaKey {
                        governorate: row.governorate,
                        zone_slug: row.zone_slug,
                        sub_zone_slug: row.sub_zone_slug,
                    },
                    crop: Crop::try_from(row.crop.as_str())?,
                    inside_pct: row.inside_pct.unwrap_or_default(),
                    farms: u64::try_from(row.farms).unwrap_or_default(),
                    farmers: u64::try_from(row.farmers).unwrap_or_default(),
                })
            })
            .collect::<Result<Vec<_>, AppError>>()?;

        Ok((counts, crop_sums))
    }

    async fn find_unplaced(
        &self,
        after_id: i32,
        limit: u64,
    ) -> Result<Vec<UnplacedFarm>, AppError> {
        farms::Entity::find()
            .filter(farms::Column::Id.gt(after_id))
            .filter(
                Condition::any()
                    .add(farms::Column::ZoneSlug.is_null())
                    .add(farms::Column::AreaM2.is_null()),
            )
            .order_by_asc(farms::Column::Id)
            .limit(limit)
            .all(&self.conn)
            .await
            .map_err(database_error)?
            .into_iter()
            .map(UnplacedFarm::try_from)
            .collect()
    }

    async fn fill_place(
        &self,
        farm: &UnplacedFarm,
        place: Option<&FarmPlace>,
    ) -> Result<bool, AppError> {
        // The time of the last write is the condition: an edit, a repaint
        // or a rename since the farm was read moves it, and then nothing is
        // written here. An edit has stored the place of its own outline.
        let result = farms::Entity::update_many()
            .col_expr(
                farms::Column::Governorate,
                Expr::value(place.map(|place| place.governorate().clone())),
            )
            .col_expr(
                farms::Column::ZoneSlug,
                Expr::value(place.map(|place| place.zone_slug().clone())),
            )
            .col_expr(
                farms::Column::SubZoneSlug,
                Expr::value(place.map(|place| place.sub_zone_slug().clone())),
            )
            .col_expr(farms::Column::AreaM2, Expr::value(farm.outline().area_m2()))
            .filter(farms::Column::Id.eq(*farm.id()))
            .filter(farms::Column::UpdatedAt.eq(farm.updated_at().naive_utc()))
            .exec(&self.conn)
            .await
            .map_err(database_error)?;

        Ok(result.rows_affected > 0)
    }

    async fn find_by_id(&self, id: i32) -> Result<Option<Farm>, AppError> {
        let model = farms::Entity::find_by_id(id)
            .one(&self.conn)
            .await
            .map_err(database_error)?;

        match model {
            Some(model) => Ok(Some(Self::load(&self.conn, model).await?)),
            None => Ok(None),
        }
    }

    async fn rename(
        &self,
        id: i32,
        name: &FarmName,
        now: DateTime<Utc>,
    ) -> Result<Option<Farm>, AppError> {
        // One statement decides whether there is a farm to rename, and it
        // writes the name only: a repaint by the farmer at the same moment
        // is not undone.
        let renamed = farms::Entity::update_many()
            .col_expr(farms::Column::Name, Expr::value(name.as_str()))
            .col_expr(farms::Column::UpdatedAt, Expr::value(now.naive_utc()))
            .filter(farms::Column::Id.eq(id))
            .exec_with_returning(&self.conn)
            .await
            .map_err(database_error)?;

        match renamed.into_iter().next() {
            Some(model) => Ok(Some(Self::load(&self.conn, model).await?)),
            None => Ok(None),
        }
    }

    async fn delete_by_id(&self, id: i32) -> Result<bool, AppError> {
        let result = farms::Entity::delete_many()
            .filter(farms::Column::Id.eq(id))
            .exec(&self.conn)
            .await
            .map_err(database_error)?;

        Ok(result.rows_affected > 0)
    }

    async fn delete_all_by_owner(&self, owner: &Phone) -> Result<u64, AppError> {
        // The cells go with their farms: the foreign key cascades.
        let result = farms::Entity::delete_many()
            .filter(farms::Column::Phone.eq(owner.as_str()))
            .exec(&self.conn)
            .await
            .map_err(database_error)?;

        Ok(result.rows_affected)
    }
}

fn farm_id(farm: &Farm) -> Result<i32, AppError> {
    farm.id().ok_or_else(|| {
        GlobalAppError::MissingValue("Cannot update a farm that has not been persisted".to_string())
            .into()
    })
}

fn cell_id(cell: &Cell) -> Result<i32, AppError> {
    cell.id().ok_or_else(|| {
        GlobalAppError::MissingValue(
            "Cannot update a farm cell that has not been persisted".to_string(),
        )
        .into()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_search_for_a_percent_or_an_underscore_looks_for_that_character() {
        assert_eq!(like_literal("50%"), "50\\%");
        assert_eq!(like_literal("north_field"), "north\\_field");
        assert_eq!(like_literal("a\\b"), "a\\\\b");
        assert_eq!(like_literal("Upper field"), "Upper field");
        assert_eq!(like_literal("کێڵگە"), "کێڵگە");
    }

    #[test]
    fn the_public_sums_never_group_by_sub_zone() {
        let sets = grouping_sets(AreaLevel::Zone, None);

        assert_eq!(
            sets,
            format!("GROUPING SETS ((), ({GOVERNORATE}), ({GOVERNORATE}, {ZONE_SLUG}))")
        );
        assert!(!rolled_up(AreaLevel::Zone).contains("sub_zone_slug"));
        assert!(!sub_zone_column(AreaLevel::Zone).contains("sub_zone_slug"));
    }

    #[test]
    fn the_dashboard_sums_group_down_to_sub_zones_and_every_set_can_carry_the_crop() {
        let sets = grouping_sets(AreaLevel::SubZone, Some(CELL_CROP));

        assert_eq!(sets.matches(CELL_CROP).count(), 4);
        assert!(sets.contains(&format!(
            "({GOVERNORATE}, {ZONE_SLUG}, {SUB_ZONE_SLUG}, {CELL_CROP})"
        )));
        assert!(sets.starts_with(&format!("GROUPING SETS (({CELL_CROP}), ")));
    }

    #[test]
    fn each_grouping_set_is_read_back_as_its_level() {
        assert_eq!(level_of(7).expect("level"), AreaLevel::Region);
        assert_eq!(level_of(3).expect("level"), AreaLevel::Governorate);
        assert_eq!(level_of(1).expect("level"), AreaLevel::Zone);
        assert_eq!(level_of(0).expect("level"), AreaLevel::SubZone);
        assert!(level_of(5).is_err());
    }
}
