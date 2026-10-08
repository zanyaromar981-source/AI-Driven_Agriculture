/// One 10 m square of the Sentinel-2 pixel grid in UTM zone 38N:
/// `e = floor(easting / 10)`, `n = floor(northing / 10)`. One cell is one
/// satellite pixel.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct GridCell {
    e: i32,
    n: i32,
}

impl GridCell {
    pub const SIZE_M: f64 = 10.0;

    /// 1 dunam = 2,500 square metres = 25 cells.
    pub const PER_DUNAM: f64 = 25.0;

    pub fn new(e: i32, n: i32) -> Self {
        Self { e, n }
    }

    pub fn e(&self) -> i32 {
        self.e
    }

    pub fn n(&self) -> i32 {
        self.n
    }

    /// The centre of the cell as `(easting, northing)` in metres.
    pub fn centre(&self) -> (f64, f64) {
        (
            (f64::from(self.e) + 0.5) * Self::SIZE_M,
            (f64::from(self.n) + 0.5) * Self::SIZE_M,
        )
    }

    pub fn dunams(cells: usize) -> f64 {
        cells as f64 / Self::PER_DUNAM
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_centre_is_five_metres_in_from_the_south_west_corner() {
        assert_eq!(
            GridCell::new(46_415, 398_748).centre(),
            (464_155.0, 3_987_485.0)
        );
    }

    #[test]
    fn twenty_five_cells_make_one_dunam() {
        assert_eq!(GridCell::dunams(25), 1.0);
        assert_eq!(GridCell::dunams(3_000), 120.0);
    }
}
