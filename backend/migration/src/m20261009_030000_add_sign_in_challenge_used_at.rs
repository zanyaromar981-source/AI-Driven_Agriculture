use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(SignInChallenges::Table)
                    .add_column(timestamp_null(SignInChallenges::UsedAt))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(SignInChallenges::Table)
                    .drop_column(SignInChallenges::UsedAt)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum SignInChallenges {
    Table,
    UsedAt,
}
