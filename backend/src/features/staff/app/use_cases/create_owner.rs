use std::sync::Arc;

use chrono::Utc;

use crate::features::staff::{
    app::{AppError, PasswordHasher, StaffRepository},
    domain::{Password, Staff, StaffEmail, StaffName},
};

pub struct CreateOwnerInput {
    pub email: StaffEmail,
    pub name: StaffName,
    pub password: Password,
}

impl CreateOwnerInput {
    /// Builds the input from what was typed on the command line, under the
    /// same rules as an account made through the dashboard.
    pub fn new(email: String, name: String, password: String) -> Result<Self, AppError> {
        Ok(Self {
            email: StaffEmail::new(email)?,
            name: StaffName::new(name)?,
            password: Password::new(password)?,
        })
    }
}

/// How the first account comes to exist: there is nobody yet to sign in and
/// add it through the dashboard.
pub struct CreateOwnerUseCase {
    staff: Arc<dyn StaffRepository>,
    hasher: Arc<dyn PasswordHasher>,
}

impl CreateOwnerUseCase {
    pub fn new(staff: Arc<dyn StaffRepository>, hasher: Arc<dyn PasswordHasher>) -> Self {
        Self { staff, hasher }
    }

    /// Returns whether the owner was created. An email that already has an
    /// account is left exactly as it is: running the command again must
    /// never reset a password or hand out the owner role.
    pub async fn execute(&self, input: CreateOwnerInput) -> Result<bool, AppError> {
        let password_hash = self.hasher.hash(&input.password).await?;

        let owner = Staff::new(input.email, input.name, password_hash, Utc::now());

        let created = self.staff.create_owner_if_absent(&owner).await?;

        tracing::info!(created, "owner account requested from the command line");

        Ok(created)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::staff::app::testing::{
        Call, Fakes, OWNER_EMAIL, OWNER_ID, PASSWORD, hash_of,
    };

    fn use_case(fakes: &Fakes) -> CreateOwnerUseCase {
        CreateOwnerUseCase::new(Arc::new(fakes.clone()), Arc::new(fakes.clone()))
    }

    fn input(email: &str, password: &str) -> CreateOwnerInput {
        CreateOwnerInput::new(
            email.to_string(),
            "First Owner".to_string(),
            password.to_string(),
        )
        .expect("input")
    }

    #[tokio::test]
    async fn creates_an_active_owner_for_a_new_email() {
        let fakes = Fakes::new();

        let created = use_case(&fakes)
            .execute(input("first@example.org", "a long enough password"))
            .await
            .expect("created");

        assert!(created);
        assert!(fakes.stored_staff(2).expect("stored").is_active_owner());
        assert_eq!(
            fakes.calls(),
            vec![
                Call::HashPassword,
                Call::CreateOwnerIfAbsent {
                    email: "first@example.org".to_string()
                }
            ]
        );
    }

    #[tokio::test]
    async fn an_existing_email_is_left_exactly_as_it_is() {
        let fakes = Fakes::new();

        let created = use_case(&fakes)
            .execute(input(OWNER_EMAIL, "another long password"))
            .await
            .expect("answered");

        assert!(!created);
        assert_eq!(
            *fakes
                .stored_staff(OWNER_ID)
                .expect("stored")
                .password_hash(),
            hash_of(PASSWORD),
            "running the command again must not reset the password"
        );
    }

    #[test]
    fn a_password_that_is_too_short_is_refused() {
        assert!(
            CreateOwnerInput::new(
                "first@example.org".to_string(),
                "First Owner".to_string(),
                "short".to_string()
            )
            .is_err()
        );
    }
}
