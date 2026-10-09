use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Where the farm lies, found from the centre of its outline when it
        // is created or edited. All three are null for a farm outside every
        // sub-zone shape. `area_m2` is the area inside the outline, kept so
        // that farms can be sorted and added up by size in SQL.
        //
        // Farms stored before this migration have all four null until
        // `farm-doctor-api backfill-farm-places` is run once: the place and
        // the area are worked out by the server's own code, which a
        // migration cannot call.
        manager
            .alter_table(
                Table::alter()
                    .table(Farms::Table)
                    .add_column(string_null(Farms::Governorate))
                    .add_column(string_null(Farms::ZoneSlug))
                    .add_column(string_null(Farms::SubZoneSlug))
                    .add_column(double_null(Farms::AreaM2))
                    .to_owned(),
            )
            .await?;

        // The dashboard lists the farms of one district. A governorate is
        // matched without regard to case and the totals read every farm, so
        // neither would use an index.
        manager
            .create_index(
                Index::create()
                    .name("idx_farms_zone_slug")
                    .table(Farms::Table)
                    .col(Farms::ZoneSlug)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name("idx_farms_zone_slug")
                    .table(Farms::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Farms::Table)
                    .drop_column(Farms::Governorate)
                    .drop_column(Farms::ZoneSlug)
                    .drop_column(Farms::SubZoneSlug)
                    .drop_column(Farms::AreaM2)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum Farms {
    Table,
    Governorate,
    ZoneSlug,
    SubZoneSlug,
    AreaM2,
}
