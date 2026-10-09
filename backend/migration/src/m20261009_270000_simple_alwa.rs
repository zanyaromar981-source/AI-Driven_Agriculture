use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

/// Where each seeded alwa is: slug, latitude, longitude (WGS84). These are
/// the centres of the four towns, not the gates of the produce markets
/// themselves, which is near enough to tell which alwa a farmer is closest
/// to. Staff can put the exact point in on the dashboard.
const MARKET_POINTS: [(&str, f64, f64); 4] = [
    ("sulaymaniyah", 35.5572, 45.4356),
    ("erbil", 36.1911, 44.0092),
    ("duhok", 36.8669, 42.9503),
    ("kalar", 34.6292, 45.3222),
];

const LISTING_POINT_CHECK: &str = "chk_alwa_listings_lat_lon_together";
const MARKET_POINT_CHECK: &str = "chk_alwa_markets_lat_lon_together";

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // The app's listing is only a crop, a quantity, a price and the
        // place the farmer stands: the point is new, and the market and the
        // pickup, which the app does not ask for, may now be missing.
        manager
            .alter_table(
                Table::alter()
                    .table(AlwaListings::Table)
                    .add_column(double_null(AlwaListings::Lat))
                    .add_column(double_null(AlwaListings::Lon))
                    .modify_column(integer_null(AlwaListings::MarketId))
                    .modify_column(string_null(AlwaListings::Pickup))
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(AlwaMarkets::Table)
                    .add_column(double_null(AlwaMarkets::Lat))
                    .add_column(double_null(AlwaMarkets::Lon))
                    .to_owned(),
            )
            .await?;

        let conn = manager.get_connection();

        // Half a point is no point.
        for (table, check) in [
            ("alwa_listings", LISTING_POINT_CHECK),
            ("alwa_markets", MARKET_POINT_CHECK),
        ] {
            conn.execute_unprepared(&format!(
                "ALTER TABLE {table} ADD CONSTRAINT {check} CHECK ((lat IS NULL) = (lon IS NULL))"
            ))
            .await?;
        }

        for (slug, lat, lon) in MARKET_POINTS {
            manager
                .exec_stmt(
                    Query::update()
                        .table(AlwaMarkets::Table)
                        .value(AlwaMarkets::Lat, lat)
                        .value(AlwaMarkets::Lon, lon)
                        .and_where(Expr::col(AlwaMarkets::Slug).eq(slug))
                        .to_owned(),
                )
                .await?;
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // A listing without a market or a pickup cannot go back under the
        // old rules, so the columns stay nullable; only what was added goes.
        manager
            .alter_table(
                Table::alter()
                    .table(AlwaMarkets::Table)
                    .drop_column(AlwaMarkets::Lat)
                    .drop_column(AlwaMarkets::Lon)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(AlwaListings::Table)
                    .drop_column(AlwaListings::Lat)
                    .drop_column(AlwaListings::Lon)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum AlwaListings {
    Table,
    Lat,
    Lon,
    MarketId,
    Pickup,
}

#[derive(DeriveIden)]
enum AlwaMarkets {
    Table,
    Slug,
    Lat,
    Lon,
}
