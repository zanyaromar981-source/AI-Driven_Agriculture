use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

/// The wholesale produce markets of the four cities: slug, English name,
/// Sorani name.
const MARKETS: [(&str, &str, &str); 4] = [
    ("sulaymaniyah", "Sulaymaniyah", "سلێمانی"),
    ("erbil", "Erbil", "هەولێر"),
    ("duhok", "Duhok", "دهۆک"),
    ("kalar", "Kalar", "کەلار"),
];

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(AlwaMarkets::Table)
                    .if_not_exists()
                    .col(pk_auto(AlwaMarkets::Id))
                    .col(string_len_uniq(AlwaMarkets::Slug, 40))
                    .col(string(AlwaMarkets::NameEn))
                    .col(string(AlwaMarkets::NameKu))
                    .to_owned(),
            )
            .await?;

        let mut markets = Query::insert()
            .into_table(AlwaMarkets::Table)
            .columns([
                AlwaMarkets::Slug,
                AlwaMarkets::NameEn,
                AlwaMarkets::NameKu,
            ])
            .to_owned();

        for (slug, name_en, name_ku) in MARKETS {
            markets.values_panic([slug.into(), name_en.into(), name_ku.into()]);
        }

        manager.exec_stmt(markets).await?;

        manager
            .create_table(
                Table::create()
                    .table(AlwaPrices::Table)
                    .if_not_exists()
                    .col(pk_auto(AlwaPrices::Id))
                    .col(integer(AlwaPrices::MarketId))
                    .col(string(AlwaPrices::Crop))
                    .col(date(AlwaPrices::Day))
                    .col(
                        integer(AlwaPrices::PriceIqdPerKg)
                            .check(Expr::col(AlwaPrices::PriceIqdPerKg).gt(0)),
                    )
                    .col(boolean(AlwaPrices::Fixed).default(false))
                    .col(string(AlwaPrices::Source))
                    .col(
                        timestamp(AlwaPrices::UpdatedAt)
                            .not_null()
                            .default(SimpleExpr::Custom("CURRENT_TIMESTAMP".into())),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_alwa_prices_market_id")
                            .from(AlwaPrices::Table, AlwaPrices::MarketId)
                            .to(AlwaMarkets::Table, AlwaMarkets::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_alwa_prices_market_id_crop_day")
                    .table(AlwaPrices::Table)
                    .col(AlwaPrices::MarketId)
                    .col(AlwaPrices::Crop)
                    .col(AlwaPrices::Day)
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(AlwaListings::Table)
                    .if_not_exists()
                    .col(pk_auto(AlwaListings::Id))
                    .col(string(AlwaListings::SellerPhone))
                    .col(string_len_null(AlwaListings::SellerName, 60))
                    .col(string(AlwaListings::Crop))
                    .col(
                        integer(AlwaListings::QuantityKg)
                            .check(Expr::col(AlwaListings::QuantityKg).gt(0)),
                    )
                    .col(
                        integer(AlwaListings::AskingPriceIqdPerKg)
                            .check(Expr::col(AlwaListings::AskingPriceIqdPerKg).gt(0)),
                    )
                    .col(string_null(AlwaListings::Grade))
                    .col(string(AlwaListings::Pickup))
                    .col(integer(AlwaListings::MarketId))
                    .col(string_len_null(AlwaListings::ZoneSlug, 40))
                    .col(string_len_null(AlwaListings::Note, 200))
                    .col(timestamp(AlwaListings::ClosesAt))
                    .col(string(AlwaListings::Status))
                    .col(
                        timestamp(AlwaListings::CreatedAt)
                            .not_null()
                            .default(SimpleExpr::Custom("CURRENT_TIMESTAMP".into())),
                    )
                    .col(
                        timestamp(AlwaListings::UpdatedAt)
                            .not_null()
                            .default(SimpleExpr::Custom("CURRENT_TIMESTAMP".into())),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_alwa_listings_market_id")
                            .from(AlwaListings::Table, AlwaListings::MarketId)
                            .to(AlwaMarkets::Table, AlwaMarkets::Id)
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_alwa_listings_seller_phone")
                    .table(AlwaListings::Table)
                    .col(AlwaListings::SellerPhone)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_alwa_listings_status_market_id_created_at")
                    .table(AlwaListings::Table)
                    .col(AlwaListings::Status)
                    .col(AlwaListings::MarketId)
                    .col(AlwaListings::CreatedAt)
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(AlwaOffers::Table)
                    .if_not_exists()
                    .col(pk_auto(AlwaOffers::Id))
                    .col(integer(AlwaOffers::ListingId))
                    .col(string(AlwaOffers::BuyerPhone))
                    .col(string_len(AlwaOffers::BuyerName, 60))
                    .col(string(AlwaOffers::BuyerKind))
                    .col(
                        integer(AlwaOffers::QuantityKg)
                            .check(Expr::col(AlwaOffers::QuantityKg).gt(0)),
                    )
                    .col(
                        integer(AlwaOffers::PriceIqdPerKg)
                            .check(Expr::col(AlwaOffers::PriceIqdPerKg).gt(0)),
                    )
                    .col(string(AlwaOffers::Status))
                    .col(
                        timestamp(AlwaOffers::CreatedAt)
                            .not_null()
                            .default(SimpleExpr::Custom("CURRENT_TIMESTAMP".into())),
                    )
                    .col(
                        timestamp(AlwaOffers::UpdatedAt)
                            .not_null()
                            .default(SimpleExpr::Custom("CURRENT_TIMESTAMP".into())),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_alwa_offers_listing_id")
                            .from(AlwaOffers::Table, AlwaOffers::ListingId)
                            .to(AlwaListings::Table, AlwaListings::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_alwa_offers_listing_id")
                    .table(AlwaOffers::Table)
                    .col(AlwaOffers::ListingId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_alwa_offers_buyer_phone")
                    .table(AlwaOffers::Table)
                    .col(AlwaOffers::BuyerPhone)
                    .to_owned(),
            )
            .await?;

        // Deals are read by the day an offer was accepted.
        manager
            .create_index(
                Index::create()
                    .name("idx_alwa_offers_status_updated_at")
                    .table(AlwaOffers::Table)
                    .col(AlwaOffers::Status)
                    .col(AlwaOffers::UpdatedAt)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(AlwaOffers::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(AlwaListings::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(AlwaPrices::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(AlwaMarkets::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum AlwaMarkets {
    Table,
    Id,
    Slug,
    NameEn,
    NameKu,
}

#[derive(DeriveIden)]
enum AlwaPrices {
    Table,
    Id,
    MarketId,
    Crop,
    Day,
    PriceIqdPerKg,
    Fixed,
    Source,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum AlwaListings {
    Table,
    Id,
    SellerPhone,
    SellerName,
    Crop,
    QuantityKg,
    AskingPriceIqdPerKg,
    Grade,
    Pickup,
    MarketId,
    ZoneSlug,
    Note,
    ClosesAt,
    Status,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum AlwaOffers {
    Table,
    Id,
    ListingId,
    BuyerPhone,
    BuyerName,
    BuyerKind,
    QuantityKg,
    PriceIqdPerKg,
    Status,
    CreatedAt,
    UpdatedAt,
}
