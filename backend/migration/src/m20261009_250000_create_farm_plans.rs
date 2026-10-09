use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // `farm_id` carries no foreign key: the farms table belongs to
        // another feature. A plan left behind by a deleted farm is never
        // served, because every read starts from a farm that exists.
        manager
            .create_table(
                Table::create()
                    .table(FarmPlans::Table)
                    .if_not_exists()
                    .col(pk_auto(FarmPlans::Id))
                    .col(integer(FarmPlans::FarmId))
                    .col(date(FarmPlans::FromDay))
                    .col(timestamp(FarmPlans::Issued))
                    .col(json_binary(FarmPlans::RainMm))
                    .col(json_binary(FarmPlans::Tmin))
                    .col(json_binary(FarmPlans::Tmax))
                    .col(json_binary(FarmPlans::Alerts))
                    .col(json_binary(FarmPlans::Decisions))
                    .col(string_len(FarmPlans::Source, 120))
                    .col(
                        timestamp(FarmPlans::UpdatedAt)
                            .not_null()
                            .default(SimpleExpr::Custom("CURRENT_TIMESTAMP".into())),
                    )
                    .to_owned(),
            )
            .await?;

        // One current plan per farm: the index is what makes a push an
        // upsert.
        manager
            .create_index(
                Index::create()
                    .name("idx_farm_plans_farm_id")
                    .table(FarmPlans::Table)
                    .col(FarmPlans::FarmId)
                    .unique()
                    .to_owned(),
            )
            .await?;

        let connection = manager.get_connection();

        connection
            .execute_unprepared(
                "DROP TRIGGER IF EXISTS bump_data_version ON farm_plans;
                 CREATE TRIGGER bump_data_version
                   AFTER INSERT OR UPDATE OR DELETE ON farm_plans
                   FOR EACH STATEMENT EXECUTE FUNCTION bump_data_version('farms')",
            )
            .await?;

        // The job that fills the table, so the job status page knows it.
        // Its Sorani name is left empty until a speaker writes it.
        connection
            .execute_unprepared(
                "INSERT INTO jobs (job, name_en, name_ku, every_hours)
                 VALUES ('plans', '10-day farm plans', NULL, 6)
                 ON CONFLICT (job) DO NOTHING",
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared("DELETE FROM jobs WHERE job = 'plans'")
            .await?;
        manager
            .drop_table(Table::drop().table(FarmPlans::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum FarmPlans {
    Table,
    Id,
    FarmId,
    FromDay,
    Issued,
    RainMm,
    Tmin,
    Tmax,
    Alerts,
    Decisions,
    Source,
    UpdatedAt,
}
