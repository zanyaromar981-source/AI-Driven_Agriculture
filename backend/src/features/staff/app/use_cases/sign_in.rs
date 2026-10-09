use std::sync::Arc;

use crate::features::staff::{
    app::{AppError, PasswordHasher, StaffRepository, StaffTokenIssuer},
    domain::{Password, StaffAccess, StaffEmail, StaffError},
};

pub struct SignInInput {
    pub email: StaffEmail,
    pub password: Password,
}

pub struct SignedInStaff {
    pub token: String,
    pub access: StaffAccess,
}

pub struct SignInUseCase {
    staff: Arc<dyn StaffRepository>,
    hasher: Arc<dyn PasswordHasher>,
    tokens: Arc<dyn StaffTokenIssuer>,
}

impl SignInUseCase {
    pub fn new(
        staff: Arc<dyn StaffRepository>,
        hasher: Arc<dyn PasswordHasher>,
        tokens: Arc<dyn StaffTokenIssuer>,
    ) -> Self {
        Self {
            staff,
            hasher,
            tokens,
        }
    }

    /// An unknown email, a wrong password and an account that is switched
    /// off all get the same answer after the same work, so the endpoint
    /// cannot be used to find out who has an account.
    pub async fn execute(&self, input: SignInInput) -> Result<SignedInStaff, AppError> {
        let found = self.staff.find_by_email(&input.email).await?;

        // The password is checked before anything else is looked at, also
        // when there is no account to check it against.
        let password_matches = self
            .hasher
            .verify(
                &input.password,
                found.as_ref().map(|staff| staff.password_hash()),
            )
            .await?;

        let Some((staff, staff_id)) = found
            .filter(|staff| password_matches && *staff.active())
            .and_then(|staff| staff.id().map(|id| (staff, id)))
        else {
            tracing::info!("staff sign-in refused");

            return Err(StaffError::BadCredentials.into());
        };

        let granted = self.staff.permissions_granted_to(staff_id).await?;
        let token = self.tokens.issue(staff_id)?;

        tracing::info!(staff_id, "staff signed in");

        Ok(SignedInStaff {
            token,
            access: StaffAccess::new(staff, granted),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        app::Permission,
        features::staff::app::testing::{
            Call, DAM_OFFICER_ROLE_ID, Fakes, OWNER_EMAIL, OWNER_ID, PASSWORD, a_staff_member,
        },
    };

    fn use_case(fakes: &Fakes) -> SignInUseCase {
        SignInUseCase::new(
            Arc::new(fakes.clone()),
            Arc::new(fakes.clone()),
            Arc::new(fakes.clone()),
        )
    }

    fn input(email: &str, password: &str) -> SignInInput {
        SignInInput {
            email: StaffEmail::new(email.to_string()).expect("email"),
            password: Password::presented(password.to_string()).expect("password"),
        }
    }

    #[tokio::test]
    async fn the_right_password_signs_in_with_a_token_for_the_staff_id() {
        let fakes = Fakes::new();

        let signed_in = use_case(&fakes)
            .execute(input(OWNER_EMAIL, PASSWORD))
            .await
            .expect("signed in");

        assert_eq!(signed_in.token, format!("token-for-{OWNER_ID}"));
        assert_eq!(*signed_in.access.permissions(), Permission::all());
        assert!(
            fakes
                .calls()
                .contains(&Call::IssueToken { staff_id: OWNER_ID })
        );
    }

    #[tokio::test]
    async fn a_wrong_password_is_refused_and_issues_no_token() {
        let fakes = Fakes::new();

        let result = use_case(&fakes)
            .execute(input(OWNER_EMAIL, "not the password"))
            .await;

        assert!(matches!(
            result,
            Err(AppError::Staff(StaffError::BadCredentials))
        ));
        assert!(
            !fakes
                .calls()
                .iter()
                .any(|call| matches!(call, Call::IssueToken { .. }))
        );
    }

    #[tokio::test]
    async fn an_unknown_email_gets_the_same_answer_and_still_spends_one_verification() {
        let fakes = Fakes::new();

        let result = use_case(&fakes)
            .execute(input("nobody@example.org", PASSWORD))
            .await;

        assert!(matches!(
            result,
            Err(AppError::Staff(StaffError::BadCredentials))
        ));
        assert_eq!(
            fakes.calls(),
            vec![
                Call::FindStaffByEmail {
                    email: "nobody@example.org".to_string()
                },
                Call::VerifyPassword {
                    against_a_stored_hash: false
                },
            ],
            "skipping the hash for an unknown email would show in the response time"
        );
    }

    #[tokio::test]
    async fn an_inactive_account_gets_the_same_answer_even_with_the_right_password() {
        let fakes = Fakes::new().with_staff(a_staff_member(
            2,
            "gone@example.org",
            false,
            &[DAM_OFFICER_ROLE_ID],
        ));

        let result = use_case(&fakes)
            .execute(input("gone@example.org", PASSWORD))
            .await;

        assert!(matches!(
            result,
            Err(AppError::Staff(StaffError::BadCredentials))
        ));
        assert!(
            fakes.calls().contains(&Call::VerifyPassword {
                against_a_stored_hash: true
            }),
            "an inactive account must cost the same time as an active one"
        );
        assert!(
            !fakes
                .calls()
                .iter()
                .any(|call| matches!(call, Call::IssueToken { .. }))
        );
    }

    #[tokio::test]
    async fn a_repository_failure_surfaces_and_is_not_mistaken_for_bad_credentials() {
        let result = use_case(&Fakes::failing())
            .execute(input(OWNER_EMAIL, PASSWORD))
            .await;

        assert!(matches!(result, Err(AppError::GlobalAppError(_))));
    }
}
