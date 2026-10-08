use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(AlwaListings::Table)
                    .add_column(string_len_null(AlwaListings::IdempotencyKey, 128))
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_alwa_listings_seller_phone_idempotency_key")
                    .table(AlwaListings::Table)
                    .col(AlwaListings::SellerPhone)
                    .col(AlwaListings::IdempotencyKey)
                    .unique()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name("idx_alwa_listings_seller_phone_idempotency_key")
                    .table(AlwaListings::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(AlwaListings::Table)
                    .drop_column(AlwaListings::IdempotencyKey)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum AlwaListings {
    Table,
    SellerPhone,
    IdempotencyKey,
}
