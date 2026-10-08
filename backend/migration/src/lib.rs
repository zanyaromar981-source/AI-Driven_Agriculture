pub use sea_orm_migration::prelude::*;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20261008_200000_create_farms::Migration),
            Box::new(m20261008_210000_create_farmers::Migration),
            Box::new(m20261008_224000_create_fires::Migration),
            Box::new(m20261008_225000_create_farm_insights::Migration),
        ]
    }
}

mod m20261008_200000_create_farms;
mod m20261008_210000_create_farmers;
mod m20261008_224000_create_fires;
mod m20261008_225000_create_farm_insights;
