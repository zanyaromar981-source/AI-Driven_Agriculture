use crate::features::zones::domain::ZoneError;

const MIN_CORNERS: usize = 3;

/// The edge of a sub-zone on the map: one ring of corners for each separate
/// piece of land, every corner as `(lon, lat)` in WGS84 degrees. A ring
/// closes itself, and no ring is a hole in another, so a point is inside
/// the shape when it is inside any ring.
#[derive(Clone, Debug, PartialEq)]
pub struct Shape {
    rings: Vec<Vec<(f64, f64)>>,
    /// The box around every ring as `(west, south, east, north)`. Most
    /// points are outside most shapes, and the box says so without walking
    /// the ring.
    bounds: (f64, f64, f64, f64),
}

impl Shape {
    pub fn new(rings: Vec<Vec<(f64, f64)>>) -> Result<Self, ZoneError> {
        let mut rings = rings;

        for ring in &mut rings {
            // A ring that repeats its first corner at the end describes the
            // same land as one that does not.
            if ring.len() > 1 && ring.first() == ring.last() {
                ring.pop();
            }

            let finite = ring
                .iter()
                .all(|(lon, lat)| lon.is_finite() && lat.is_finite());

            if ring.len() < MIN_CORNERS || !finite {
                return Err(ZoneError::BadShape);
            }
        }

        if rings.is_empty() {
            return Err(ZoneError::BadShape);
        }

        let corners = || rings.iter().flatten();
        let bounds = (
            corners().map(|(lon, _)| *lon).fold(f64::INFINITY, f64::min),
            corners().map(|(_, lat)| *lat).fold(f64::INFINITY, f64::min),
            corners()
                .map(|(lon, _)| *lon)
                .fold(f64::NEG_INFINITY, f64::max),
            corners()
                .map(|(_, lat)| *lat)
                .fold(f64::NEG_INFINITY, f64::max),
        );

        Ok(Self { rings, bounds })
    }

    pub fn rings(&self) -> &[Vec<(f64, f64)>] {
        &self.rings
    }

    /// Whether the point lies inside the shape.
    ///
    /// A point exactly on an edge counts for the side the rule below gives
    /// it: inside when the land is to the east or to the north of the edge,
    /// outside when it is to the west or to the south. Two shapes that share
    /// a border therefore never both claim a point on it, and never both
    /// refuse it, as long as they share the border's corners, which the
    /// seeded shapes do.
    pub fn contains(&self, lon: f64, lat: f64) -> bool {
        let (west, south, east, north) = self.bounds;

        if lon < west || lon > east || lat < south || lat > north {
            return false;
        }

        self.rings.iter().any(|ring| ring_contains(ring, lon, lat))
    }
}

/// Counts the edges a ray from the point towards the east crosses: an odd
/// count is inside. An edge counts when one end is above the point and the
/// other is not, so a corner on the ray is counted once, not twice.
fn ring_contains(ring: &[(f64, f64)], lon: f64, lat: f64) -> bool {
    let mut inside = false;

    for (index, (lon1, lat1)) in ring.iter().enumerate() {
        let (lon2, lat2) = ring[(index + 1) % ring.len()];

        if (*lat1 > lat) != (lat2 > lat)
            && lon < (lon2 - lon1) * (lat - lat1) / (lat2 - lat1) + lon1
        {
            inside = !inside;
        }
    }

    inside
}

#[cfg(test)]
mod tests {
    use super::*;

    fn square(west: f64, south: f64, size: f64) -> Vec<(f64, f64)> {
        vec![
            (west, south),
            (west + size, south),
            (west + size, south + size),
            (west, south + size),
        ]
    }

    fn shape(rings: Vec<Vec<(f64, f64)>>) -> Shape {
        Shape::new(rings).expect("shape")
    }

    #[test]
    fn a_point_inside_the_ring_is_inside() {
        assert!(shape(vec![square(44.0, 36.0, 1.0)]).contains(44.5, 36.5));
    }

    #[test]
    fn a_point_outside_the_ring_is_outside() {
        let shape = shape(vec![square(44.0, 36.0, 1.0)]);

        assert!(!shape.contains(45.5, 36.5), "east of it");
        assert!(!shape.contains(43.5, 36.5), "west of it");
        assert!(!shape.contains(44.5, 37.5), "north of it");
        assert!(!shape.contains(44.5, 35.5), "south of it");
    }

    #[test]
    fn a_point_in_the_notch_of_a_bent_ring_is_outside() {
        // An L: the square with its north-east quarter cut away.
        let bent = shape(vec![vec![
            (44.0, 36.0),
            (45.0, 36.0),
            (45.0, 36.5),
            (44.5, 36.5),
            (44.5, 37.0),
            (44.0, 37.0),
        ]]);

        assert!(bent.contains(44.25, 36.75));
        assert!(bent.contains(44.75, 36.25));
        assert!(
            !bent.contains(44.75, 36.75),
            "inside the box around the ring, outside the ring"
        );
    }

    #[test]
    fn a_point_on_a_border_two_shapes_share_belongs_to_exactly_one_of_them() {
        let west = shape(vec![square(44.0, 36.0, 1.0)]);
        let east = shape(vec![square(45.0, 36.0, 1.0)]);
        let north = shape(vec![square(44.0, 37.0, 1.0)]);

        assert!(!west.contains(45.0, 36.5));
        assert!(east.contains(45.0, 36.5), "the land to the east takes it");

        assert!(!west.contains(44.5, 37.0));
        assert!(north.contains(44.5, 37.0), "the land to the north takes it");
    }

    #[test]
    fn a_point_level_with_a_corner_is_counted_once() {
        // A diamond: a ray from its west side passes exactly through the
        // east corner, which must not be counted as two crossings.
        let diamond = shape(vec![vec![
            (44.0, 36.5),
            (44.5, 36.0),
            (45.0, 36.5),
            (44.5, 37.0),
        ]]);

        assert!(diamond.contains(44.5, 36.5));
        assert!(!diamond.contains(43.5, 36.5));
        assert!(!diamond.contains(45.5, 36.5));
    }

    #[test]
    fn a_shape_of_several_rings_holds_the_points_of_each_ring() {
        let islands = shape(vec![square(44.0, 36.0, 1.0), square(46.0, 36.0, 1.0)]);

        assert!(islands.contains(44.5, 36.5));
        assert!(islands.contains(46.5, 36.5));
        assert!(
            !islands.contains(45.5, 36.5),
            "the gap between the rings belongs to neither"
        );
    }

    #[test]
    fn a_ring_closed_by_repeating_its_first_corner_is_the_same_ring() {
        let mut closed = square(44.0, 36.0, 1.0);
        closed.push((44.0, 36.0));

        assert_eq!(shape(vec![closed]), shape(vec![square(44.0, 36.0, 1.0)]));
    }

    #[test]
    fn a_shape_needs_a_ring_and_a_ring_needs_three_corners() {
        assert!(matches!(Shape::new(vec![]), Err(ZoneError::BadShape)));
        assert!(matches!(
            Shape::new(vec![vec![(44.0, 36.0), (45.0, 36.0)]]),
            Err(ZoneError::BadShape)
        ));
        assert!(matches!(
            Shape::new(vec![square(44.0, 36.0, 1.0), vec![]]),
            Err(ZoneError::BadShape)
        ));
    }

    #[test]
    fn a_corner_that_is_not_a_number_is_refused() {
        assert!(matches!(
            Shape::new(vec![vec![(44.0, 36.0), (f64::NAN, 36.0), (45.0, 37.0)]]),
            Err(ZoneError::BadShape)
        ));
    }
}
