use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

use crate::{
    app::{Action, Permission, Resource, StaffContext},
    features::staff::{
        app::{
            AppError,
            use_cases::{
                AddStaffInput, CreateRoleInput, EditRoleInput, EditStaffInput, SignInInput,
                SignedInStaff,
            },
        },
        domain::{
            Password, Role, RoleDescription, RoleName, RoleRef, RoleSelection, Staff, StaffEmail,
            StaffError, StaffName,
        },
    },
};

/// A kind of data the dashboard manages.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum StaffResource {
    Zones,
    Dams,
    Outlooks,
    Water,
    Fires,
    Alwa,
    Farmers,
    Farms,
    Insights,
    Staff,
    Roles,
}

impl From<StaffResource> for Resource {
    fn from(value: StaffResource) -> Self {
        match value {
            StaffResource::Zones => Resource::Zones,
            StaffResource::Dams => Resource::Dams,
            StaffResource::Outlooks => Resource::Outlooks,
            StaffResource::Water => Resource::Water,
            StaffResource::Fires => Resource::Fires,
            StaffResource::Alwa => Resource::Alwa,
            StaffResource::Farmers => Resource::Farmers,
            StaffResource::Farms => Resource::Farms,
            StaffResource::Insights => Resource::Insights,
            StaffResource::Staff => Resource::Staff,
            StaffResource::Roles => Resource::Roles,
        }
    }
}

impl From<Resource> for StaffResource {
    fn from(value: Resource) -> Self {
        match value {
            Resource::Zones => StaffResource::Zones,
            Resource::Dams => StaffResource::Dams,
            Resource::Outlooks => StaffResource::Outlooks,
            Resource::Water => StaffResource::Water,
            Resource::Fires => StaffResource::Fires,
            Resource::Alwa => StaffResource::Alwa,
            Resource::Farmers => StaffResource::Farmers,
            Resource::Farms => StaffResource::Farms,
            Resource::Insights => StaffResource::Insights,
            Resource::Staff => StaffResource::Staff,
            Resource::Roles => StaffResource::Roles,
        }
    }
}

/// What can be done to a resource.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum StaffAction {
    Create,
    Read,
    Update,
    Delete,
}

impl From<StaffAction> for Action {
    fn from(value: StaffAction) -> Self {
        match value {
            StaffAction::Create => Action::Create,
            StaffAction::Read => Action::Read,
            StaffAction::Update => Action::Update,
            StaffAction::Delete => Action::Delete,
        }
    }
}

impl From<Action> for StaffAction {
    fn from(value: Action) -> Self {
        match value {
            Action::Create => StaffAction::Create,
            Action::Read => StaffAction::Read,
            Action::Update => StaffAction::Update,
            Action::Delete => StaffAction::Delete,
        }
    }
}

/// One action on one resource. Sent when a role is saved, and returned for
/// roles and for the signed-in staff member.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, ToSchema)]
pub struct StaffPermission {
    pub resource: StaffResource,
    pub action: StaffAction,
}

impl From<StaffPermission> for Permission {
    fn from(value: StaffPermission) -> Self {
        Permission::new(value.resource.into(), value.action.into())
    }
}

impl From<&Permission> for StaffPermission {
    fn from(value: &Permission) -> Self {
        Self {
            resource: value.resource.into(),
            action: value.action.into(),
        }
    }
}

/// What a role editor can choose from.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct StaffPermissionCatalogueResponse {
    pub resources: Vec<StaffResource>,
    pub actions: Vec<StaffAction>,
}

impl StaffPermissionCatalogueResponse {
    pub fn everything() -> Self {
        Self {
            resources: Resource::ALL.into_iter().map(Into::into).collect(),
            actions: Action::ALL.into_iter().map(Into::into).collect(),
        }
    }
}

/// Role ids travel as strings. One that is not a number cannot name a role.
fn role_selection(role_ids: Vec<String>) -> Result<RoleSelection, AppError> {
    let role_ids = role_ids
        .iter()
        .map(|role_id| role_id.parse::<i32>().map_err(|_| StaffError::UnknownRole))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(RoleSelection::new(role_ids)?)
}

// The params that carry a password derive neither `Debug` nor `Serialize`,
// so a password cannot be printed or echoed by accident.

#[derive(Deserialize, Validate, ToSchema)]
pub struct StaffLoginParams {
    pub email: String,
    pub password: String,
}

impl StaffLoginParams {
    /// An email or password that could not belong to any account is
    /// answered like a wrong one, not with a hint about its shape.
    pub fn into_input(self) -> Result<SignInInput, AppError> {
        Ok(SignInInput {
            email: StaffEmail::new(self.email).map_err(|_| StaffError::BadCredentials)?,
            password: Password::presented(self.password)?,
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct StaffRoleRefResponse {
    pub id: String,
    pub name: String,
}

impl From<&RoleRef> for StaffRoleRefResponse {
    fn from(role: &RoleRef) -> Self {
        Self {
            id: role.id().to_string(),
            name: role.name().clone(),
        }
    }
}

/// A staff member. The password hash is never part of it.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct StaffResponse {
    pub id: String,
    pub email: String,
    pub name: String,
    pub active: bool,
    pub roles: Vec<StaffRoleRefResponse>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<&Staff> for StaffResponse {
    fn from(staff: &Staff) -> Self {
        Self {
            id: staff.id().unwrap_or_default().to_string(),
            email: staff.email().into(),
            name: staff.name().into(),
            active: *staff.active(),
            roles: staff.roles().iter().map(Into::into).collect(),
            created_at: *staff.created_at(),
            updated_at: *staff.updated_at(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct StaffOneResponse {
    pub staff: StaffResponse,
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct StaffListResponse {
    pub staff: Vec<StaffResponse>,
}

#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct StaffSignedInResponse {
    pub token: String,
    pub staff: StaffResponse,
    pub permissions: Vec<StaffPermission>,
}

impl From<&SignedInStaff> for StaffSignedInResponse {
    fn from(signed_in: &SignedInStaff) -> Self {
        Self {
            token: signed_in.token.clone(),
            staff: signed_in.access.staff().into(),
            permissions: signed_in
                .access
                .permissions()
                .iter()
                .map(Into::into)
                .collect(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct StaffMeResponse {
    pub staff: StaffResponse,
    pub permissions: Vec<StaffPermission>,
}

impl From<(&Staff, &StaffContext)> for StaffMeResponse {
    /// The permissions are the ones this very request was let in with.
    fn from((staff, context): (&Staff, &StaffContext)) -> Self {
        let mut permissions: Vec<Permission> = context.permissions().iter().copied().collect();
        permissions.sort_unstable();

        Self {
            staff: staff.into(),
            permissions: permissions.iter().map(Into::into).collect(),
        }
    }
}

#[derive(Serialize, Deserialize, Validate, Debug, Clone, ToSchema)]
pub struct SaveStaffRoleParams {
    pub name: String,
    /// Send `null` or an empty text for no description.
    pub description: Option<String>,
    /// The whole set: on an edit, a permission left out is removed.
    pub permissions: Vec<StaffPermission>,
}

impl SaveStaffRoleParams {
    pub fn into_create_input(self) -> Result<CreateRoleInput, AppError> {
        Ok(CreateRoleInput {
            name: RoleName::new(self.name)?,
            description: RoleDescription::optional(self.description)?,
            permissions: self.permissions.into_iter().map(Into::into).collect(),
        })
    }

    pub fn into_edit_input(self) -> Result<EditRoleInput, AppError> {
        Ok(EditRoleInput {
            name: RoleName::new(self.name)?,
            description: RoleDescription::optional(self.description)?,
            permissions: self.permissions.into_iter().map(Into::into).collect(),
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct StaffRoleResponse {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    /// A system role cannot be edited or deleted.
    pub system: bool,
    pub permissions: Vec<StaffPermission>,
    pub staff_count: u64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<&Role> for StaffRoleResponse {
    fn from(role: &Role) -> Self {
        Self {
            id: role.id().unwrap_or_default().to_string(),
            name: role.name().into(),
            description: role.description().as_ref().map(Into::into),
            system: *role.system(),
            permissions: role.permissions().iter().map(Into::into).collect(),
            staff_count: *role.staff_count(),
            created_at: *role.created_at(),
            updated_at: *role.updated_at(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct StaffOneRoleResponse {
    pub role: StaffRoleResponse,
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct StaffRolesResponse {
    pub roles: Vec<StaffRoleResponse>,
}

#[derive(Deserialize, Validate, ToSchema)]
pub struct CreateStaffParams {
    pub email: String,
    pub name: String,
    /// 10 to 200 characters.
    pub password: String,
    /// Role ids, as strings.
    #[serde(default)]
    pub role_ids: Vec<String>,
}

impl CreateStaffParams {
    pub fn into_input(self) -> Result<AddStaffInput, AppError> {
        Ok(AddStaffInput {
            email: StaffEmail::new(self.email)?,
            name: StaffName::new(self.name)?,
            password: Password::new(self.password)?,
            roles: role_selection(self.role_ids)?,
        })
    }
}

#[derive(Deserialize, Validate, ToSchema)]
pub struct UpdateStaffParams {
    pub name: String,
    pub active: bool,
    /// The whole set of role ids, as strings: a role left out is removed.
    pub role_ids: Vec<String>,
    /// Leave it out to keep the password the account has.
    pub password: Option<String>,
}

impl UpdateStaffParams {
    pub fn into_input(self) -> Result<EditStaffInput, AppError> {
        Ok(EditStaffInput {
            name: StaffName::new(self.name)?,
            active: self.active,
            roles: role_selection(self.role_ids)?,
            password: self.password.map(Password::new).transpose()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_resource_and_action_survives_the_trip_through_its_dto() {
        for resource in Resource::ALL {
            assert_eq!(Resource::from(StaffResource::from(resource)), resource);
        }

        for action in Action::ALL {
            assert_eq!(Action::from(StaffAction::from(action)), action);
        }
    }

    #[test]
    fn a_dto_resource_is_spelled_as_it_is_stored() {
        for resource in Resource::ALL {
            let sent = serde_json::to_value(StaffResource::from(resource)).expect("json");

            assert_eq!(sent, serde_json::Value::String(String::from(resource)));
        }

        for action in Action::ALL {
            let sent = serde_json::to_value(StaffAction::from(action)).expect("json");

            assert_eq!(sent, serde_json::Value::String(String::from(action)));
        }
    }

    #[test]
    fn the_catalogue_lists_every_resource_and_action() {
        let catalogue = StaffPermissionCatalogueResponse::everything();

        assert_eq!(catalogue.resources.len(), Resource::ALL.len());
        assert_eq!(catalogue.actions.len(), Action::ALL.len());
    }

    #[test]
    fn a_role_id_that_is_not_a_number_is_an_unknown_role() {
        assert!(matches!(
            role_selection(vec!["1".to_string(), "abc".to_string()]),
            Err(AppError::Staff(StaffError::UnknownRole))
        ));
        assert_eq!(
            role_selection(vec!["2".to_string(), "1".to_string()])
                .expect("roles")
                .ids(),
            [1, 2]
        );
    }

    #[test]
    fn a_malformed_email_at_sign_in_is_just_bad_credentials() {
        let params = StaffLoginParams {
            email: "not-an-email".to_string(),
            password: "whatever it is".to_string(),
        };

        assert!(matches!(
            params.into_input(),
            Err(AppError::Staff(StaffError::BadCredentials))
        ));
    }
}
