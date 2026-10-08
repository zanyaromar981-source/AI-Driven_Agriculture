use crate::shared::DomainError;

#[derive(thiserror::Error, Debug)]
pub enum StaffError {
    #[error("A system role cannot be changed or deleted")]
    SystemRole,

    #[error("The role is still assigned to at least one staff member")]
    RoleInUse,

    #[error("Another role already has this name")]
    RoleNameTaken,

    #[error("Another staff member already has this email")]
    EmailTaken,

    #[error("The email or the password is wrong")]
    BadCredentials,

    #[error("You cannot delete or deactivate your own account")]
    OwnAccount,

    #[error("The last active owner cannot be deleted, deactivated or lose the owner role")]
    LastOwner,

    #[error("A role that does not exist cannot be assigned")]
    UnknownRole,

    #[error(transparent)]
    DomainError(#[from] DomainError),
}
