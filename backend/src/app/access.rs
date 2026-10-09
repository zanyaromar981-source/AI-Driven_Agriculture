//! Who may do what on the dashboard. A staff member holds roles, a role holds
//! permissions, and a permission is one action on one kind of data. Roles
//! are made and removed by staff themselves; the kinds of data and the four
//! actions are fixed here, because each one corresponds to routes in code.

use std::collections::HashSet;

use getset::Getters;

use crate::{app::AppError, shared::DomainError};

/// A kind of data the dashboard manages.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Resource {
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

impl Resource {
    pub const ALL: [Resource; 17] = [
        Resource::Zones,
        Resource::Dams,
        Resource::Outlooks,
        Resource::Water,
        Resource::Fires,
        Resource::Alwa,
        Resource::Farmers,
        Resource::Farms,
        Resource::Insights,
        Resource::Staff,
        Resource::Roles,
        Resource::Briefs,
        Resource::Crops,
        Resource::Rules,
        Resource::Messages,
        Resource::App,
        Resource::Jobs,
    ];
}

impl From<Resource> for String {
    fn from(value: Resource) -> Self {
        match value {
            Resource::Zones => "zones".to_string(),
            Resource::Dams => "dams".to_string(),
            Resource::Outlooks => "outlooks".to_string(),
            Resource::Water => "water".to_string(),
            Resource::Fires => "fires".to_string(),
            Resource::Alwa => "alwa".to_string(),
            Resource::Farmers => "farmers".to_string(),
            Resource::Farms => "farms".to_string(),
            Resource::Insights => "insights".to_string(),
            Resource::Staff => "staff".to_string(),
            Resource::Roles => "roles".to_string(),
            Resource::Briefs => "briefs".to_string(),
            Resource::Crops => "crops".to_string(),
            Resource::Rules => "rules".to_string(),
            Resource::Messages => "messages".to_string(),
            Resource::App => "app".to_string(),
            Resource::Jobs => "jobs".to_string(),
        }
    }
}

impl TryFrom<&str> for Resource {
    type Error = DomainError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "zones" => Ok(Resource::Zones),
            "dams" => Ok(Resource::Dams),
            "outlooks" => Ok(Resource::Outlooks),
            "water" => Ok(Resource::Water),
            "fires" => Ok(Resource::Fires),
            "alwa" => Ok(Resource::Alwa),
            "farmers" => Ok(Resource::Farmers),
            "farms" => Ok(Resource::Farms),
            "insights" => Ok(Resource::Insights),
            "staff" => Ok(Resource::Staff),
            "roles" => Ok(Resource::Roles),
            "briefs" => Ok(Resource::Briefs),
            "crops" => Ok(Resource::Crops),
            "rules" => Ok(Resource::Rules),
            "messages" => Ok(Resource::Messages),
            "app" => Ok(Resource::App),
            "jobs" => Ok(Resource::Jobs),
            _ => Err(DomainError::InvalidValue(format!(
                "Invalid resource: {value}"
            ))),
        }
    }
}

/// What can be done to a resource.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Action {
    Create,
    Read,
    Update,
    Delete,
}

impl Action {
    pub const ALL: [Action; 4] = [Action::Create, Action::Read, Action::Update, Action::Delete];
}

impl From<Action> for String {
    fn from(value: Action) -> Self {
        match value {
            Action::Create => "create".to_string(),
            Action::Read => "read".to_string(),
            Action::Update => "update".to_string(),
            Action::Delete => "delete".to_string(),
        }
    }
}

impl TryFrom<&str> for Action {
    type Error = DomainError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "create" => Ok(Action::Create),
            "read" => Ok(Action::Read),
            "update" => Ok(Action::Update),
            "delete" => Ok(Action::Delete),
            _ => Err(DomainError::InvalidValue(format!(
                "Invalid action: {value}"
            ))),
        }
    }
}

/// One action on one resource, for example "update dams".
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Permission {
    pub resource: Resource,
    pub action: Action,
}

impl Permission {
    pub const fn new(resource: Resource, action: Action) -> Self {
        Self { resource, action }
    }

    /// Every permission there is: what the owner role holds.
    pub fn all() -> Vec<Permission> {
        Resource::ALL
            .into_iter()
            .flat_map(|resource| {
                Action::ALL
                    .into_iter()
                    .map(move |action| Permission::new(resource, action))
            })
            .collect()
    }
}

/// The signed-in staff member and everything their roles allow, worked out
/// afresh on every request so that a change to a role takes effect at once.
#[derive(Clone, Debug, Getters)]
#[getset(get = "pub")]
pub struct StaffContext {
    staff_id: i32,
    email: String,
    permissions: HashSet<Permission>,
}

impl StaffContext {
    pub fn new(staff_id: i32, email: String, permissions: HashSet<Permission>) -> Self {
        Self {
            staff_id,
            email,
            permissions,
        }
    }

    pub fn can(&self, resource: Resource, action: Action) -> bool {
        self.permissions
            .contains(&Permission::new(resource, action))
    }

    pub fn require(&self, resource: Resource, action: Action) -> Result<(), AppError> {
        if self.can(resource, action) {
            Ok(())
        } else {
            Err(AppError::Forbidden)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn staff(permissions: &[Permission]) -> StaffContext {
        StaffContext::new(
            1,
            "officer@example.org".to_string(),
            permissions.iter().copied().collect(),
        )
    }

    /// Every variant must survive a trip to the database and back. A mismatch
    /// between the two directions corrupts rows silently rather than failing.
    #[test]
    fn every_resource_and_action_round_trips() {
        for resource in Resource::ALL {
            let stored = String::from(resource);

            assert_eq!(
                Resource::try_from(stored.as_str()).expect("resource"),
                resource
            );
        }

        for action in Action::ALL {
            let stored = String::from(action);

            assert_eq!(Action::try_from(stored.as_str()).expect("action"), action);
        }
    }

    #[test]
    fn briefs_is_a_resource_stored_under_its_own_name() {
        assert!(Resource::ALL.contains(&Resource::Briefs));
        assert_eq!(String::from(Resource::Briefs), "briefs");
        assert_eq!(
            Resource::try_from("briefs").expect("resource"),
            Resource::Briefs
        );
    }

    #[test]
    fn an_unknown_stored_value_is_rejected_rather_than_defaulted() {
        assert!(Resource::try_from("everything").is_err());
        assert!(Action::try_from("write").is_err());
        assert!(Action::try_from("Read").is_err());
    }

    #[test]
    fn all_permissions_is_every_action_on_every_resource() {
        assert_eq!(
            Permission::all().len(),
            Resource::ALL.len() * Action::ALL.len()
        );
    }

    #[test]
    fn a_permission_allows_exactly_its_own_action_on_its_own_resource() {
        let staff = staff(&[Permission::new(Resource::Dams, Action::Read)]);

        assert!(staff.can(Resource::Dams, Action::Read));
        assert!(
            !staff.can(Resource::Dams, Action::Update),
            "reading must not imply writing"
        );
        assert!(
            !staff.can(Resource::Fires, Action::Read),
            "one resource must not imply another"
        );
    }

    #[test]
    fn requiring_a_missing_permission_is_forbidden() {
        let staff = staff(&[]);

        assert!(matches!(
            staff.require(Resource::Roles, Action::Delete),
            Err(AppError::Forbidden)
        ));
    }
}
