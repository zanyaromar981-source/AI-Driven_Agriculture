use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Fires::Table)
                    .if_not_exists()
                    .col(pk_auto(Fires::Id))
                    .col(string_len_uniq(Fires::ExternalId, 100))
                    .col(double(Fires::Lat))
                    .col(double(Fires::Lon))
                    .col(string_len_null(Fires::ZoneSlug, 40))
                    .col(string_len_null(Fires::PlaceEn, 120))
                    .col(string_len_null(Fires::PlaceKu, 120))
                    .col(timestamp(Fires::DetectedAt))
                    .col(double_null(Fires::AreaHa))
                    .col(double_null(Fires::WindKmh))
                    .col(string_null(Fires::WindDirection))
                    .col(string(Fires::Status))
                    .col(integer_null(Fires::FarmsWithin5km))
                    .col(integer_null(Fires::FarmersAlerted))
                    .col(string_len(Fires::Source, 120))
                    .col(
                        timestamp(Fires::UpdatedAt)
                            .not_null()
                            .default(SimpleExpr::Custom("CURRENT_TIMESTAMP".into())),
                    )
                    .to_owned(),
            )
            .await?;

        // The dashboard always asks for the newest fires of a window.
        manager
            .create_index(
                Index::create()
                    .name("idx_fires_detected_at")
                    .table(Fires::Table)
                    .col(Fires::DetectedAt)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Fires::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum Fires {
    Table,
    Id,
    ExternalId,
    Lat,
    Lon,
    ZoneSlug,
    PlaceEn,
    PlaceKu,
    DetectedAt,
    AreaHa,
    WindKmh,
    WindDirection,
    Status,
    #[sea_orm(iden = "farms_within_5km")]
    FarmsWithin5km,
    FarmersAlerted,
    Source,
    UpdatedAt,
}
