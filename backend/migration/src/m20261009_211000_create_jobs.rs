use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

/// The automatic data jobs: code, English name, Sorani name and how often
/// each is meant to run, in hours. `None` means it runs on demand and so is
/// never late.
const JOBS: [(&str, &str, &str, Option<i32>); 6] = [
    ("dryness", "Dryness", "وشکی", Some(12)),
    ("fires", "Fires", "ئاگرەکان", Some(3)),
    ("groundwater", "Groundwater", "ئاوی ژێر زەوی", Some(24)),
    ("dams", "Dams", "بەنداوەکان", Some(24)),
    ("briefs", "Daily briefs", "کورتەی ڕۆژانە", Some(24)),
    ("farm_analysis", "Farm analysis", "شیکاریی کێڵگە", None),
];

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Jobs::Table)
                    .if_not_exists()
                    .col(string_len(Jobs::Job, 40).primary_key())
                    .col(string(Jobs::NameEn))
                    .col(string_null(Jobs::NameKu))
                    .col(integer_null(Jobs::EveryHours))
                    .to_owned(),
            )
            .await?;

        let mut seed = Query::insert()
            .into_table(Jobs::Table)
            .columns([Jobs::Job, Jobs::NameEn, Jobs::NameKu, Jobs::EveryHours])
            .on_conflict(OnConflict::column(Jobs::Job).do_nothing().to_owned())
            .to_owned();

        for (job, name_en, name_ku, every_hours) in JOBS {
            seed.values_panic([
                job.into(),
                name_en.into(),
                name_ku.into(),
                every_hours.into(),
            ]);
        }

        manager.exec_stmt(seed).await?;

        manager
            .create_table(
                Table::create()
                    .table(JobRuns::Table)
                    .if_not_exists()
                    .col(pk_auto(JobRuns::Id))
                    .col(string_len(JobRuns::Job, 40))
                    .col(timestamp(JobRuns::StartedAt))
                    .col(timestamp_null(JobRuns::FinishedAt))
                    .col(boolean_null(JobRuns::Ok))
                    .col(big_integer_null(JobRuns::Rows))
                    .col(string_len_null(JobRuns::Message, 500))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_job_runs_job")
                            .from(JobRuns::Table, JobRuns::Job)
                            .to(Jobs::Table, Jobs::Job)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // The job and the moment it started are the key of a run: a job that
        // reports the same run twice replaces its earlier report.
        manager
            .create_index(
                Index::create()
                    .name("idx_job_runs_job_started_at")
                    .table(JobRuns::Table)
                    .col(JobRuns::Job)
                    .col(JobRuns::StartedAt)
                    .unique()
                    .to_owned(),
            )
            .await?;

        let connection = manager.get_connection();

        for table in ["jobs", "job_runs"] {
            connection
                .execute_unprepared(&format!(
                    "DROP TRIGGER IF EXISTS bump_data_version ON {table};
                     CREATE TRIGGER bump_data_version
                       AFTER INSERT OR UPDATE OR DELETE ON {table}
                       FOR EACH STATEMENT EXECUTE FUNCTION bump_data_version('jobs')"
                ))
                .await?;
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(JobRuns::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Jobs::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum Jobs {
    Table,
    Job,
    NameEn,
    NameKu,
    EveryHours,
}

#[derive(DeriveIden)]
enum JobRuns {
    Table,
    Id,
    Job,
    StartedAt,
    FinishedAt,
    Ok,
    Rows,
    Message,
}
