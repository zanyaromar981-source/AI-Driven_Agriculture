use async_trait::async_trait;

use crate::{
    app::Permission,
    features::staff::{
        app::AppError,
        domain::{Role, RoleSelection, Staff, StaffChange, StaffEmail},
    },
};

#[async_trait]
pub trait RoleRepository: Send + Sync + std::fmt::Debug {
    /// Returns every role, oldest first, each with its permissions and the
    /// number of staff holding it.
    async fn find_all(&self) -> Result<Vec<Role>, AppError>;

    async fn find_by_id(&self, id: i32) -> Result<Option<Role>, AppError>;

    /// Creates a new role with its permissions. `entity.id()` must be
    /// `None`. Fails with `RoleNameTaken` when the name is in use; the
    /// unique index decides, so of two requests at once one fails.
    async fn create(&self, entity: &Role) -> Result<Role, AppError>;

    /// Replaces the name, the description and the whole permission set of
    /// an existing role. `entity.id()` must be `Some`. A system role is
    /// refused by the statement itself with `SystemRole`.
    async fn update(&self, entity: &Role) -> Result<Role, AppError>;

    /// Deletes the role in one statement. Fails with `SystemRole` for a
    /// system role and with `RoleInUse` while any staff member holds it.
    async fn delete(&self, id: i32) -> Result<(), AppError>;
}

#[async_trait]
pub trait StaffRepository: Send + Sync + std::fmt::Debug {
    /// Returns every staff member, oldest first, with the roles they hold.
    async fn find_all(&self) -> Result<Vec<Staff>, AppError>;

    async fn find_by_id(&self, id: i32) -> Result<Option<Staff>, AppError>;

    async fn find_by_email(&self, email: &StaffEmail) -> Result<Option<Staff>, AppError>;

    /// Returns the permissions of every role the staff member holds. A
    /// permission that two roles grant is returned twice.
    async fn permissions_granted_to(&self, staff_id: i32) -> Result<Vec<Permission>, AppError>;

    /// Creates a new staff member holding `roles`. `entity.id()` must be
    /// `None`. Fails with `EmailTaken` when the email is in use (the unique
    /// index decides) and with `UnknownRole` when a role does not exist
    /// (the foreign key decides).
    async fn create(&self, entity: &Staff, roles: &RoleSelection) -> Result<Staff, AppError>;

    /// Applies the change in one transaction. Fails with `LastOwner` when
    /// it would leave no active owner, checked with the rows locked, and
    /// with `UnknownRole` when a role does not exist.
    async fn update(&self, id: i32, change: &StaffChange) -> Result<Staff, AppError>;

    /// Deletes the staff member in one transaction. Fails with `LastOwner`
    /// when they are the last active owner, checked with the rows locked.
    async fn delete(&self, id: i32) -> Result<(), AppError>;

    /// Creates the staff member with the owner role unless the email
    /// already has an account. Returns whether it was created; an existing
    /// account is left exactly as it is.
    async fn create_owner_if_absent(&self, entity: &Staff) -> Result<bool, AppError>;
}
