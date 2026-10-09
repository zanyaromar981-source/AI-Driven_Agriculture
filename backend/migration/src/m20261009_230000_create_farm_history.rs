use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // `farm_id` carries no foreign key in either table: the farms table
        // belongs to another feature. Every read starts from a farm that
        // exists, so months of a farm that is gone are never served, and
        // the trigger at the end of this migration removes them.
        //
        // One row per farm, metric and month. The primary key is the
        // natural key, with `farm_id` leading, so it is also the index
        // that finds a farm's months and the one an upsert conflicts on.
        manager
            .create_table(
                Table::create()
                    .table(FarmHistory::Table)
                    .if_not_exists()
                    .col(integer(FarmHistory::FarmId))
                    .col(string(FarmHistory::Metric))
                    // The first day of the month.
                    .col(date(FarmHistory::Month))
                    .col(double(FarmHistory::Value))
                    .primary_key(
                        Index::create()
                            .col(FarmHistory::FarmId)
                            .col(FarmHistory::Metric)
                            .col(FarmHistory::Month),
                    )
                    .to_owned(),
            )
            .await?;

        // What every month of one series has in common.
        manager
            .create_table(
                Table::create()
                    .table(FarmHistorySeries::Table)
                    .if_not_exists()
                    .col(integer(FarmHistorySeries::FarmId))
                    .col(string(FarmHistorySeries::Metric))
                    .col(string(FarmHistorySeries::Unit))
                    .col(string_len(FarmHistorySeries::Source, 200))
                    .col(timestamp(FarmHistorySeries::AsOf))
                    .primary_key(
                        Index::create()
                            .col(FarmHistorySeries::FarmId)
                            .col(FarmHistorySeries::Metric),
                    )
                    .to_owned(),
            )
            .await?;

        let connection = manager.get_connection();

        // Per statement, so a push of 120 months raises the version once.
        for table in ["farm_history", "farm_history_series"] {
            connection
                .execute_unprepared(&format!(
                    "CREATE TRIGGER bump_data_version
                       AFTER INSERT OR UPDATE OR DELETE ON {table}
                       FOR EACH STATEMENT EXECUTE FUNCTION bump_data_version('farms')"
                ))
                .await?;
        }

        // A deleted farm takes its history with it, inside the delete's own
        // transaction and whichever route deleted it (the farmer, staff, or
        // the removal of a farmer with all farms). It stands in for the
        // foreign key these tables do not have. The series row goes first,
        // the order a push locks in.
        connection
            .execute_unprepared(
                "CREATE OR REPLACE FUNCTION clear_farm_history() RETURNS trigger AS $$
                 BEGIN
                   DELETE FROM farm_history_series WHERE farm_id = OLD.id;
                   DELETE FROM farm_history WHERE farm_id = OLD.id;
                   RETURN NULL;
                 END
                 $$ LANGUAGE plpgsql;
                 CREATE TRIGGER clear_farm_history
                   AFTER DELETE ON farms
                   FOR EACH ROW EXECUTE FUNCTION clear_farm_history()",
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "DROP TRIGGER IF EXISTS clear_farm_history ON farms;
                 DROP FUNCTION IF EXISTS clear_farm_history()",
            )
            .await?;

        manager
            .drop_table(Table::drop().table(FarmHistory::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(FarmHistorySeries::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum FarmHistory {
    Table,
    FarmId,
    Metric,
    Month,
    Value,
}

#[derive(DeriveIden)]
enum FarmHistorySeries {
    Table,
    FarmId,
    Metric,
    Unit,
    Source,
    AsOf,
}
