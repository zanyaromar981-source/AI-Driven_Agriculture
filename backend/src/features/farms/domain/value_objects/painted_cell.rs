use getset::CopyGetters;

use crate::features::farms::domain::{Crop, GridCell};

/// A cell the farmer painted with a crop in the app.
#[derive(Clone, Copy, Debug, PartialEq, Eq, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct PaintedCell {
    position: GridCell,
    crop: Crop,
}

impl PaintedCell {
    pub fn new(position: GridCell, crop: Crop) -> Self {
        Self { position, crop }
    }
}
