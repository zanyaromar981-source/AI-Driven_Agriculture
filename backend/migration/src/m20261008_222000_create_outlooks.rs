use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(SeasonOutlooks::Table)
                    .if_not_exists()
                    .col(pk_auto(SeasonOutlooks::Id))
                    .col(string_len(SeasonOutlooks::ZoneSlug, 40))
                    .col(string_len(SeasonOutlooks::Season, 7))
                    .col(date(SeasonOutlooks::Issued))
                    .col(string(SeasonOutlooks::Outlook))
                    .col(double(SeasonOutlooks::ConfidencePct))
                    .col(string_len_null(SeasonOutlooks::ReasonEn, 200))
                    .col(string_len_null(SeasonOutlooks::ReasonKu, 200))
                    .col(
                        timestamp(SeasonOutlooks::UpdatedAt)
                            .not_null()
                            .default(SimpleExpr::Custom("CURRENT_TIMESTAMP".into())),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_season_outlooks_zone_slug_season_issued")
                    .table(SeasonOutlooks::Table)
                    .col(SeasonOutlooks::ZoneSlug)
                    .col(SeasonOutlooks::Season)
                    .col(SeasonOutlooks::Issued)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // The dashboard reads one whole issue at a time.
        manager
            .create_index(
                Index::create()
                    .name("idx_season_outlooks_season_issued")
                    .table(SeasonOutlooks::Table)
                    .col(SeasonOutlooks::Season)
                    .col(SeasonOutlooks::Issued)
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(OutlookRuns::Table)
                    .if_not_exists()
                    .col(pk_auto(OutlookRuns::Id))
                    .col(string_len(OutlookRuns::Season, 7))
                    .col(date(OutlookRuns::Issued))
                    .col(integer(OutlookRuns::SeasonsTested))
                    .col(integer(OutlookRuns::SeasonsRight))
                    .col(string_len(OutlookRuns::Method, 200))
                    .col(
                        timestamp(OutlookRuns::UpdatedAt)
                            .not_null()
                            .default(SimpleExpr::Custom("CURRENT_TIMESTAMP".into())),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_outlook_runs_season_issued")
                    .table(OutlookRuns::Table)
                    .col(OutlookRuns::Season)
                    .col(OutlookRuns::Issued)
                    .unique()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(OutlookRuns::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(SeasonOutlooks::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum SeasonOutlooks {
    Table,
    Id,
    ZoneSlug,
    Season,
    Issued,
    Outlook,
    ConfidencePct,
    ReasonEn,
    ReasonKu,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum OutlookRuns {
    Table,
    Id,
    Season,
    Issued,
    SeasonsTested,
    SeasonsRight,
    Method,
    UpdatedAt,
}
