use async_trait::async_trait;

use crate::features::staff::{
    app::AppError,
    domain::{Password, PasswordHash},
};

/// Turns a password into the form that is stored, and checks one against it.
#[async_trait]
pub trait PasswordHasher: Send + Sync + std::fmt::Debug {
    async fn hash(&self, password: &Password) -> Result<PasswordHash, AppError>;

    /// Whether `password` is the one behind `stored`. With no stored hash
    /// the answer is always `false`, but it must take as long as a real
    /// check, so that timing does not show which emails have an account.
    async fn verify(
        &self,
        password: &Password,
        stored: Option<&PasswordHash>,
    ) -> Result<bool, AppError>;
}

/// Issues the token the dashboard sends with every later request.
pub trait StaffTokenIssuer: Send + Sync + std::fmt::Debug {
    fn issue(&self, staff_id: i32) -> Result<String, AppError>;
}
