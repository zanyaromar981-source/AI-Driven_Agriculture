use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use chrono::Utc;

use crate::{
    app::{Action, AppError as GlobalAppError, Permission, Resource, StaffContext},
    features::staff::{
        app::{AppError, PasswordHasher, RoleRepository, StaffRepository, StaffTokenIssuer},
        domain::{
            OwnerStanding, Password, PasswordHash, Role, RoleName, RoleRef, RoleSelection, Staff,
            StaffChange, StaffEmail, StaffError, StaffName,
        },
    },
};

pub const OWNER_ID: i32 = 1;
pub const OWNER_EMAIL: &str = "owner@example.org";
pub const PASSWORD: &str = "correct horse battery";
pub const OWNER_ROLE_ID: i32 = 1;
pub const DAM_OFFICER_ROLE_ID: i32 = 2;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Call {
    FindAllRoles,
    FindRole {
        id: i32,
    },
    CreateRole {
        name: String,
    },
    UpdateRole {
        id: i32,
    },
    DeleteRole {
        id: i32,
    },
    FindAllStaff,
    FindStaff {
        id: i32,
    },
    FindStaffByEmail {
        email: String,
    },
    PermissionsGrantedTo {
        staff_id: i32,
    },
    CreateStaff {
        email: String,
        role_ids: Vec<i32>,
    },
    UpdateStaff {
        id: i32,
        active: bool,
        role_ids: Vec<i32>,
        sets_password: bool,
    },
    DeleteStaff {
        id: i32,
    },
    CreateOwnerIfAbsent {
        email: String,
    },
    HashPassword,
    VerifyPassword {
        against_a_stored_hash: bool,
    },
    IssueToken {
        staff_id: i32,
    },
}

#[derive(Debug, Default)]
struct Script {
    roles: Vec<Role>,
    staff: Vec<Staff>,
    fail_with_database_error: bool,
}

/// One fake standing in for every port of the feature, so a test can read
/// the calls of a whole use case in the order they happened. It keeps roles
/// and staff in memory and refuses what the database would refuse.
#[derive(Debug, Clone, Default)]
pub struct Fakes {
    script: Arc<Mutex<Script>>,
    calls: Arc<Mutex<Vec<Call>>>,
}

impl Fakes {
    /// What a fresh installation holds after `create-owner`: the owner role,
    /// one custom role and one owner.
    pub fn new() -> Self {
        let fake = Self::default();

        {
            let mut script = fake.script.lock().expect("script lock");

            script.roles = vec![owner_role(), dam_officer_role()];
            script.staff = vec![a_staff_member(
                OWNER_ID,
                OWNER_EMAIL,
                true,
                &[OWNER_ROLE_ID],
            )];
        }

        fake
    }

    pub fn with_staff(self, staff: Staff) -> Self {
        self.script.lock().expect("script lock").staff.push(staff);
        self
    }

    pub fn failing() -> Self {
        let fake = Self::new();
        fake.script
            .lock()
            .expect("script lock")
            .fail_with_database_error = true;
        fake
    }

    pub fn calls(&self) -> Vec<Call> {
        self.calls.lock().expect("calls lock").clone()
    }

    pub fn stored_staff(&self, id: i32) -> Option<Staff> {
        self.script
            .lock()
            .expect("script lock")
            .staff
            .iter()
            .find(|staff| *staff.id() == Some(id))
            .cloned()
    }

    pub fn stored_role(&self, id: i32) -> Option<Role> {
        self.script
            .lock()
            .expect("script lock")
            .roles
            .iter()
            .find(|role| *role.id() == Some(id))
            .cloned()
    }

    fn record(&self, call: Call) {
        self.calls.lock().expect("calls lock").push(call);
    }

    fn guard(&self) -> Result<(), AppError> {
        if self
            .script
            .lock()
            .expect("script lock")
            .fail_with_database_error
        {
            return Err(GlobalAppError::DatabaseError("connection refused".to_string()).into());
        }

        Ok(())
    }
}

impl Script {
    fn role_refs(&self, selection: &RoleSelection) -> Result<Vec<RoleRef>, AppError> {
        selection
            .ids()
            .iter()
            .map(|id| {
                self.roles
                    .iter()
                    .find(|role| *role.id() == Some(*id))
                    .map(|role| RoleRef::new(*id, role.name().into(), *role.system()))
                    .ok_or_else(|| StaffError::UnknownRole.into())
            })
            .collect()
    }

    fn system_role_ids(&self) -> Vec<i32> {
        self.roles
            .iter()
            .filter(|role| *role.system())
            .filter_map(|role| *role.id())
            .collect()
    }

    fn standing_of(&self, target: &Staff) -> OwnerStanding {
        OwnerStanding {
            target_is_active_owner: target.is_active_owner(),
            other_active_owners: self
                .staff
                .iter()
                .filter(|staff| staff.id() != target.id() && staff.is_active_owner())
                .count() as u64,
        }
    }
}

#[async_trait]
impl RoleRepository for Fakes {
    async fn find_all(&self) -> Result<Vec<Role>, AppError> {
        self.record(Call::FindAllRoles);
        self.guard()?;

        Ok(self.script.lock().expect("script lock").roles.clone())
    }

    async fn find_by_id(&self, id: i32) -> Result<Option<Role>, AppError> {
        self.record(Call::FindRole { id });
        self.guard()?;

        Ok(self.stored_role(id))
    }

    async fn create(&self, entity: &Role) -> Result<Role, AppError> {
        self.record(Call::CreateRole {
            name: entity.name().into(),
        });
        self.guard()?;

        let mut script = self.script.lock().expect("script lock");

        if script.roles.iter().any(|role| role.name() == entity.name()) {
            return Err(StaffError::RoleNameTaken.into());
        }

        let created = Role::rehydrate(
            script.roles.len() as i32 + 1,
            entity.name().clone(),
            entity.description().clone(),
            false,
            entity.permissions().clone(),
            0,
            *entity.created_at(),
            *entity.updated_at(),
        );
        script.roles.push(created.clone());

        Ok(created)
    }

    async fn update(&self, entity: &Role) -> Result<Role, AppError> {
        let id = entity.id().expect("a stored role");

        self.record(Call::UpdateRole { id });
        self.guard()?;

        let mut script = self.script.lock().expect("script lock");

        if script
            .roles
            .iter()
            .any(|role| *role.id() != Some(id) && role.name() == entity.name())
        {
            return Err(StaffError::RoleNameTaken.into());
        }

        let Some(stored) = script.roles.iter_mut().find(|role| *role.id() == Some(id)) else {
            return Err(GlobalAppError::NotFound.into());
        };

        stored.ensure_changeable()?;
        *stored = entity.clone();

        Ok(entity.clone())
    }

    async fn delete(&self, id: i32) -> Result<(), AppError> {
        self.record(Call::DeleteRole { id });
        self.guard()?;

        let mut script = self.script.lock().expect("script lock");

        let Some(stored) = script.roles.iter().find(|role| *role.id() == Some(id)) else {
            return Err(GlobalAppError::NotFound.into());
        };

        stored.ensure_changeable()?;

        if script
            .staff
            .iter()
            .any(|staff| staff.roles().iter().any(|role| *role.id() == id))
        {
            return Err(StaffError::RoleInUse.into());
        }

        script.roles.retain(|role| *role.id() != Some(id));

        Ok(())
    }
}

#[async_trait]
impl StaffRepository for Fakes {
    async fn find_all(&self) -> Result<Vec<Staff>, AppError> {
        self.record(Call::FindAllStaff);
        self.guard()?;

        Ok(self.script.lock().expect("script lock").staff.clone())
    }

    async fn find_by_id(&self, id: i32) -> Result<Option<Staff>, AppError> {
        self.record(Call::FindStaff { id });
        self.guard()?;

        Ok(self.stored_staff(id))
    }

    async fn find_by_email(&self, email: &StaffEmail) -> Result<Option<Staff>, AppError> {
        self.record(Call::FindStaffByEmail {
            email: email.into(),
        });
        self.guard()?;

        Ok(self
            .script
            .lock()
            .expect("script lock")
            .staff
            .iter()
            .find(|staff| staff.email() == email)
            .cloned())
    }

    async fn permissions_granted_to(&self, staff_id: i32) -> Result<Vec<Permission>, AppError> {
        self.record(Call::PermissionsGrantedTo { staff_id });
        self.guard()?;

        let script = self.script.lock().expect("script lock");

        let Some(staff) = script
            .staff
            .iter()
            .find(|staff| *staff.id() == Some(staff_id))
        else {
            return Ok(Vec::new());
        };

        Ok(staff
            .roles()
            .iter()
            .filter_map(|held| {
                script
                    .roles
                    .iter()
                    .find(|role| role.id() == &Some(*held.id()))
            })
            .flat_map(|role| role.permissions().clone())
            .collect())
    }

    async fn create(&self, entity: &Staff, roles: &RoleSelection) -> Result<Staff, AppError> {
        self.record(Call::CreateStaff {
            email: entity.email().into(),
            role_ids: roles.ids().to_vec(),
        });
        self.guard()?;

        let mut script = self.script.lock().expect("script lock");

        if script
            .staff
            .iter()
            .any(|staff| staff.email() == entity.email())
        {
            return Err(StaffError::EmailTaken.into());
        }

        let created = Staff::rehydrate(
            script.staff.len() as i32 + 1,
            entity.email().clone(),
            entity.name().clone(),
            entity.password_hash().clone(),
            *entity.active(),
            script.role_refs(roles)?,
            *entity.created_at(),
            *entity.updated_at(),
        );
        script.staff.push(created.clone());

        Ok(created)
    }

    async fn update(&self, id: i32, change: &StaffChange) -> Result<Staff, AppError> {
        self.record(Call::UpdateStaff {
            id,
            active: *change.active(),
            role_ids: change.roles().ids().to_vec(),
            sets_password: change.password_hash().is_some(),
        });
        self.guard()?;

        let mut script = self.script.lock().expect("script lock");

        let Some(stored) = script
            .staff
            .iter()
            .find(|staff| *staff.id() == Some(id))
            .cloned()
        else {
            return Err(GlobalAppError::NotFound.into());
        };

        script
            .standing_of(&stored)
            .ensure_an_owner_remains(change.keeps_an_active_owner(&script.system_role_ids()))?;

        let updated = Staff::rehydrate(
            id,
            stored.email().clone(),
            change.name().clone(),
            change
                .password_hash()
                .clone()
                .unwrap_or_else(|| stored.password_hash().clone()),
            *change.active(),
            script.role_refs(change.roles())?,
            *stored.created_at(),
            Utc::now(),
        );

        script.staff.retain(|staff| *staff.id() != Some(id));
        script.staff.push(updated.clone());

        Ok(updated)
    }

    async fn delete(&self, id: i32) -> Result<(), AppError> {
        self.record(Call::DeleteStaff { id });
        self.guard()?;

        let mut script = self.script.lock().expect("script lock");

        let Some(stored) = script
            .staff
            .iter()
            .find(|staff| *staff.id() == Some(id))
            .cloned()
        else {
            return Err(GlobalAppError::NotFound.into());
        };

        script.standing_of(&stored).ensure_an_owner_remains(false)?;
        script.staff.retain(|staff| *staff.id() != Some(id));

        Ok(())
    }

    async fn create_owner_if_absent(&self, entity: &Staff) -> Result<bool, AppError> {
        self.record(Call::CreateOwnerIfAbsent {
            email: entity.email().into(),
        });
        self.guard()?;

        let mut script = self.script.lock().expect("script lock");

        if script
            .staff
            .iter()
            .any(|staff| staff.email() == entity.email())
        {
            return Ok(false);
        }

        let created = Staff::rehydrate(
            script.staff.len() as i32 + 1,
            entity.email().clone(),
            entity.name().clone(),
            entity.password_hash().clone(),
            true,
            vec![RoleRef::new(OWNER_ROLE_ID, "Owner".to_string(), true)],
            *entity.created_at(),
            *entity.updated_at(),
        );
        script.staff.push(created);

        Ok(true)
    }
}

#[async_trait]
impl PasswordHasher for Fakes {
    async fn hash(&self, password: &Password) -> Result<PasswordHash, AppError> {
        self.record(Call::HashPassword);

        Ok(hash_of(password.as_str()))
    }

    async fn verify(
        &self,
        password: &Password,
        stored: Option<&PasswordHash>,
    ) -> Result<bool, AppError> {
        self.record(Call::VerifyPassword {
            against_a_stored_hash: stored.is_some(),
        });

        Ok(stored.is_some_and(|stored| *stored == hash_of(password.as_str())))
    }
}

impl StaffTokenIssuer for Fakes {
    fn issue(&self, staff_id: i32) -> Result<String, AppError> {
        self.record(Call::IssueToken { staff_id });

        Ok(format!("token-for-{staff_id}"))
    }
}

pub fn hash_of(password: &str) -> PasswordHash {
    PasswordHash::new(format!("hash-of-{password}")).expect("hash")
}

pub fn owner_role() -> Role {
    Role::rehydrate(
        OWNER_ROLE_ID,
        RoleName::new("Owner".to_string()).expect("role name"),
        None,
        true,
        Permission::all(),
        1,
        Utc::now(),
        Utc::now(),
    )
}

pub fn dam_officer_role() -> Role {
    Role::rehydrate(
        DAM_OFFICER_ROLE_ID,
        RoleName::new("Dam officer".to_string()).expect("role name"),
        None,
        false,
        vec![
            Permission::new(Resource::Dams, Action::Read),
            Permission::new(Resource::Dams, Action::Update),
        ],
        0,
        Utc::now(),
        Utc::now(),
    )
}

/// A stored staff member whose password is [`PASSWORD`].
pub fn a_staff_member(id: i32, email: &str, active: bool, role_ids: &[i32]) -> Staff {
    let roles = role_ids
        .iter()
        .map(|role_id| match *role_id {
            OWNER_ROLE_ID => RoleRef::new(OWNER_ROLE_ID, "Owner".to_string(), true),
            other => RoleRef::new(other, "Dam officer".to_string(), false),
        })
        .collect();

    Staff::rehydrate(
        id,
        StaffEmail::new(email.to_string()).expect("email"),
        StaffName::new("Hiwa K.".to_string()).expect("name"),
        hash_of(PASSWORD),
        active,
        roles,
        Utc::now(),
        Utc::now(),
    )
}

/// The signed-in staff member a use case is acting for.
pub fn actor(staff_id: i32) -> StaffContext {
    StaffContext::new(
        staff_id,
        OWNER_EMAIL.to_string(),
        Permission::all().into_iter().collect(),
    )
}
