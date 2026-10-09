use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // The share of the cell inside the farm's outline, above 0 and at
        // most 100. Cells stored before this column were the ones whose
        // centre is inside the outline, each counted as a whole cell, so
        // they read as 100: an old farm keeps the crop areas it had. Its
        // cells are worked out again when the farmer next edits it.
        manager
            .alter_table(
                Table::alter()
                    .table(FarmCells::Table)
                    .add_column(double(FarmCells::InsidePct).default(100.0))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(FarmCells::Table)
                    .drop_column(FarmCells::InsidePct)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum FarmCells {
    Table,
    InsidePct,
}
