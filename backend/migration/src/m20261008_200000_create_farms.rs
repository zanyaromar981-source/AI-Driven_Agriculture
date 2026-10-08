use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Farms::Table)
                    .if_not_exists()
                    .col(pk_auto(Farms::Id))
                    .col(string_len(Farms::Name, 100))
                    .col(string(Farms::Phone))
                    .col(json_binary(Farms::Outline))
                    .col(timestamp_null(Farms::CreatedOfflineAt))
                    .col(
                        timestamp(Farms::CreatedAt)
                            .not_null()
                            .default(SimpleExpr::Custom("CURRENT_TIMESTAMP".into())),
                    )
                    .col(
                        timestamp(Farms::UpdatedAt)
                            .not_null()
                            .default(SimpleExpr::Custom("CURRENT_TIMESTAMP".into())),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_farms_phone")
                    .table(Farms::Table)
                    .col(Farms::Phone)
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(FarmCells::Table)
                    .if_not_exists()
                    .col(pk_auto(FarmCells::Id))
                    .col(integer(FarmCells::FarmId))
                    .col(integer(FarmCells::E))
                    .col(integer(FarmCells::N))
                    .col(string(FarmCells::Crop))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_farm_cells_farm_id")
                            .from(FarmCells::Table, FarmCells::FarmId)
                            .to(Farms::Table, Farms::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_farm_cells_farm_id_e_n")
                    .table(FarmCells::Table)
                    .col(FarmCells::FarmId)
                    .col(FarmCells::E)
                    .col(FarmCells::N)
                    .unique()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(FarmCells::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Farms::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum Farms {
    Table,
    Id,
    Name,
    Phone,
    Outline,
    CreatedOfflineAt,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum FarmCells {
    Table,
    Id,
    FarmId,
    E,
    N,
    Crop,
}
