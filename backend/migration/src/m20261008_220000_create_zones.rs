use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

/// One zone of the reference list: its English name, its Sorani name, its
/// governorate and its sub-zones as `(English name, Sorani name)`.
struct ZoneSeed {
    name: &'static str,
    ku: &'static str,
    governorate: &'static str,
    sub_zones: &'static [(&'static str, &'static str)],
}

/// The 33 districts of the Kurdistan Region in its four governorates, with
/// their 72 sub-districts, as drawn on the team's map
/// (`web/map_demo/kri_map_data.js`). This is reference data, not
/// measurements: readings only ever arrive through the ingest routes.
const ZONES: [ZoneSeed; 33] = [
    ZoneSeed {
        name: "Akre",
        ku: "ئاکرێ",
        governorate: "Duhok",
        sub_zones: &[
            ("Begeel", "بجیل"),
            ("Kurdsein", "گردەسێن"),
            ("Dinarta", "دینارتێ"),
            ("Markaz Aqra", "ناوەندی ئاکرێ"),
        ],
    },
    ZoneSeed {
        name: "Amedi",
        ku: "ئامێدی",
        governorate: "Duhok",
        sub_zones: &[
            ("Barwari Bala", "کانی ماسێ"),
            ("Markaz Al-Amadiya", "ناوەندی ئامێدی"),
            ("Nerwa Rekan", "دێرەلووک و شیلادزێ"),
            ("Sarsank", "سەرسەنگ"),
        ],
    },
    ZoneSeed {
        name: "Bardarash",
        ku: "بەردەڕەش",
        governorate: "Duhok",
        sub_zones: &[("Bardarash", "بەردەڕەش")],
    },
    ZoneSeed {
        name: "Duhok",
        ku: "دهۆک",
        governorate: "Duhok",
        sub_zones: &[
            ("Al-Duski", "مانگێشک"),
            ("Markaz Duhok", "ناوەندی دهۆک"),
            ("Zawita", "زاویتە"),
        ],
    },
    ZoneSeed {
        name: "Shekhan",
        ku: "شێخان",
        governorate: "Duhok",
        sub_zones: &[
            ("Markaz Al-Shikhan", "ناوەندی شێخان"),
            ("Qasruk", "قەسرۆک"),
            ("Atreesh", "ئەترووش"),
        ],
    },
    ZoneSeed {
        name: "Sumel",
        ku: "سێمێل",
        governorate: "Duhok",
        sub_zones: &[
            ("Bateel", "باتێل"),
            ("Fayde", "فایدە"),
            ("Markaz Sumail", "ناوەندی سێمێل"),
        ],
    },
    ZoneSeed {
        name: "Zakho",
        ku: "زاخۆ",
        governorate: "Duhok",
        sub_zones: &[
            ("Sindi", "ڕزگاری"),
            ("Markaz Zakho", "ناوەندی زاخۆ"),
            ("Batifa", "باتیفا"),
            ("Dercar", "دەرکار"),
        ],
    },
    ZoneSeed {
        name: "Chuman",
        ku: "چۆمان",
        governorate: "Erbil",
        sub_zones: &[("Balak", "باڵەکایەتی"), ("Haji Omaran", "حاجی ئۆمەران")],
    },
    ZoneSeed {
        name: "Erbil",
        ku: "هەولێر",
        governorate: "Erbil",
        sub_zones: &[("Markaz Erbil", "ناوەندی هەولێر")],
    },
    ZoneSeed {
        name: "Harir",
        ku: "هەریر",
        governorate: "Erbil",
        sub_zones: &[("Harir", "هەریر")],
    },
    ZoneSeed {
        name: "Khalifan",
        ku: "خەلیفان",
        governorate: "Erbil",
        sub_zones: &[("Khailfan", "خەلیفان")],
    },
    ZoneSeed {
        name: "Koya",
        ku: "کۆیە",
        governorate: "Erbil",
        sub_zones: &[("Shorsh", "شۆڕش"), ("Markaz Koysinjaq", "ناوەندی کۆیە")],
    },
    ZoneSeed {
        name: "Makhmur",
        ku: "مەخموور",
        governorate: "Erbil",
        sub_zones: &[
            ("Qaraj", "قەراج"),
            ("Markaz Makhmour", "ناوەندی مەخموور"),
            ("Dibaga", "دیبەگە"),
            ("Gwyer", "گوێر"),
        ],
    },
    ZoneSeed {
        name: "Mergasor",
        ku: "مێرگەسۆر",
        governorate: "Erbil",
        sub_zones: &[
            ("Barzan", "بارزان"),
            ("Mazouri Bala", "مزووری باڵا"),
            ("Mergasur", "ناوەندی مێرگەسۆر"),
        ],
    },
    ZoneSeed {
        name: "Pirmam",
        ku: "پیرمام",
        governorate: "Erbil",
        sub_zones: &[("Salah Al-Din", "سەڵاحەدین")],
    },
    ZoneSeed {
        name: "Qushtapa",
        ku: "قوشتەپە",
        governorate: "Erbil",
        sub_zones: &[("Qushtappa", "قوشتەپە")],
    },
    ZoneSeed {
        name: "Rawanduz",
        ku: "ڕەواندز",
        governorate: "Erbil",
        sub_zones: &[("Markaz Rawanduz", "ناوەندی ڕەواندز")],
    },
    ZoneSeed {
        name: "Shaqlawa",
        ku: "شەقڵاوە",
        governorate: "Erbil",
        sub_zones: &[("Khoshnaw", "خۆشناو")],
    },
    ZoneSeed {
        name: "Sidakan",
        ku: "سیدەکان",
        governorate: "Erbil",
        sub_zones: &[("Bradost", "برادۆست")],
    },
    ZoneSeed {
        name: "Soran",
        ku: "سۆران",
        governorate: "Erbil",
        sub_zones: &[("Diana", "دیانا")],
    },
    ZoneSeed {
        name: "Taqtaq",
        ku: "تەقتەق",
        governorate: "Erbil",
        sub_zones: &[("Taq Taq", "تەقتەق")],
    },
    ZoneSeed {
        name: "Chamchamal",
        ku: "چەمچەماڵ",
        governorate: "Sulaymaniyah",
        sub_zones: &[
            ("Agjalare", "ئاغجەلەر"),
            ("Markaz Chamchamal", "ناوەندی چەمچەماڵ"),
            ("Kadr Karam", "قادرکەرەم"),
            ("Cenkaw", "سەنگاو"),
        ],
    },
    ZoneSeed {
        name: "Darbandikhan",
        ku: "دەربەندیخان",
        governorate: "Sulaymaniyah",
        sub_zones: &[("Markaz Derbendikhan", "ناوەندی دەربەندیخان")],
    },
    ZoneSeed {
        name: "Dukan",
        ku: "دوکان",
        governorate: "Sulaymaniyah",
        sub_zones: &[("Gnareen", "None"), ("Sourdash", "سورداش")],
    },
    ZoneSeed {
        name: "Kalar",
        ku: "کەلار",
        governorate: "Sulaymaniyah",
        sub_zones: &[
            ("Markaz Kalar", "ناوەندی کەلار"),
            ("Bebaz", "None"),
            ("Tilako", "تیلەکۆ"),
        ],
    },
    ZoneSeed {
        name: "Penjwen",
        ku: "پێنجوێن",
        governorate: "Sulaymaniyah",
        sub_zones: &[("Karmak", "None"), ("Markaz Panjwin", "ناوەندی پێنجوێن")],
    },
    ZoneSeed {
        name: "Pshdar",
        ku: "پشدەر",
        governorate: "Sulaymaniyah",
        sub_zones: &[
            ("Hero", "هێرۆ"),
            ("Nawdasht", "ناوداشت"),
            ("Markaz Pshdar", "قەڵادزێ"),
        ],
    },
    ZoneSeed {
        name: "Ranya",
        ku: "ڕانیە",
        governorate: "Sulaymaniyah",
        sub_zones: &[
            ("Mirka", "None"),
            ("Markaz Rania", "ناوەندی ڕانیە"),
            ("Bitwana", "بتوێن"),
        ],
    },
    ZoneSeed {
        name: "Sharazur",
        ku: "شارەزوور",
        governorate: "Sulaymaniyah",
        sub_zones: &[("Shahrazur", "زەڕایەن")],
    },
    ZoneSeed {
        name: "Sharbazher",
        ku: "شارباژێڕ",
        governorate: "Sulaymaniyah",
        sub_zones: &[
            ("Saruchik", "None"),
            ("Markaz Sharbazher", "چوارتا"),
            ("Siwail", "سیوەیل"),
            ("Mawat", "ماوەت"),
        ],
    },
    ZoneSeed {
        name: "Sulaymaniyah",
        ku: "سلێمانی",
        governorate: "Sulaymaniyah",
        sub_zones: &[
            ("Bazian", "بازیان"),
            ("Qaradagh", "قەرەداغ"),
            ("Sarchnar", "سەرچنار"),
        ],
    },
    ZoneSeed {
        name: "Halabja",
        ku: "هەڵەبجە",
        governorate: "Halabja",
        sub_zones: &[("Beyara", "بیارە"), ("Markaz Halabja", "هەڵەبجە و سیروان")],
    },
    ZoneSeed {
        name: "Khurmal",
        ku: "خورماڵ",
        governorate: "Halabja",
        sub_zones: &[("Khourmal", "خورماڵ")],
    },
];

/// Lower-case name with hyphens for spaces: `Markaz Zakho` is
/// `markaz-zakho`.
fn slug(name: &str) -> String {
    name.to_lowercase().replace(' ', "-")
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Zones::Table)
                    .if_not_exists()
                    .col(pk_auto(Zones::Id))
                    .col(string_len_uniq(Zones::Slug, 60))
                    .col(string_len(Zones::NameEn, 100))
                    .col(string_len(Zones::NameKu, 100))
                    .col(string_len(Zones::Governorate, 100))
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(SubZones::Table)
                    .if_not_exists()
                    .col(pk_auto(SubZones::Id))
                    .col(integer(SubZones::ZoneId))
                    .col(string_len(SubZones::Slug, 60))
                    .col(string_len(SubZones::NameEn, 100))
                    .col(string_len(SubZones::NameKu, 100))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_sub_zones_zone_id")
                            .from(SubZones::Table, SubZones::ZoneId)
                            .to(Zones::Table, Zones::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_sub_zones_zone_id_slug")
                    .table(SubZones::Table)
                    .col(SubZones::ZoneId)
                    .col(SubZones::Slug)
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(ZoneReadings::Table)
                    .if_not_exists()
                    .col(pk_auto(ZoneReadings::Id))
                    .col(integer(ZoneReadings::ZoneId))
                    .col(date(ZoneReadings::Month))
                    .col(integer(ZoneReadings::Dryness))
                    .col(double_null(ZoneReadings::RainPctOfNormal))
                    .col(double_null(ZoneReadings::GreennessPctVsNormal))
                    .col(integer_null(ZoneReadings::WaterNeed))
                    .col(boolean(ZoneReadings::NitrogenHold))
                    .col(json_binary(ZoneReadings::BestCrops))
                    .col(string_len(ZoneReadings::Source, 100))
                    .col(
                        timestamp(ZoneReadings::UpdatedAt)
                            .not_null()
                            .default(SimpleExpr::Custom("CURRENT_TIMESTAMP".into())),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_zone_readings_zone_id")
                            .from(ZoneReadings::Table, ZoneReadings::ZoneId)
                            .to(Zones::Table, Zones::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_zone_readings_zone_id_month")
                    .table(ZoneReadings::Table)
                    .col(ZoneReadings::ZoneId)
                    .col(ZoneReadings::Month)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // The overview and the comparison load whole months.
        manager
            .create_index(
                Index::create()
                    .name("idx_zone_readings_month")
                    .table(ZoneReadings::Table)
                    .col(ZoneReadings::Month)
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(SubZoneReadings::Table)
                    .if_not_exists()
                    .col(pk_auto(SubZoneReadings::Id))
                    .col(integer(SubZoneReadings::SubZoneId))
                    .col(date(SubZoneReadings::Month))
                    .col(integer(SubZoneReadings::Dryness))
                    .col(
                        timestamp(SubZoneReadings::UpdatedAt)
                            .not_null()
                            .default(SimpleExpr::Custom("CURRENT_TIMESTAMP".into())),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_sub_zone_readings_sub_zone_id")
                            .from(SubZoneReadings::Table, SubZoneReadings::SubZoneId)
                            .to(SubZones::Table, SubZones::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_sub_zone_readings_sub_zone_id_month")
                    .table(SubZoneReadings::Table)
                    .col(SubZoneReadings::SubZoneId)
                    .col(SubZoneReadings::Month)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // Zone ids are given here, 1 to 30 in list order, so the sub-zones
        // can point at them and the dashboard's north to south order is the
        // id order.
        let mut zones = Query::insert()
            .into_table(Zones::Table)
            .columns([
                Zones::Id,
                Zones::Slug,
                Zones::NameEn,
                Zones::NameKu,
                Zones::Governorate,
            ])
            .to_owned();

        let mut sub_zones = Query::insert()
            .into_table(SubZones::Table)
            .columns([
                SubZones::ZoneId,
                SubZones::Slug,
                SubZones::NameEn,
                SubZones::NameKu,
            ])
            .to_owned();

        for (zone, zone_id) in ZONES.iter().zip(1i32..) {
            zones.values_panic([
                zone_id.into(),
                slug(zone.name).into(),
                zone.name.into(),
                zone.ku.into(),
                zone.governorate.into(),
            ]);

            for (name, ku) in zone.sub_zones {
                sub_zones.values_panic([
                    zone_id.into(),
                    slug(name).into(),
                    (*name).into(),
                    (*ku).into(),
                ]);
            }
        }

        manager.exec_stmt(zones).await?;
        manager.exec_stmt(sub_zones).await?;

        // The ids were written by hand, so the sequence has to be moved past
        // them or the next zone added later would collide with id 1.
        manager
            .get_connection()
            .execute_unprepared(
                "SELECT setval(pg_get_serial_sequence('zones', 'id'), (SELECT MAX(id) FROM zones))",
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(SubZoneReadings::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(ZoneReadings::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(SubZones::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Zones::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum Zones {
    Table,
    Id,
    Slug,
    NameEn,
    NameKu,
    Governorate,
}

#[derive(DeriveIden)]
enum SubZones {
    Table,
    Id,
    ZoneId,
    Slug,
    NameEn,
    NameKu,
}

#[derive(DeriveIden)]
enum ZoneReadings {
    Table,
    Id,
    ZoneId,
    Month,
    Dryness,
    RainPctOfNormal,
    GreennessPctVsNormal,
    WaterNeed,
    NitrogenHold,
    BestCrops,
    Source,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum SubZoneReadings {
    Table,
    Id,
    SubZoneId,
    Month,
    Dryness,
    UpdatedAt,
}
