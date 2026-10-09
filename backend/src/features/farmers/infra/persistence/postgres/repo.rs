use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, DatabaseConnection, DbErr, EntityTrait, ExprTrait,
    Order, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, TransactionTrait, TryInsertResult,
    sea_query::{Expr, NullOrdering, OnConflict, extension::postgres::PgExpr},
};

use crate::{
    app::{AppError as GlobalAppError, Pagination},
    features::farmers::{
        app::{
            AppError, FarmerFilter, FarmerRepository, FarmerSort, LetterRecord, LetterRepository,
            SignInChallengeRepository,
        },
        domain::{
            Farmer, FarmerChange, FarmerName, Letter, LetterLanguage, LetterNumber, LetterPurpose,
            SignInChallenge,
        },
        infra::persistence::postgres::entities::{farmers, letters, sign_in_challenges},
    },
    shared::Phone,
};

/// Turns what staff typed into a pattern that matches it anywhere in a
/// text. `%` and `_` mean "anything" to `LIKE`, so they are escaped: a
/// search for `50%` finds that text, not every farmer.
fn contains_pattern(text: &str) -> String {
    let mut pattern = String::with_capacity(text.len() + 2);

    pattern.push('%');

    for character in text.chars() {
        if matches!(character, '%' | '_' | '\\') {
            pattern.push('\\');
        }

        pattern.push(character);
    }

    pattern.push('%');
    pattern
}

fn database_error(error: DbErr) -> AppError {
    tracing::error!(%error, "farmer repository query failed");

    GlobalAppError::DatabaseError(error.to_string()).into()
}

#[derive(Debug)]
pub struct FarmerPostgresRepository {
    conn: DatabaseConnection,
}

impl FarmerPostgresRepository {
    pub fn new(conn: DatabaseConnection) -> Self {
        Self { conn }
    }
}

#[async_trait]
impl FarmerRepository for FarmerPostgresRepository {
    async fn find_by_phone(&self, phone: &Phone) -> Result<Option<Farmer>, AppError> {
        let model = farmers::Entity::find()
            .filter(farmers::Column::Phone.eq(phone.as_str()))
            .one(&self.conn)
            .await
            .map_err(database_error)?;

        model.map(Farmer::try_from).transpose()
    }

    async fn create_if_absent(&self, entity: &Farmer) -> Result<(), AppError> {
        // The phone is unique, so two sign-ins racing for a new phone cannot
        // both insert: the second does nothing.
        farmers::Entity::insert(farmers::ActiveModel::from(entity))
            .on_conflict_do_nothing_on([farmers::Column::Phone])
            .exec(&self.conn)
            .await
            .map_err(database_error)?;

        Ok(())
    }

    async fn update(&self, entity: &Farmer) -> Result<Farmer, AppError> {
        if entity.id().is_none() {
            return Err(GlobalAppError::MissingValue(
                "Cannot update a farmer that has not been persisted".to_string(),
            )
            .into());
        }

        let model = farmers::ActiveModel::from(entity)
            .update(&self.conn)
            .await
            .map_err(database_error)?;

        Farmer::try_from(model)
    }

    async fn find_by_id(&self, id: i32) -> Result<Option<Farmer>, AppError> {
        let model = farmers::Entity::find_by_id(id)
            .one(&self.conn)
            .await
            .map_err(database_error)?;

        model.map(Farmer::try_from).transpose()
    }

    async fn find_page(
        &self,
        filter: &FarmerFilter,
        pagination: &Pagination,
    ) -> Result<(Vec<Farmer>, u64), AppError> {
        let mut query = farmers::Entity::find();

        if let Some(phone) = &filter.phone {
            query = query.filter(farmers::Column::Phone.eq(phone.as_str()));
        }

        if let Some(search) = &filter.search {
            let pattern = contains_pattern(search.as_str());

            query = query.filter(
                Condition::any()
                    .add(Expr::cust_with_values(
                        r"name ILIKE $1 ESCAPE '\'",
                        [pattern.clone()],
                    ))
                    .add(Expr::cust_with_values(
                        r"phone ILIKE $1 ESCAPE '\'",
                        [pattern],
                    )),
            );
        }

        if let Some(governorate) = &filter.governorate {
            query = query.filter(farmers::Column::Governorate.eq(governorate.as_str()));
        }

        if let Some(zone) = &filter.zone {
            query = query.filter(farmers::Column::ZoneSlug.eq(zone.as_str()));
        }

        if let Some(blocked) = filter.blocked {
            query = query.filter(farmers::Column::Blocked.eq(blocked));
        }

        let direction = if filter.is_descending() {
            Order::Desc
        } else {
            Order::Asc
        };

        let count = query
            .clone()
            .count(&self.conn)
            .await
            .map_err(database_error)?;

        // The id breaks ties, so a farmer is on exactly one page.
        let ordered = match filter.sort {
            FarmerSort::CreatedAt => query.order_by(farmers::Column::CreatedAt, direction.clone()),
            // A farmer without a name comes after every named one, whichever
            // way the names run.
            FarmerSort::Name => query.order_by_with_nulls(
                farmers::Column::Name,
                direction.clone(),
                NullOrdering::Last,
            ),
        };

        let models = ordered
            .order_by(farmers::Column::Id, direction)
            .offset(pagination.skip())
            .limit(*pagination.rows_per_page())
            .all(&self.conn)
            .await
            .map_err(database_error)?;

        let farmers = models
            .into_iter()
            .map(Farmer::try_from)
            .collect::<Result<Vec<_>, _>>()?;

        Ok((farmers, count))
    }

    async fn create(&self, entity: &Farmer) -> Result<Option<Farmer>, AppError> {
        // The unique index on the phone decides: a phone that already has a
        // farmer inserts nothing, so no row comes back. (The single-row
        // `exec_with_returning` reports that as an error, not as a conflict.)
        let inserted = farmers::Entity::insert(farmers::ActiveModel::from(entity))
            .on_conflict_do_nothing_on([farmers::Column::Phone])
            .exec_with_returning_many(&self.conn)
            .await
            .map_err(database_error)?;

        match inserted {
            TryInsertResult::Inserted(models) => {
                models.into_iter().next().map(Farmer::try_from).transpose()
            }
            TryInsertResult::Conflicted | TryInsertResult::Empty => Ok(None),
        }
    }

    async fn update_by_id(
        &self,
        id: i32,
        change: &FarmerChange,
        now: DateTime<Utc>,
    ) -> Result<Option<Farmer>, AppError> {
        let details = &change.details;

        let mut update = farmers::Entity::update_many()
            .col_expr(
                farmers::Column::Name,
                Expr::value(change.name.as_ref().map(String::from)),
            )
            .col_expr(
                farmers::Column::Language,
                Expr::value(String::from(change.language)),
            )
            .col_expr(
                farmers::Column::Gender,
                Expr::value(details.gender.map(String::from)),
            )
            .col_expr(
                farmers::Column::BirthYear,
                Expr::value(details.birth_year.map(|year| year.value())),
            )
            .col_expr(
                farmers::Column::Village,
                Expr::value(details.village.as_ref().map(String::from)),
            )
            .col_expr(
                farmers::Column::Governorate,
                Expr::value(details.governorate.as_ref().map(String::from)),
            )
            .col_expr(
                farmers::Column::ZoneSlug,
                Expr::value(details.zone_slug.as_ref().map(String::from)),
            )
            .col_expr(
                farmers::Column::SubZoneSlug,
                Expr::value(details.sub_zone_slug.as_ref().map(String::from)),
            )
            .col_expr(
                farmers::Column::Notes,
                Expr::value(details.notes.as_ref().map(String::from)),
            )
            .col_expr(farmers::Column::UpdatedAt, Expr::value(now.naive_utc()));

        // Left out of the statement when the edit does not say: the farmer
        // then stays blocked or not, also against a block set this moment.
        if let Some(blocked) = change.blocked {
            update = update.col_expr(farmers::Column::Blocked, Expr::value(blocked));
        }

        let updated = update
            .filter(farmers::Column::Id.eq(id))
            .exec_with_returning(&self.conn)
            .await
            .map_err(database_error)?;

        updated.into_iter().next().map(Farmer::try_from).transpose()
    }

    async fn delete_with_challenge(&self, id: i32) -> Result<bool, AppError> {
        let transaction = self.conn.begin().await.map_err(database_error)?;

        let deleted = farmers::Entity::delete_many()
            .filter(farmers::Column::Id.eq(id))
            .exec_with_returning(&transaction)
            .await
            .map_err(database_error)?;

        // A code still open for the phone would otherwise sign the removed
        // farmer straight back in.
        for farmer in &deleted {
            sign_in_challenges::Entity::delete_many()
                .filter(sign_in_challenges::Column::Phone.eq(farmer.phone.as_str()))
                .exec(&transaction)
                .await
                .map_err(database_error)?;
        }

        transaction.commit().await.map_err(database_error)?;

        Ok(!deleted.is_empty())
    }

    async fn find_many(
        &self,
        ids: Option<&[i32]>,
        matching: Option<&str>,
        limit: u64,
    ) -> Result<Vec<Farmer>, AppError> {
        let mut query = farmers::Entity::find();

        if let Some(ids) = ids {
            query = query.filter(farmers::Column::Id.is_in(ids.to_vec()));
        }

        if let Some(matching) = matching {
            // The text is looked for as it is: `%` and `_` typed by a person
            // are characters, not wildcards. The backslash is the escape
            // character Postgres uses for `LIKE` when none is named.
            let pattern = format!("%{}%", escape_like(matching));

            query = query.filter(
                Condition::any()
                    .add(Expr::col((farmers::Entity, farmers::Column::Name)).ilike(pattern.clone()))
                    .add(Expr::col((farmers::Entity, farmers::Column::Phone)).ilike(pattern)),
            );
        }

        let models = query
            .order_by_desc(farmers::Column::CreatedAt)
            .order_by_desc(farmers::Column::Id)
            .limit(limit)
            .all(&self.conn)
            .await
            .map_err(database_error)?;

        models.into_iter().map(Farmer::try_from).collect()
    }
}

/// Puts a backslash before everything `LIKE` reads as a wildcard.
fn escape_like(text: &str) -> String {
    let mut escaped = String::with_capacity(text.len());

    for character in text.chars() {
        if matches!(character, '%' | '_' | '\\') {
            escaped.push('\\');
        }

        escaped.push(character);
    }

    escaped
}

#[derive(Debug)]
pub struct LetterPostgresRepository {
    conn: DatabaseConnection,
}

impl LetterPostgresRepository {
    pub fn new(conn: DatabaseConnection) -> Self {
        Self { conn }
    }
}

#[async_trait]
impl LetterRepository for LetterPostgresRepository {
    async fn issue(
        &self,
        farmer_id: i32,
        staff_id: i32,
        purpose: &LetterPurpose,
        language: LetterLanguage,
        now: DateTime<Utc>,
    ) -> Result<Option<(Letter, Farmer)>, AppError> {
        let transaction = self.conn.begin().await.map_err(database_error)?;

        // The farmer's row is held until the letter is stored. A second
        // letter for the same farmer waits here, so it counts the first one
        // and takes the next number. Letters are never deleted, so the
        // count is also the highest number given so far.
        let Some(farmer) = farmers::Entity::find_by_id(farmer_id)
            .lock_exclusive()
            .one(&transaction)
            .await
            .map_err(database_error)?
        else {
            return Ok(None);
        };

        let earlier = letters::Entity::find()
            .filter(letters::Column::FarmerId.eq(farmer_id))
            .count(&transaction)
            .await
            .map_err(database_error)?;

        let letter = Letter::issue(
            farmer_id,
            earlier + 1,
            staff_id,
            purpose.clone(),
            language,
            now,
        );

        // The unique index on the number stands behind the lock: a number
        // can never be stored twice, whatever else goes wrong.
        let stored = letters::ActiveModel::from(&letter)
            .insert(&transaction)
            .await
            .map_err(database_error)?;

        transaction.commit().await.map_err(database_error)?;

        Ok(Some((Letter::try_from(stored)?, Farmer::try_from(farmer)?)))
    }

    async fn find_by_number(
        &self,
        number: &LetterNumber,
    ) -> Result<Option<LetterRecord>, AppError> {
        let Some(letter) = letters::Entity::find()
            .filter(letters::Column::Number.eq(number.as_str()))
            .one(&self.conn)
            .await
            .map_err(database_error)?
        else {
            return Ok(None);
        };

        let farmer = farmers::Entity::find_by_id(letter.farmer_id)
            .one(&self.conn)
            .await
            .map_err(database_error)?;

        Ok(Some(LetterRecord {
            letter: Letter::try_from(letter)?,
            farmer_name: farmer
                .and_then(|farmer| farmer.name)
                .map(FarmerName::new)
                .transpose()?,
        }))
    }
}

#[derive(Debug)]
pub struct SignInChallengePostgresRepository {
    conn: DatabaseConnection,
}

impl SignInChallengePostgresRepository {
    pub fn new(conn: DatabaseConnection) -> Self {
        Self { conn }
    }
}

#[async_trait]
impl SignInChallengeRepository for SignInChallengePostgresRepository {
    async fn find_by_phone(&self, phone: &Phone) -> Result<Option<SignInChallenge>, AppError> {
        let model = sign_in_challenges::Entity::find_by_id(phone.as_str())
            .one(&self.conn)
            .await
            .map_err(database_error)?;

        model.map(SignInChallenge::try_from).transpose()
    }

    async fn save_if_due(
        &self,
        challenge: &SignInChallenge,
        sent_before: DateTime<Utc>,
    ) -> Result<bool, AppError> {
        // The phone is the key, so saving a new challenge replaces the old,
        // but only an old one that was sent long enough ago. One statement
        // checks and writes, so requests at the same moment cannot all pass.
        let stored =
            sign_in_challenges::Entity::insert(sign_in_challenges::ActiveModel::from(challenge))
                .on_conflict(
                    OnConflict::column(sign_in_challenges::Column::Phone)
                        .update_columns([
                            sign_in_challenges::Column::CodeHash,
                            sign_in_challenges::Column::Language,
                            sign_in_challenges::Column::Attempts,
                            sign_in_challenges::Column::SentAt,
                            sign_in_challenges::Column::ExpiresAt,
                            sign_in_challenges::Column::UsedAt,
                        ])
                        // A code that was already used does not hold the
                        // phone waiting; an unused one does until it is old
                        // enough.
                        .action_and_where(
                            Expr::col((
                                sign_in_challenges::Entity,
                                sign_in_challenges::Column::SentAt,
                            ))
                            .lte(sent_before.naive_utc())
                            .or(Expr::col((
                                sign_in_challenges::Entity,
                                sign_in_challenges::Column::UsedAt,
                            ))
                            .is_not_null()),
                        )
                        .to_owned(),
                )
                .exec_without_returning(&self.conn)
                .await
                .map_err(database_error)?;

        Ok(stored > 0)
    }

    async fn record_attempt(
        &self,
        phone: &Phone,
        max_attempts: u32,
    ) -> Result<Option<SignInChallenge>, AppError> {
        // One statement counts the attempt and enforces the limit, so
        // guesses sent at the same moment are counted one after another.
        let counted = sign_in_challenges::Entity::update_many()
            .col_expr(
                sign_in_challenges::Column::Attempts,
                Expr::col(sign_in_challenges::Column::Attempts).add(1),
            )
            .filter(sign_in_challenges::Column::Phone.eq(phone.as_str()))
            .filter(
                sign_in_challenges::Column::Attempts
                    .lt(i32::try_from(max_attempts).unwrap_or(i32::MAX)),
            )
            .exec_with_returning(&self.conn)
            .await
            .map_err(database_error)?;

        counted
            .into_iter()
            .next()
            .map(SignInChallenge::try_from)
            .transpose()
    }

    async fn consume(
        &self,
        phone: &Phone,
        code_hash: &str,
        now: DateTime<Utc>,
        reusable_since: DateTime<Utc>,
    ) -> Result<bool, AppError> {
        // One statement decides: the first use stamps the time, a repeat
        // inside the window leaves the stamp alone, anything later matches
        // no row.
        let result = sign_in_challenges::Entity::update_many()
            .col_expr(
                sign_in_challenges::Column::UsedAt,
                Expr::cust_with_values("COALESCE(used_at, $1)", [now.naive_utc()]),
            )
            .filter(sign_in_challenges::Column::Phone.eq(phone.as_str()))
            .filter(sign_in_challenges::Column::CodeHash.eq(code_hash))
            .filter(
                Condition::any()
                    .add(sign_in_challenges::Column::UsedAt.is_null())
                    .add(sign_in_challenges::Column::UsedAt.gt(reusable_since.naive_utc())),
            )
            .exec(&self.conn)
            .await
            .map_err(database_error)?;

        Ok(result.rows_affected > 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_search_matches_its_text_anywhere() {
        assert_eq!(contains_pattern("hiwa"), "%hiwa%");
    }

    #[test]
    fn the_wildcards_of_like_are_searched_for_as_plain_characters() {
        assert_eq!(contains_pattern("50%"), r"%50\%%");
        assert_eq!(contains_pattern("a_b"), r"%a\_b%");
        assert_eq!(contains_pattern(r"a\b"), r"%a\\b%");
    }
}
