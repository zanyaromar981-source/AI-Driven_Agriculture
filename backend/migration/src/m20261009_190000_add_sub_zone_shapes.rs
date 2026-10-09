use sea_orm_migration::{
    prelude::*,
    schema::*,
    sea_orm::{DbBackend, Statement},
};

use crate::sub_zone_shapes::SUB_ZONE_SHAPES;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // The edge of the sub-zone as drawn on the team's map: an array of
        // rings, each an array of `[lon, lat]` in WGS84 degrees. This is
        // reference data like the names beside it, and it is what tells
        // which sub-zone a farm lies in. Null means no shape is known.
        manager
            .alter_table(
                Table::alter()
                    .table(SubZones::Table)
                    .add_column(json_binary_null(SubZones::Outline))
                    .to_owned(),
            )
            .await?;

        let connection = manager.get_connection();

        for (zone_slug, sub_zone_slug, outline) in SUB_ZONE_SHAPES {
            let written = connection
                .execute_raw(Statement::from_sql_and_values(
                    DbBackend::Postgres,
                    "UPDATE sub_zones
                        SET outline = $1::jsonb
                       FROM zones
                      WHERE zones.id = sub_zones.zone_id
                        AND zones.slug = $2
                        AND sub_zones.slug = $3",
                    [outline.into(), zone_slug.into(), sub_zone_slug.into()],
                ))
                .await?;

            // A shape that names no seeded sub-zone would leave a hole in
            // the map that nothing reports later, so it stops the migration.
            if written.rows_affected() != 1 {
                return Err(DbErr::Migration(format!(
                    "no sub-zone {zone_slug}/{sub_zone_slug} to give its shape to"
                )));
            }
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(SubZones::Table)
                    .drop_column(SubZones::Outline)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum SubZones {
    Table,
    Outline,
}
