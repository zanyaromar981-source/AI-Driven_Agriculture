use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // `farm_id` carries no foreign key: the farms table belongs to
        // another feature. A reading left behind by a deleted farm is never
        // served, because every read starts from a farm that exists.
        manager
            .create_table(
                Table::create()
                    .table(FarmInsights::Table)
                    .if_not_exists()
                    .col(pk_auto(FarmInsights::Id))
                    .col(integer(FarmInsights::FarmId))
                    .col(string(FarmInsights::Topic))
                    .col(date(FarmInsights::AsOf))
                    .col(string_len(FarmInsights::Source, 120))
                    .col(string(FarmInsights::Confidence))
                    .col(string_len_null(FarmInsights::SummaryEn, 300))
                    .col(string_len_null(FarmInsights::SummaryKu, 300))
                    .col(json_binary(FarmInsights::Measures))
                    .col(
                        timestamp(FarmInsights::UpdatedAt)
                            .not_null()
                            .default(SimpleExpr::Custom("CURRENT_TIMESTAMP".into())),
                    )
                    .to_owned(),
            )
            .await?;

        // One current reading per farm and topic. `farm_id` leads, so this
        // is also the index that finds all readings of one farm.
        manager
            .create_index(
                Index::create()
                    .name("idx_farm_insights_farm_id_topic")
                    .table(FarmInsights::Table)
                    .col(FarmInsights::FarmId)
                    .col(FarmInsights::Topic)
                    .unique()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(FarmInsights::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum FarmInsights {
    Table,
    Id,
    FarmId,
    Topic,
    AsOf,
    Source,
    Confidence,
    SummaryEn,
    SummaryKu,
    Measures,
    UpdatedAt,
}
