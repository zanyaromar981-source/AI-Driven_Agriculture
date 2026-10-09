use std::collections::HashSet;

use crate::features::alwa::domain::{AlwaError, Crop};

/// The crops staff have switched on, as they were when a request began. A
/// new listing and a new or changed price may only name one of these; what
/// is already stored with a crop that was switched off since is left alone.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ActiveCrops(HashSet<Crop>);

impl ActiveCrops {
    pub fn new(crops: impl IntoIterator<Item = Crop>) -> Self {
        Self(crops.into_iter().collect())
    }

    /// Refuses a crop that is not switched on, by name.
    pub fn allow(&self, crop: Crop) -> Result<(), AlwaError> {
        if !self.0.contains(&crop) {
            return Err(AlwaError::UnknownCrop(String::from(crop)));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn active() -> ActiveCrops {
        ActiveCrops::new([Crop::of("wheat"), Crop::of("tomato")])
    }

    #[test]
    fn a_crop_that_is_switched_on_is_allowed() {
        assert!(active().allow(Crop::of("wheat")).is_ok());
        assert!(active().allow(Crop::of("tomato")).is_ok());
    }

    #[test]
    fn a_crop_that_is_not_in_the_list_is_refused_by_name() {
        let result = active().allow(Crop::of("rice"));

        assert!(matches!(result, Err(AlwaError::UnknownCrop(code)) if code == "rice"));
    }

    #[test]
    fn with_no_crop_switched_on_every_crop_is_refused() {
        assert!(ActiveCrops::default().allow(Crop::of("wheat")).is_err());
    }
}
