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
                AddStaffInput, CreateRoleInput, EditOwnProfileInput, EditRoleInput, EditStaffInput,
                OwnPasswordInput, SignInInput, SignedInStaff,
            },
        },
        domain::{
            JobTitle, Password, Role, RoleDescription, RoleName, RoleRef, RoleSelection, Staff,
            StaffEmail, StaffError, StaffName,
        },
    },
    shared::{DomainError, Phone},
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
    Briefs,
    Crops,
    Rules,
    Messages,
    App,
    Jobs,
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
            StaffResource::Briefs => Resource::Briefs,
            StaffResource::Crops => Resource::Crops,
            StaffResource::Rules => Resource::Rules,
            StaffResource::Messages => Resource::Messages,
            StaffResource::App => Resource::App,
            StaffResource::Jobs => Resource::Jobs,
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
            Resource::Briefs => StaffResource::Briefs,
            Resource::Crops => StaffResource::Crops,
            Resource::Rules => StaffResource::Rules,
            Resource::Messages => StaffResource::Messages,
            Resource::App => StaffResource::App,
            Resource::Jobs => StaffResource::Jobs,
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

/// A form sends an untouched phone as an empty text, which means the same
/// as none. Anything else must be an Iraqi mobile number.
fn optional_phone(phone: Option<String>) -> Result<Option<Phone>, AppError> {
    Ok(phone
        .filter(|phone| !phone.trim().is_empty())
        .map(Phone::new)
        .transpose()?)
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
    pub phone: Option<String>,
    pub job_title: Option<String>,
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
            phone: staff.phone().as_ref().map(Into::into),
            job_title: staff.job_title().as_ref().map(Into::into),
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
    /// An Iraqi mobile number, for example `+9647501234567`. Optional.
    pub phone: Option<String>,
    /// Up to 80 characters. Optional.
    pub job_title: Option<String>,
}

impl CreateStaffParams {
    pub fn into_input(self) -> Result<AddStaffInput, AppError> {
        Ok(AddStaffInput {
            email: StaffEmail::new(self.email)?,
            name: StaffName::new(self.name)?,
            phone: optional_phone(self.phone)?,
            job_title: JobTitle::optional(self.job_title)?,
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
    /// An Iraqi mobile number. `null`, empty or left out clears it.
    pub phone: Option<String>,
    /// Up to 80 characters. `null`, empty or left out clears it.
    pub job_title: Option<String>,
}

impl UpdateStaffParams {
    pub fn into_input(self) -> Result<EditStaffInput, AppError> {
        Ok(EditStaffInput {
            name: StaffName::new(self.name)?,
            phone: optional_phone(self.phone)?,
            job_title: JobTitle::optional(self.job_title)?,
            active: self.active,
            roles: role_selection(self.role_ids)?,
            password: self.password.map(Password::new).transpose()?,
        })
    }
}

/// What a staff member changes on their own account. Anything else in the
/// body (`email`, `active`, `role_ids`, ...) is refused, not ignored, so
/// that nothing looks changed that was not.
#[derive(Deserialize, Validate, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct UpdateOwnStaffProfileParams {
    pub name: String,
    /// An Iraqi mobile number. `null`, empty or left out clears it.
    pub phone: Option<String>,
    /// The password the account has now. Needed with `new_password`.
    pub current_password: Option<String>,
    /// 10 to 200 characters. Needed with `current_password`. Leave both
    /// out to keep the password.
    pub new_password: Option<String>,
}

impl UpdateOwnStaffProfileParams {
    pub fn into_input(self) -> Result<EditOwnProfileInput, AppError> {
        let password = match (self.current_password, self.new_password) {
            (None, None) => None,
            (Some(current), Some(new)) => Some(OwnPasswordInput {
                // A current password too long to be anyone's is a wrong
                // one, not a hint about how a password must look.
                current: Password::presented(current).map_err(|_| StaffError::WrongPassword)?,
                new: Password::new(new)?,
            }),
            _ => {
                return Err(DomainError::InvalidValue(
                    "current_password and new_password must be sent together".to_string(),
                )
                .into());
            }
        };

        Ok(EditOwnProfileInput {
            name: StaffName::new(self.name)?,
            phone: optional_phone(self.phone)?,
            password,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn own_profile(body: serde_json::Value) -> Result<EditOwnProfileInput, AppError> {
        serde_json::from_value::<UpdateOwnStaffProfileParams>(body)
            .expect("shape")
            .into_input()
    }

    #[test]
    fn the_two_passwords_of_an_own_profile_change_go_together() {
        assert!(
            own_profile(serde_json::json!({"name": "Hiwa"}))
                .expect("valid")
                .password
                .is_none()
        );
        assert!(
            own_profile(serde_json::json!({
                "name": "Hiwa", "current_password": "whatever", "new_password": "long enough new"
            }))
            .expect("valid")
            .password
            .is_some()
        );
        assert!(
            own_profile(serde_json::json!({"name": "Hiwa", "new_password": "long enough new"}))
                .is_err(),
            "a new password without the current one proves nothing"
        );
        assert!(
            own_profile(serde_json::json!({"name": "Hiwa", "current_password": "whatever"}))
                .is_err()
        );
    }

    #[test]
    fn a_new_own_password_follows_the_password_rules_and_the_phone_must_be_a_mobile() {
        assert!(
            own_profile(serde_json::json!({
                "name": "Hiwa", "current_password": "whatever", "new_password": "short"
            }))
            .is_err()
        );
        assert!(own_profile(serde_json::json!({"name": "Hiwa", "phone": "0750"})).is_err());
        assert!(
            own_profile(serde_json::json!({"name": "Hiwa", "phone": ""}))
                .expect("valid")
                .phone
                .is_none()
        );
        assert!(matches!(
            own_profile(serde_json::json!({
                "name": "Hiwa", "current_password": "a".repeat(201), "new_password": "long enough new"
            })),
            Err(AppError::Staff(StaffError::WrongPassword))
        ));
    }

    #[test]
    fn an_own_profile_change_cannot_carry_roles_the_active_state_or_an_email() {
        for extra in ["email", "active", "role_ids", "job_title"] {
            let mut body = serde_json::json!({"name": "Hiwa"});
            body[extra] = serde_json::json!("x");

            assert!(
                serde_json::from_value::<UpdateOwnStaffProfileParams>(body).is_err(),
                "{extra}"
            );
        }
    }

    #[test]
    fn a_staff_member_is_created_and_edited_with_an_optional_phone_and_job_title() {
        let created = serde_json::from_value::<CreateStaffParams>(serde_json::json!({
            "email": "dilan@example.org", "name": "Dilan", "password": "long enough password",
            "phone": "+9647501234567", "job_title": " Dam engineer "
        }))
        .expect("shape")
        .into_input()
        .expect("valid");

        assert_eq!(created.phone.expect("phone").as_str(), "+9647501234567");
        assert_eq!(created.job_title.expect("title").as_str(), "Dam engineer");

        let edit = |extra: (&str, serde_json::Value)| {
            let mut body = serde_json::json!({"name": "Dilan", "active": true, "role_ids": []});
            body[extra.0] = extra.1;

            serde_json::from_value::<UpdateStaffParams>(body)
                .expect("shape")
                .into_input()
        };

        assert!(edit(("phone", serde_json::json!("+9645301234567"))).is_err());
        assert!(edit(("job_title", serde_json::json!("x".repeat(81)))).is_err());
        assert!(
            edit(("phone", serde_json::Value::Null))
                .expect("valid")
                .phone
                .is_none()
        );
    }

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
