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

    pub const AREA_M2: f64 = Self::SIZE_M * Self::SIZE_M;

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

    /// The south-west corner of the cell as `(easting, northing)` in metres.
    pub fn south_west(&self) -> (f64, f64) {
        (
            f64::from(self.e) * Self::SIZE_M,
            f64::from(self.n) * Self::SIZE_M,
        )
    }

    /// The dunams in cells whose shares inside an outline add up to
    /// `inside_pct`: one whole cell is 100, so 2,500 is one dunam.
    pub fn dunams(inside_pct: f64) -> f64 {
        inside_pct / 100.0 / Self::PER_DUNAM
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
    fn the_south_west_corner_is_the_cell_index_times_ten_metres() {
        assert_eq!(
            GridCell::new(46_415, 398_748).south_west(),
            (464_150.0, 3_987_480.0)
        );
    }

    #[test]
    fn twenty_five_whole_cells_make_one_dunam() {
        assert_eq!(GridCell::dunams(25.0 * 100.0), 1.0);
        assert_eq!(GridCell::dunams(3_000.0 * 100.0), 120.0);
    }

    #[test]
    fn half_a_cell_is_half_a_cells_share_of_a_dunam() {
        assert_eq!(GridCell::dunams(50.0), 0.02);
    }
}
