use std::collections::HashMap;

use async_trait::async_trait;
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseConnection, DbErr, EntityTrait,
    FromQueryResult, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, SqlErr,
    TransactionTrait,
    sea_query::{Expr, OnConflict, Query},
};

use crate::{
    app::{AppError as GlobalAppError, Permission},
    features::staff::{
        app::{AppError, RoleRepository, StaffRepository},
        domain::{
            Grantor, OwnerStanding, Role, RoleRef, RoleSelection, Staff, StaffChange, StaffEmail,
            StaffError,
        },
        infra::persistence::postgres::{
            entities::{role_permissions, roles, staff, staff_roles},
            mappings::{permission_active_model, permission_from, staff_role_active_model},
        },
    },
};

/// Postgres accepts 65,535 bind parameters per statement. An id list binds
/// one per id.
const IDS_PER_QUERY: usize = 30_000;

fn database_error(error: DbErr) -> AppError {
    tracing::error!(%error, "staff repository query failed");

    GlobalAppError::DatabaseError(error.to_string()).into()
}

/// For a write whose unique index stands for a rule: the index refusing the
/// row is that rule broken, anything else is a fault.
fn unique_violation_means(rule: StaffError) -> impl FnOnce(DbErr) -> AppError {
    move |error| match error.sql_err() {
        Some(SqlErr::UniqueConstraintViolation(_)) => rule.into(),
        _ => database_error(error),
    }
}

/// The same for a write whose foreign key stands for a rule.
fn foreign_key_violation_means(rule: StaffError) -> impl FnOnce(DbErr) -> AppError {
    move |error| match error.sql_err() {
        Some(SqlErr::ForeignKeyConstraintViolation(_)) => rule.into(),
        _ => database_error(error),
    }
}

/// Takes the system role rows until the transaction ends and returns their
/// ids. Every write that could remove an owner, hand out a permission or
/// take one away takes them first, so such writes run one after another.
/// Each one therefore counts the owners the one before it left, and checks
/// a grant against what the one before it left the actor holding.
async fn lock_system_roles<C: ConnectionTrait>(conn: &C) -> Result<Vec<i32>, AppError> {
    let system_roles = roles::Entity::find()
        .filter(roles::Column::System.eq(true))
        .order_by_asc(roles::Column::Id)
        .lock_exclusive()
        .all(conn)
        .await
        .map_err(database_error)?;

    Ok(system_roles.into_iter().map(|role| role.id).collect())
}

async fn permissions_of_roles<C: ConnectionTrait>(
    conn: &C,
    role_ids: &[i32],
) -> Result<Vec<Permission>, AppError> {
    let mut permissions = Vec::new();

    for role_ids in role_ids.chunks(IDS_PER_QUERY) {
        let granted = role_permissions::Entity::find()
            .filter(role_permissions::Column::RoleId.is_in(role_ids.to_vec()))
            .all(conn)
            .await
            .map_err(database_error)?;

        for permission in &granted {
            permissions.push(permission_from(permission)?);
        }
    }

    Ok(permissions)
}

async fn roles_held_by<C: ConnectionTrait>(conn: &C, staff_id: i32) -> Result<Vec<i32>, AppError> {
    let held = staff_roles::Entity::find()
        .filter(staff_roles::Column::StaffId.eq(staff_id))
        .all(conn)
        .await
        .map_err(database_error)?;

    Ok(held.into_iter().map(|link| link.role_id).collect())
}

/// Reads what the acting staff member holds right now. Must run after
/// `lock_system_roles` in the same transaction: the token was checked a
/// moment ago, and a role or the account may have been changed since.
async fn grantor<C: ConnectionTrait>(conn: &C, actor_id: i32) -> Result<Grantor, AppError> {
    let actor = staff::Entity::find_by_id(actor_id)
        .one(conn)
        .await
        .map_err(database_error)?;

    let Some(actor) = actor else {
        return Ok(Grantor::new(false, Vec::new()));
    };

    let held = roles_held_by(conn, actor_id).await?;

    Ok(Grantor::new(
        actor.active,
        permissions_of_roles(conn, &held).await?,
    ))
}

#[derive(FromQueryResult)]
struct StaffPerRole {
    role_id: i32,
    staff: i64,
}

#[derive(Debug)]
pub struct RolePostgresRepository {
    conn: DatabaseConnection,
}

impl RolePostgresRepository {
    pub fn new(conn: DatabaseConnection) -> Self {
        Self { conn }
    }

    async fn load<C: ConnectionTrait>(
        conn: &C,
        models: Vec<roles::Model>,
    ) -> Result<Vec<Role>, AppError> {
        let role_ids: Vec<i32> = models.iter().map(|model| model.id).collect();

        let mut permissions: HashMap<i32, Vec<role_permissions::Model>> = HashMap::new();
        let mut staff_counts: HashMap<i32, u64> = HashMap::new();

        for role_ids in role_ids.chunks(IDS_PER_QUERY) {
            let granted = role_permissions::Entity::find()
                .filter(role_permissions::Column::RoleId.is_in(role_ids.to_vec()))
                .all(conn)
                .await
                .map_err(database_error)?;

            for permission in granted {
                permissions
                    .entry(permission.role_id)
                    .or_default()
                    .push(permission);
            }

            let counts = staff_roles::Entity::find()
                .select_only()
                .column(staff_roles::Column::RoleId)
                .column_as(staff_roles::Column::StaffId.count(), "staff")
                .filter(staff_roles::Column::RoleId.is_in(role_ids.to_vec()))
                .group_by(staff_roles::Column::RoleId)
                .into_model::<StaffPerRole>()
                .all(conn)
                .await
                .map_err(database_error)?;

            for count in counts {
                staff_counts.insert(
                    count.role_id,
                    u64::try_from(count.staff).unwrap_or_default(),
                );
            }
        }

        models
            .into_iter()
            .map(|model| {
                let permissions = permissions.remove(&model.id).unwrap_or_default();
                let staff_count = staff_counts.remove(&model.id).unwrap_or_default();

                Role::try_from((model, permissions, staff_count))
            })
            .collect()
    }

    async fn load_one<C: ConnectionTrait>(conn: &C, id: i32) -> Result<Option<Role>, AppError> {
        let model = roles::Entity::find_by_id(id)
            .one(conn)
            .await
            .map_err(database_error)?;

        match model {
            Some(model) => Ok(Self::load(conn, vec![model]).await?.pop()),
            None => Ok(None),
        }
    }

    async fn grant<C: ConnectionTrait>(
        conn: &C,
        role_id: i32,
        permissions: &[Permission],
    ) -> Result<(), AppError> {
        if permissions.is_empty() {
            return Ok(());
        }

        role_permissions::Entity::insert_many(
            permissions
                .iter()
                .map(|permission| permission_active_model(role_id, permission)),
        )
        .exec_without_returning(conn)
        .await
        .map_err(database_error)?;

        Ok(())
    }

    /// Says why a guarded write touched no row. The write has already
    /// decided; this only picks the answer.
    async fn why_unchanged<C: ConnectionTrait>(conn: &C, id: i32) -> AppError {
        match Self::load_one(conn, id).await {
            Ok(Some(role)) => match role.ensure_changeable() {
                Err(error) => error.into(),
                // A changeable role that the write did not find was removed
                // by another request in between.
                Ok(()) => GlobalAppError::NotFound.into(),
            },
            Ok(None) => GlobalAppError::NotFound.into(),
            Err(error) => error,
        }
    }
}

#[async_trait]
impl RoleRepository for RolePostgresRepository {
    async fn find_all(&self) -> Result<Vec<Role>, AppError> {
        let models = roles::Entity::find()
            .order_by_asc(roles::Column::Id)
            .all(&self.conn)
            .await
            .map_err(database_error)?;

        Self::load(&self.conn, models).await
    }

    async fn find_by_id(&self, id: i32) -> Result<Option<Role>, AppError> {
        Self::load_one(&self.conn, id).await
    }

    async fn create(&self, entity: &Role, actor_id: i32) -> Result<Role, AppError> {
        let transaction = self.conn.begin().await.map_err(database_error)?;

        lock_system_roles(&transaction).await?;

        grantor(&transaction, actor_id)
            .await?
            .ensure_can_set_role_permissions(&[], entity.permissions())?;

        // The name is unique, so of two requests creating the same role at
        // the same moment the second waits for the first and is refused.
        let model = roles::ActiveModel::from(entity)
            .insert(&transaction)
            .await
            .map_err(unique_violation_means(StaffError::RoleNameTaken))?;

        Self::grant(&transaction, model.id, entity.permissions()).await?;

        let role = Self::load(&transaction, vec![model])
            .await?
            .pop()
            .ok_or(GlobalAppError::NotFound)?;

        transaction.commit().await.map_err(database_error)?;

        Ok(role)
    }

    async fn update(&self, entity: &Role, actor_id: i32) -> Result<Role, AppError> {
        let Some(id) = *entity.id() else {
            return Err(GlobalAppError::MissingValue(
                "Cannot update a role that has not been persisted".to_string(),
            )
            .into());
        };

        let transaction = self.conn.begin().await.map_err(database_error)?;

        lock_system_roles(&transaction).await?;

        // What the role holds is read here, under the lock, and not taken
        // from the caller: only what this write adds to it is a grant.
        let held_before = permissions_of_roles(&transaction, &[id]).await?;

        grantor(&transaction, actor_id)
            .await?
            .ensure_can_set_role_permissions(&held_before, entity.permissions())?;

        // The statement itself refuses a system role. It also takes the
        // row, so two edits of one role run one after the other and the
        // permission set below is never a mix of both.
        let result = roles::Entity::update_many()
            .col_expr(
                roles::Column::Name,
                Expr::value(String::from(entity.name())),
            )
            .col_expr(
                roles::Column::Description,
                Expr::value(entity.description().as_ref().map(String::from)),
            )
            .col_expr(
                roles::Column::UpdatedAt,
                Expr::value(entity.updated_at().naive_utc()),
            )
            .filter(roles::Column::Id.eq(id))
            .filter(roles::Column::System.eq(false))
            .exec(&transaction)
            .await
            .map_err(unique_violation_means(StaffError::RoleNameTaken))?;

        if result.rows_affected == 0 {
            return Err(Self::why_unchanged(&transaction, id).await);
        }

        role_permissions::Entity::delete_many()
            .filter(role_permissions::Column::RoleId.eq(id))
            .exec(&transaction)
            .await
            .map_err(database_error)?;

        Self::grant(&transaction, id, entity.permissions()).await?;

        let role = Self::load_one(&transaction, id)
            .await?
            .ok_or(GlobalAppError::NotFound)?;

        transaction.commit().await.map_err(database_error)?;

        Ok(role)
    }

    async fn delete(&self, id: i32) -> Result<(), AppError> {
        // One statement decides everything: the filter refuses a system
        // role, and the foreign key from `staff_roles` refuses a role that
        // any staff member holds, also one assigned at this very moment.
        let result = roles::Entity::delete_many()
            .filter(roles::Column::Id.eq(id))
            .filter(roles::Column::System.eq(false))
            .exec(&self.conn)
            .await
            .map_err(foreign_key_violation_means(StaffError::RoleInUse))?;

        if result.rows_affected == 0 {
            return Err(Self::why_unchanged(&self.conn, id).await);
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct StaffPostgresRepository {
    conn: DatabaseConnection,
}

impl StaffPostgresRepository {
    pub fn new(conn: DatabaseConnection) -> Self {
        Self { conn }
    }

    async fn load<C: ConnectionTrait>(
        conn: &C,
        models: Vec<staff::Model>,
    ) -> Result<Vec<Staff>, AppError> {
        let staff_ids: Vec<i32> = models.iter().map(|model| model.id).collect();

        let mut roles_held: HashMap<i32, Vec<RoleRef>> = HashMap::new();

        for staff_ids in staff_ids.chunks(IDS_PER_QUERY) {
            let held = staff_roles::Entity::find()
                .filter(staff_roles::Column::StaffId.is_in(staff_ids.to_vec()))
                .find_also_related(roles::Entity)
                .order_by_asc(staff_roles::Column::RoleId)
                .all(conn)
                .await
                .map_err(database_error)?;

            for (link, role) in held {
                if let Some(role) = role {
                    roles_held
                        .entry(link.staff_id)
                        .or_default()
                        .push(RoleRef::from(&role));
                }
            }
        }

        models
            .into_iter()
            .map(|model| {
                let roles = roles_held.remove(&model.id).unwrap_or_default();

                Staff::try_from((model, roles))
            })
            .collect()
    }

    async fn load_one<C: ConnectionTrait>(
        conn: &C,
        model: staff::Model,
    ) -> Result<Staff, AppError> {
        Self::load(conn, vec![model])
            .await?
            .pop()
            .ok_or_else(|| GlobalAppError::NotFound.into())
    }

    /// Gives the staff member each role in `roles` they do not hold yet.
    /// A role that does not exist is refused by the foreign key.
    async fn assign<C: ConnectionTrait>(
        conn: &C,
        staff_id: i32,
        roles: &RoleSelection,
    ) -> Result<(), AppError> {
        if roles.ids().is_empty() {
            return Ok(());
        }

        staff_roles::Entity::insert_many(
            roles
                .ids()
                .iter()
                .map(|role_id| staff_role_active_model(staff_id, *role_id)),
        )
        .on_conflict(
            OnConflict::columns([staff_roles::Column::StaffId, staff_roles::Column::RoleId])
                .do_nothing()
                .to_owned(),
        )
        .exec_without_returning(conn)
        .await
        .map_err(foreign_key_violation_means(StaffError::UnknownRole))?;

        Ok(())
    }

    /// Reads the staff row and holds it until the transaction ends.
    async fn lock_staff<C: ConnectionTrait>(conn: &C, id: i32) -> Result<staff::Model, AppError> {
        staff::Entity::find_by_id(id)
            .lock_exclusive()
            .one(conn)
            .await
            .map_err(database_error)?
            .ok_or_else(|| GlobalAppError::NotFound.into())
    }

    /// Must run after `lock_system_roles` in the same transaction.
    async fn owner_standing<C: ConnectionTrait>(
        conn: &C,
        target: &staff::Model,
        system_role_ids: &[i32],
    ) -> Result<OwnerStanding, AppError> {
        let holds_a_system_role = staff_roles::Entity::find()
            .filter(staff_roles::Column::StaffId.eq(target.id))
            .filter(staff_roles::Column::RoleId.is_in(system_role_ids.to_vec()))
            .count(conn)
            .await
            .map_err(database_error)?
            > 0;

        let other_active_owners = staff::Entity::find()
            .filter(staff::Column::Active.eq(true))
            .filter(staff::Column::Id.ne(target.id))
            .filter(
                staff::Column::Id.in_subquery(
                    Query::select()
                        .column(staff_roles::Column::StaffId)
                        .from(staff_roles::Entity)
                        .and_where(staff_roles::Column::RoleId.is_in(system_role_ids.to_vec()))
                        .to_owned(),
                ),
            )
            .count(conn)
            .await
            .map_err(database_error)?;

        Ok(OwnerStanding {
            target_is_active_owner: target.active && holds_a_system_role,
            other_active_owners,
        })
    }
}

#[async_trait]
impl StaffRepository for StaffPostgresRepository {
    async fn find_all(&self) -> Result<Vec<Staff>, AppError> {
        let models = staff::Entity::find()
            .order_by_asc(staff::Column::Id)
            .all(&self.conn)
            .await
            .map_err(database_error)?;

        Self::load(&self.conn, models).await
    }

    async fn find_by_id(&self, id: i32) -> Result<Option<Staff>, AppError> {
        let model = staff::Entity::find_by_id(id)
            .one(&self.conn)
            .await
            .map_err(database_error)?;

        match model {
            Some(model) => Ok(Some(Self::load_one(&self.conn, model).await?)),
            None => Ok(None),
        }
    }

    async fn find_by_email(&self, email: &StaffEmail) -> Result<Option<Staff>, AppError> {
        let model = staff::Entity::find()
            .filter(staff::Column::Email.eq(email.as_str()))
            .one(&self.conn)
            .await
            .map_err(database_error)?;

        match model {
            Some(model) => Ok(Some(Self::load_one(&self.conn, model).await?)),
            None => Ok(None),
        }
    }

    async fn permissions_granted_to(&self, staff_id: i32) -> Result<Vec<Permission>, AppError> {
        let granted = role_permissions::Entity::find()
            .filter(
                role_permissions::Column::RoleId.in_subquery(
                    Query::select()
                        .column(staff_roles::Column::RoleId)
                        .from(staff_roles::Entity)
                        .and_where(staff_roles::Column::StaffId.eq(staff_id))
                        .to_owned(),
                ),
            )
            .all(&self.conn)
            .await
            .map_err(database_error)?;

        granted.iter().map(permission_from).collect()
    }

    async fn create(
        &self,
        entity: &Staff,
        roles: &RoleSelection,
        actor_id: i32,
    ) -> Result<Staff, AppError> {
        let transaction = self.conn.begin().await.map_err(database_error)?;

        lock_system_roles(&transaction).await?;

        // A role that does not exist grants nothing here and is refused by
        // the foreign key below.
        grantor(&transaction, actor_id)
            .await?
            .ensure_can_assign(&permissions_of_roles(&transaction, roles.ids()).await?)?;

        // The email is unique, so of two requests adding the same person at
        // the same moment the second waits for the first and is refused.
        let model = staff::ActiveModel::from(entity)
            .insert(&transaction)
            .await
            .map_err(unique_violation_means(StaffError::EmailTaken))?;

        Self::assign(&transaction, model.id, roles).await?;

        let created = Self::load_one(&transaction, model).await?;

        transaction.commit().await.map_err(database_error)?;

        Ok(created)
    }

    async fn update(
        &self,
        id: i32,
        change: &StaffChange,
        actor_id: i32,
    ) -> Result<Staff, AppError> {
        let transaction = self.conn.begin().await.map_err(database_error)?;

        let system_role_ids = lock_system_roles(&transaction).await?;
        let stored = Self::lock_staff(&transaction, id).await?;

        let grantor = grantor(&transaction, actor_id).await?;
        let held = roles_held_by(&transaction, id).await?;

        // Roles the staff member already holds stay without a check: only
        // the ones this edit hands out are a grant.
        grantor.ensure_can_assign(
            &permissions_of_roles(&transaction, &change.roles().newly_assigned(&held)).await?,
        )?;

        // The account is measured by the roles it holds, also while it is
        // switched off: with its password it could be switched on again.
        if change.password_hash().is_some() {
            grantor.ensure_can_set_password(
                actor_id,
                id,
                &permissions_of_roles(&transaction, &held).await?,
            )?;
        }

        Self::owner_standing(&transaction, &stored, &system_role_ids)
            .await?
            .ensure_an_owner_remains(change.keeps_an_active_owner(&system_role_ids))?;

        // Only what an edit sets is written. The password is left alone
        // unless a new one was given.
        let mut update = staff::Entity::update_many()
            .col_expr(
                staff::Column::Name,
                Expr::value(String::from(change.name())),
            )
            .col_expr(staff::Column::Active, Expr::value(*change.active()))
            .col_expr(
                staff::Column::UpdatedAt,
                Expr::value(Utc::now().naive_utc()),
            )
            .filter(staff::Column::Id.eq(id));

        if let Some(password_hash) = change.password_hash() {
            update = update.col_expr(
                staff::Column::PasswordHash,
                Expr::value(password_hash.as_str().to_string()),
            );
        }

        update.exec(&transaction).await.map_err(database_error)?;

        // Roles the staff member keeps are not touched: those no longer
        // chosen are removed and the new ones added.
        staff_roles::Entity::delete_many()
            .filter(staff_roles::Column::StaffId.eq(id))
            .filter(staff_roles::Column::RoleId.is_not_in(change.roles().ids().to_vec()))
            .exec(&transaction)
            .await
            .map_err(database_error)?;

        Self::assign(&transaction, id, change.roles()).await?;

        let model = staff::Entity::find_by_id(id)
            .one(&transaction)
            .await
            .map_err(database_error)?
            .ok_or(GlobalAppError::NotFound)?;

        let updated = Self::load_one(&transaction, model).await?;

        transaction.commit().await.map_err(database_error)?;

        Ok(updated)
    }

    async fn delete(&self, id: i32) -> Result<(), AppError> {
        let transaction = self.conn.begin().await.map_err(database_error)?;

        let system_role_ids = lock_system_roles(&transaction).await?;
        let stored = Self::lock_staff(&transaction, id).await?;

        Self::owner_standing(&transaction, &stored, &system_role_ids)
            .await?
            .ensure_an_owner_remains(false)?;

        staff::Entity::delete_by_id(id)
            .exec(&transaction)
            .await
            .map_err(database_error)?;

        transaction.commit().await.map_err(database_error)?;

        Ok(())
    }

    async fn create_owner_if_absent(&self, entity: &Staff) -> Result<bool, AppError> {
        let transaction = self.conn.begin().await.map_err(database_error)?;

        // The email is unique, so running the command twice, also at the
        // same moment, creates one account and leaves it as first created.
        let inserted = staff::Entity::insert(staff::ActiveModel::from(entity))
            .on_conflict(
                OnConflict::column(staff::Column::Email)
                    .do_nothing()
                    .to_owned(),
            )
            .exec_without_returning(&transaction)
            .await
            .map_err(database_error)?;

        if inserted == 0 {
            return Ok(false);
        }

        let created = staff::Entity::find()
            .filter(staff::Column::Email.eq(entity.email().as_str()))
            .one(&transaction)
            .await
            .map_err(database_error)?
            .ok_or_else(|| {
                GlobalAppError::MissingValue("The owner just created is gone".to_string())
            })?;

        let owner_role = roles::Entity::find()
            .filter(roles::Column::System.eq(true))
            .order_by_asc(roles::Column::Id)
            .one(&transaction)
            .await
            .map_err(database_error)?
            .ok_or_else(|| {
                GlobalAppError::MissingValue(
                    "The owner role is missing: run the migrations first".to_string(),
                )
            })?;

        Self::assign(
            &transaction,
            created.id,
            &RoleSelection::new(vec![owner_role.id])?,
        )
        .await?;

        transaction.commit().await.map_err(database_error)?;

        Ok(true)
    }
}
