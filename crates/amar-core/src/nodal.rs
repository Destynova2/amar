use crate::astro::AstronomicalTerms;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct FactorTerm {
    pub(crate) formula: NodalFactorFormula,
    pub(crate) power: u8,
}

impl FactorTerm {
    pub(crate) const fn none() -> Self {
        Self {
            formula: NodalFactorFormula::Unity,
            power: 0,
        }
    }

    pub(crate) const fn new(formula: NodalFactorFormula, power: u8) -> Self {
        Self { formula, power }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NodalFactorFormula {
    Unity,
    Mm,
    Mf,
    O1,
    J1,
    Oo1,
    M2,
    M3,
    M1,
    L2,
    K1,
    K2,
}

impl NodalFactorFormula {
    pub(crate) fn value(self, nodal: &NodalTerms) -> f64 {
        match self {
            Self::Unity => 1.0,
            Self::Mm => f_mm(nodal.inclination),
            Self::Mf => f_mf(nodal.inclination),
            Self::O1 => f_o1(nodal.inclination),
            Self::J1 => f_j1(nodal.inclination),
            Self::Oo1 => f_oo1(nodal.inclination),
            Self::M2 => f_m2(nodal.inclination),
            Self::M3 => f_m3(nodal.inclination),
            Self::M1 => f_m1(nodal.inclination, nodal.p),
            Self::L2 => f_l2(nodal.inclination, nodal.p),
            Self::K1 => f_k1(nodal.inclination, nodal.nu),
            Self::K2 => f_k2(nodal.inclination, nodal.nu),
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) struct NodalTerms {
    pub(crate) inclination: f64,
    pub(crate) xi: f64,
    pub(crate) nu: f64,
    pub(crate) nu_prime: f64,
    pub(crate) two_nu_double_prime: f64,
    pub(crate) q: f64,
    pub(crate) r: f64,
    pub(crate) q_u: f64,
    pub(crate) p: f64,
}

pub(crate) fn nodal_terms(astro: &AstronomicalTerms) -> NodalTerms {
    let node = astro.node_degrees;
    let inclination = inclination(node);
    let xi = xi(node);
    let nu = nu(node);
    let nu_prime = nu_prime(node);
    let two_nu_double_prime = two_nu_double_prime(node);
    let p = astro.p * 360.0 - xi;
    let q = q(p);
    let r = r(p, inclination);
    let q_u = p - q;

    NodalTerms {
        inclination,
        xi,
        nu,
        nu_prime,
        two_nu_double_prime,
        q,
        r,
        q_u,
        p,
    }
}

fn sind(degrees: f64) -> f64 {
    degrees.to_radians().sin()
}

fn cosd(degrees: f64) -> f64 {
    degrees.to_radians().cos()
}

fn tand(degrees: f64) -> f64 {
    degrees.to_radians().tan()
}

fn atan2d(y: f64, x: f64) -> f64 {
    y.atan2(x).to_degrees()
}

fn asind(value: f64) -> f64 {
    value.clamp(-1.0, 1.0).asin().to_degrees()
}

fn acosd(value: f64) -> f64 {
    value.clamp(-1.0, 1.0).acos().to_degrees()
}

fn cos_inclination(node: f64) -> f64 {
    const OBLIQUITY: f64 = 23.0 + 27.0 / 60.0 + 8.26 / 3600.0;
    const LUNAR_INCLINATION: f64 = 5.0 + 8.0 / 60.0 + 43.3546 / 3600.0;
    cosd(OBLIQUITY) * cosd(LUNAR_INCLINATION)
        - sind(OBLIQUITY) * sind(LUNAR_INCLINATION) * cosd(node)
}

fn sin_inclination(node: f64) -> f64 {
    let value = cos_inclination(node);
    (1.0 - value * value).sqrt()
}

fn inclination(node: f64) -> f64 {
    acosd(cos_inclination(node))
}

fn sin_nu(node: f64) -> f64 {
    const LUNAR_INCLINATION: f64 = 5.0 + 8.0 / 60.0 + 43.3546 / 3600.0;
    sind(LUNAR_INCLINATION) * sind(node) / sin_inclination(node)
}

fn cos_nu(node: f64) -> f64 {
    let value = sin_nu(node);
    (1.0 - value * value).sqrt()
}

fn sin_omega_arc(node: f64) -> f64 {
    const OBLIQUITY: f64 = 23.0 + 27.0 / 60.0 + 8.26 / 3600.0;
    sind(OBLIQUITY) * sind(node) / sin_inclination(node)
}

fn cos_omega_arc(node: f64) -> f64 {
    const OBLIQUITY: f64 = 23.0 + 27.0 / 60.0 + 8.26 / 3600.0;
    cosd(node) * cos_nu(node) + sind(node) * sin_nu(node) * cosd(OBLIQUITY)
}

fn xi(node: f64) -> f64 {
    node - atan2d(sin_omega_arc(node), cos_omega_arc(node))
}

fn nu(node: f64) -> f64 {
    asind(sin_nu(node))
}

fn nu_prime(node: f64) -> f64 {
    let inclination = inclination(node);
    let multiplier = sind(2.0 * inclination);
    // Schureman SP-98, Table 15: K1 nodal satellite ratio.
    atan2d(
        multiplier * sin_nu(node),
        multiplier * cos_nu(node) + 0.3347,
    )
}

fn two_nu_double_prime(node: f64) -> f64 {
    let sin_i = sin_inclination(node);
    let term = sin_i * sin_i;
    let double_nu = 2.0 * nu(node);
    // Schureman SP-98, Table 15: K2 nodal satellite ratio.
    atan2d(term * sind(double_nu), term * cosd(double_nu) + 0.0727)
}

fn q(p: f64) -> f64 {
    // Schureman SP-98, Table 15: M1 elliptic constituent ratio.
    atan2d(0.483 * sind(p), cosd(p))
}

fn q_amplitude(p: f64) -> f64 {
    1.0 / (2.31 + 1.435 * cosd(2.0 * p)).sqrt()
}

fn r(p: f64, inclination: f64) -> f64 {
    let cot_i2 = 1.0 / tand(inclination / 2.0);
    atan2d(sind(2.0 * p), cot_i2 * cot_i2 / 6.0 - cosd(2.0 * p))
}

fn r_amplitude(p: f64, inclination: f64) -> f64 {
    let term = tand(inclination / 2.0).powi(2);
    1.0 / (1.0 - 12.0 * term * cosd(2.0 * p) + 36.0 * term * term).sqrt()
}

fn f_mm(inclination: f64) -> f64 {
    let sin_i = sind(inclination);
    (2.0 / 3.0 - sin_i * sin_i) / 0.5021
}

fn f_mf(inclination: f64) -> f64 {
    sind(inclination).powi(2) / 0.1578
}

fn f_o1(inclination: f64) -> f64 {
    sind(inclination) * cosd(inclination / 2.0).powi(2) / 0.38
}

fn f_j1(inclination: f64) -> f64 {
    sind(2.0 * inclination) / 0.7214
}

fn f_oo1(inclination: f64) -> f64 {
    sind(inclination) * sind(inclination / 2.0).powi(2) / 0.0164
}

fn f_m2(inclination: f64) -> f64 {
    cosd(inclination / 2.0).powi(4) / 0.9154
}

fn f_m3(inclination: f64) -> f64 {
    cosd(inclination / 2.0).powi(6) / 0.8758
}

fn f_m1(inclination: f64, p: f64) -> f64 {
    f_o1(inclination) / q_amplitude(p)
}

fn f_l2(inclination: f64, p: f64) -> f64 {
    f_m2(inclination) / r_amplitude(p, inclination)
}

fn f_k1(inclination: f64, nu: f64) -> f64 {
    let term = sind(2.0 * inclination);
    (0.8965 * term * term + 0.6001 * term * cosd(nu) + 0.1006).sqrt()
}

fn f_k2(inclination: f64, nu: f64) -> f64 {
    let term = sind(inclination).powi(2);
    (19.0444 * term * term + 2.7702 * term * cosd(2.0 * nu) + 0.0981).sqrt()
}
