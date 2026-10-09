use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

const PRICES_MARKET_FK: &str = "fk_alwa_prices_market_id";

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Staff can now delete a market. One that still has prices must
        // refuse, as one with listings already does, so the prices no longer
        // go with it.
        replace_prices_market_fk(manager, ForeignKeyAction::Restrict).await?;

        // Who on the staff closed a listing, and why. The staff member is
        // another slice's row, so there is no foreign key.
        manager
            .alter_table(
                Table::alter()
                    .table(AlwaListings::Table)
                    .add_column(integer_null(AlwaListings::ClosedByStaffId))
                    .add_column(string_len_null(AlwaListings::ModerationNote, 200))
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(AlwaListings::Table)
                    .drop_column(AlwaListings::ClosedByStaffId)
                    .drop_column(AlwaListings::ModerationNote)
                    .to_owned(),
            )
            .await?;

        replace_prices_market_fk(manager, ForeignKeyAction::Cascade).await
    }
}

async fn replace_prices_market_fk(
    manager: &SchemaManager<'_>,
    on_delete: ForeignKeyAction,
) -> Result<(), DbErr> {
    manager
        .drop_foreign_key(
            ForeignKey::drop()
                .name(PRICES_MARKET_FK)
                .table(AlwaPrices::Table)
                .to_owned(),
        )
        .await?;

    manager
        .create_foreign_key(
            ForeignKey::create()
                .name(PRICES_MARKET_FK)
                .from(AlwaPrices::Table, AlwaPrices::MarketId)
                .to(AlwaMarkets::Table, AlwaMarkets::Id)
                .on_delete(on_delete)
                .to_owned(),
        )
        .await
}

#[derive(DeriveIden)]
enum AlwaMarkets {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum AlwaPrices {
    Table,
    MarketId,
}

#[derive(DeriveIden)]
enum AlwaListings {
    Table,
    ClosedByStaffId,
    ModerationNote,
}
