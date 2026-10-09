use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

const BRIEFS_RESOURCE: &str = "briefs";

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // `scope` is the literal `region` or a zone slug. The zones belong
        // to another feature, so there is no foreign key. The text columns
        // are sized in characters, which is how Postgres counts them.
        manager
            .create_table(
                Table::create()
                    .table(DailyBriefs::Table)
                    .if_not_exists()
                    .col(pk_auto(DailyBriefs::Id))
                    .col(date(DailyBriefs::Day))
                    .col(string_len(DailyBriefs::Scope, 40))
                    .col(string_len(DailyBriefs::HeadlineEn, 120))
                    .col(string_len(DailyBriefs::HeadlineKu, 120))
                    .col(string_len(DailyBriefs::SummaryEn, 2000))
                    .col(string_len(DailyBriefs::SummaryKu, 2000))
                    .col(json_binary(DailyBriefs::Points))
                    .col(json_binary(DailyBriefs::Sources))
                    .col(string_len(DailyBriefs::Author, 80))
                    .col(timestamp(DailyBriefs::GeneratedAt).not_null())
                    .col(
                        timestamp(DailyBriefs::UpdatedAt)
                            .not_null()
                            .default(SimpleExpr::Custom("CURRENT_TIMESTAMP".into())),
                    )
                    .to_owned(),
            )
            .await?;

        // One brief per day and scope. `scope` leads, so this is also the
        // index that finds the newest brief of one scope.
        manager
            .create_index(
                Index::create()
                    .name("idx_daily_briefs_scope_day")
                    .table(DailyBriefs::Table)
                    .col(DailyBriefs::Scope)
                    .col(DailyBriefs::Day)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // `farm_id` carries no foreign key: the farms table belongs to
        // another feature. A row left behind by a deleted farm is never
        // read, because every read starts from a farm that exists.
        manager
            .create_table(
                Table::create()
                    .table(FarmBriefZones::Table)
                    .if_not_exists()
                    .col(pk_auto(FarmBriefZones::Id))
                    .col(integer_uniq(FarmBriefZones::FarmId))
                    .col(string_len(FarmBriefZones::ZoneSlug, 40))
                    .col(
                        timestamp(FarmBriefZones::UpdatedAt)
                            .not_null()
                            .default(SimpleExpr::Custom("CURRENT_TIMESTAMP".into())),
                    )
                    .to_owned(),
            )
            .await?;

        // The owner role holds every permission there is, so it gets the
        // four of the new resource. Safe to run twice.
        manager
            .get_connection()
            .execute_unprepared(&format!(
                "INSERT INTO role_permissions (role_id, resource, action) \
                 SELECT roles.id, '{BRIEFS_RESOURCE}', actions.action \
                 FROM roles, (VALUES ('create'), ('read'), ('update'), ('delete')) \
                 AS actions (action) \
                 WHERE roles.system \
                 ON CONFLICT DO NOTHING"
            ))
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(&format!(
                "DELETE FROM role_permissions WHERE resource = '{BRIEFS_RESOURCE}'"
            ))
            .await?;
        manager
            .drop_table(Table::drop().table(FarmBriefZones::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(DailyBriefs::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum DailyBriefs {
    Table,
    Id,
    Day,
    Scope,
    HeadlineEn,
    HeadlineKu,
    SummaryEn,
    SummaryKu,
    Points,
    Sources,
    Author,
    GeneratedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum FarmBriefZones {
    Table,
    Id,
    FarmId,
    ZoneSlug,
    UpdatedAt,
}
