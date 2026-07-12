use crate::UtcDateTime;

#[derive(Clone, Copy)]
pub(crate) struct AstronomicalTerms {
    pub(crate) tau: f64,
    pub(crate) s: f64,
    pub(crate) h: f64,
    pub(crate) p: f64,
    pub(crate) p1: f64,
    pub(crate) node_degrees: f64,
}

pub(crate) fn astronomical_terms(at: UtcDateTime) -> AstronomicalTerms {
    // Schureman SP-98, Table 1: epoch 1899-12-31 12:00 UTC.
    let d = at.ordinal_days() - 693_595.5;
    let julian_centuries = d / 36_525.0;
    let powers = [
        1.0,
        julian_centuries,
        julian_centuries * julian_centuries,
        julian_centuries * julian_centuries * julian_centuries,
    ];
    let s = polynomial_cycles(
        [
            270.0 + 26.0 / 60.0 + 14.72 / 3600.0,
            1336.0 * 360.0 + 1_108_411.2 / 3600.0,
            9.09 / 3600.0,
            0.0068 / 3600.0,
        ],
        powers,
    );
    let h = polynomial_cycles(
        [
            279.0 + 41.0 / 60.0 + 48.04 / 3600.0,
            129_602_768.13 / 3600.0,
            1.089 / 3600.0,
            0.0,
        ],
        powers,
    );
    let p = polynomial_cycles(
        [
            334.0 + 19.0 / 60.0 + 40.87 / 3600.0,
            11.0 * 360.0 + 392_515.94 / 3600.0,
            -37.24 / 3600.0,
            -0.045 / 3600.0,
        ],
        powers,
    );
    let p1 = polynomial_cycles(
        [
            281.0 + 13.0 / 60.0 + 15.0 / 3600.0,
            6189.03 / 3600.0,
            1.63 / 3600.0,
            0.012 / 3600.0,
        ],
        powers,
    );
    let node_degrees = polynomial_degrees(
        [
            259.0 + 10.0 / 60.0 + 57.12 / 3600.0,
            -(5.0 * 360.0 + 482_912.63 / 3600.0),
            7.58 / 3600.0,
            0.008 / 3600.0,
        ],
        powers,
    );
    let mean_sun_hour_angle = d.rem_euclid(1.0);
    let tau = (mean_sun_hour_angle + h - s).rem_euclid(1.0);

    AstronomicalTerms {
        tau,
        s,
        h,
        p,
        p1,
        node_degrees,
    }
}

fn polynomial_cycles(coefficients: [f64; 4], powers: [f64; 4]) -> f64 {
    (polynomial_degrees(coefficients, powers) / 360.0).rem_euclid(1.0)
}

fn polynomial_degrees(coefficients: [f64; 4], powers: [f64; 4]) -> f64 {
    let degrees = coefficients
        .iter()
        .zip(powers)
        .map(|(coefficient, power)| coefficient * power)
        .sum::<f64>();
    degrees.rem_euclid(360.0)
}
