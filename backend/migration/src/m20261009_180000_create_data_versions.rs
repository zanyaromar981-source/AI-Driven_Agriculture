use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

/// Every topic a website caches as one piece.
const TOPICS: [&str; 17] = [
    "zones",
    "sub_zones",
    "dams",
    "fires",
    "outlooks",
    "water",
    "alwa_prices",
    "alwa_listings",
    "crops",
    "rules",
    "briefs",
    "app_config",
    "farmers",
    "farms",
    "messages",
    "jobs",
    "staff_roles",
];

/// Which topic each existing table belongs to. A write to the table raises
/// the topic's version. Tables added later attach their own trigger in their
/// own migration, with the same function:
///
/// ```sql
/// CREATE TRIGGER bump_data_version AFTER INSERT OR UPDATE OR DELETE ON my_table
///   FOR EACH STATEMENT EXECUTE FUNCTION bump_data_version('my_topic');
/// ```
const TABLES: [(&str, &str); 19] = [
    ("zones", "zones"),
    ("zone_readings", "zones"),
    ("sub_zones", "sub_zones"),
    ("sub_zone_readings", "sub_zones"),
    ("dams", "dams"),
    ("dam_readings", "dams"),
    ("fires", "fires"),
    ("season_outlooks", "outlooks"),
    ("outlook_runs", "outlooks"),
    ("water_plan_entries", "water"),
    ("alwa_markets", "alwa_prices"),
    ("alwa_prices", "alwa_prices"),
    ("alwa_listings", "alwa_listings"),
    ("alwa_offers", "alwa_listings"),
    ("daily_briefs", "briefs"),
    ("farmers", "farmers"),
    ("farms", "farms"),
    ("farm_cells", "farms"),
    ("farm_insights", "farms"),
];

/// Staff and role tables, all one topic.
const STAFF_TABLES: [&str; 4] = ["staff", "roles", "role_permissions", "staff_roles"];

/// The resources added for the website, each with the four actions, granted
/// to the role that holds everything.
const NEW_RESOURCES: [&str; 5] = ["crops", "rules", "messages", "app", "jobs"];
const ACTIONS: [&str; 4] = ["create", "read", "update", "delete"];

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(DataVersions::Table)
                    .if_not_exists()
                    .col(string_len(DataVersions::Topic, 40).primary_key())
                    .col(big_integer(DataVersions::Version).default(1))
                    .col(
                        timestamp(DataVersions::ChangedAt)
                            .not_null()
                            .default(SimpleExpr::Custom("CURRENT_TIMESTAMP".into())),
                    )
                    .to_owned(),
            )
            .await?;

        let connection = manager.get_connection();

        for topic in TOPICS {
            connection
                .execute_unprepared(&format!(
                    "INSERT INTO data_versions (topic) VALUES ('{topic}') ON CONFLICT DO NOTHING"
                ))
                .await?;
        }

        // The database raises the version itself, inside the transaction of
        // the write, so no code path can change data and forget to say so.
        // It fires once per statement, not once per row.
        connection
            .execute_unprepared(
                "CREATE OR REPLACE FUNCTION bump_data_version() RETURNS trigger AS $$
                 BEGIN
                   UPDATE data_versions
                      SET version = version + 1, changed_at = CURRENT_TIMESTAMP
                    WHERE topic = TG_ARGV[0];
                   RETURN NULL;
                 END
                 $$ LANGUAGE plpgsql",
            )
            .await?;

        let staff = STAFF_TABLES.iter().map(|table| (*table, "staff_roles"));

        for (table, topic) in TABLES.into_iter().chain(staff) {
            connection
                .execute_unprepared(&format!(
                    "DROP TRIGGER IF EXISTS bump_data_version ON {table};
                     CREATE TRIGGER bump_data_version
                       AFTER INSERT OR UPDATE OR DELETE ON {table}
                       FOR EACH STATEMENT EXECUTE FUNCTION bump_data_version('{topic}')"
                ))
                .await?;
        }

        for resource in NEW_RESOURCES {
            for action in ACTIONS {
                connection
                    .execute_unprepared(&format!(
                        "INSERT INTO role_permissions (role_id, resource, action)
                         SELECT id, '{resource}', '{action}' FROM roles WHERE system
                         ON CONFLICT DO NOTHING"
                    ))
                    .await?;
            }
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let connection = manager.get_connection();
        let staff = STAFF_TABLES.iter().map(|table| (*table, "staff_roles"));

        for (table, _) in TABLES.into_iter().chain(staff) {
            connection
                .execute_unprepared(&format!(
                    "DROP TRIGGER IF EXISTS bump_data_version ON {table}"
                ))
                .await?;
        }

        connection
            .execute_unprepared("DROP FUNCTION IF EXISTS bump_data_version()")
            .await?;

        for resource in NEW_RESOURCES {
            connection
                .execute_unprepared(&format!(
                    "DELETE FROM role_permissions WHERE resource = '{resource}'"
                ))
                .await?;
        }

        manager
            .drop_table(Table::drop().table(DataVersions::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum DataVersions {
    Table,
    Topic,
    Version,
    ChangedAt,
}
