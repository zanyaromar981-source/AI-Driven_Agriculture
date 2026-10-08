use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

/// The large reservoirs of the region: slug, English name, Sorani name and
/// design capacity in billion m3. Reference data, not measurements.
const DAMS: [(&str, &str, &str, f64); 2] = [
    ("dukan", "Dukan", "دووکان", 6.97),
    ("darbandikhan", "Darbandikhan", "دەربەندیخان", 3.0),
];

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Dams::Table)
                    .if_not_exists()
                    .col(pk_auto(Dams::Id))
                    .col(string_len_uniq(Dams::Slug, 40))
                    .col(string(Dams::NameEn))
                    .col(string(Dams::NameKu))
                    .col(double(Dams::CapacityBnM3))
                    .to_owned(),
            )
            .await?;

        let mut seed = Query::insert()
            .into_table(Dams::Table)
            .columns([
                Dams::Slug,
                Dams::NameEn,
                Dams::NameKu,
                Dams::CapacityBnM3,
            ])
            .on_conflict(OnConflict::column(Dams::Slug).do_nothing().to_owned())
            .to_owned();

        for (slug, name_en, name_ku, capacity_bn_m3) in DAMS {
            seed.values_panic([
                slug.into(),
                name_en.into(),
                name_ku.into(),
                capacity_bn_m3.into(),
            ]);
        }

        manager.exec_stmt(seed).await?;

        manager
            .create_table(
                Table::create()
                    .table(DamReadings::Table)
                    .if_not_exists()
                    .col(pk_auto(DamReadings::Id))
                    .col(integer(DamReadings::DamId))
                    .col(date(DamReadings::Day))
                    .col(double(DamReadings::PctFull))
                    .col(double_null(DamReadings::VolumeBnM3))
                    .col(double_null(DamReadings::LakeAreaKm2))
                    .col(double_null(DamReadings::FarmSupplyBnM3))
                    .col(string(DamReadings::Source))
                    .col(
                        timestamp(DamReadings::UpdatedAt)
                            .not_null()
                            .default(SimpleExpr::Custom("CURRENT_TIMESTAMP".into())),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_dam_readings_dam_id")
                            .from(DamReadings::Table, DamReadings::DamId)
                            .to(Dams::Table, Dams::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_dam_readings_dam_id_day")
                    .table(DamReadings::Table)
                    .col(DamReadings::DamId)
                    .col(DamReadings::Day)
                    .unique()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(DamReadings::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Dams::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum Dams {
    Table,
    Id,
    Slug,
    NameEn,
    NameKu,
    CapacityBnM3,
}

#[derive(DeriveIden)]
enum DamReadings {
    Table,
    Id,
    DamId,
    Day,
    PctFull,
    VolumeBnM3,
    LakeAreaKm2,
    FarmSupplyBnM3,
    Source,
    UpdatedAt,
}
