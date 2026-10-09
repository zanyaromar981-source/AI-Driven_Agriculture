use std::collections::HashSet;

use crate::features::farms::domain::{Crop, FarmError};

/// The crops staff have switched on, as they were when a request began. New
/// data may only be painted with these; what is already stored with a crop
/// that was switched off since is left alone.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ActiveCrops(HashSet<Crop>);

impl ActiveCrops {
    pub fn new(crops: impl IntoIterator<Item = Crop>) -> Self {
        Self(crops.into_iter().collect())
    }

    /// Refuses the first crop that is not switched on, by name. The empty
    /// mark is always allowed: unpainting a cell needs no crop.
    pub fn allow(&self, crops: impl IntoIterator<Item = Crop>) -> Result<(), FarmError> {
        for crop in crops {
            if !crop.is_empty() && !self.0.contains(&crop) {
                return Err(FarmError::UnknownCrop(String::from(crop)));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn active() -> ActiveCrops {
        ActiveCrops::new([Crop::of("wheat"), Crop::of("barley")])
    }

    #[test]
    fn crops_that_are_switched_on_are_allowed() {
        assert!(
            active()
                .allow([Crop::of("wheat"), Crop::of("barley")])
                .is_ok()
        );
        assert!(active().allow([]).is_ok());
    }

    #[test]
    fn a_crop_that_is_not_in_the_list_is_refused_by_name() {
        let result = active().allow([Crop::of("wheat"), Crop::of("rice")]);

        assert!(matches!(result, Err(FarmError::UnknownCrop(code)) if code == "rice"));
    }

    #[test]
    fn the_empty_mark_is_allowed_though_it_is_never_in_the_list() {
        assert!(active().allow([Crop::EMPTY]).is_ok());
        assert!(ActiveCrops::default().allow([Crop::EMPTY]).is_ok());
    }

    #[test]
    fn with_no_crop_switched_on_every_crop_is_refused() {
        assert!(ActiveCrops::default().allow([Crop::of("wheat")]).is_err());
    }
}
