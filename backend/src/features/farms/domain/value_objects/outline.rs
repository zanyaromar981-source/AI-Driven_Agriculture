use crate::features::farms::domain::{FarmError, GridCell, Point, TouchedCell, utm::to_utm_38n};

const MIN_CORNERS: usize = 3;
const MAX_CORNERS: usize = 50;

/// How many cells the box around an outline may hold for each cell the farm
/// is allowed. A long diagonal field fills a small share of its box, so the
/// box is given room; eight times is far more than any real field needs.
const BOX_CELLS_PER_CELL: i64 = 8;

/// A cell with this many square metres or fewer inside the outline is not
/// touched by it. The app's clipping code (`cellsTouching` in `geo.dart`)
/// has the same cut-off, so both sides list the same cells, and an edge
/// that runs along a grid line does not add a row of cells holding nothing
/// but rounding noise.
const MIN_INSIDE_M2: f64 = 0.01;

/// Half the diagonal of a cell, rounded up as the app rounds it. A cell
/// whose centre is at least this far from every edge is not crossed by the
/// outline, so it is whole inside or whole outside and needs no clipping.
const HALF_DIAGONAL_M: f64 = 7.0711;

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
        shoelace(&self.ring)
    }

    pub fn area_dunam(&self) -> f64 {
        self.area_m2() / (GridCell::AREA_M2 * GridCell::PER_DUNAM)
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

    /// Every cell of the grid the outline touches, row by row from the
    /// south, each with the share of it that lies inside. The shares add up
    /// to the area of the outline, less the slivers under the cut-off.
    ///
    /// `max_cells` counts touched cells, the rows a farm may hold, so a farm
    /// is refused when its edge cells take it over the limit even if its
    /// area alone would fit. Refuses before scanning when the outline, or
    /// the box around it, is too large: a thin sliver stretched across the
    /// region has a small area but a box of billions of cells, and scanning
    /// that would tie up the server.
    pub fn cells(&self, max_cells: usize) -> Result<Vec<TouchedCell>, FarmError> {
        // The touched cells cover the outline, so there are never fewer of
        // them than its area in whole cells.
        if self.area_m2() / GridCell::AREA_M2 > max_cells as f64 {
            return Err(FarmError::TooManyCells(max_cells));
        }

        let index = |metres: f64| (metres / GridCell::SIZE_M).floor() as i32;
        let eastings = self.ring.iter().map(|(x, _)| *x);
        let northings = self.ring.iter().map(|(_, y)| *y);

        let west = index(eastings.clone().fold(f64::INFINITY, f64::min));
        let east = index(eastings.fold(f64::NEG_INFINITY, f64::max));
        let south = index(northings.clone().fold(f64::INFINITY, f64::min));
        let north = index(northings.fold(f64::NEG_INFINITY, f64::max));

        let columns = i64::from(east) - i64::from(west) + 1;
        let rows = i64::from(north) - i64::from(south) + 1;

        if columns.saturating_mul(rows) > (max_cells as i64).saturating_mul(BOX_CELLS_PER_CELL) {
            return Err(FarmError::TooManyCells(max_cells));
        }

        let mut cells = Vec::new();

        for n in south..=north {
            for e in west..=east {
                let cell = GridCell::new(e, n);
                let inside_m2 = self.inside_m2(&cell);

                if inside_m2 > MIN_INSIDE_M2 {
                    if cells.len() == max_cells {
                        return Err(FarmError::TooManyCells(max_cells));
                    }

                    cells.push(TouchedCell::new(cell, inside_m2));
                }
            }
        }

        if cells.is_empty() {
            return Err(FarmError::OutlineEnclosesNoCells);
        }

        Ok(cells)
    }

    /// The square metres of the cell that lie inside the outline, found by
    /// clipping the outline to the cell. The steps and constants are those
    /// of `cellsTouching` in the app's `geo.dart`, so the app shows the same
    /// numbers before saving as the backend returns after.
    fn inside_m2(&self, cell: &GridCell) -> f64 {
        let centre = cell.centre();

        let crossed = edges(&self.ring)
            .any(|(start, end)| distance_to_segment(centre, start, end) < HALF_DIAGONAL_M);

        if !crossed {
            return if self.contains(centre) {
                GridCell::AREA_M2
            } else {
                0.0
            };
        }

        let (west, south) = cell.south_west();
        let piece = clip_to_box(
            &self.ring,
            west,
            south,
            west + GridCell::SIZE_M,
            south + GridCell::SIZE_M,
        );

        shoelace(&piece).min(GridCell::AREA_M2)
    }

    fn contains(&self, (x, y): (f64, f64)) -> bool {
        let mut inside = false;

        for ((x1, y1), (x2, y2)) in edges(&self.ring) {
            if (y1 > y) != (y2 > y) && x < (x2 - x1) * (y - y1) / (y2 - y1) + x1 {
                inside = !inside;
            }
        }

        inside
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

/// The area of a closed ring in square metres. Each term multiplies a sum
/// of eastings by a difference of northings, which keeps far more digits
/// than `x1 * y2 - x2 * y1` on coordinates in the millions; it is also the
/// form the app uses, so both sides get the same area.
fn shoelace(ring: &[(f64, f64)]) -> f64 {
    if ring.len() < MIN_CORNERS {
        return 0.0;
    }

    let doubled: f64 = edges(ring)
        .map(|((x1, y1), (x2, y2))| (x1 + x2) * (y1 - y2))
        .sum();

    doubled.abs() / 2.0
}

/// One side of a cell, as the line everything on the far side of is cut off.
#[derive(Clone, Copy)]
enum Side {
    West(f64),
    East(f64),
    South(f64),
    North(f64),
}

impl Side {
    fn keeps(self, (x, y): (f64, f64)) -> bool {
        match self {
            Side::West(west) => x >= west,
            Side::East(east) => x <= east,
            Side::South(south) => y >= south,
            Side::North(north) => y <= north,
        }
    }

    /// Where the segment from `start` to `end` meets this side.
    fn cut(self, start: (f64, f64), end: (f64, f64)) -> (f64, f64) {
        match self {
            Side::West(x) | Side::East(x) => {
                let along = (x - start.0) / (end.0 - start.0);

                (x, start.1 + along * (end.1 - start.1))
            }
            Side::South(y) | Side::North(y) => {
                let along = (y - start.1) / (end.1 - start.1);

                (start.0 + along * (end.0 - start.0), y)
            }
        }
    }
}

/// Sutherland-Hodgman: the part of `ring` inside the box. The box is convex,
/// so this is exact for any outline that does not cross itself, concave or
/// not. Where a concave outline leaves the box in several pieces, they come
/// back joined by runs along the box edge that enclose nothing, so the area
/// of the result is still the area inside.
fn clip_to_box(
    ring: &[(f64, f64)],
    west: f64,
    south: f64,
    east: f64,
    north: f64,
) -> Vec<(f64, f64)> {
    let mut kept = ring.to_vec();

    for side in [
        Side::West(west),
        Side::East(east),
        Side::South(south),
        Side::North(north),
    ] {
        let input = std::mem::take(&mut kept);

        let Some(mut previous) = input.last().copied() else {
            break;
        };

        for current in input {
            match (side.keeps(previous), side.keeps(current)) {
                (true, true) => kept.push(current),
                (false, true) => {
                    kept.push(side.cut(previous, current));
                    kept.push(current);
                }
                (true, false) => kept.push(side.cut(previous, current)),
                (false, false) => {}
            }

            previous = current;
        }
    }

    kept
}

fn distance_to_segment(point: (f64, f64), start: (f64, f64), end: (f64, f64)) -> f64 {
    let (dx, dy) = (end.0 - start.0, end.1 - start.1);
    let length_squared = dx * dx + dy * dy;

    let along = if length_squared == 0.0 {
        0.0
    } else {
        (((point.0 - start.0) * dx + (point.1 - start.1) * dy) / length_squared).clamp(0.0, 1.0)
    };

    let (x, y) = (
        start.0 + along * dx - point.0,
        start.1 + along * dy - point.1,
    );

    (x * x + y * y).sqrt()
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

    /// The south-west corner of a cell, so offsets from it sit on the grid.
    const X0: f64 = 464_100.0;
    const Y0: f64 = 3_987_400.0;

    /// An outline given straight in grid metres, as offsets from a cell
    /// corner. The clipping rules are about the ring, not the GPS corners.
    fn on_the_grid(offsets: &[(f64, f64)]) -> Outline {
        Outline {
            points: Vec::new(),
            ring: offsets.iter().map(|(x, y)| (X0 + x, Y0 + y)).collect(),
        }
    }

    /// The share of the cell `(column, row)` cells east and north of the
    /// corner the offsets start from, or `None` when the cell is absent.
    fn share(cells: &[TouchedCell], column: i32, row: i32) -> Option<f64> {
        let position = GridCell::new(46_410 + column, 398_740 + row);

        cells
            .iter()
            .find(|cell| cell.position() == position)
            .map(TouchedCell::inside_pct)
    }

    fn inside_m2(cells: &[TouchedCell]) -> f64 {
        cells.iter().map(TouchedCell::inside_pct).sum::<f64>() / 100.0 * GridCell::AREA_M2
    }

    fn assert_the_shares_add_up_to_the_area(outline: &Outline, cells: &[TouchedCell]) {
        let (sum, area) = (inside_m2(cells), outline.area_m2());

        assert!(
            (sum - area).abs() < 1e-6,
            "{sum} square metres in the cells for {area} of outline"
        );
    }

    #[test]
    fn a_square_on_the_grid_lines_has_only_whole_cells() {
        let outline = on_the_grid(&[(0.0, 0.0), (50.0, 0.0), (50.0, 50.0), (0.0, 50.0)]);
        let cells = outline.cells(1_000).expect("cells");

        assert_eq!(
            cells.len(),
            25,
            "cells that only share an edge with the square are not touched"
        );
        assert!(cells.iter().all(|cell| cell.inside_pct() == 100.0));
        assert_the_shares_add_up_to_the_area(&outline, &cells);
    }

    #[test]
    fn a_square_half_a_cell_off_the_grid_halves_its_edges_and_quarters_its_corners() {
        let outline = on_the_grid(&[(5.0, 5.0), (55.0, 5.0), (55.0, 55.0), (5.0, 55.0)]);
        let cells = outline.cells(1_000).expect("cells");

        assert_eq!(cells.len(), 36);

        for row in 0..6 {
            for column in 0..6 {
                let on_an_edge = |index: i32| index == 0 || index == 5;

                let expected = match (on_an_edge(column), on_an_edge(row)) {
                    (true, true) => 25.0,
                    (true, false) | (false, true) => 50.0,
                    (false, false) => 100.0,
                };

                assert_eq!(
                    share(&cells, column, row),
                    Some(expected),
                    "cell {column}, {row}"
                );
            }
        }

        assert_the_shares_add_up_to_the_area(&outline, &cells);
    }

    #[test]
    fn a_triangle_halves_the_cells_its_long_side_cuts() {
        let outline = on_the_grid(&[(0.0, 0.0), (40.0, 0.0), (0.0, 40.0)]);
        let cells = outline.cells(1_000).expect("cells");

        for row in 0..4 {
            for column in 0..4 {
                let expected = match column + row {
                    0..=2 => Some(100.0),
                    3 => Some(50.0),
                    // Beyond the long side; the nearest ones meet it at one
                    // corner only, which is no overlap.
                    _ => None,
                };

                assert_eq!(share(&cells, column, row), expected, "cell {column}, {row}");
            }
        }

        assert_eq!(outline.area_m2(), 800.0);
        assert_the_shares_add_up_to_the_area(&outline, &cells);
    }

    #[test]
    fn the_notch_of_an_l_shape_holds_no_cells() {
        let outline = on_the_grid(&[
            (0.0, 0.0),
            (40.0, 0.0),
            (40.0, 20.0),
            (20.0, 20.0),
            (20.0, 40.0),
            (0.0, 40.0),
        ]);
        let cells = outline.cells(1_000).expect("cells");

        assert_eq!(cells.len(), 12);
        assert!(cells.iter().all(|cell| cell.inside_pct() == 100.0));

        for (column, row) in [(2, 2), (3, 2), (2, 3), (3, 3)] {
            assert_eq!(
                share(&cells, column, row),
                None,
                "cell {column}, {row} is in the notch: 0 inside, so absent"
            );
        }

        assert_the_shares_add_up_to_the_area(&outline, &cells);
    }

    #[test]
    fn an_l_shape_off_the_grid_still_adds_up_and_cuts_the_notch_corner_cell() {
        let outline = on_the_grid(&[
            (5.0, 5.0),
            (45.0, 5.0),
            (45.0, 25.0),
            (25.0, 25.0),
            (25.0, 45.0),
            (5.0, 45.0),
        ]);
        let cells = outline.cells(1_000).expect("cells");

        assert_eq!(
            share(&cells, 2, 2),
            Some(75.0),
            "the inner corner takes a quarter out of this cell"
        );
        assert_eq!(share(&cells, 3, 3), None);
        assert_eq!(outline.area_m2(), 1_200.0);
        assert_the_shares_add_up_to_the_area(&outline, &cells);
    }

    #[test]
    fn two_prongs_of_one_outline_in_the_same_cell_are_both_counted() {
        // A U whose two arms, 3 m wide each, reach 5 m into the row above.
        let outline = on_the_grid(&[
            (0.0, 5.0),
            (10.0, 5.0),
            (10.0, 15.0),
            (7.0, 15.0),
            (7.0, 8.0),
            (3.0, 8.0),
            (3.0, 15.0),
            (0.0, 15.0),
        ]);
        let cells = outline.cells(1_000).expect("cells");

        assert_eq!(share(&cells, 0, 1), Some(30.0), "two arms of 3 m by 5 m");
        assert_eq!(share(&cells, 0, 0), Some(42.0), "5 m by 10 m less the gap");
        assert_the_shares_add_up_to_the_area(&outline, &cells);
    }

    #[test]
    fn a_field_smaller_than_a_cell_is_the_cells_it_lies_in_with_small_shares() {
        // 3 m wide and 2 m tall, across the line between two cells.
        let outline = on_the_grid(&[(8.5, 4.0), (11.5, 4.0), (10.0, 6.0)]);
        let cells = outline.cells(1_000).expect("cells");

        assert_eq!(cells.len(), 2);
        assert!((share(&cells, 0, 0).expect("west half") - 1.5).abs() < 1e-9);
        assert!((share(&cells, 1, 0).expect("east half") - 1.5).abs() < 1e-9);
        assert!((outline.area_m2() - 3.0).abs() < 1e-9);
        assert_the_shares_add_up_to_the_area(&outline, &cells);
    }

    #[test]
    fn a_sliver_under_the_cut_off_does_not_make_a_cell_touched() {
        let just_over = on_the_grid(&[(1.0, 1.0), (9.0, 1.0), (9.0, 1.003)]);
        let just_under = on_the_grid(&[(1.0, 1.0), (9.0, 1.0), (9.0, 1.002)]);

        assert_eq!(just_over.cells(1_000).expect("cells").len(), 1);
        assert!(
            matches!(
                just_under.cells(1_000),
                Err(FarmError::OutlineEnclosesNoCells)
            ),
            "0.008 square metres is under the 0.01 cut-off"
        );
    }

    #[test]
    fn corners_in_one_straight_line_enclose_nothing() {
        let line = on_the_grid(&[(0.0, 0.0), (10.0, 0.0), (20.0, 0.0)]);

        assert!(matches!(
            line.cells(1_000),
            Err(FarmError::OutlineEnclosesNoCells)
        ));
    }

    #[test]
    fn the_shares_of_a_walked_field_add_up_to_its_area() {
        let outline = Outline::new(a_field()).expect("outline");
        let cells = outline.cells(1_000).expect("cells");

        assert!(
            cells
                .iter()
                .all(|cell| cell.inside_pct() > 0.0 && cell.inside_pct() <= 100.0)
        );
        assert!(
            (inside_m2(&cells) - outline.area_m2()).abs() < 0.05,
            "{} square metres in the cells for {} of outline; only slivers under the cut-off may be missing",
            inside_m2(&cells),
            outline.area_m2()
        );
    }

    #[test]
    fn the_walking_direction_does_not_change_the_cells() {
        let clockwise: Vec<Point> = a_field().into_iter().rev().collect();

        let one_way = Outline::new(a_field()).expect("outline").cells(1_000);
        let other_way = Outline::new(clockwise).expect("outline").cells(1_000);

        let (one_way, other_way) = (one_way.expect("cells"), other_way.expect("cells"));

        assert_eq!(one_way.len(), other_way.len());
        assert!(one_way.iter().zip(&other_way).all(|(first, second)| {
            first.position() == second.position()
                && (first.inside_pct() - second.inside_pct()).abs() < 1e-6
        }));
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
    fn the_limit_counts_touched_cells_not_the_area() {
        // 25 cells of area, 36 cells touched.
        let outline = on_the_grid(&[(5.0, 5.0), (55.0, 5.0), (55.0, 55.0), (5.0, 55.0)]);

        assert!(matches!(
            outline.cells(35),
            Err(FarmError::TooManyCells(35))
        ));
        assert_eq!(outline.cells(36).expect("cells").len(), 36);
    }

    #[test]
    fn a_thin_sliver_across_the_region_is_refused_without_scanning_its_box() {
        let sliver = vec![point(33.5, 41.5), point(38.5, 47.5), point(38.5, 47.499_99)];
        let outline = Outline::new(sliver).expect("outline");

        assert!(
            outline.area_m2() / 100.0 < 50_000.0,
            "the area alone would pass the limit, which is the point"
        );

        let started = std::time::Instant::now();

        assert!(matches!(
            outline.cells(50_000),
            Err(FarmError::TooManyCells(50_000))
        ));
        assert!(
            started.elapsed() < std::time::Duration::from_millis(50),
            "the box guard answers before any cell is clipped"
        );
    }

    #[test]
    fn the_centroid_sits_between_the_corners() {
        let (lat, lon) = Outline::new(a_field()).expect("outline").centroid();

        assert!((lat - 36.0305).abs() < 1e-9);
        assert!((lon - 44.6005).abs() < 1e-9);
    }
}
