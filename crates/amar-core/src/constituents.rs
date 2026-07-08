use crate::nodal::{FactorTerm, NodalFactorFormula, NodalTerms};

const CANONICAL_CONSTITUENTS: &[&str] = &[
    "2MK3", "2MK5", "2MK6", "2MN6", "2MS6", "2N2", "2Q1", "2SK5", "2SM2", "2SM6", "3MK7", "ALP1",
    "BET1", "CHI1", "EPS2", "ETA2", "GAM2", "H1", "H2", "J1", "K1", "K2", "L2", "LAM2", "M1", "M2",
    "M3", "M4", "M6", "M8", "MF", "MK3", "MK4", "MKS2", "MM", "MN4", "MO3", "MS4", "MSF", "MSK6",
    "MSM", "MSN2", "MU2", "N2", "NO1", "NU2", "O1", "OO1", "OQ2", "P1", "PHI1", "PI1", "PSI1",
    "Q1", "R2", "RHO", "S1", "S2", "S4", "S6", "SA", "SIG1", "SK3", "SK4", "SN4", "SO1", "SO3",
    "SSA", "T2", "TAU1", "THE1", "UPS1",
];

const PORT_SELECTION_CONSTITUENTS: &[&str] = &[
    "2MK5", "2MK6", "2MN6", "2MS6", "2N2", "2Q1", "2SK5", "2SM6", "3MK7", "ALP1", "BET1", "CHI1",
    "EPS2", "ETA2", "GAM2", "H1", "H2", "J1", "K1", "K2", "L2", "LAM2", "M2", "M3", "M4", "M6",
    "M8", "MF", "MK3", "MK4", "MKS2", "MM", "MN4", "MO3", "MS4", "MSF", "MSK6", "MSM", "MSN2",
    "MU2", "N2", "NO1", "NU2", "O1", "OO1", "OQ2", "P1", "PHI1", "PI1", "PSI1", "Q1", "R2", "RHO",
    "S1", "S2", "S4", "SA", "SIG1", "SK3", "SK4", "SN4", "SO1", "SO3", "SSA", "T2", "TAU1", "THE1",
    "UPS1",
];

#[derive(Clone, Copy)]
pub(crate) struct ConstituentDefinition {
    pub(crate) coefficients: [i8; 5],
    pub(crate) semi_cycles: f64,
    pub(crate) u_coefficients: [i8; 7],
    pub(crate) factor_terms: [FactorTerm; 2],
}

impl ConstituentDefinition {
    fn new(
        coefficients: [i8; 5],
        semi_cycles: f64,
        u_coefficients: [i8; 7],
        factor_terms: [FactorTerm; 2],
    ) -> Self {
        Self {
            coefficients,
            semi_cycles,
            u_coefficients,
            factor_terms,
        }
    }

    pub(crate) fn nodal_phase_degrees(self, nodal: &NodalTerms) -> f64 {
        let values = [
            nodal.xi,
            nodal.nu,
            nodal.nu_prime,
            nodal.two_nu_double_prime,
            nodal.q,
            nodal.r,
            nodal.q_u,
        ];
        self.u_coefficients
            .iter()
            .zip(values)
            .map(|(coefficient, value)| f64::from(*coefficient) * value)
            .sum()
    }

    pub(crate) fn nodal_factor(self, nodal: &NodalTerms) -> f64 {
        self.factor_terms
            .iter()
            .filter(|term| term.power != 0)
            .map(|term| term.formula.value(nodal).powi(i32::from(term.power)))
            .product()
    }
}

#[derive(Clone, Copy)]
pub(crate) struct CompoundTerm {
    coefficient: i8,
    name: &'static str,
}

impl CompoundTerm {
    pub(crate) const fn new(coefficient: i8, name: &'static str) -> Self {
        Self { coefficient, name }
    }
}

pub fn supported_constituent_names() -> &'static [&'static str] {
    CANONICAL_CONSTITUENTS
}

pub fn port_selection_constituent_names() -> &'static [&'static str] {
    PORT_SELECTION_CONSTITUENTS
}

pub fn constituent_speed_degrees_per_hour(name: &str) -> Option<f64> {
    let speed = match name {
        "2MK3" => compound_speed(&[CompoundTerm::new(2, "M2"), CompoundTerm::new(-1, "K1")])?,
        "2MK5" => compound_speed(&[CompoundTerm::new(2, "M2"), CompoundTerm::new(1, "K1")])?,
        "2MK6" => compound_speed(&[CompoundTerm::new(2, "M2"), CompoundTerm::new(1, "K2")])?,
        "2MN6" => compound_speed(&[CompoundTerm::new(2, "M2"), CompoundTerm::new(1, "N2")])?,
        "2MS6" => compound_speed(&[CompoundTerm::new(2, "M2"), CompoundTerm::new(1, "S2")])?,
        "2N2" => 27.895_355,
        "2Q1" => 12.854_286,
        "2SK5" => compound_speed(&[CompoundTerm::new(2, "S2"), CompoundTerm::new(1, "K1")])?,
        "2SM2" => compound_speed(&[CompoundTerm::new(2, "S2"), CompoundTerm::new(-1, "M2")])?,
        "2SM6" => compound_speed(&[CompoundTerm::new(2, "S2"), CompoundTerm::new(1, "M2")])?,
        "3MK7" => compound_speed(&[CompoundTerm::new(3, "M2"), CompoundTerm::new(1, "K1")])?,
        "ALP1" => 12.382_765_164,
        "BET1" => 14.414_556_708,
        "CHI1" => 14.569_547_544,
        "EPS2" => 27.423_833_796,
        "ETA2" => 30.626_511_948,
        "GAM2" => 28.911_250_656,
        "H1" => 28.943_037_576,
        "H2" => 29.025_170_928,
        "J1" => 15.585_443_5,
        "K1" => 15.041_069,
        "K2" => 30.082_138,
        "L2" => 29.528_479,
        "LAM2" | "LDA2" => 29.455_626,
        "M1" => 14.496_694,
        "M2" => 28.984_104,
        "M3" => 43.476_16,
        "M4" => compound_speed(&[CompoundTerm::new(2, "M2")])?,
        "M6" => compound_speed(&[CompoundTerm::new(3, "M2")])?,
        "M8" => compound_speed(&[CompoundTerm::new(4, "M2")])?,
        "MF" => 1.098_033_1,
        "MK3" => compound_speed(&[CompoundTerm::new(1, "M2"), CompoundTerm::new(1, "K1")])?,
        "MK4" => compound_speed(&[CompoundTerm::new(1, "M2"), CompoundTerm::new(1, "K2")])?,
        "MKS2" => compound_speed(&[
            CompoundTerm::new(1, "M2"),
            CompoundTerm::new(1, "K2"),
            CompoundTerm::new(-1, "S2"),
        ])?,
        "MM" => 0.544_374_7,
        "MN4" => compound_speed(&[CompoundTerm::new(1, "M2"), CompoundTerm::new(1, "N2")])?,
        "MO3" => compound_speed(&[CompoundTerm::new(1, "M2"), CompoundTerm::new(1, "O1")])?,
        "MS4" => compound_speed(&[CompoundTerm::new(1, "M2"), CompoundTerm::new(1, "S2")])?,
        "MSF" => compound_speed(&[CompoundTerm::new(1, "S2"), CompoundTerm::new(-1, "M2")])?,
        "MSK6" => compound_speed(&[
            CompoundTerm::new(1, "M2"),
            CompoundTerm::new(1, "S2"),
            CompoundTerm::new(1, "K2"),
        ])?,
        "MSM" => compound_speed(&[CompoundTerm::new(1, "MSF"), CompoundTerm::new(-1, "MM")])?,
        "MSN2" => compound_speed(&[
            CompoundTerm::new(1, "M2"),
            CompoundTerm::new(1, "S2"),
            CompoundTerm::new(-1, "N2"),
        ])?,
        "MU2" => 27.968_208,
        "N2" => 28.439_73,
        "NO1" => compound_speed(&[CompoundTerm::new(1, "N2"), CompoundTerm::new(-1, "O1")])?,
        "NU2" => 28.512_583,
        "O1" => 13.943_035,
        "OO1" => 16.139_101,
        "OQ2" => 27.350_980_236,
        "P1" => 14.958_931,
        "PHI1" => 15.123_205_908,
        "PI1" => 14.917_864_68,
        "PSI1" => 15.082_135_308,
        "Q1" => 13.398_661,
        "R2" => 30.041_067,
        "RHO" | "RHO1" => 13.471_515,
        "S1" => 15.0,
        "S2" => 30.0,
        "S4" => compound_speed(&[CompoundTerm::new(2, "S2")])?,
        "S6" => compound_speed(&[CompoundTerm::new(3, "S2")])?,
        "SA" => 0.041_068_6,
        "SIG1" => 12.927_139_848,
        "SK3" => compound_speed(&[CompoundTerm::new(1, "S2"), CompoundTerm::new(1, "K1")])?,
        "SK4" => compound_speed(&[CompoundTerm::new(1, "S2"), CompoundTerm::new(1, "K2")])?,
        "SN4" => compound_speed(&[CompoundTerm::new(1, "S2"), CompoundTerm::new(1, "N2")])?,
        "SO1" => compound_speed(&[CompoundTerm::new(1, "S2"), CompoundTerm::new(-1, "O1")])?,
        "SO3" => compound_speed(&[CompoundTerm::new(1, "S2"), CompoundTerm::new(1, "O1")])?,
        "SSA" => 0.082_137_3,
        "T2" => 29.958_933,
        "TAU1" => 14.025_172_896,
        "THE1" => 15.512_589_72,
        "UPS1" => 16.683_476_328,
        _ => return None,
    };
    Some(speed)
}

fn compound_speed(terms: &[CompoundTerm]) -> Option<f64> {
    let mut speed = 0.0;
    for term in terms {
        speed += f64::from(term.coefficient) * constituent_speed_degrees_per_hour(term.name)?;
    }
    Some(speed)
}

pub(crate) fn constituent_definition(name: &str) -> Option<ConstituentDefinition> {
    let n = FactorTerm::none();
    let f = FactorTerm::new;
    let definition = match name {
        "2MK3" => compound_definition(&[CompoundTerm::new(2, "M2"), CompoundTerm::new(-1, "K1")])?,
        "2MK5" => compound_definition(&[CompoundTerm::new(2, "M2"), CompoundTerm::new(1, "K1")])?,
        "2MK6" => compound_definition(&[CompoundTerm::new(2, "M2"), CompoundTerm::new(1, "K2")])?,
        "2MN6" => compound_definition(&[CompoundTerm::new(2, "M2"), CompoundTerm::new(1, "N2")])?,
        "2MS6" => compound_definition(&[CompoundTerm::new(2, "M2"), CompoundTerm::new(1, "S2")])?,
        "2N2" => ConstituentDefinition::new(
            [2, -2, 0, 2, 0],
            0.0,
            [2, -2, 0, 0, 0, 0, 0],
            [f(NodalFactorFormula::M2, 1), n],
        ),
        "2Q1" => ConstituentDefinition::new(
            [1, -3, 0, 2, 0],
            0.25,
            [2, -1, 0, 0, 0, 0, 0],
            [f(NodalFactorFormula::O1, 1), n],
        ),
        "2SK5" => compound_definition(&[CompoundTerm::new(2, "S2"), CompoundTerm::new(1, "K1")])?,
        "2SM2" => compound_definition(&[CompoundTerm::new(2, "S2"), CompoundTerm::new(-1, "M2")])?,
        "2SM6" => compound_definition(&[CompoundTerm::new(2, "S2"), CompoundTerm::new(1, "M2")])?,
        "3MK7" => compound_definition(&[CompoundTerm::new(3, "M2"), CompoundTerm::new(1, "K1")])?,
        "ALP1" => ConstituentDefinition::new(
            [1, -4, 2, 1, 0],
            0.25,
            [2, -1, 0, 0, 0, 0, 0],
            [f(NodalFactorFormula::O1, 1), n],
        ),
        "BET1" => ConstituentDefinition::new(
            [1, 0, -2, 1, 0],
            -0.25,
            [2, -1, 0, 0, 0, 0, 0],
            [f(NodalFactorFormula::O1, 1), n],
        ),
        "CHI1" => ConstituentDefinition::new(
            [1, 0, 2, -1, 0],
            -0.25,
            [0, -1, 0, 0, 0, 0, 0],
            [f(NodalFactorFormula::J1, 1), n],
        ),
        "EPS2" => ConstituentDefinition::new(
            [2, -3, 2, 1, 0],
            0.0,
            [2, -2, 0, 0, 0, 0, 0],
            [f(NodalFactorFormula::M2, 1), n],
        ),
        "ETA2" => ConstituentDefinition::new(
            [2, 3, 0, -1, 0],
            0.0,
            [2, -2, 0, 0, 0, 0, 0],
            [f(NodalFactorFormula::M2, 1), n],
        ),
        "GAM2" => ConstituentDefinition::new(
            [2, 0, -2, 2, 0],
            0.5,
            [2, -2, 0, 0, 0, 0, 0],
            [f(NodalFactorFormula::M2, 1), n],
        ),
        "H1" => ConstituentDefinition::new(
            [2, 0, -1, 0, 1],
            0.5,
            [2, -2, 0, 0, 0, 0, 0],
            [f(NodalFactorFormula::M2, 1), n],
        ),
        "H2" => ConstituentDefinition::new(
            [2, 0, 1, 0, -1],
            0.0,
            [2, -2, 0, 0, 0, 0, 0],
            [f(NodalFactorFormula::M2, 1), n],
        ),
        "J1" => ConstituentDefinition::new(
            [1, 2, 0, -1, 0],
            -0.25,
            [0, -1, 0, 0, 0, 0, 0],
            [f(NodalFactorFormula::J1, 1), n],
        ),
        "K1" => ConstituentDefinition::new(
            [1, 1, 0, 0, 0],
            -0.25,
            [0, 0, -1, 0, 0, 0, 0],
            [f(NodalFactorFormula::K1, 1), n],
        ),
        "K2" => ConstituentDefinition::new(
            [2, 2, 0, 0, 0],
            0.0,
            [0, 0, 0, -1, 0, 0, 0],
            [f(NodalFactorFormula::K2, 1), n],
        ),
        "L2" => ConstituentDefinition::new(
            [2, 1, 0, -1, 0],
            0.5,
            [2, -2, 0, 0, 0, -1, 0],
            [f(NodalFactorFormula::L2, 1), n],
        ),
        "LAM2" | "LDA2" => ConstituentDefinition::new(
            [2, 1, -2, 1, 0],
            0.5,
            [2, -2, 0, 0, 0, 0, 0],
            [f(NodalFactorFormula::M2, 1), n],
        ),
        "M1" => ConstituentDefinition::new(
            [1, 0, 0, 0, 0],
            -0.25,
            [1, -1, 0, 0, 1, 0, 0],
            [f(NodalFactorFormula::M1, 1), n],
        ),
        "M2" => ConstituentDefinition::new(
            [2, 0, 0, 0, 0],
            0.0,
            [2, -2, 0, 0, 0, 0, 0],
            [f(NodalFactorFormula::M2, 1), n],
        ),
        "M3" => ConstituentDefinition::new(
            [3, 0, 0, 0, 0],
            0.0,
            [3, -3, 0, 0, 0, 0, 0],
            [f(NodalFactorFormula::M3, 1), n],
        ),
        "M4" => compound_definition(&[CompoundTerm::new(2, "M2")])?,
        "M6" => compound_definition(&[CompoundTerm::new(3, "M2")])?,
        "M8" => compound_definition(&[CompoundTerm::new(4, "M2")])?,
        "MF" => ConstituentDefinition::new(
            [0, 2, 0, 0, 0],
            0.0,
            [-2, 0, 0, 0, 0, 0, 0],
            [f(NodalFactorFormula::Mf, 1), n],
        ),
        "MK3" => compound_definition(&[CompoundTerm::new(1, "M2"), CompoundTerm::new(1, "K1")])?,
        "MK4" => compound_definition(&[CompoundTerm::new(1, "M2"), CompoundTerm::new(1, "K2")])?,
        "MKS2" => compound_definition(&[
            CompoundTerm::new(1, "M2"),
            CompoundTerm::new(1, "K2"),
            CompoundTerm::new(-1, "S2"),
        ])?,
        "MM" => ConstituentDefinition::new(
            [0, 1, 0, -1, 0],
            0.0,
            [0, 0, 0, 0, 0, 0, 0],
            [f(NodalFactorFormula::Mm, 1), n],
        ),
        "MN4" => compound_definition(&[CompoundTerm::new(1, "M2"), CompoundTerm::new(1, "N2")])?,
        "MO3" => compound_definition(&[CompoundTerm::new(1, "M2"), CompoundTerm::new(1, "O1")])?,
        "MS4" => compound_definition(&[CompoundTerm::new(1, "M2"), CompoundTerm::new(1, "S2")])?,
        "MSF" => compound_definition(&[CompoundTerm::new(1, "S2"), CompoundTerm::new(-1, "M2")])?,
        "MSK6" => compound_definition(&[
            CompoundTerm::new(1, "M2"),
            CompoundTerm::new(1, "S2"),
            CompoundTerm::new(1, "K2"),
        ])?,
        "MSM" => compound_definition(&[CompoundTerm::new(1, "MSF"), CompoundTerm::new(-1, "MM")])?,
        "MSN2" => compound_definition(&[
            CompoundTerm::new(1, "M2"),
            CompoundTerm::new(1, "S2"),
            CompoundTerm::new(-1, "N2"),
        ])?,
        "MU2" => ConstituentDefinition::new(
            [2, -2, 2, 0, 0],
            0.0,
            [2, -2, 0, 0, 0, 0, 0],
            [f(NodalFactorFormula::M2, 1), n],
        ),
        "N2" => ConstituentDefinition::new(
            [2, -1, 0, 1, 0],
            0.0,
            [2, -2, 0, 0, 0, 0, 0],
            [f(NodalFactorFormula::M2, 1), n],
        ),
        "NU2" => ConstituentDefinition::new(
            [2, -1, 2, -1, 0],
            0.0,
            [2, -2, 0, 0, 0, 0, 0],
            [f(NodalFactorFormula::M2, 1), n],
        ),
        "NO1" => compound_definition(&[CompoundTerm::new(1, "N2"), CompoundTerm::new(-1, "O1")])?,
        "O1" => ConstituentDefinition::new(
            [1, -1, 0, 0, 0],
            0.25,
            [2, -1, 0, 0, 0, 0, 0],
            [f(NodalFactorFormula::O1, 1), n],
        ),
        "OO1" => ConstituentDefinition::new(
            [1, 3, 0, 0, 0],
            -0.25,
            [-2, -1, 0, 0, 0, 0, 0],
            [f(NodalFactorFormula::Oo1, 1), n],
        ),
        "OQ2" => ConstituentDefinition::new(
            [2, -3, 0, 3, 0],
            0.0,
            [2, -2, 0, 0, 0, 0, 0],
            [f(NodalFactorFormula::M2, 1), n],
        ),
        "P1" => ConstituentDefinition::new([1, 1, -2, 0, 0], 0.25, [0, 0, 0, 0, 0, 0, 0], [n, n]),
        "PHI1" => ConstituentDefinition::new([1, 1, 2, 0, 0], -0.25, [0, 0, 0, 0, 0, 0, 0], [n, n]),
        "PI1" => ConstituentDefinition::new([1, 1, -3, 0, 1], 0.25, [0, 0, 0, 0, 0, 0, 0], [n, n]),
        "PSI1" => {
            ConstituentDefinition::new([1, 1, 1, 0, -1], -0.25, [0, 0, 0, 0, 0, 0, 0], [n, n])
        }
        "Q1" => ConstituentDefinition::new(
            [1, -2, 0, 1, 0],
            0.25,
            [2, -1, 0, 0, 0, 0, 0],
            [f(NodalFactorFormula::O1, 1), n],
        ),
        "R2" => ConstituentDefinition::new([2, 2, -1, 0, -1], 0.5, [0, 0, 0, 0, 0, 0, 0], [n, n]),
        "RHO" | "RHO1" => ConstituentDefinition::new(
            [1, -2, 2, -1, 0],
            0.25,
            [2, -1, 0, 0, 0, 0, 0],
            [f(NodalFactorFormula::O1, 1), n],
        ),
        "S1" => ConstituentDefinition::new([1, 1, -1, 0, 0], 0.0, [0, 0, 0, 0, 0, 0, 0], [n, n]),
        "S2" => ConstituentDefinition::new([2, 2, -2, 0, 0], 0.0, [0, 0, 0, 0, 0, 0, 0], [n, n]),
        "S4" => compound_definition(&[CompoundTerm::new(2, "S2")])?,
        "S6" => compound_definition(&[CompoundTerm::new(3, "S2")])?,
        "SA" => ConstituentDefinition::new([0, 0, 1, 0, 0], 0.0, [0, 0, 0, 0, 0, 0, 0], [n, n]),
        "SIG1" => ConstituentDefinition::new(
            [1, -3, 2, 0, 0],
            0.25,
            [2, -1, 0, 0, 0, 0, 0],
            [f(NodalFactorFormula::O1, 1), n],
        ),
        "SK3" => compound_definition(&[CompoundTerm::new(1, "S2"), CompoundTerm::new(1, "K1")])?,
        "SK4" => compound_definition(&[CompoundTerm::new(1, "S2"), CompoundTerm::new(1, "K2")])?,
        "SN4" => compound_definition(&[CompoundTerm::new(1, "S2"), CompoundTerm::new(1, "N2")])?,
        "SO1" => compound_definition(&[CompoundTerm::new(1, "S2"), CompoundTerm::new(-1, "O1")])?,
        "SO3" => compound_definition(&[CompoundTerm::new(1, "S2"), CompoundTerm::new(1, "O1")])?,
        "SSA" => ConstituentDefinition::new([0, 0, 2, 0, 0], 0.0, [0, 0, 0, 0, 0, 0, 0], [n, n]),
        "T2" => ConstituentDefinition::new([2, 2, -3, 0, 1], 0.0, [0, 0, 0, 0, 0, 0, 0], [n, n]),
        "TAU1" => ConstituentDefinition::new(
            [1, -1, 2, 0, 0],
            -0.25,
            [2, -1, 0, 0, 0, 0, 0],
            [f(NodalFactorFormula::O1, 1), n],
        ),
        "THE1" => ConstituentDefinition::new(
            [1, 2, -2, 1, 0],
            -0.25,
            [0, -1, 0, 0, 0, 0, 0],
            [f(NodalFactorFormula::J1, 1), n],
        ),
        "UPS1" => ConstituentDefinition::new(
            [1, 4, 0, -1, 0],
            -0.25,
            [-2, -1, 0, 0, 0, 0, 0],
            [f(NodalFactorFormula::Oo1, 1), n],
        ),
        _ => return None,
    };
    Some(definition)
}

pub(crate) fn compound_definition(terms: &[CompoundTerm]) -> Option<ConstituentDefinition> {
    let mut coefficients = [0_i8; 5];
    let mut semi_cycles = 0.0;
    let mut u_coefficients = [0_i8; 7];
    let mut factor_terms = [FactorTerm::none(), FactorTerm::none()];

    for term in terms {
        let definition = constituent_definition(term.name)?;
        add_scaled_coefficients(
            &mut coefficients,
            &definition.coefficients,
            term.coefficient,
        )?;
        semi_cycles += f64::from(term.coefficient) * definition.semi_cycles;
        add_scaled_coefficients(
            &mut u_coefficients,
            &definition.u_coefficients,
            term.coefficient,
        )?;
        for factor_term in definition.factor_terms {
            if factor_term.power == 0 {
                continue;
            }
            let power = factor_term
                .power
                .checked_mul(term.coefficient.unsigned_abs())?;
            add_factor_power(&mut factor_terms, factor_term.formula, power)?;
        }
    }

    Some(ConstituentDefinition::new(
        coefficients,
        semi_cycles,
        u_coefficients,
        factor_terms,
    ))
}

fn add_scaled_coefficients<const N: usize>(
    target: &mut [i8; N],
    source: &[i8; N],
    coefficient: i8,
) -> Option<()> {
    for (target_value, source_value) in target.iter_mut().zip(source) {
        let value = i16::from(*target_value) + i16::from(coefficient) * i16::from(*source_value);
        *target_value = i8::try_from(value).ok()?;
    }
    Some(())
}

fn add_factor_power(
    terms: &mut [FactorTerm; 2],
    formula: NodalFactorFormula,
    power: u8,
) -> Option<()> {
    for term in terms.iter_mut() {
        if term.power != 0 && term.formula == formula {
            term.power = term.power.checked_add(power)?;
            return Some(());
        }
    }
    for term in terms.iter_mut() {
        if term.power == 0 {
            *term = FactorTerm::new(formula, power);
            return Some(());
        }
    }
    None
}
