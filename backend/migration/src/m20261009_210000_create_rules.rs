use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

/// One number a data job uses to decide a warning or a colour.
struct Seed {
    code: &'static str,
    grp: &'static str,
    name_en: &'static str,
    meaning_en: &'static str,
    value: f64,
    unit: &'static str,
    min_value: f64,
    max_value: f64,
    used_by: &'static str,
}

/// The numbers in use on the day this was written, each read from the code
/// that uses it: the weather planner (`farm_doctor/weather_planner.py`), the
/// dryness bands (`zones/domain/enums.rs`) and Field Eye
/// (`farm_doctor/field_eye.py`). The value is also the default a reset goes
/// back to. The allowed range is a guard against a slip of the hand, not a
/// measured limit. The Sorani columns are left empty until a speaker writes
/// them: a wrong name is worse than a missing one.
const RULES: [Seed; 15] = [
    Seed {
        code: "frost_c",
        grp: "temperature",
        name_en: "Frost night",
        meaning_en: "A night is a frost night when its lowest temperature is at or below this.",
        value: 0.0,
        unit: "c",
        min_value: -1.0,
        max_value: 5.0,
        used_by: "weather_planner",
    },
    Seed {
        code: "hard_frost_c",
        grp: "temperature",
        name_en: "Hard frost night",
        meaning_en: "A night is a hard frost night when its lowest temperature is at or below this.",
        value: -2.0,
        unit: "c",
        min_value: -10.0,
        max_value: -1.0,
        used_by: "weather_planner",
    },
    Seed {
        code: "heat_c",
        grp: "temperature",
        name_en: "Heat day",
        meaning_en: "A day is a heat day when its highest temperature is at or above this.",
        value: 31.0,
        unit: "c",
        min_value: 25.0,
        max_value: 45.0,
        used_by: "weather_planner",
    },
    Seed {
        code: "heavy_rain_mm",
        grp: "rain",
        name_en: "Heavy rain day",
        meaning_en: "A day is a heavy rain day when its rain is at or above this.",
        value: 12.0,
        unit: "mm",
        min_value: 5.0,
        max_value: 50.0,
        used_by: "weather_planner",
    },
    Seed {
        code: "sowing_rain_mm",
        grp: "rain",
        name_en: "Sowing rain",
        meaning_en: "The sowing window opens when three days in a row bring at least this much rain together.",
        value: 20.0,
        unit: "mm",
        min_value: 5.0,
        max_value: 60.0,
        used_by: "weather_planner",
    },
    Seed {
        code: "rust_weather_hours",
        grp: "disease",
        name_en: "Rust weather",
        meaning_en: "The risk of rust is high when at least this many forecast hours are cool and humid (6 to 16 C, humidity 90% or more).",
        value: 24.0,
        unit: "h",
        min_value: 9.0,
        max_value: 96.0,
        used_by: "weather_planner",
    },
    Seed {
        code: "spray_window_hours",
        grp: "spraying",
        name_en: "Spray window",
        meaning_en: "A spray window is this many daytime hours in a row with no rain, 15 to 24 C and wind under 15 km/h.",
        value: 6.0,
        unit: "h",
        min_value: 2.0,
        max_value: 12.0,
        used_by: "weather_planner",
    },
    Seed {
        code: "sunn_pest_degree_days",
        grp: "pests",
        name_en: "Sunn pest nymphs",
        meaning_en: "Sunn pest nymphs are expected once the degree-days since 1 January (base 13.3 C) reach this.",
        value: 84.0,
        unit: "degree_days",
        min_value: 40.0,
        max_value: 222.0,
        used_by: "weather_planner",
    },
    Seed {
        code: "dust_pm10",
        grp: "air",
        name_en: "Dust alert",
        meaning_en: "A dust alert is raised when the highest PM10 forecast for the next five days is at or above this.",
        value: 150.0,
        unit: "ug_m3",
        min_value: 50.0,
        max_value: 500.0,
        used_by: "weather_planner",
    },
    Seed {
        code: "dryness_greener_from",
        grp: "bands",
        name_en: "Greener starts at",
        meaning_en: "A dryness index below this is much greener than normal; from this up to the next edge it is greener.",
        value: 25.0,
        unit: "index",
        min_value: 5.0,
        max_value: 40.0,
        used_by: "dryness",
    },
    Seed {
        code: "dryness_normal_from",
        grp: "bands",
        name_en: "Normal starts at",
        meaning_en: "A dryness index from this up to the next edge is normal.",
        value: 45.0,
        unit: "index",
        min_value: 41.0,
        max_value: 55.0,
        used_by: "dryness",
    },
    Seed {
        code: "dryness_dry_from",
        grp: "bands",
        name_en: "Dry starts at",
        meaning_en: "A dryness index from this up to the next edge is dry.",
        value: 60.0,
        unit: "index",
        min_value: 56.0,
        max_value: 75.0,
        used_by: "dryness",
    },
    Seed {
        code: "dryness_very_dry_from",
        grp: "bands",
        name_en: "Very dry starts at",
        meaning_en: "A dryness index at or above this is very dry.",
        value: 80.0,
        unit: "index",
        min_value: 76.0,
        max_value: 95.0,
        used_by: "dryness",
    },
    Seed {
        code: "field_eye_max_cloud_pct",
        grp: "pictures",
        name_en: "Cloud limit",
        meaning_en: "A satellite picture is skipped when its cloud cover is at or above this.",
        value: 60.0,
        unit: "pct",
        min_value: 10.0,
        max_value: 90.0,
        used_by: "field_eye",
    },
    Seed {
        code: "field_eye_weak_pixel_pct",
        grp: "greenness",
        name_en: "Weak spot",
        meaning_en: "A spot in a field is weak when its greenness is below this share of the field's own median.",
        value: 70.0,
        unit: "pct",
        min_value: 40.0,
        max_value: 95.0,
        used_by: "field_eye",
    },
];

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Rules::Table)
                    .if_not_exists()
                    .col(string_len(Rules::Code, 60).primary_key())
                    .col(string_len(Rules::Grp, 40))
                    .col(string(Rules::NameEn))
                    .col(string_null(Rules::NameKu))
                    .col(text(Rules::MeaningEn))
                    .col(text_null(Rules::MeaningKu))
                    .col(double(Rules::Value))
                    .col(string_len(Rules::Unit, 20))
                    .col(double(Rules::MinValue))
                    .col(double(Rules::MaxValue))
                    .col(double(Rules::DefaultValue))
                    .col(string_len(Rules::UsedBy, 40))
                    .col(integer_null(Rules::UpdatedBy))
                    .col(
                        timestamp(Rules::UpdatedAt)
                            .not_null()
                            .default(SimpleExpr::Custom("CURRENT_TIMESTAMP".into())),
                    )
                    .to_owned(),
            )
            .await?;

        let mut seed = Query::insert()
            .into_table(Rules::Table)
            .columns([
                Rules::Code,
                Rules::Grp,
                Rules::NameEn,
                Rules::MeaningEn,
                Rules::Value,
                Rules::Unit,
                Rules::MinValue,
                Rules::MaxValue,
                Rules::DefaultValue,
                Rules::UsedBy,
            ])
            .on_conflict(OnConflict::column(Rules::Code).do_nothing().to_owned())
            .to_owned();

        for rule in RULES {
            seed.values_panic([
                rule.code.into(),
                rule.grp.into(),
                rule.name_en.into(),
                rule.meaning_en.into(),
                rule.value.into(),
                rule.unit.into(),
                rule.min_value.into(),
                rule.max_value.into(),
                rule.value.into(),
                rule.used_by.into(),
            ]);
        }

        manager.exec_stmt(seed).await?;

        manager
            .create_table(
                Table::create()
                    .table(RuleChanges::Table)
                    .if_not_exists()
                    .col(pk_auto(RuleChanges::Id))
                    .col(string_len(RuleChanges::Code, 60))
                    .col(double(RuleChanges::OldValue))
                    .col(double(RuleChanges::NewValue))
                    .col(string_len(RuleChanges::Reason, 500))
                    .col(integer(RuleChanges::StaffId))
                    .col(
                        timestamp(RuleChanges::At)
                            .not_null()
                            .default(SimpleExpr::Custom("CURRENT_TIMESTAMP".into())),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_rule_changes_code")
                            .from(RuleChanges::Table, RuleChanges::Code)
                            .to(Rules::Table, Rules::Code),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_rule_changes_code_id")
                    .table(RuleChanges::Table)
                    .col(RuleChanges::Code)
                    .col(RuleChanges::Id)
                    .to_owned(),
            )
            .await?;

        let connection = manager.get_connection();

        // The log is insert only, and the database holds that line itself:
        // no code path, present or future, can rewrite or drop a change.
        connection
            .execute_unprepared(
                "CREATE OR REPLACE FUNCTION rule_changes_insert_only() RETURNS trigger AS $$
                 BEGIN
                   RAISE EXCEPTION 'rule_changes is insert only';
                 END
                 $$ LANGUAGE plpgsql;
                 DROP TRIGGER IF EXISTS rule_changes_insert_only ON rule_changes;
                 CREATE TRIGGER rule_changes_insert_only
                   BEFORE UPDATE OR DELETE ON rule_changes
                   FOR EACH ROW EXECUTE FUNCTION rule_changes_insert_only()",
            )
            .await?;

        for table in ["rules", "rule_changes"] {
            connection
                .execute_unprepared(&format!(
                    "DROP TRIGGER IF EXISTS bump_data_version ON {table};
                     CREATE TRIGGER bump_data_version
                       AFTER INSERT OR UPDATE OR DELETE ON {table}
                       FOR EACH STATEMENT EXECUTE FUNCTION bump_data_version('rules')"
                ))
                .await?;
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(RuleChanges::Table).to_owned())
            .await?;
        manager
            .get_connection()
            .execute_unprepared("DROP FUNCTION IF EXISTS rule_changes_insert_only()")
            .await?;
        manager
            .drop_table(Table::drop().table(Rules::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum Rules {
    Table,
    Code,
    Grp,
    NameEn,
    NameKu,
    MeaningEn,
    MeaningKu,
    Value,
    Unit,
    MinValue,
    MaxValue,
    DefaultValue,
    UsedBy,
    UpdatedBy,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum RuleChanges {
    Table,
    Id,
    Code,
    OldValue,
    NewValue,
    Reason,
    StaffId,
    At,
}
