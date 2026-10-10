use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

/// Crops grown around Akre that MapSPAM 2020 and local knowledge name but
/// the first list did not have: code, English name, Sorani name, colour,
/// category, season, sort order. Sort order carries on the steps of ten
/// after the sixteen seeded crops.
const CROPS: [(&str, &str, &str, &str, &str, &str, i32); 4] = [
    ("rice", "Rice", "برنج", "#d8cfa0", "cereal", "summer", 170),
    ("fig", "Fig", "هەنجیر", "#8e5a7a", "fruit", "perennial", 180),
    ("sumac", "Sumac", "سماق", "#a8323e", "fruit", "perennial", 190),
    ("pistachio", "Pistachio", "فستق", "#93c572", "fruit", "perennial", 200),
];

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
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

        for (code, name_en, name_ku, color, category, season, sort_order) in CROPS {
            seed.values_panic([
                code.into(),
                name_en.into(),
                name_ku.into(),
                color.into(),
                category.into(),
                season.into(),
                sort_order.into(),
            ]);
        }

        manager.exec_stmt(seed).await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let codes: Vec<&str> = CROPS.iter().map(|crop| crop.0).collect();
        manager
            .exec_stmt(
                Query::delete()
                    .from_table(Crops::Table)
                    .and_where(Expr::col(Crops::Code).is_in(codes))
                    .to_owned(),
            )
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
    SortOrder,
}
