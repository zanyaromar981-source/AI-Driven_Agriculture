use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Farmers::Table)
                    .if_not_exists()
                    .col(pk_auto(Farmers::Id))
                    .col(string_uniq(Farmers::Phone))
                    .col(string_len_null(Farmers::Name, 60))
                    .col(string(Farmers::Language))
                    .col(
                        timestamp(Farmers::CreatedAt)
                            .not_null()
                            .default(SimpleExpr::Custom("CURRENT_TIMESTAMP".into())),
                    )
                    .col(
                        timestamp(Farmers::UpdatedAt)
                            .not_null()
                            .default(SimpleExpr::Custom("CURRENT_TIMESTAMP".into())),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(SignInChallenges::Table)
                    .if_not_exists()
                    .col(string(SignInChallenges::Phone).primary_key())
                    .col(string(SignInChallenges::CodeHash))
                    .col(string(SignInChallenges::Language))
                    .col(integer(SignInChallenges::Attempts).default(0))
                    .col(timestamp(SignInChallenges::SentAt))
                    .col(timestamp(SignInChallenges::ExpiresAt))
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(SignInChallenges::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Farmers::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum Farmers {
    Table,
    Id,
    Phone,
    Name,
    Language,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum SignInChallenges {
    Table,
    Phone,
    CodeHash,
    Language,
    Attempts,
    SentAt,
    ExpiresAt,
}
