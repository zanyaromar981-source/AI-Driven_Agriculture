use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

const OWNER_ROLE: &str = "Owner";

/// Every permission there is when this migration was written: each of the
/// four actions on each resource. A resource added to the code later needs
/// a new migration that gives it to the owner role.
const OWNER_PERMISSIONS: [(&str, &str); 44] = [
    ("zones", "create"),
    ("zones", "read"),
    ("zones", "update"),
    ("zones", "delete"),
    ("dams", "create"),
    ("dams", "read"),
    ("dams", "update"),
    ("dams", "delete"),
    ("outlooks", "create"),
    ("outlooks", "read"),
    ("outlooks", "update"),
    ("outlooks", "delete"),
    ("water", "create"),
    ("water", "read"),
    ("water", "update"),
    ("water", "delete"),
    ("fires", "create"),
    ("fires", "read"),
    ("fires", "update"),
    ("fires", "delete"),
    ("alwa", "create"),
    ("alwa", "read"),
    ("alwa", "update"),
    ("alwa", "delete"),
    ("farmers", "create"),
    ("farmers", "read"),
    ("farmers", "update"),
    ("farmers", "delete"),
    ("farms", "create"),
    ("farms", "read"),
    ("farms", "update"),
    ("farms", "delete"),
    ("insights", "create"),
    ("insights", "read"),
    ("insights", "update"),
    ("insights", "delete"),
    ("staff", "create"),
    ("staff", "read"),
    ("staff", "update"),
    ("staff", "delete"),
    ("roles", "create"),
    ("roles", "read"),
    ("roles", "update"),
    ("roles", "delete"),
];

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Staff::Table)
                    .if_not_exists()
                    .col(pk_auto(Staff::Id))
                    // Stored in lower case, so this index is what makes an
                    // email belong to one account only.
                    .col(string_len_uniq(Staff::Email, 254))
                    .col(string_len(Staff::Name, 80))
                    .col(string(Staff::PasswordHash))
                    .col(boolean(Staff::Active).default(true))
                    .col(
                        timestamp(Staff::CreatedAt)
                            .not_null()
                            .default(SimpleExpr::Custom("CURRENT_TIMESTAMP".into())),
                    )
                    .col(
                        timestamp(Staff::UpdatedAt)
                            .not_null()
                            .default(SimpleExpr::Custom("CURRENT_TIMESTAMP".into())),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Roles::Table)
                    .if_not_exists()
                    .col(pk_auto(Roles::Id))
                    .col(string_len_uniq(Roles::Name, 60))
                    .col(string_len_null(Roles::Description, 200))
                    .col(boolean(Roles::System).default(false))
                    .col(
                        timestamp(Roles::CreatedAt)
                            .not_null()
                            .default(SimpleExpr::Custom("CURRENT_TIMESTAMP".into())),
                    )
                    .col(
                        timestamp(Roles::UpdatedAt)
                            .not_null()
                            .default(SimpleExpr::Custom("CURRENT_TIMESTAMP".into())),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(RolePermissions::Table)
                    .if_not_exists()
                    .col(integer(RolePermissions::RoleId))
                    .col(string(RolePermissions::Resource))
                    .col(string(RolePermissions::Action))
                    .primary_key(
                        Index::create()
                            .name("pk_role_permissions")
                            .col(RolePermissions::RoleId)
                            .col(RolePermissions::Resource)
                            .col(RolePermissions::Action),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_role_permissions_role_id")
                            .from(RolePermissions::Table, RolePermissions::RoleId)
                            .to(Roles::Table, Roles::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(StaffRoles::Table)
                    .if_not_exists()
                    .col(integer(StaffRoles::StaffId))
                    .col(integer(StaffRoles::RoleId))
                    .primary_key(
                        Index::create()
                            .name("pk_staff_roles")
                            .col(StaffRoles::StaffId)
                            .col(StaffRoles::RoleId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_staff_roles_staff_id")
                            .from(StaffRoles::Table, StaffRoles::StaffId)
                            .to(Staff::Table, Staff::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    // Restrict is the rule that a role still held by anyone
                    // cannot be deleted.
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_staff_roles_role_id")
                            .from(StaffRoles::Table, StaffRoles::RoleId)
                            .to(Roles::Table, Roles::Id)
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_staff_roles_role_id")
                    .table(StaffRoles::Table)
                    .col(StaffRoles::RoleId)
                    .to_owned(),
            )
            .await?;

        let owner = Query::insert()
            .into_table(Roles::Table)
            .columns([Roles::Name, Roles::System])
            .values_panic([OWNER_ROLE.into(), true.into()])
            .on_conflict(OnConflict::column(Roles::Name).do_nothing().to_owned())
            .to_owned();

        manager.exec_stmt(owner).await?;

        let pairs = OWNER_PERMISSIONS
            .iter()
            .map(|(resource, action)| format!("('{resource}', '{action}')"))
            .collect::<Vec<_>>()
            .join(", ");

        manager
            .get_connection()
            .execute_unprepared(&format!(
                "INSERT INTO role_permissions (role_id, resource, action) \
                 SELECT roles.id, pairs.resource, pairs.action \
                 FROM roles, (VALUES {pairs}) AS pairs (resource, action) \
                 WHERE roles.name = '{OWNER_ROLE}' AND roles.system \
                 ON CONFLICT DO NOTHING"
            ))
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(StaffRoles::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(RolePermissions::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Roles::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Staff::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum Staff {
    Table,
    Id,
    Email,
    Name,
    PasswordHash,
    Active,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Roles {
    Table,
    Id,
    Name,
    Description,
    System,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum RolePermissions {
    Table,
    RoleId,
    Resource,
    Action,
}

#[derive(DeriveIden)]
enum StaffRoles {
    Table,
    StaffId,
    RoleId,
}
