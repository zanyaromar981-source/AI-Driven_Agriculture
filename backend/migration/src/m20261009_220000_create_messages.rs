use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

/// Both tables are the `messages` topic: a write to either tells the
/// website that the inbox changed.
const TABLES: [&str; 2] = ["messages", "message_photos"];

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Messages::Table)
                    .if_not_exists()
                    .col(pk_auto(Messages::Id))
                    // The farmer's id and the farm's id, with no foreign key:
                    // both belong to other features, and a message stays for
                    // the record when its farmer or farm is removed.
                    .col(integer(Messages::FarmerId))
                    .col(integer_null(Messages::FarmId))
                    .col(
                        string_len(Messages::Kind, 20).check(Expr::col(Messages::Kind).is_in([
                            "question",
                            "report",
                            "complaint",
                            "request",
                            "other",
                        ])),
                    )
                    .col(text(Messages::Text))
                    .col(string_len(Messages::State, 20).default("new").check(
                        Expr::col(Messages::State).is_in(["new", "read", "replied", "closed"]),
                    ))
                    .col(text_null(Messages::ReplyTextKu))
                    .col(text_null(Messages::ReplyTextEn))
                    .col(integer_null(Messages::RepliedBy))
                    .col(timestamp_null(Messages::RepliedAt))
                    .col(string_len_null(Messages::IdempotencyKey, 128))
                    .col(
                        timestamp(Messages::CreatedAt)
                            .not_null()
                            .default(SimpleExpr::Custom("CURRENT_TIMESTAMP".into())),
                    )
                    .col(
                        timestamp(Messages::UpdatedAt)
                            .not_null()
                            .default(SimpleExpr::Custom("CURRENT_TIMESTAMP".into())),
                    )
                    .to_owned(),
            )
            .await?;

        // A farmer's own list, and the count of what they sent in the last
        // 24 hours.
        manager
            .create_index(
                Index::create()
                    .name("idx_messages_farmer_id_created_at")
                    .table(Messages::Table)
                    .col(Messages::FarmerId)
                    .col(Messages::CreatedAt)
                    .to_owned(),
            )
            .await?;

        // A retried send carries the key of the first: the index lets only
        // one of them in. Rows without a key never collide.
        manager
            .create_index(
                Index::create()
                    .name("idx_messages_farmer_id_idempotency_key")
                    .table(Messages::Table)
                    .col(Messages::FarmerId)
                    .col(Messages::IdempotencyKey)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // The staff inbox: newest first, often one state at a time.
        manager
            .create_index(
                Index::create()
                    .name("idx_messages_state_created_at")
                    .table(Messages::Table)
                    .col(Messages::State)
                    .col(Messages::CreatedAt)
                    .to_owned(),
            )
            .await?;

        // The photos live in the database, not on disk, so the server keeps
        // no files to lose. They are in their own table so that reading a
        // message never reads megabytes.
        manager
            .create_table(
                Table::create()
                    .table(MessagePhotos::Table)
                    .if_not_exists()
                    .col(pk_auto(MessagePhotos::Id))
                    .col(integer(MessagePhotos::MessageId))
                    .col(string_len(MessagePhotos::ContentType, 40))
                    .col(blob(MessagePhotos::Bytes))
                    .col(integer(MessagePhotos::Size))
                    .col(
                        timestamp(MessagePhotos::CreatedAt)
                            .not_null()
                            .default(SimpleExpr::Custom("CURRENT_TIMESTAMP".into())),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_message_photos_message_id")
                            .from(MessagePhotos::Table, MessagePhotos::MessageId)
                            .to(Messages::Table, Messages::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_message_photos_message_id")
                    .table(MessagePhotos::Table)
                    .col(MessagePhotos::MessageId)
                    .to_owned(),
            )
            .await?;

        let connection = manager.get_connection();

        for table in TABLES {
            connection
                .execute_unprepared(&format!(
                    "DROP TRIGGER IF EXISTS bump_data_version ON {table};
                     CREATE TRIGGER bump_data_version
                       AFTER INSERT OR UPDATE OR DELETE ON {table}
                       FOR EACH STATEMENT EXECUTE FUNCTION bump_data_version('messages')"
                ))
                .await?;
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(MessagePhotos::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Messages::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum Messages {
    Table,
    Id,
    FarmerId,
    FarmId,
    Kind,
    Text,
    State,
    ReplyTextKu,
    ReplyTextEn,
    RepliedBy,
    RepliedAt,
    IdempotencyKey,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum MessagePhotos {
    Table,
    Id,
    MessageId,
    ContentType,
    Bytes,
    Size,
    CreatedAt,
}
