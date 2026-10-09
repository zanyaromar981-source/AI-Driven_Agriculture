use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Staff::Table)
                    .add_column(string_len_null(Staff::Phone, 20))
                    .add_column(string_len_null(Staff::JobTitle, 80))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Staff::Table)
                    .drop_column(Staff::Phone)
                    .drop_column(Staff::JobTitle)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum Staff {
    Table,
    Phone,
    JobTitle,
}
