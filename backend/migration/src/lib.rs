pub use sea_orm_migration::prelude::*;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20261008_200000_create_farms::Migration),
            Box::new(m20261008_210000_create_farmers::Migration),
            Box::new(m20261008_220000_create_zones::Migration),
            Box::new(m20261008_221000_create_dams::Migration),
            Box::new(m20261008_222000_create_outlooks::Migration),
            Box::new(m20261008_223000_create_water_plans::Migration),
            Box::new(m20261008_224000_create_fires::Migration),
            Box::new(m20261008_225000_create_farm_insights::Migration),
            Box::new(m20261008_226000_create_alwa::Migration),
            Box::new(m20261009_010000_add_alwa_listing_idempotency_key::Migration),
            Box::new(m20261009_020000_create_staff::Migration),
            Box::new(m20261009_030000_add_sign_in_challenge_used_at::Migration),
            Box::new(m20261009_140000_alwa_dashboard::Migration),
            Box::new(m20261009_160000_create_briefs::Migration),
            Box::new(m20261009_170000_add_farm_cell_inside_pct::Migration),
            Box::new(m20261009_180000_create_data_versions::Migration),
            Box::new(m20261009_190000_add_sub_zone_shapes::Migration),
            Box::new(m20261009_191000_add_farm_place::Migration),
            Box::new(m20261009_200000_add_farmer_details::Migration),
            Box::new(m20261009_201000_create_letters::Migration),
            Box::new(m20261009_202000_add_staff_details::Migration),
            Box::new(m20261009_210000_create_rules::Migration),
            Box::new(m20261009_211000_create_jobs::Migration),
            Box::new(m20261009_220000_create_messages::Migration),
            Box::new(m20261009_221000_create_app_config::Migration),
            Box::new(m20261009_230000_create_farm_history::Migration),
            Box::new(m20261009_240000_create_crops::Migration),
            Box::new(m20261009_250000_create_farm_plans::Migration),
            Box::new(m20261009_260000_create_alerts::Migration),
        ]
    }
}

mod m20261008_200000_create_farms;
mod m20261008_210000_create_farmers;
mod m20261008_220000_create_zones;
mod m20261008_221000_create_dams;
mod m20261008_222000_create_outlooks;
mod m20261008_223000_create_water_plans;
mod m20261008_224000_create_fires;
mod m20261008_225000_create_farm_insights;
mod m20261008_226000_create_alwa;
mod m20261009_010000_add_alwa_listing_idempotency_key;
mod m20261009_020000_create_staff;
mod m20261009_030000_add_sign_in_challenge_used_at;
mod m20261009_140000_alwa_dashboard;
mod m20261009_160000_create_briefs;
mod m20261009_170000_add_farm_cell_inside_pct;
mod m20261009_180000_create_data_versions;
mod m20261009_190000_add_sub_zone_shapes;
mod m20261009_191000_add_farm_place;
mod m20261009_200000_add_farmer_details;
mod m20261009_201000_create_letters;
mod m20261009_202000_add_staff_details;
mod m20261009_210000_create_rules;
mod m20261009_211000_create_jobs;
mod sub_zone_shapes;
mod m20261009_220000_create_messages;
mod m20261009_221000_create_app_config;
mod m20261009_240000_create_crops;
mod m20261009_230000_create_farm_history;
mod m20261009_250000_create_farm_plans;
mod m20261009_260000_create_alerts;
