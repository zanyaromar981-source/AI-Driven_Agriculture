use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sea_orm::{
    ColumnTrait, DatabaseConnection, DbErr, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder,
    QuerySelect, TransactionTrait, sea_query::Expr,
};

use crate::{
    app::{AppError as GlobalAppError, Pagination},
    features::rules::{
        app::{AppError, RuleRepository},
        domain::{ChangeOutcome, ChangeReason, NewValue, Rule, RuleChange, RuleCode, RuleUser},
        infra::persistence::postgres::entities::{rule_changes, rules},
    },
};

fn database_error(error: DbErr) -> AppError {
    tracing::error!(%error, "rule repository query failed");

    GlobalAppError::DatabaseError(error.to_string()).into()
}

#[derive(Debug)]
pub struct RulePostgresRepository {
    conn: DatabaseConnection,
}

impl RulePostgresRepository {
    pub fn new(conn: DatabaseConnection) -> Self {
        Self { conn }
    }
}

#[async_trait]
impl RuleRepository for RulePostgresRepository {
    async fn find_all(&self, used_by: Option<RuleUser>) -> Result<Vec<Rule>, AppError> {
        let mut query = rules::Entity::find();

        if let Some(used_by) = used_by {
            query = query.filter(rules::Column::UsedBy.eq(String::from(used_by)));
        }

        let models = query
            .order_by_asc(rules::Column::UsedBy)
            .order_by_asc(rules::Column::Grp)
            .order_by_asc(rules::Column::Code)
            .all(&self.conn)
            .await
            .map_err(database_error)?;

        models.into_iter().map(Rule::try_from).collect()
    }

    async fn find_by_code(&self, code: &RuleCode) -> Result<Option<Rule>, AppError> {
        let model = rules::Entity::find_by_id(code.as_str().to_string())
            .one(&self.conn)
            .await
            .map_err(database_error)?;

        model.map(Rule::try_from).transpose()
    }

    async fn change_value(
        &self,
        code: &RuleCode,
        requested: NewValue,
        reason: &ChangeReason,
        staff_id: i32,
        now: DateTime<Utc>,
    ) -> Result<Option<ChangeOutcome>, AppError> {
        let transaction = self.conn.begin().await.map_err(database_error)?;

        // The lock is what makes the log a true chain: a second change of
        // this rule waits here until this one has committed, and then reads
        // the value this one stored as its old value. Finding no row decides
        // the 404.
        let locked = rules::Entity::find_by_id(code.as_str().to_string())
            .lock_exclusive()
            .one(&transaction)
            .await
            .map_err(database_error)?;

        let Some(model) = locked else {
            return Ok(None);
        };

        let current = Rule::try_from(model)?;

        // Leaving here drops the transaction, which rolls it back: a refused
        // value and a repeat of the stored one both write nothing.
        let Some(value) = current.decide(requested)? else {
            return Ok(Some(ChangeOutcome::new(current, None)));
        };

        let change = RuleChange::of(&current, value, reason.clone(), staff_id, now);

        // Only what a change changes is written; names, range and default
        // belong to the migration.
        let updated = rules::Entity::update_many()
            .col_expr(rules::Column::Value, Expr::value(value))
            .col_expr(rules::Column::UpdatedBy, Expr::value(staff_id))
            .col_expr(rules::Column::UpdatedAt, Expr::value(now.naive_utc()))
            .filter(rules::Column::Code.eq(code.as_str()))
            .exec_with_returning(&transaction)
            .await
            .map_err(database_error)?;

        let Some(model) = updated.into_iter().next() else {
            return Ok(None);
        };

        let logged = rule_changes::Entity::insert(rule_changes::ActiveModel::from(&change))
            .exec_with_returning(&transaction)
            .await
            .map_err(database_error)?;

        transaction.commit().await.map_err(database_error)?;

        Ok(Some(ChangeOutcome::new(
            Rule::try_from(model)?,
            Some(RuleChange::try_from(logged)?),
        )))
    }

    async fn find_changes_page(
        &self,
        code: &RuleCode,
        pagination: &Pagination,
    ) -> Result<(Vec<RuleChange>, u64), AppError> {
        let query =
            rule_changes::Entity::find().filter(rule_changes::Column::Code.eq(code.as_str()));

        let count = query
            .clone()
            .count(&self.conn)
            .await
            .map_err(database_error)?;

        // The id rises with every insert, so it orders two changes made in
        // the same instant, which the timestamp cannot.
        let models = query
            .order_by_desc(rule_changes::Column::Id)
            .offset(pagination.skip())
            .limit(*pagination.rows_per_page())
            .all(&self.conn)
            .await
            .map_err(database_error)?;

        Ok((
            models
                .into_iter()
                .map(RuleChange::try_from)
                .collect::<Result<Vec<_>, _>>()?,
            count,
        ))
    }
}
