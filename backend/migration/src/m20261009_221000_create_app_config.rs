use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

/// Both tables are the `app_config` topic.
// Only the settings raise the `app_config` topic. Version sightings are
// written by ordinary farmer requests and would raise this public topic
// all day; the versions page is read fresh each time it is opened.
const TABLES: [&str; 1] = ["app_config"];

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(AppConfig::Table)
                    .if_not_exists()
                    // There is one config. The key can only be 1, so a
                    // second row cannot exist.
                    .col(
                        small_integer(AppConfig::Id)
                            .primary_key()
                            .default(1)
                            .check(Expr::col(AppConfig::Id).eq(1)),
                    )
                    .col(string_len(AppConfig::LatestVersion, 32))
                    .col(string_len(AppConfig::MinVersion, 32))
                    .col(text_null(AppConfig::UpdateMessageKu))
                    .col(text_null(AppConfig::UpdateMessageEn))
                    .col(boolean(AppConfig::Maintenance).default(false))
                    .col(text_null(AppConfig::MaintenanceMessageKu))
                    .col(text_null(AppConfig::MaintenanceMessageEn))
                    .col(timestamp_null(AppConfig::MaintenanceFrom))
                    .col(timestamp_null(AppConfig::MaintenanceUntil))
                    .col(boolean(AppConfig::AnnouncementOn).default(false))
                    .col(text_null(AppConfig::AnnouncementKu))
                    .col(text_null(AppConfig::AnnouncementEn))
                    .col(boolean(AppConfig::FeatureAddFarm))
                    .col(boolean(AppConfig::FeatureWalkMode))
                    .col(boolean(AppConfig::FeatureSatellite))
                    .col(boolean(AppConfig::FeatureDoctor))
                    .col(boolean(AppConfig::FeatureReports))
                    .col(boolean(AppConfig::FeatureAlwa))
                    .col(boolean(AppConfig::FeaturePlan))
                    .col(boolean(AppConfig::FeaturePush))
                    .col(integer(AppConfig::LimitFarmsPerPhone))
                    .col(integer(AppConfig::LimitMaxFarmDunam))
                    .col(integer(AppConfig::LimitMinCorners))
                    .col(integer(AppConfig::LimitMaxCorners))
                    .col(integer(AppConfig::LimitGpsMeters))
                    .col(string_len_null(AppConfig::HelpPhone, 20))
                    .col(boolean(AppConfig::PublicFarmTotals))
                    // The staff member who saved it last, with no foreign
                    // key: staff belong to another feature.
                    .col(integer_null(AppConfig::UpdatedBy))
                    .col(
                        timestamp(AppConfig::UpdatedAt)
                            .not_null()
                            .default(SimpleExpr::Custom("CURRENT_TIMESTAMP".into())),
                    )
                    .to_owned(),
            )
            .await?;

        let connection = manager.get_connection();

        // The one row, with what is true today:
        // - both versions 1.0.0, no maintenance, no announcement;
        // - `reports`, `plan` and `push` off, because the backend has no
        //   reports route, no weather plan route and no push yet; every
        //   other switch on;
        // - the limits the backend enforces: 20 farms per phone
        //   (FARMS__MAX_FARMS_PER_USER), 3 to 50 corners (the outline rule),
        //   and 2,000 dunam, which is the 50,000 cell limit
        //   (FARMS__MAX_CELLS_PER_FARM) at 100 m2 a cell and 2,500 m2 a
        //   dunam; 10 m of GPS accuracy is the app's own setting;
        // - no help phone, and the public farm totals on.
        connection
            .execute_unprepared(
                "INSERT INTO app_config (
                   id, latest_version, min_version, maintenance, announcement_on,
                   feature_add_farm, feature_walk_mode, feature_satellite, feature_doctor,
                   feature_reports, feature_alwa, feature_plan, feature_push,
                   limit_farms_per_phone, limit_max_farm_dunam, limit_min_corners,
                   limit_max_corners, limit_gps_meters, public_farm_totals
                 ) VALUES (
                   1, '1.0.0', '1.0.0', FALSE, FALSE,
                   TRUE, TRUE, TRUE, TRUE,
                   FALSE, TRUE, FALSE, FALSE,
                   20, 2000, 3,
                   50, 10, TRUE
                 ) ON CONFLICT DO NOTHING",
            )
            .await?;

        // Which farmer was last seen on which version of the app. The
        // farmer's id, never the phone, and no foreign key: farmers belong
        // to another feature.
        manager
            .create_table(
                Table::create()
                    .table(AppVersionsSeen::Table)
                    .if_not_exists()
                    .col(integer(AppVersionsSeen::FarmerId))
                    .col(string_len(AppVersionsSeen::Version, 32))
                    .col(timestamp(AppVersionsSeen::LastSeen))
                    .primary_key(
                        Index::create()
                            .col(AppVersionsSeen::FarmerId)
                            .col(AppVersionsSeen::Version),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_app_versions_seen_last_seen")
                    .table(AppVersionsSeen::Table)
                    .col(AppVersionsSeen::LastSeen)
                    .to_owned(),
            )
            .await?;

        for table in TABLES {
            connection
                .execute_unprepared(&format!(
                    "DROP TRIGGER IF EXISTS bump_data_version ON {table};
                     CREATE TRIGGER bump_data_version
                       AFTER INSERT OR UPDATE OR DELETE ON {table}
                       FOR EACH STATEMENT EXECUTE FUNCTION bump_data_version('app_config')"
                ))
                .await?;
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(AppVersionsSeen::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(AppConfig::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum AppConfig {
    Table,
    Id,
    LatestVersion,
    MinVersion,
    UpdateMessageKu,
    UpdateMessageEn,
    Maintenance,
    MaintenanceMessageKu,
    MaintenanceMessageEn,
    MaintenanceFrom,
    MaintenanceUntil,
    AnnouncementOn,
    AnnouncementKu,
    AnnouncementEn,
    FeatureAddFarm,
    FeatureWalkMode,
    FeatureSatellite,
    FeatureDoctor,
    FeatureReports,
    FeatureAlwa,
    FeaturePlan,
    FeaturePush,
    LimitFarmsPerPhone,
    LimitMaxFarmDunam,
    LimitMinCorners,
    LimitMaxCorners,
    LimitGpsMeters,
    HelpPhone,
    PublicFarmTotals,
    UpdatedBy,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum AppVersionsSeen {
    Table,
    FarmerId,
    Version,
    LastSeen,
}
