//! Forward projection from WGS84 latitude and longitude to UTM zone 38N
//! (EPSG:32638), the grid the Sentinel-2 pixels over Kurdistan sit on.
//! Krüger series to the third order, accurate to well under a millimetre
//! inside the zone.

const SEMI_MAJOR_AXIS_M: f64 = 6_378_137.0;
const FLATTENING: f64 = 1.0 / 298.257_223_563;
const SCALE_FACTOR: f64 = 0.9996;
const FALSE_EASTING_M: f64 = 500_000.0;
const CENTRAL_MERIDIAN_DEG: f64 = 45.0;

/// Returns `(easting, northing)` in metres.
pub(super) fn to_utm_38n(lat: f64, lon: f64) -> (f64, f64) {
    let n = FLATTENING / (2.0 - FLATTENING);
    let rectifying_radius =
        SEMI_MAJOR_AXIS_M / (1.0 + n) * (1.0 + n.powi(2) / 4.0 + n.powi(4) / 64.0);

    let alpha = [
        n / 2.0 - 2.0 * n.powi(2) / 3.0 + 5.0 * n.powi(3) / 16.0,
        13.0 * n.powi(2) / 48.0 - 3.0 * n.powi(3) / 5.0,
        61.0 * n.powi(3) / 240.0,
    ];

    let phi = lat.to_radians();
    let lambda = (lon - CENTRAL_MERIDIAN_DEG).to_radians();

    let c = 2.0 * n.sqrt() / (1.0 + n);
    let t = (phi.sin().atanh() - c * (c * phi.sin()).atanh()).sinh();
    let xi = t.atan2(lambda.cos());
    let eta = (lambda.sin() / (1.0 + t * t).sqrt()).atanh();

    let mut easting = eta;
    let mut northing = xi;

    for (index, coefficient) in alpha.iter().enumerate() {
        let order = 2.0 * (index as f64 + 1.0);

        easting += coefficient * (order * xi).cos() * (order * eta).sinh();
        northing += coefficient * (order * xi).sin() * (order * eta).cosh();
    }

    (
        FALSE_EASTING_M + SCALE_FACTOR * rectifying_radius * easting,
        SCALE_FACTOR * rectifying_radius * northing,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Reference values computed with PROJ (pyproj, EPSG:4326 to EPSG:32638).
    const REFERENCE: [(f64, f64, f64, f64); 4] = [
        (36.0312, 44.6021, 464_152.260_6, 3_987_482.218_6),
        (35.5613, 45.4329, 539_231.313_6, 3_935_378.185_0),
        (36.8679, 42.9488, 317_166.010_0, 4_082_182.290_1),
        (34.6200, 45.3200, 529_334.934_8, 3_830_950.369_3),
    ];

    #[test]
    fn matches_proj_to_the_centimetre() {
        for (lat, lon, easting, northing) in REFERENCE {
            let (e, n) = to_utm_38n(lat, lon);

            assert!(
                (e - easting).abs() < 0.01,
                "easting for {lat}, {lon}: {e} vs {easting}"
            );
            assert!(
                (n - northing).abs() < 0.01,
                "northing for {lat}, {lon}: {n} vs {northing}"
            );
        }
    }

    #[test]
    fn the_central_meridian_sits_on_the_false_easting() {
        let (easting, _) = to_utm_38n(36.0, 45.0);

        assert!((easting - FALSE_EASTING_M).abs() < 1e-6);
    }
}
