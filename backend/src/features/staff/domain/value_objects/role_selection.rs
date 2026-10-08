use crate::{features::staff::domain::StaffError, shared::DomainError};

const MAX_ROLES: usize = 50;

/// The roles chosen for one staff member: each role once, in a fixed order.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct RoleSelection(Vec<i32>);

impl RoleSelection {
    pub fn new(mut role_ids: Vec<i32>) -> Result<Self, StaffError> {
        role_ids.sort_unstable();
        role_ids.dedup();

        if role_ids.len() > MAX_ROLES {
            return Err(DomainError::InvalidValue(format!(
                "A staff member can hold {MAX_ROLES} roles at most"
            ))
            .into());
        }

        Ok(Self(role_ids))
    }

    pub fn ids(&self) -> &[i32] {
        &self.0
    }

    pub fn contains(&self, role_id: i32) -> bool {
        self.0.binary_search(&role_id).is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_role_chosen_twice_counts_once() {
        let selection = RoleSelection::new(vec![3, 1, 3, 2, 1]).expect("selection");

        assert_eq!(selection.ids(), [1, 2, 3]);
        assert!(selection.contains(2));
        assert!(!selection.contains(4));
    }

    #[test]
    fn no_roles_at_all_is_a_valid_choice() {
        assert!(
            RoleSelection::new(Vec::new())
                .expect("empty")
                .ids()
                .is_empty()
        );
    }

    #[test]
    fn the_number_of_roles_is_bounded() {
        assert!(RoleSelection::new((1..=MAX_ROLES as i32).collect()).is_ok());
        assert!(RoleSelection::new((1..=MAX_ROLES as i32 + 1).collect()).is_err());
    }
}
