use chrono::{DateTime, Utc};
use getset::Getters;

use crate::{
    app::Permission,
    features::staff::domain::{
        PasswordHash, RoleDescription, RoleName, RoleSelection, StaffEmail, StaffError, StaffName,
    },
};

/// Each permission once, in the order the role editor shows them.
fn as_a_set(mut permissions: Vec<Permission>) -> Vec<Permission> {
    permissions.sort_unstable();
    permissions.dedup();
    permissions
}

/// A named set of permissions that staff members can hold.
#[derive(Clone, Debug, Getters)]
#[getset(get = "pub")]
pub struct Role {
    /// None = new (not yet persisted), Some = existing (persisted)
    id: Option<i32>,
    name: RoleName,
    description: Option<RoleDescription>,
    /// A system role is seeded by a migration and is what keeps the
    /// dashboard manageable, so staff cannot change or remove it.
    system: bool,
    permissions: Vec<Permission>,
    /// How many staff members hold the role.
    staff_count: u64,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl Role {
    /// A role made by staff. It is never a system role.
    pub fn new(
        name: RoleName,
        description: Option<RoleDescription>,
        permissions: Vec<Permission>,
        now: DateTime<Utc>,
    ) -> Self {
        Self {
            id: None,
            name,
            description,
            system: false,
            permissions: as_a_set(permissions),
            staff_count: 0,
            created_at: now,
            updated_at: now,
        }
    }

    /// Reconstruct from persisted state.
    #[allow(clippy::too_many_arguments)]
    pub fn rehydrate(
        id: i32,
        name: RoleName,
        description: Option<RoleDescription>,
        system: bool,
        permissions: Vec<Permission>,
        staff_count: u64,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id: Some(id),
            name,
            description,
            system,
            permissions: as_a_set(permissions),
            staff_count,
            created_at,
            updated_at,
        }
    }

    pub fn ensure_changeable(&self) -> Result<(), StaffError> {
        if self.system {
            return Err(StaffError::SystemRole);
        }

        Ok(())
    }

    /// Replaces the name, the description and the whole permission set.
    pub fn edit(
        &mut self,
        name: RoleName,
        description: Option<RoleDescription>,
        permissions: Vec<Permission>,
        now: DateTime<Utc>,
    ) -> Result<(), StaffError> {
        self.ensure_changeable()?;

        self.name = name;
        self.description = description;
        self.permissions = as_a_set(permissions);
        self.updated_at = now;

        Ok(())
    }
}

/// A role as it appears on a staff member: enough to name it and to know
/// whether it is the owner role.
#[derive(Clone, Debug, PartialEq, Eq, Getters)]
#[getset(get = "pub")]
pub struct RoleRef {
    id: i32,
    name: String,
    system: bool,
}

impl RoleRef {
    pub fn new(id: i32, name: String, system: bool) -> Self {
        Self { id, name, system }
    }
}

/// Someone who signs in to the dashboard.
#[derive(Clone, Debug, Getters)]
#[getset(get = "pub")]
pub struct Staff {
    /// None = new (not yet persisted), Some = existing (persisted)
    id: Option<i32>,
    email: StaffEmail,
    name: StaffName,
    password_hash: PasswordHash,
    active: bool,
    roles: Vec<RoleRef>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl Staff {
    /// A new account is active. Its roles are assigned when it is stored.
    pub fn new(
        email: StaffEmail,
        name: StaffName,
        password_hash: PasswordHash,
        now: DateTime<Utc>,
    ) -> Self {
        Self {
            id: None,
            email,
            name,
            password_hash,
            active: true,
            roles: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Reconstruct from persisted state.
    #[allow(clippy::too_many_arguments)]
    pub fn rehydrate(
        id: i32,
        email: StaffEmail,
        name: StaffName,
        password_hash: PasswordHash,
        active: bool,
        roles: Vec<RoleRef>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id: Some(id),
            email,
            name,
            password_hash,
            active,
            roles,
            created_at,
            updated_at,
        }
    }

    /// An owner counts only while the account can still sign in.
    pub fn is_active_owner(&self) -> bool {
        self.active && self.roles.iter().any(|role| role.system)
    }
}

/// What an edit sets on a staff member. The email is the account and never
/// changes; the password changes only when a new hash is given.
#[derive(Clone, Debug, Getters)]
#[getset(get = "pub")]
pub struct StaffChange {
    name: StaffName,
    active: bool,
    roles: RoleSelection,
    password_hash: Option<PasswordHash>,
}

impl StaffChange {
    pub fn new(
        name: StaffName,
        active: bool,
        roles: RoleSelection,
        password_hash: Option<PasswordHash>,
    ) -> Self {
        Self {
            name,
            active,
            roles,
            password_hash,
        }
    }

    /// Nobody can lock themselves out by switching their own account off.
    pub fn ensure_not_deactivating_own_account(
        &self,
        actor_id: i32,
        target_id: i32,
    ) -> Result<(), StaffError> {
        if actor_id == target_id && !self.active {
            return Err(StaffError::OwnAccount);
        }

        Ok(())
    }

    /// Whether the staff member is an active owner once the change is made.
    pub fn keeps_an_active_owner(&self, system_role_ids: &[i32]) -> bool {
        self.active
            && system_role_ids
                .iter()
                .any(|role_id| self.roles.contains(*role_id))
    }
}

/// Nobody can delete the account they are signed in with.
pub fn ensure_not_own_account(actor_id: i32, target_id: i32) -> Result<(), StaffError> {
    if actor_id == target_id {
        return Err(StaffError::OwnAccount);
    }

    Ok(())
}

/// What is known about owners at the moment a staff member is changed. It
/// must be read inside the transaction that makes the change, with the rows
/// locked, or two changes at the same moment could each see the other's
/// owner and together leave none.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OwnerStanding {
    pub target_is_active_owner: bool,
    pub other_active_owners: u64,
}

impl OwnerStanding {
    /// Without an active owner nobody could manage staff and roles again,
    /// so the last one cannot be deleted, deactivated or lose the role.
    pub fn ensure_an_owner_remains(
        &self,
        target_stays_active_owner: bool,
    ) -> Result<(), StaffError> {
        if self.target_is_active_owner
            && !target_stays_active_owner
            && self.other_active_owners == 0
        {
            return Err(StaffError::LastOwner);
        }

        Ok(())
    }
}

/// A staff member together with everything their roles allow.
#[derive(Clone, Debug, Getters)]
#[getset(get = "pub")]
pub struct StaffAccess {
    staff: Staff,
    permissions: Vec<Permission>,
}

impl StaffAccess {
    /// `granted` holds the permissions of every role the staff member has,
    /// one entry per role that grants it. The result is their union, and
    /// nothing at all for an account that is switched off.
    pub fn new(staff: Staff, granted: Vec<Permission>) -> Self {
        let permissions = if staff.active {
            as_a_set(granted)
        } else {
            Vec::new()
        };

        Self { staff, permissions }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::{Action, Resource};

    fn role_name(value: &str) -> RoleName {
        RoleName::new(value.to_string()).expect("role name")
    }

    fn a_role(system: bool) -> Role {
        Role::rehydrate(
            1,
            role_name("Dam officer"),
            None,
            system,
            vec![Permission::new(Resource::Dams, Action::Read)],
            0,
            Utc::now(),
            Utc::now(),
        )
    }

    fn a_staff_member(active: bool, roles: Vec<RoleRef>) -> Staff {
        Staff::rehydrate(
            7,
            StaffEmail::new("hiwa@example.org".to_string()).expect("email"),
            StaffName::new("Hiwa K.".to_string()).expect("name"),
            PasswordHash::new("stored-hash".to_string()).expect("hash"),
            active,
            roles,
            Utc::now(),
            Utc::now(),
        )
    }

    fn owner_role() -> RoleRef {
        RoleRef::new(1, "Owner".to_string(), true)
    }

    fn custom_role() -> RoleRef {
        RoleRef::new(2, "Dam officer".to_string(), false)
    }

    fn a_change(active: bool, role_ids: Vec<i32>) -> StaffChange {
        StaffChange::new(
            StaffName::new("Hiwa K.".to_string()).expect("name"),
            active,
            RoleSelection::new(role_ids).expect("roles"),
            None,
        )
    }

    #[test]
    fn a_role_holds_each_permission_once_in_a_fixed_order() {
        let role = Role::new(
            role_name("Dam officer"),
            None,
            vec![
                Permission::new(Resource::Dams, Action::Update),
                Permission::new(Resource::Zones, Action::Read),
                Permission::new(Resource::Dams, Action::Update),
                Permission::new(Resource::Dams, Action::Read),
            ],
            Utc::now(),
        );

        assert_eq!(
            *role.permissions(),
            vec![
                Permission::new(Resource::Zones, Action::Read),
                Permission::new(Resource::Dams, Action::Read),
                Permission::new(Resource::Dams, Action::Update),
            ]
        );
    }

    #[test]
    fn a_role_made_by_staff_is_never_a_system_role() {
        let role = Role::new(role_name("Dam officer"), None, Vec::new(), Utc::now());

        assert!(!role.system());
        assert!(role.id().is_none());
    }

    #[test]
    fn editing_a_role_replaces_its_whole_permission_set() {
        let mut role = a_role(false);
        let now = Utc::now();

        role.edit(
            role_name("Fire officer"),
            None,
            vec![Permission::new(Resource::Fires, Action::Read)],
            now,
        )
        .expect("edited");

        assert_eq!(role.name().as_str(), "Fire officer");
        assert_eq!(
            *role.permissions(),
            vec![Permission::new(Resource::Fires, Action::Read)],
            "the permission the role had before must be gone"
        );
        assert_eq!(*role.updated_at(), now);
    }

    #[test]
    fn a_system_role_cannot_be_edited() {
        let mut role = a_role(true);

        let result = role.edit(role_name("Renamed"), None, Vec::new(), Utc::now());

        assert!(matches!(result, Err(StaffError::SystemRole)));
        assert_eq!(role.name().as_str(), "Dam officer");
        assert_eq!(role.permissions().len(), 1);
    }

    #[test]
    fn only_a_system_role_is_unchangeable() {
        assert!(matches!(
            a_role(true).ensure_changeable(),
            Err(StaffError::SystemRole)
        ));
        assert!(a_role(false).ensure_changeable().is_ok());
    }

    #[test]
    fn a_new_staff_member_is_active_and_has_no_roles_yet() {
        let staff = Staff::new(
            StaffEmail::new("hiwa@example.org".to_string()).expect("email"),
            StaffName::new("Hiwa K.".to_string()).expect("name"),
            PasswordHash::new("stored-hash".to_string()).expect("hash"),
            Utc::now(),
        );

        assert!(staff.active());
        assert!(staff.roles().is_empty());
    }

    #[test]
    fn an_owner_counts_only_while_the_account_is_active() {
        assert!(a_staff_member(true, vec![owner_role()]).is_active_owner());
        assert!(!a_staff_member(false, vec![owner_role()]).is_active_owner());
        assert!(!a_staff_member(true, vec![custom_role()]).is_active_owner());
    }

    #[test]
    fn debug_output_of_a_staff_member_never_shows_the_password_hash() {
        let shown = format!("{:?}", a_staff_member(true, Vec::new()));

        assert!(!shown.contains("stored-hash"));
    }

    #[test]
    fn nobody_can_deactivate_their_own_account() {
        assert!(matches!(
            a_change(false, vec![1]).ensure_not_deactivating_own_account(7, 7),
            Err(StaffError::OwnAccount)
        ));
        assert!(
            a_change(true, vec![1])
                .ensure_not_deactivating_own_account(7, 7)
                .is_ok(),
            "editing your own name or roles is allowed"
        );
        assert!(
            a_change(false, vec![1])
                .ensure_not_deactivating_own_account(7, 8)
                .is_ok()
        );
    }

    #[test]
    fn nobody_can_delete_their_own_account() {
        assert!(matches!(
            ensure_not_own_account(7, 7),
            Err(StaffError::OwnAccount)
        ));
        assert!(ensure_not_own_account(7, 8).is_ok());
    }

    #[test]
    fn a_change_keeps_an_owner_only_if_active_and_still_holding_the_role() {
        assert!(a_change(true, vec![1, 2]).keeps_an_active_owner(&[1]));
        assert!(!a_change(false, vec![1, 2]).keeps_an_active_owner(&[1]));
        assert!(!a_change(true, vec![2]).keeps_an_active_owner(&[1]));
    }

    #[test]
    fn the_last_active_owner_cannot_stop_being_one() {
        let alone = OwnerStanding {
            target_is_active_owner: true,
            other_active_owners: 0,
        };

        assert!(matches!(
            alone.ensure_an_owner_remains(false),
            Err(StaffError::LastOwner)
        ));
        assert!(
            alone.ensure_an_owner_remains(true).is_ok(),
            "a change that leaves them an active owner is harmless"
        );
    }

    #[test]
    fn an_owner_can_go_while_another_active_owner_remains() {
        let one_of_two = OwnerStanding {
            target_is_active_owner: true,
            other_active_owners: 1,
        };

        assert!(one_of_two.ensure_an_owner_remains(false).is_ok());
    }

    #[test]
    fn changing_someone_who_is_not_an_active_owner_never_trips_the_rule() {
        let not_an_owner = OwnerStanding {
            target_is_active_owner: false,
            other_active_owners: 0,
        };

        assert!(not_an_owner.ensure_an_owner_remains(false).is_ok());
    }

    #[test]
    fn a_staff_members_permissions_are_the_union_of_their_roles() {
        let access = StaffAccess::new(
            a_staff_member(true, vec![custom_role()]),
            vec![
                Permission::new(Resource::Dams, Action::Read),
                Permission::new(Resource::Fires, Action::Read),
                Permission::new(Resource::Dams, Action::Read),
                Permission::new(Resource::Dams, Action::Update),
            ],
        );

        assert_eq!(
            *access.permissions(),
            vec![
                Permission::new(Resource::Dams, Action::Read),
                Permission::new(Resource::Dams, Action::Update),
                Permission::new(Resource::Fires, Action::Read),
            ],
            "a permission two roles grant must appear once"
        );
    }

    #[test]
    fn an_inactive_staff_member_has_no_access_at_all() {
        let access = StaffAccess::new(a_staff_member(false, vec![owner_role()]), Permission::all());

        assert!(access.permissions().is_empty());
    }
}
