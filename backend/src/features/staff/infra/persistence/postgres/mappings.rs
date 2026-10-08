use sea_orm::ActiveValue::{NotSet, Set};

use crate::{
    app::{Action, Permission, Resource},
    features::staff::{
        app::AppError,
        domain::{
            PasswordHash, Role, RoleDescription, RoleName, RoleRef, Staff, StaffEmail, StaffName,
        },
        infra::persistence::postgres::entities::{role_permissions, roles, staff, staff_roles},
    },
};

pub fn permission_from(model: &role_permissions::Model) -> Result<Permission, AppError> {
    Ok(Permission::new(
        Resource::try_from(model.resource.as_str())?,
        Action::try_from(model.action.as_str())?,
    ))
}

pub fn permission_active_model(
    role_id: i32,
    permission: &Permission,
) -> role_permissions::ActiveModel {
    role_permissions::ActiveModel {
        role_id: Set(role_id),
        resource: Set(permission.resource.into()),
        action: Set(permission.action.into()),
    }
}

pub fn staff_role_active_model(staff_id: i32, role_id: i32) -> staff_roles::ActiveModel {
    staff_roles::ActiveModel {
        staff_id: Set(staff_id),
        role_id: Set(role_id),
    }
}

/// A role row, its permission rows and the number of staff holding it.
impl TryFrom<(roles::Model, Vec<role_permissions::Model>, u64)> for Role {
    type Error = AppError;

    fn try_from(
        (model, permissions, staff_count): (roles::Model, Vec<role_permissions::Model>, u64),
    ) -> Result<Self, Self::Error> {
        let permissions = permissions
            .iter()
            .map(permission_from)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Role::rehydrate(
            model.id,
            RoleName::new(model.name)?,
            model.description.map(RoleDescription::new).transpose()?,
            model.system,
            permissions,
            staff_count,
            model.created_at.and_utc(),
            model.updated_at.and_utc(),
        ))
    }
}

impl From<&Role> for roles::ActiveModel {
    fn from(role: &Role) -> Self {
        roles::ActiveModel {
            id: match *role.id() {
                Some(id) => Set(id),
                None => NotSet,
            },
            name: Set(role.name().into()),
            description: Set(role.description().as_ref().map(Into::into)),
            system: Set(*role.system()),
            created_at: Set(role.created_at().naive_utc()),
            updated_at: Set(role.updated_at().naive_utc()),
        }
    }
}

impl From<&roles::Model> for RoleRef {
    fn from(model: &roles::Model) -> Self {
        RoleRef::new(model.id, model.name.clone(), model.system)
    }
}

/// A staff row and the roles it holds.
impl TryFrom<(staff::Model, Vec<RoleRef>)> for Staff {
    type Error = AppError;

    fn try_from((model, roles): (staff::Model, Vec<RoleRef>)) -> Result<Self, Self::Error> {
        Ok(Staff::rehydrate(
            model.id,
            StaffEmail::new(model.email)?,
            StaffName::new(model.name)?,
            PasswordHash::new(model.password_hash)?,
            model.active,
            roles,
            model.created_at.and_utc(),
            model.updated_at.and_utc(),
        ))
    }
}

impl From<&Staff> for staff::ActiveModel {
    fn from(entity: &Staff) -> Self {
        staff::ActiveModel {
            id: match *entity.id() {
                Some(id) => Set(id),
                None => NotSet,
            },
            email: Set(entity.email().into()),
            name: Set(entity.name().into()),
            password_hash: Set(entity.password_hash().as_str().to_string()),
            active: Set(*entity.active()),
            created_at: Set(entity.created_at().naive_utc()),
            updated_at: Set(entity.updated_at().naive_utc()),
        }
    }
}
