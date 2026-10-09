use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

/// The crop codes the farms and alwa features accepted when the list was
/// still fixed in code, so every stored cell, listing and price keeps
/// naming a crop that exists: code, English name, Sorani name, colour,
/// category, season.
///
/// The first eleven are the farm crops, in the order of the app's picker;
/// their Sorani names and colours are the app's own (`app/lib/l10n/strings.dart`
/// and `app/lib/crops.dart`). The last five were traded at the alwa only:
/// the app has no Sorani name for them, so it is left empty for staff to
/// write, and their colours were chosen here. Category and season are the
/// usual ones for the Kurdistan Region; staff can correct them. No yield is
/// seeded: nobody has entered one.
const CROPS: [(&str, &str, Option<&str>, &str, &str, &str); 16] = [
    ("wheat", "Wheat", Some("گەنم"), "#e0b13a", "cereal", "winter"),
    ("barley", "Barley", Some("جۆ"), "#c8b560", "cereal", "winter"),
    ("tomato", "Tomato", Some("تەماتە"), "#d9483b", "vegetable", "summer"),
    ("cucumber", "Cucumber", Some("خەیار"), "#6db352", "vegetable", "summer"),
    ("potato", "Potato", Some("پەتاتە"), "#a9784a", "vegetable", "summer"),
    ("onion", "Onion", Some("پیاز"), "#b46fa8", "vegetable", "summer"),
    ("watermelon", "Watermelon", Some("شووتی"), "#ef7c8e", "fruit", "summer"),
    ("grape", "Grape", Some("ترێ"), "#7e57c2", "fruit", "perennial"),
    ("olive", "Olive", Some("زەیتوون"), "#7d8b3a", "oil", "perennial"),
    ("sunflower", "Sunflower", Some("گوڵەبەڕۆژە"), "#f5c518", "oil", "summer"),
    ("chickpea", "Chickpea", Some("نۆک"), "#d9b88a", "legume", "winter"),
    ("pomegranate", "Pomegranate", None, "#b5173a", "fruit", "perennial"),
    ("okra", "Okra", None, "#4e8f3a", "vegetable", "summer"),
    ("eggplant", "Eggplant", None, "#5b2a6e", "vegetable", "summer"),
    ("pepper", "Pepper", None, "#e2572b", "vegetable", "summer"),
    ("apple", "Apple", None, "#9ccc65", "fruit", "perennial"),
];

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // The code is the primary key, which is also the unique index that
        // stops two crops taking the same code.
        manager
            .create_table(
                Table::create()
                    .table(Crops::Table)
                    .if_not_exists()
                    .col(string_len(Crops::Code, 24).primary_key())
                    .col(string_len(Crops::NameEn, 60))
                    .col(string_len_null(Crops::NameKu, 60))
                    .col(string_len(Crops::Color, 7))
                    .col(string_len(Crops::Category, 20))
                    .col(string_len(Crops::Season, 20))
                    .col(double_null(Crops::YieldKgPerDunam))
                    .col(boolean(Crops::Active).default(true))
                    .col(integer(Crops::SortOrder).default(0))
                    .col(timestamp(Crops::CreatedAt).default(Expr::current_timestamp()))
                    .col(timestamp(Crops::UpdatedAt).default(Expr::current_timestamp()))
                    .to_owned(),
            )
            .await?;

        let mut seed = Query::insert()
            .into_table(Crops::Table)
            .columns([
                Crops::Code,
                Crops::NameEn,
                Crops::NameKu,
                Crops::Color,
                Crops::Category,
                Crops::Season,
                Crops::SortOrder,
            ])
            .on_conflict(OnConflict::column(Crops::Code).do_nothing().to_owned())
            .to_owned();

        // Steps of ten leave room to put a new crop between two old ones.
        for (position, (code, name_en, name_ku, color, category, season)) in
            CROPS.into_iter().enumerate()
        {
            seed.values_panic([
                code.into(),
                name_en.into(),
                name_ku.into(),
                color.into(),
                category.into(),
                season.into(),
                ((position as i32 + 1) * 10).into(),
            ]);
        }

        manager.exec_stmt(seed).await?;

        manager
            .get_connection()
            .execute_unprepared(
                "DROP TRIGGER IF EXISTS bump_data_version ON crops;
                 CREATE TRIGGER bump_data_version
                   AFTER INSERT OR UPDATE OR DELETE ON crops
                   FOR EACH STATEMENT EXECUTE FUNCTION bump_data_version('crops')",
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Crops::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Crops {
    Table,
    Code,
    NameEn,
    NameKu,
    Color,
    Category,
    Season,
    YieldKgPerDunam,
    Active,
    SortOrder,
    CreatedAt,
    UpdatedAt,
}
