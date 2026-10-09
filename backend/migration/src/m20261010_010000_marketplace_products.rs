use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

/// The products of the Marketplace that are not crops (BACKEND.md 2.16):
/// code, group, unit, English name, Sorani name, colour. The names are the
/// contract's own, as the app team wrote them.
///
/// A product that is not a plant has no plant category and no season, so
/// these take `other` and `perennial`, the values that claim the least.
/// No yield is seeded: a yield per dunam means nothing for them.
const PRODUCTS: [(&str, &str, &str, &str, &str, &str); 14] = [
    ("fish", "fish_meat_eggs", "kg", "Fish", "ماسی", "#3f8fb5"),
    (
        "chicken",
        "fish_meat_eggs",
        "kg",
        "Chicken",
        "مریشک",
        "#e8a05c",
    ),
    (
        "eggs",
        "fish_meat_eggs",
        "tray_30",
        "Eggs (tray of 30)",
        "هێلکە (تەبەقەی ٣٠)",
        "#f2e2c4",
    ),
    ("honey", "honey_dairy", "kg", "Honey", "هەنگوین", "#d99a1c"),
    ("milk", "honey_dairy", "litre", "Milk", "شیر", "#dfe9f2"),
    ("yogurt", "honey_dairy", "kg", "Yogurt", "ماست", "#c9d6c0"),
    ("cheese", "honey_dairy", "kg", "Cheese", "پەنیر", "#f0d264"),
    ("sheep", "animals", "head", "Sheep", "مەڕ", "#9e9485"),
    ("goat", "animals", "head", "Goat", "بزن", "#6f5b4b"),
    ("cow", "animals", "head", "Cow", "مانگا", "#4a3728"),
    ("walnut", "nuts_dried", "kg", "Walnuts", "گوێز", "#8a5a33"),
    ("almond", "nuts_dried", "kg", "Almonds", "بادەم", "#c69c6d"),
    ("raisin", "nuts_dried", "kg", "Raisins", "مێوژ", "#5a2d4a"),
    (
        "dried_fig",
        "nuts_dried",
        "kg",
        "Dried figs",
        "هەنجیری وشک",
        "#94618e",
    ),
];

/// The crops were seeded at 10 to 160; the products follow from here, in
/// the contract's order, again in steps of ten.
const FIRST_SORT_ORDER: i32 = 210;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();

        // The crops table becomes the products table. Every row there
        // today is a crop sold by the kg, which is what the defaults say,
        // so nothing stored changes its meaning. `group` is a reserved
        // word in SQL, hence `grp`.
        conn.execute_unprepared(
            "ALTER TABLE crops
               ADD COLUMN grp varchar(20) NOT NULL DEFAULT 'crops',
               ADD COLUMN unit varchar(10) NOT NULL DEFAULT 'kg'",
        )
        .await?;

        let mut seed = Query::insert()
            .into_table(Crops::Table)
            .columns([
                Crops::Code,
                Crops::Grp,
                Crops::Unit,
                Crops::NameEn,
                Crops::NameKu,
                Crops::Color,
                Crops::Category,
                Crops::Season,
                Crops::SortOrder,
            ])
            .on_conflict(OnConflict::column(Crops::Code).do_nothing().to_owned())
            .to_owned();

        for (position, (code, group, unit, name_en, name_ku, color)) in
            PRODUCTS.into_iter().enumerate()
        {
            seed.values_panic([
                code.into(),
                group.into(),
                unit.into(),
                name_en.into(),
                name_ku.into(),
                color.into(),
                "other".into(),
                "perennial".into(),
                (FIRST_SORT_ORDER + position as i32 * 10).into(),
            ]);
        }

        manager.exec_stmt(seed).await?;

        // A listing keeps the group and the unit its product had when it
        // was posted, and a price the unit it was typed in. The quantity
        // and price columns keep their names and now count in that unit;
        // every row stored so far is a crop by the kg.
        conn.execute_unprepared(
            "ALTER TABLE alwa_listings
               ADD COLUMN grp varchar(20) NOT NULL DEFAULT 'crops',
               ADD COLUMN unit varchar(10) NOT NULL DEFAULT 'kg';
             ALTER TABLE alwa_prices
               ADD COLUMN unit varchar(10) NOT NULL DEFAULT 'kg';
             CREATE INDEX idx_alwa_listings_grp ON alwa_listings (grp)",
        )
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();

        conn.execute_unprepared(
            "DROP INDEX IF EXISTS idx_alwa_listings_grp;
             ALTER TABLE alwa_prices DROP COLUMN unit;
             ALTER TABLE alwa_listings DROP COLUMN grp, DROP COLUMN unit;
             DELETE FROM crops WHERE grp <> 'crops';
             ALTER TABLE crops DROP COLUMN grp, DROP COLUMN unit",
        )
        .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Crops {
    Table,
    Code,
    Grp,
    Unit,
    NameEn,
    NameKu,
    Color,
    Category,
    Season,
    SortOrder,
}
