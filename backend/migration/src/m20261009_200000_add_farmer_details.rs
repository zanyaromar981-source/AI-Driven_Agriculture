use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Everything is optional or has a default, so the farmers already
        // stored stay valid: nothing is known about them and none is blocked.
        manager
            .alter_table(
                Table::alter()
                    .table(Farmers::Table)
                    .add_column(string_len_null(Farmers::Gender, 10))
                    .add_column(integer_null(Farmers::BirthYear))
                    .add_column(string_len_null(Farmers::Village, 80))
                    .add_column(string_len_null(Farmers::Governorate, 40))
                    .add_column(string_len_null(Farmers::ZoneSlug, 40))
                    .add_column(string_len_null(Farmers::SubZoneSlug, 40))
                    .add_column(string_len_null(Farmers::Notes, 1000))
                    .add_column(boolean(Farmers::Blocked).default(false))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Farmers::Table)
                    .drop_column(Farmers::Gender)
                    .drop_column(Farmers::BirthYear)
                    .drop_column(Farmers::Village)
                    .drop_column(Farmers::Governorate)
                    .drop_column(Farmers::ZoneSlug)
                    .drop_column(Farmers::SubZoneSlug)
                    .drop_column(Farmers::Notes)
                    .drop_column(Farmers::Blocked)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum Farmers {
    Table,
    Gender,
    BirthYear,
    Village,
    Governorate,
    ZoneSlug,
    SubZoneSlug,
    Notes,
    Blocked,
}
