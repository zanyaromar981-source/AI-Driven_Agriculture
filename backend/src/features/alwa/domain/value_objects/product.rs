use crate::features::alwa::domain::{Crop, ProductGroup, Unit};

/// Something that can be sold at the Marketplace, as the crops feature
/// describes it right now: its code, the shelf it stands on, and what one
/// of it is. A listing copies the group and the unit when it is posted and
/// a price the unit when it is typed in, so both keep their meaning if
/// staff edit the product later.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Product {
    code: Crop,
    group: ProductGroup,
    unit: Unit,
}

impl Product {
    pub fn new(code: Crop, group: ProductGroup, unit: Unit) -> Self {
        Self { code, group, unit }
    }

    pub fn code(&self) -> Crop {
        self.code
    }

    pub fn group(&self) -> ProductGroup {
        self.group
    }

    pub fn unit(&self) -> Unit {
        self.unit
    }

    /// A crop sold by the kg for a test, as every product was before the
    /// Marketplace.
    #[cfg(test)]
    pub fn crop(code: &str) -> Self {
        Self::new(Crop::of(code), ProductGroup::Crops, Unit::Kg)
    }

    /// Any product for a test.
    #[cfg(test)]
    pub fn of(code: &str, group: ProductGroup, unit: Unit) -> Self {
        Self::new(Crop::of(code), group, unit)
    }
}
