use crate::features::farms::domain::{FarmError, GridCell, Point, utm::to_utm_38n};

const MIN_CORNERS: usize = 3;
const MAX_CORNERS: usize = 50;

/// The edge of a farm: the corners the farmer walked, in walking order. The
/// polygon closes itself, so the last corner joins back to the first.
#[derive(Clone, Debug, PartialEq)]
pub struct Outline {
    points: Vec<Point>,
    /// The same corners as `(easting, northing)` in metres, UTM zone 38N.
    ring: Vec<(f64, f64)>,
}

impl Outline {
    pub fn new(mut points: Vec<Point>) -> Result<Self, FarmError> {
        // A corner tapped twice in a row, or the first corner repeated at the
        // end to close the ring, describes the same polygon, so the repeats
        // are dropped rather than rejected.
        points.dedup_by(|next, previous| position(next) == position(previous));

        if points.len() > 1 && points.first().map(position) == points.last().map(position) {
            points.pop();
        }

        if !(MIN_CORNERS..=MAX_CORNERS).contains(&points.len()) {
            return Err(FarmError::OutlineCornerCount {
                min: MIN_CORNERS,
                max: MAX_CORNERS,
            });
        }

        let ring: Vec<(f64, f64)> = points
            .iter()
            .map(|point| to_utm_38n(point.lat(), point.lon()))
            .collect();

        if crosses_itself(&ring) {
            return Err(FarmError::OutlineSelfIntersects);
        }

        Ok(Self { points, ring })
    }

    pub fn points(&self) -> &[Point] {
        &self.points
    }

    pub fn area_m2(&self) -> f64 {
        let doubled: f64 = edges(&self.ring)
            .map(|((x1, y1), (x2, y2))| x1 * y2 - x2 * y1)
            .sum();

        doubled.abs() / 2.0
    }

    pub fn area_dunam(&self) -> f64 {
        self.area_m2() / (GridCell::SIZE_M * GridCell::SIZE_M * GridCell::PER_DUNAM)
    }

    /// The mean of the corners as `(lat, lon)`. Good enough to place a pin
    /// or pick the nearest weather point; it is not the centre of mass.
    pub fn centroid(&self) -> (f64, f64) {
        let count = self.points.len() as f64;

        (
            self.points.iter().map(Point::lat).sum::<f64>() / count,
            self.points.iter().map(Point::lon).sum::<f64>() / count,
        )
    }

    /// A cell belongs to the farm when its centre is inside the outline.
    pub fn contains(&self, cell: &GridCell) -> bool {
        let (x, y) = cell.centre();
        let mut inside = false;

        for ((x1, y1), (x2, y2)) in edges(&self.ring) {
            if (y1 > y) != (y2 > y) && x < (x2 - x1) * (y - y1) / (y2 - y1) + x1 {
                inside = !inside;
            }
        }

        inside
    }

    /// Every cell of the grid that belongs to the farm, row by row from the
    /// south. Refuses before scanning when the outline is larger than
    /// `max_cells`, so an oversized polygon cannot be made to enumerate a
    /// whole province.
    pub fn cells(&self, max_cells: usize) -> Result<Vec<GridCell>, FarmError> {
        let cell_area = GridCell::SIZE_M * GridCell::SIZE_M;

        if self.area_m2() / cell_area > max_cells as f64 {
            return Err(FarmError::TooManyCells(max_cells));
        }

        let index = |metres: f64| (metres / GridCell::SIZE_M).floor() as i32;
        let eastings = self.ring.iter().map(|(x, _)| *x);
        let northings = self.ring.iter().map(|(_, y)| *y);

        let west = index(eastings.clone().fold(f64::INFINITY, f64::min));
        let east = index(eastings.fold(f64::NEG_INFINITY, f64::max));
        let south = index(northings.clone().fold(f64::INFINITY, f64::min));
        let north = index(northings.fold(f64::NEG_INFINITY, f64::max));

        let mut cells = Vec::new();

        for n in south..=north {
            for e in west..=east {
                let cell = GridCell::new(e, n);

                if self.contains(&cell) {
                    cells.push(cell);
                }
            }
        }

        if cells.is_empty() {
            return Err(FarmError::OutlineEnclosesNoCells);
        }

        if cells.len() > max_cells {
            return Err(FarmError::TooManyCells(max_cells));
        }

        Ok(cells)
    }
}

fn position(point: &Point) -> (f64, f64) {
    (point.lat(), point.lon())
}

fn edges(ring: &[(f64, f64)]) -> impl Iterator<Item = ((f64, f64), (f64, f64))> + '_ {
    ring.iter()
        .zip(ring.iter().cycle().skip(1))
        .map(|(start, end)| (*start, *end))
}

fn crosses_itself(ring: &[(f64, f64)]) -> bool {
    let edges: Vec<_> = edges(ring).collect();
    let count = edges.len();

    for first in 0..count {
        for second in (first + 1)..count {
            // Neighbouring edges share a corner by construction.
            let neighbours = second == first + 1 || (first == 0 && second == count - 1);

            if neighbours {
                continue;
            }

            if segments_touch(edges[first], edges[second]) {
                return true;
            }
        }
    }

    false
}

fn segments_touch(first: ((f64, f64), (f64, f64)), second: ((f64, f64), (f64, f64))) -> bool {
    let (a, b) = first;
    let (c, d) = second;

    let d1 = turn(c, d, a);
    let d2 = turn(c, d, b);
    let d3 = turn(a, b, c);
    let d4 = turn(a, b, d);

    if ((d1 > 0.0 && d2 < 0.0) || (d1 < 0.0 && d2 > 0.0))
        && ((d3 > 0.0 && d4 < 0.0) || (d3 < 0.0 && d4 > 0.0))
    {
        return true;
    }

    (d1 == 0.0 && within(c, d, a))
        || (d2 == 0.0 && within(c, d, b))
        || (d3 == 0.0 && within(a, b, c))
        || (d4 == 0.0 && within(a, b, d))
}

/// Positive when `point` is to the left of the line from `start` to `end`.
fn turn(start: (f64, f64), end: (f64, f64), point: (f64, f64)) -> f64 {
    (end.0 - start.0) * (point.1 - start.1) - (end.1 - start.1) * (point.0 - start.0)
}

fn within(start: (f64, f64), end: (f64, f64), point: (f64, f64)) -> bool {
    point.0 >= start.0.min(end.0)
        && point.0 <= start.0.max(end.0)
        && point.1 >= start.1.min(end.1)
        && point.1 <= start.1.max(end.1)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn point(lat: f64, lon: f64) -> Point {
        Point::new(lat, lon, None, None).expect("point")
    }

    /// Roughly 90 m east to west and 111 m south to north.
    fn a_field() -> Vec<Point> {
        vec![
            point(36.0300, 44.6000),
            point(36.0300, 44.6010),
            point(36.0310, 44.6010),
            point(36.0310, 44.6000),
        ]
    }

    #[test]
    fn a_walked_field_is_a_valid_outline() {
        assert!(Outline::new(a_field()).is_ok());
    }

    #[test]
    fn fewer_than_three_corners_is_not_a_field() {
        let two = a_field().into_iter().take(2).collect();

        assert!(matches!(
            Outline::new(two),
            Err(FarmError::OutlineCornerCount { .. })
        ));
    }

    #[test]
    fn more_than_fifty_corners_is_refused() {
        let many = (0..51)
            .map(|step| point(36.03 + f64::from(step) * 0.0001, 44.60))
            .collect();

        assert!(matches!(
            Outline::new(many),
            Err(FarmError::OutlineCornerCount { .. })
        ));
    }

    #[test]
    fn a_figure_of_eight_crosses_itself() {
        let mut corners = a_field();
        corners.swap(2, 3);

        assert!(matches!(
            Outline::new(corners),
            Err(FarmError::OutlineSelfIntersects)
        ));
    }

    #[test]
    fn a_corner_tapped_twice_is_one_corner() {
        let mut corners = a_field();
        corners.insert(1, corners[1]);

        let outline = Outline::new(corners).expect("outline");

        assert_eq!(outline.points().len(), 4);
    }

    #[test]
    fn repeating_the_first_corner_at_the_end_is_the_same_field() {
        let mut closed = a_field();
        closed.push(closed[0]);

        let outline = Outline::new(closed).expect("outline");

        assert_eq!(outline.points().len(), 4);
    }

    #[test]
    fn the_walking_direction_does_not_change_the_area() {
        let clockwise: Vec<Point> = a_field().into_iter().rev().collect();

        let one_way = Outline::new(a_field()).expect("outline").area_m2();
        let other_way = Outline::new(clockwise).expect("outline").area_m2();

        assert!((one_way - other_way).abs() < 1e-6);
        assert!(
            (9_000.0..11_000.0).contains(&one_way),
            "about 90 m by 111 m, got {one_way}"
        );
    }

    #[test]
    fn the_cells_cover_about_the_area_of_the_outline() {
        let outline = Outline::new(a_field()).expect("outline");

        let cells = outline.cells(1_000).expect("cells");
        let covered = cells.len() as f64 * 100.0;

        assert!(
            (covered - outline.area_m2()).abs() < outline.area_m2() * 0.1,
            "{covered} square metres of cells for {} of outline",
            outline.area_m2()
        );
    }

    #[test]
    fn every_cell_it_lists_is_inside_and_its_neighbour_outside_is_not() {
        let outline = Outline::new(a_field()).expect("outline");
        let cells = outline.cells(1_000).expect("cells");

        assert!(cells.iter().all(|cell| outline.contains(cell)));

        let west_edge = cells.iter().map(GridCell::e).min().expect("a cell");

        assert!(!outline.contains(&GridCell::new(west_edge - 1, cells[0].n())));
    }

    #[test]
    fn an_outline_over_the_cell_limit_is_refused() {
        let outline = Outline::new(a_field()).expect("outline");

        assert!(matches!(
            outline.cells(10),
            Err(FarmError::TooManyCells(10))
        ));
    }

    #[test]
    fn a_sliver_too_thin_to_hold_a_cell_centre_is_refused() {
        let sliver = vec![
            point(36.030_00, 44.600_00),
            point(36.030_00, 44.600_02),
            point(36.030_01, 44.600_01),
        ];

        assert!(matches!(
            Outline::new(sliver).expect("outline").cells(1_000),
            Err(FarmError::OutlineEnclosesNoCells)
        ));
    }

    #[test]
    fn the_centroid_sits_between_the_corners() {
        let (lat, lon) = Outline::new(a_field()).expect("outline").centroid();

        assert!((lat - 36.0305).abs() < 1e-9);
        assert!((lon - 44.6005).abs() < 1e-9);
    }
}
