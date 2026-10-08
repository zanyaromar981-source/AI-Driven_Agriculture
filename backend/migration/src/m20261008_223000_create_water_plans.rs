use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(WaterPlanEntries::Table)
                    .if_not_exists()
                    .col(pk_auto(WaterPlanEntries::Id))
                    .col(string_len(WaterPlanEntries::Season, 7))
                    .col(string_len(WaterPlanEntries::ZoneSlug, 40))
                    .col(double(WaterPlanEntries::Need))
                    .col(string_len_null(WaterPlanEntries::DamSlug, 40))
                    .col(double_null(WaterPlanEntries::SendMillionM3))
                    .col(boolean(WaterPlanEntries::Urgent).default(false))
                    .col(string_len_null(WaterPlanEntries::NoteEn, 200))
                    .col(string_len_null(WaterPlanEntries::NoteKu, 200))
                    .col(
                        timestamp(WaterPlanEntries::UpdatedAt)
                            .not_null()
                            .default(SimpleExpr::Custom("CURRENT_TIMESTAMP".into())),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_water_plan_entries_season_zone_slug")
                    .table(WaterPlanEntries::Table)
                    .col(WaterPlanEntries::Season)
                    .col(WaterPlanEntries::ZoneSlug)
                    .unique()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(WaterPlanEntries::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum WaterPlanEntries {
    Table,
    Id,
    Season,
    ZoneSlug,
    Need,
    DamSlug,
    SendMillionM3,
    Urgent,
    NoteEn,
    NoteKu,
    UpdatedAt,
}
