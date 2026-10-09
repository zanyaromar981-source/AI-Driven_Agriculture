use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Workers::Table)
                    .if_not_exists()
                    .col(pk_auto(Workers::Id))
                    // The phone the person signed in with, with no foreign
                    // key: the account belongs to the farmers feature, which
                    // removes the card when it removes the account.
                    .col(string_len(Workers::Phone, 20))
                    .col(string_len(Workers::Name, 80))
                    .col(
                        integer(Workers::CostIqd)
                            .check(Expr::col(Workers::CostIqd).between(1_000, 10_000_000)),
                    )
                    .col(
                        string_len(Workers::CostPer, 10)
                            .default("day")
                            .check(Expr::col(Workers::CostPer).is_in(["day", "hour"])),
                    )
                    .col(string_len_null(Workers::Note, 200))
                    .col(string_len_null(Workers::ZoneSlug, 40))
                    .col(double_null(Workers::Lat))
                    .col(double_null(Workers::Lon))
                    .col(boolean(Workers::Available).default(true))
                    .col(
                        timestamp(Workers::CreatedAt)
                            .not_null()
                            .default(SimpleExpr::Custom("CURRENT_TIMESTAMP".into())),
                    )
                    .col(
                        timestamp(Workers::UpdatedAt)
                            .not_null()
                            .default(SimpleExpr::Custom("CURRENT_TIMESTAMP".into())),
                    )
                    // A point is both numbers or neither.
                    .check(
                        Expr::col(Workers::Lat)
                            .is_null()
                            .eq(Expr::col(Workers::Lon).is_null()),
                    )
                    .to_owned(),
            )
            .await?;

        // One card per phone: the index is what a put upserts on, so two
        // puts at the same moment leave one row.
        manager
            .create_index(
                Index::create()
                    .name("idx_workers_phone")
                    .table(Workers::Table)
                    .col(Workers::Phone)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // The farmers' list: available cards, newest update first.
        manager
            .create_index(
                Index::create()
                    .name("idx_workers_available_updated_at")
                    .table(Workers::Table)
                    .col(Workers::Available)
                    .col(Workers::UpdatedAt)
                    .to_owned(),
            )
            .await?;

        // A card is part of what the website shows about farmers' accounts,
        // so it rides on the `farmers` topic: no new topic.
        manager
            .get_connection()
            .execute_unprepared(
                "DROP TRIGGER IF EXISTS bump_data_version ON workers;
                 CREATE TRIGGER bump_data_version
                   AFTER INSERT OR UPDATE OR DELETE ON workers
                   FOR EACH STATEMENT EXECUTE FUNCTION bump_data_version('farmers')",
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Workers::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Workers {
    Table,
    Id,
    Phone,
    Name,
    CostIqd,
    CostPer,
    Note,
    ZoneSlug,
    Lat,
    Lon,
    Available,
    CreatedAt,
    UpdatedAt,
}
