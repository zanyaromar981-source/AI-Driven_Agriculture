use std::collections::HashMap;

use crate::features::alwa::domain::{AlwaError, Crop, Product};

/// The products staff have switched on, crops and all the rest: the only
/// ones a new listing or a new price may name. What is already stored keeps
/// its product when staff switch that product off.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ActiveCrops(HashMap<Crop, Product>);

impl ActiveCrops {
    pub fn new(products: impl IntoIterator<Item = Product>) -> Self {
        Self(
            products
                .into_iter()
                .map(|product| (product.code(), product))
                .collect(),
        )
    }

    /// The product a new listing or price names, with its group and unit,
    /// or the refusal that names the code.
    pub fn allow(&self, crop: Crop) -> Result<Product, AlwaError> {
        self.0
            .get(&crop)
            .copied()
            .ok_or_else(|| AlwaError::UnknownCrop(String::from(crop)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::alwa::domain::{ProductGroup, Unit};

    fn active() -> ActiveCrops {
        ActiveCrops::new([
            Product::crop("wheat"),
            Product::crop("tomato"),
            Product::of("eggs", ProductGroup::FishMeatEggs, Unit::Tray30),
        ])
    }

    #[test]
    fn a_crop_that_is_switched_on_is_allowed() {
        assert!(active().allow(Crop::of("wheat")).is_ok());
        assert!(active().allow(Crop::of("tomato")).is_ok());
    }

    #[test]
    fn an_allowed_product_comes_with_its_group_and_unit() {
        let eggs = active().allow(Crop::of("eggs")).expect("eggs");

        assert_eq!(eggs.group(), ProductGroup::FishMeatEggs);
        assert_eq!(eggs.unit(), Unit::Tray30);

        let wheat = active().allow(Crop::of("wheat")).expect("wheat");

        assert_eq!(wheat.group(), ProductGroup::Crops);
        assert_eq!(wheat.unit(), Unit::Kg);
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
