use getset::CopyGetters;

use crate::features::farms::domain::GridCell;

/// A cell the outline touches and how much of it lies inside the outline.
#[derive(Clone, Copy, Debug, PartialEq, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct TouchedCell {
    position: GridCell,
    /// The share of the cell's 100 square metres inside the outline, above 0
    /// and at most 100. It is not rounded: the app adds these up and must
    /// arrive at the area of the outline.
    inside_pct: f64,
}

impl TouchedCell {
    pub fn new(position: GridCell, inside_m2: f64) -> Self {
        Self {
            position,
            inside_pct: inside_m2 / GridCell::AREA_M2 * 100.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_quarter_of_a_cell_inside_is_twenty_five_percent() {
        let cell = TouchedCell::new(GridCell::new(46_415, 398_748), 25.0);

        assert_eq!(cell.inside_pct(), 25.0);
    }
}
