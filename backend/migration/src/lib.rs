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
            Box::new(m20261009_030000_add_sign_in_challenge_used_at::Migration),
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
mod m20261009_030000_add_sign_in_challenge_used_at;
