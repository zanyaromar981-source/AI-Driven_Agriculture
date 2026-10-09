use crate::{
    features::farmers::domain::{FarmerSearch, PlaceSlug},
    shared::Phone,
};

/// What the staff listing of farmers is ordered by.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FarmerSort {
    #[default]
    CreatedAt,
    Name,
}

/// Which farmers the staff listing shows and in what order. Every filter
/// that is given must match.
#[derive(Clone, Debug, Default)]
pub struct FarmerFilter {
    /// Only the farmer with exactly this phone.
    pub phone: Option<Phone>,
    /// Only farmers whose name or phone contains this text, whatever the
    /// case.
    pub search: Option<FarmerSearch>,
    pub governorate: Option<PlaceSlug>,
    pub zone: Option<PlaceSlug>,
    pub blocked: Option<bool>,
    pub sort: FarmerSort,
    /// `None` is the natural direction of the sort: newest first by time,
    /// A to Z by name.
    pub descending: Option<bool>,
}

impl FarmerFilter {
    pub fn is_descending(&self) -> bool {
        self.descending
            .unwrap_or(matches!(self.sort, FarmerSort::CreatedAt))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn with_no_order_given_time_runs_newest_first_and_names_a_to_z() {
        let by = |sort| FarmerFilter {
            sort,
            ..FarmerFilter::default()
        };

        assert!(by(FarmerSort::CreatedAt).is_descending());
        assert!(!by(FarmerSort::Name).is_descending());
    }

    #[test]
    fn an_order_that_is_given_wins() {
        let filter = FarmerFilter {
            sort: FarmerSort::Name,
            descending: Some(true),
            ..FarmerFilter::default()
        };

        assert!(filter.is_descending());
    }
}
