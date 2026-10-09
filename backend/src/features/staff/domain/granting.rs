use std::collections::HashSet;

use crate::{app::Permission, features::staff::domain::StaffError};

/// The staff member making a change, as far as handing out access goes:
/// what they hold themselves at the moment of the change. It must be read
/// inside the transaction that makes the change, or someone being stripped
/// of a permission at the same moment could still pass it on.
#[derive(Clone, Debug)]
pub struct Grantor {
    permissions: HashSet<Permission>,
}

impl Grantor {
    /// `granted` holds the permissions of every role the staff member has.
    /// An account that is switched off, or gone, holds nothing.
    pub fn new(active: bool, granted: Vec<Permission>) -> Self {
        Self {
            permissions: if active {
                granted.into_iter().collect()
            } else {
                HashSet::new()
            },
        }
    }

    fn ensure_holds_all<'a>(
        &self,
        wanted: impl IntoIterator<Item = &'a Permission>,
    ) -> Result<(), StaffError> {
        if wanted
            .into_iter()
            .all(|permission| self.permissions.contains(permission))
        {
            Ok(())
        } else {
            Err(StaffError::CannotGrant)
        }
    }

    /// A role may gain only permissions the grantor holds. What it already
    /// had may stay, and taking permissions away is always allowed.
    pub fn ensure_can_set_role_permissions(
        &self,
        held_before: &[Permission],
        wanted: &[Permission],
    ) -> Result<(), StaffError> {
        self.ensure_holds_all(
            wanted
                .iter()
                .filter(|permission| !held_before.contains(permission)),
        )
    }

    /// `newly_assigned` holds the permissions of the roles a staff member
    /// is about to receive and does not hold yet. Each must be one the
    /// grantor holds, so only someone holding everything can make an owner.
    pub fn ensure_can_assign(&self, newly_assigned: &[Permission]) -> Result<(), StaffError> {
        self.ensure_holds_all(newly_assigned)
    }

    /// Setting someone else's password is taking over their account, so it
    /// needs everything that account can do. One's own password is one's
    /// own to change.
    pub fn ensure_can_set_password(
        &self,
        actor_id: i32,
        target_id: i32,
        target_permissions: &[Permission],
    ) -> Result<(), StaffError> {
        if actor_id == target_id {
            return Ok(());
        }

        self.ensure_holds_all(target_permissions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::{Action, Resource};

    const DAMS_READ: Permission = Permission::new(Resource::Dams, Action::Read);
    const DAMS_UPDATE: Permission = Permission::new(Resource::Dams, Action::Update);
    const FIRES_READ: Permission = Permission::new(Resource::Fires, Action::Read);

    fn holding(permissions: &[Permission]) -> Grantor {
        Grantor::new(true, permissions.to_vec())
    }

    #[test]
    fn a_role_can_be_given_what_the_grantor_holds() {
        assert!(
            holding(&[DAMS_READ])
                .ensure_can_set_role_permissions(&[], &[DAMS_READ])
                .is_ok()
        );
    }

    #[test]
    fn a_role_cannot_be_given_what_the_grantor_lacks() {
        assert!(matches!(
            holding(&[DAMS_READ]).ensure_can_set_role_permissions(&[], &[DAMS_READ, DAMS_UPDATE]),
            Err(StaffError::CannotGrant)
        ));
    }

    #[test]
    fn a_role_keeps_what_it_already_had_even_if_the_grantor_lacks_it() {
        assert!(
            holding(&[DAMS_READ])
                .ensure_can_set_role_permissions(&[DAMS_UPDATE], &[DAMS_UPDATE, DAMS_READ])
                .is_ok(),
            "only what is added is a grant"
        );
    }

    #[test]
    fn taking_permissions_away_from_a_role_is_always_allowed() {
        assert!(
            holding(&[])
                .ensure_can_set_role_permissions(&[DAMS_UPDATE, FIRES_READ], &[FIRES_READ])
                .is_ok()
        );
        assert!(
            holding(&[])
                .ensure_can_set_role_permissions(&[DAMS_UPDATE], &[])
                .is_ok()
        );
    }

    #[test]
    fn a_role_can_be_assigned_only_if_the_grantor_holds_all_it_grants() {
        let grantor = holding(&[DAMS_READ, FIRES_READ]);

        assert!(grantor.ensure_can_assign(&[DAMS_READ]).is_ok());
        assert!(matches!(
            grantor.ensure_can_assign(&[DAMS_READ, DAMS_UPDATE]),
            Err(StaffError::CannotGrant)
        ));
    }

    #[test]
    fn only_someone_holding_everything_can_hand_out_the_owner_role() {
        let everything = Permission::all();
        let almost: Vec<Permission> = everything[1..].to_vec();

        assert!(holding(&everything).ensure_can_assign(&everything).is_ok());
        assert!(matches!(
            holding(&almost).ensure_can_assign(&everything),
            Err(StaffError::CannotGrant)
        ));
    }

    #[test]
    fn assigning_nothing_new_is_always_allowed() {
        assert!(holding(&[]).ensure_can_assign(&[]).is_ok());
    }

    #[test]
    fn an_inactive_grantor_can_grant_nothing() {
        let switched_off = Grantor::new(false, Permission::all());

        assert!(matches!(
            switched_off.ensure_can_assign(&[DAMS_READ]),
            Err(StaffError::CannotGrant)
        ));
    }

    #[test]
    fn someone_elses_password_needs_everything_they_can_do() {
        let grantor = holding(&[DAMS_READ]);

        assert!(grantor.ensure_can_set_password(1, 2, &[DAMS_READ]).is_ok());
        assert!(grantor.ensure_can_set_password(1, 2, &[]).is_ok());
        assert!(matches!(
            grantor.ensure_can_set_password(1, 2, &[DAMS_READ, DAMS_UPDATE]),
            Err(StaffError::CannotGrant)
        ));
    }

    #[test]
    fn ones_own_password_is_always_ones_own_to_change() {
        assert!(
            holding(&[])
                .ensure_can_set_password(1, 1, &Permission::all())
                .is_ok()
        );
    }
}
