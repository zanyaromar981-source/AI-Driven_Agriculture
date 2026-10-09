/// The 10 m cell of the farm the farmer tapped before asking, on the grid
/// of `BACKEND.md` section 1: `e = floor(easting / 10)`,
/// `n = floor(northing / 10)` in UTM zone 38N.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TappedCell {
    e: i32,
    n: i32,
}

impl TappedCell {
    pub fn new(e: i32, n: i32) -> Self {
        Self { e, n }
    }

    pub fn e(&self) -> i32 {
        self.e
    }

    pub fn n(&self) -> i32 {
        self.n
    }
}
