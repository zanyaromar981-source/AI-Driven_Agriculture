use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // `farm_id` carries no foreign key: the farms table belongs to
        // another feature. Every read starts from a farm that exists, so an
        // alert left behind by a deleted farm is never served or pushed.
        manager
            .create_table(
                Table::create()
                    .table(Alerts::Table)
                    .if_not_exists()
                    .col(pk_auto(Alerts::Id))
                    .col(integer(Alerts::FarmId))
                    .col(string_len(Alerts::Key, 120))
                    .col(string(Alerts::Type))
                    .col(date(Alerts::Day))
                    .col(string(Alerts::Level))
                    .col(string(Alerts::Confidence))
                    .col(string_len(Alerts::Ku, 500))
                    .col(string_len(Alerts::En, 500))
                    .col(string_len(Alerts::ActionKu, 500))
                    .col(string_len(Alerts::ActionEn, 500))
                    .col(boolean(Alerts::Pushed).default(false))
                    .col(timestamp_null(Alerts::PushedAt))
                    .col(boolean(Alerts::Done).default(false))
                    .col(timestamp_null(Alerts::DoneAt))
                    .col(string_len(Alerts::Source, 120))
                    .col(
                        timestamp(Alerts::CreatedAt)
                            .not_null()
                            .default(SimpleExpr::Custom("CURRENT_TIMESTAMP".into())),
                    )
                    .col(
                        timestamp(Alerts::UpdatedAt)
                            .not_null()
                            .default(SimpleExpr::Custom("CURRENT_TIMESTAMP".into())),
                    )
                    .to_owned(),
            )
            .await?;

        // The key a job gives an alert is unique within its farm: a job that
        // runs again replaces its alert instead of adding a second one.
        // `farm_id` leads, so this also finds all alerts of one farm.
        manager
            .create_index(
                Index::create()
                    .name("idx_alerts_farm_id_key")
                    .table(Alerts::Table)
                    .col(Alerts::FarmId)
                    .col(Alerts::Key)
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                "DROP TRIGGER IF EXISTS bump_data_version ON alerts;
                 CREATE TRIGGER bump_data_version
                   AFTER INSERT OR UPDATE OR DELETE ON alerts
                   FOR EACH STATEMENT EXECUTE FUNCTION bump_data_version('farms')",
            )
            .await?;

        // No data version trigger: no website shows devices, and a push
        // token is a secret.
        manager
            .create_table(
                Table::create()
                    .table(Devices::Table)
                    .if_not_exists()
                    .col(pk_auto(Devices::Id))
                    .col(string_len(Devices::TokenHash, 64))
                    .col(text(Devices::PushToken))
                    .col(string(Devices::Phone))
                    .col(string(Devices::Platform))
                    .col(string(Devices::Lang))
                    .col(boolean(Devices::RedAlerts).default(true))
                    .col(boolean(Devices::WeeklyPlan).default(true))
                    .col(
                        timestamp(Devices::CreatedAt)
                            .not_null()
                            .default(SimpleExpr::Custom("CURRENT_TIMESTAMP".into())),
                    )
                    .col(
                        timestamp(Devices::UpdatedAt)
                            .not_null()
                            .default(SimpleExpr::Custom("CURRENT_TIMESTAMP".into())),
                    )
                    .to_owned(),
            )
            .await?;

        // A token may be 4,096 characters, more than an index entry holds,
        // so the unique key is its SHA-256.
        manager
            .create_index(
                Index::create()
                    .name("idx_devices_token_hash")
                    .table(Devices::Table)
                    .col(Devices::TokenHash)
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_devices_phone")
                    .table(Devices::Table)
                    .col(Devices::Phone)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Devices::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Alerts::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum Alerts {
    Table,
    Id,
    FarmId,
    Key,
    Type,
    Day,
    Level,
    Confidence,
    Ku,
    En,
    ActionKu,
    ActionEn,
    Pushed,
    PushedAt,
    Done,
    DoneAt,
    Source,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Devices {
    Table,
    Id,
    TokenHash,
    PushToken,
    Phone,
    Platform,
    Lang,
    RedAlerts,
    WeeklyPlan,
    CreatedAt,
    UpdatedAt,
}
