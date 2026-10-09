use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // A letter is a record of what was issued. It carries no foreign
        // key to the farmer, so it outlives a farmer who is removed, and
        // none to staff, who belong to another feature.
        manager
            .create_table(
                Table::create()
                    .table(Letters::Table)
                    .if_not_exists()
                    .col(pk_auto(Letters::Id))
                    .col(string_len_uniq(Letters::Number, 40))
                    .col(integer(Letters::FarmerId))
                    .col(integer(Letters::StaffId))
                    .col(string_len(Letters::Purpose, 300))
                    .col(string_len(Letters::Lang, 5))
                    .col(
                        timestamp(Letters::CreatedAt)
                            .not_null()
                            .default(SimpleExpr::Custom("CURRENT_TIMESTAMP".into())),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_letters_farmer_id")
                    .table(Letters::Table)
                    .col(Letters::FarmerId)
                    .to_owned(),
            )
            .await?;

        // Letters are personal data of farmers: a new one raises that topic.
        manager
            .get_connection()
            .execute_unprepared(
                "CREATE TRIGGER bump_data_version
                   AFTER INSERT OR UPDATE OR DELETE ON letters
                   FOR EACH STATEMENT EXECUTE FUNCTION bump_data_version('farmers')",
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Letters::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Letters {
    Table,
    Id,
    Number,
    FarmerId,
    StaffId,
    Purpose,
    Lang,
    CreatedAt,
}
