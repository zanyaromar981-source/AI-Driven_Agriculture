use std::sync::Arc;

use crate::{
    app::{AppError as GlobalAppError, StaffContext},
    features::staff::{
        app::{AppError, PasswordHasher, StaffRepository},
        domain::{OwnPasswordChange, OwnProfileChange, Password, Staff, StaffError, StaffName},
    },
    shared::Phone,
};

/// The current password and the one to replace it with. They only come
/// together: a new password without the current one proves nothing.
pub struct OwnPasswordInput {
    pub current: Password,
    pub new: Password,
}

pub struct EditOwnProfileInput {
    pub name: StaffName,
    /// `None` clears it.
    pub phone: Option<Phone>,
    /// `None` keeps the password the account has.
    pub password: Option<OwnPasswordInput>,
}

/// A staff member changing their own name, phone and password. It needs no
/// permission, and it cannot touch roles, the active state or the email.
pub struct EditOwnProfileUseCase {
    staff: Arc<dyn StaffRepository>,
    hasher: Arc<dyn PasswordHasher>,
}

impl EditOwnProfileUseCase {
    pub fn new(staff: Arc<dyn StaffRepository>, hasher: Arc<dyn PasswordHasher>) -> Self {
        Self { staff, hasher }
    }

    /// The account changed is always the one the token belongs to: there is
    /// no id to pass in.
    pub async fn execute(
        &self,
        actor: &StaffContext,
        input: EditOwnProfileInput,
    ) -> Result<Staff, AppError> {
        let staff_id = *actor.staff_id();

        let password = match &input.password {
            Some(password) => Some(self.checked(staff_id, password).await?),
            None => None,
        };
        let sets_password = password.is_some();

        let change = OwnProfileChange {
            name: input.name,
            phone: input.phone,
            password,
        };

        // The statement decides. With a new password it writes only while
        // the stored password is still the one checked above.
        let Some(updated) = self.staff.update_own_profile(staff_id, &change).await? else {
            tracing::info!(
                staff_id,
                sets_password,
                "own profile not changed: the account or its password changed a moment ago"
            );

            return Err(if sets_password {
                StaffError::WrongPassword.into()
            } else {
                GlobalAppError::NotFound.into()
            });
        };

        tracing::info!(
            staff_id,
            password_changed = sets_password,
            "staff member changed their own profile"
        );

        Ok(updated)
    }

    /// Checks the current password against the stored one and hashes the
    /// new one. Nothing is hashed for a wrong current password.
    async fn checked(
        &self,
        staff_id: i32,
        password: &OwnPasswordInput,
    ) -> Result<OwnPasswordChange, AppError> {
        let Some(stored) = self.staff.find_by_id(staff_id).await? else {
            return Err(GlobalAppError::NotFound.into());
        };

        if !self
            .hasher
            .verify(&password.current, Some(stored.password_hash()))
            .await?
        {
            tracing::info!(staff_id, "own password not changed: wrong current password");

            return Err(StaffError::WrongPassword.into());
        }

        Ok(OwnPasswordChange {
            verified_against: stored.password_hash().clone(),
            new_hash: self.hasher.hash(&password.new).await?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::staff::app::testing::{
        Call, DAM_OFFICER_ROLE_ID, Fakes, OWNER_EMAIL, OWNER_ID, OWNER_ROLE_ID, PASSWORD,
        a_staff_member, actor, hash_of,
    };

    const NEW_PASSWORD: &str = "a brand new password";

    fn use_case(fakes: &Fakes) -> EditOwnProfileUseCase {
        EditOwnProfileUseCase::new(Arc::new(fakes.clone()), Arc::new(fakes.clone()))
    }

    fn input(password: Option<(&str, &str)>) -> EditOwnProfileInput {
        EditOwnProfileInput {
            name: StaffName::new("Renamed".to_string()).expect("name"),
            phone: Some(Phone::new("+9647501234567".to_string()).expect("phone")),
            password: password.map(|(current, new)| OwnPasswordInput {
                current: Password::presented(current.to_string()).expect("current"),
                new: Password::new(new.to_string()).expect("new"),
            }),
        }
    }

    #[tokio::test]
    async fn changes_the_name_and_phone_without_asking_for_a_password() {
        let fakes = Fakes::new();

        let staff = use_case(&fakes)
            .execute(&actor(OWNER_ID), input(None))
            .await
            .expect("staff");

        assert_eq!(staff.name().as_str(), "Renamed");
        assert_eq!(
            staff.phone().as_ref().map(Phone::as_str),
            Some("+9647501234567")
        );
        assert_eq!(*staff.password_hash(), hash_of(PASSWORD));
        assert_eq!(
            fakes.calls(),
            vec![Call::UpdateOwnProfile {
                id: OWNER_ID,
                sets_password: false,
            }],
            "no password was sent, so nothing is read, checked or hashed"
        );
    }

    #[tokio::test]
    async fn never_touches_the_email_the_roles_or_the_active_state() {
        let fakes = Fakes::new();

        let staff = use_case(&fakes)
            .execute(&actor(OWNER_ID), input(Some((PASSWORD, NEW_PASSWORD))))
            .await
            .expect("staff");

        assert_eq!(staff.email().as_str(), OWNER_EMAIL);
        assert!(*staff.active());
        assert!(staff.is_active_owner());
        assert_eq!(staff.roles().len(), 1);
    }

    #[tokio::test]
    async fn the_right_current_password_lets_a_new_one_be_set() {
        let fakes = Fakes::new();

        let staff = use_case(&fakes)
            .execute(&actor(OWNER_ID), input(Some((PASSWORD, NEW_PASSWORD))))
            .await
            .expect("staff");

        assert_eq!(*staff.password_hash(), hash_of(NEW_PASSWORD));
        assert_eq!(
            fakes.calls(),
            vec![
                Call::FindStaff { id: OWNER_ID },
                Call::VerifyPassword {
                    against_a_stored_hash: true
                },
                Call::HashPassword,
                Call::UpdateOwnProfile {
                    id: OWNER_ID,
                    sets_password: true,
                },
            ]
        );
    }

    #[tokio::test]
    async fn a_wrong_current_password_changes_nothing_at_all() {
        let fakes = Fakes::new();

        let result = use_case(&fakes)
            .execute(
                &actor(OWNER_ID),
                input(Some(("not the password", NEW_PASSWORD))),
            )
            .await;

        assert!(matches!(
            result,
            Err(AppError::Staff(StaffError::WrongPassword))
        ));

        let stored = fakes.stored_staff(OWNER_ID).expect("stored");

        assert_eq!(*stored.password_hash(), hash_of(PASSWORD));
        assert_eq!(
            stored.name().as_str(),
            "Hiwa K.",
            "the name and phone sent along must not be stored either"
        );
        assert!(
            !fakes
                .calls()
                .iter()
                .any(|call| matches!(call, Call::UpdateOwnProfile { .. }))
        );
    }

    #[tokio::test]
    async fn a_password_changed_elsewhere_after_the_check_is_not_overwritten() {
        let fakes = Fakes::new().with_the_password_changed_before_the_write();

        let result = use_case(&fakes)
            .execute(&actor(OWNER_ID), input(Some((PASSWORD, NEW_PASSWORD))))
            .await;

        assert!(matches!(
            result,
            Err(AppError::Staff(StaffError::WrongPassword))
        ));
        assert_eq!(
            *fakes
                .stored_staff(OWNER_ID)
                .expect("stored")
                .password_hash(),
            hash_of("set by someone else"),
            "the password that was checked is no longer the account's"
        );
    }

    #[tokio::test]
    async fn anyone_signed_in_can_change_their_own_profile_whatever_their_role() {
        let fakes = Fakes::new().with_staff(a_staff_member(
            2,
            "dams@example.org",
            true,
            &[DAM_OFFICER_ROLE_ID],
        ));

        let staff = use_case(&fakes)
            .execute(&actor(2), input(Some((PASSWORD, NEW_PASSWORD))))
            .await
            .expect("staff");

        assert_eq!(*staff.id(), Some(2));
        assert_eq!(*staff.roles()[0].id(), DAM_OFFICER_ROLE_ID);
        assert_eq!(
            *fakes.stored_staff(OWNER_ID).expect("owner").password_hash(),
            hash_of(PASSWORD),
            "only the account the token belongs to is written"
        );
        assert!(
            fakes
                .stored_staff(OWNER_ID)
                .expect("owner")
                .roles()
                .iter()
                .any(|role| *role.id() == OWNER_ROLE_ID)
        );
    }

    #[tokio::test]
    async fn an_account_that_is_gone_is_not_found() {
        let result = use_case(&Fakes::new())
            .execute(&actor(99), input(None))
            .await;

        assert!(matches!(
            result,
            Err(AppError::GlobalAppError(GlobalAppError::NotFound))
        ));
    }
}
