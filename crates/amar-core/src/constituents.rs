use crate::nodal::{FactorTerm, NodalFactorFormula, NodalTerms};

#[derive(Clone, Copy)]
pub(crate) struct ConstituentDefinition {
    pub(crate) coefficients: [i8; 5],
    pub(crate) semi_cycles: f64,
    u_coefficients: [i8; 7],
    factor_terms: [FactorTerm; 2],
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

pub(crate) fn constituent_definition(name: &str) -> Option<ConstituentDefinition> {
    let n = FactorTerm::none();
    let f = FactorTerm::new;
    let definition = match name {
        "2MK3" => ConstituentDefinition::new(
            [3, -1, 0, 0, 0],
            0.25,
            [4, -4, 1, 0, 0, 0, 0],
            [f(NodalFactorFormula::M2, 2), f(NodalFactorFormula::K1, 1)],
        ),
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
        "2SM2" => ConstituentDefinition::new(
            [2, 4, -4, 0, 0],
            0.0,
            [-2, 2, 0, 0, 0, 0, 0],
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
        "M4" => ConstituentDefinition::new(
            [4, 0, 0, 0, 0],
            0.0,
            [4, -4, 0, 0, 0, 0, 0],
            [f(NodalFactorFormula::M2, 2), n],
        ),
        "M6" => ConstituentDefinition::new(
            [6, 0, 0, 0, 0],
            0.0,
            [6, -6, 0, 0, 0, 0, 0],
            [f(NodalFactorFormula::M2, 3), n],
        ),
        "M8" => ConstituentDefinition::new(
            [8, 0, 0, 0, 0],
            0.0,
            [8, -8, 0, 0, 0, 0, 0],
            [f(NodalFactorFormula::M2, 4), n],
        ),
        "MF" => ConstituentDefinition::new(
            [0, 2, 0, 0, 0],
            0.0,
            [-2, 0, 0, 0, 0, 0, 0],
            [f(NodalFactorFormula::Mf, 1), n],
        ),
        "MK3" => ConstituentDefinition::new(
            [3, 1, 0, 0, 0],
            -0.25,
            [2, -2, -1, 0, 0, 0, 0],
            [f(NodalFactorFormula::M2, 1), f(NodalFactorFormula::K1, 1)],
        ),
        "MM" => ConstituentDefinition::new(
            [0, 1, 0, -1, 0],
            0.0,
            [0, 0, 0, 0, 0, 0, 0],
            [f(NodalFactorFormula::Mm, 1), n],
        ),
        "MN4" => ConstituentDefinition::new(
            [4, -1, 0, 1, 0],
            0.0,
            [4, -4, 0, 0, 0, 0, 0],
            [f(NodalFactorFormula::M2, 2), n],
        ),
        "MS4" => ConstituentDefinition::new(
            [4, 2, -2, 0, 0],
            0.0,
            [2, -2, 0, 0, 0, 0, 0],
            [f(NodalFactorFormula::M2, 1), n],
        ),
        "MSF" => ConstituentDefinition::new(
            [0, 2, -2, 0, 0],
            0.0,
            [-2, 2, 0, 0, 0, 0, 0],
            [f(NodalFactorFormula::M2, 1), n],
        ),
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
        "P1" => ConstituentDefinition::new([1, 1, -2, 0, 0], 0.25, [0, 0, 0, 0, 0, 0, 0], [n, n]),
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
        "S4" => ConstituentDefinition::new([4, 4, -4, 0, 0], 0.0, [0, 0, 0, 0, 0, 0, 0], [n, n]),
        "S6" => ConstituentDefinition::new([6, 6, -6, 0, 0], 0.0, [0, 0, 0, 0, 0, 0, 0], [n, n]),
        "SA" => ConstituentDefinition::new([0, 0, 1, 0, 0], 0.0, [0, 0, 0, 0, 0, 0, 0], [n, n]),
        "SSA" => ConstituentDefinition::new([0, 0, 2, 0, 0], 0.0, [0, 0, 0, 0, 0, 0, 0], [n, n]),
        "T2" => ConstituentDefinition::new([2, 2, -3, 0, 1], 0.0, [0, 0, 0, 0, 0, 0, 0], [n, n]),
        _ => return None,
    };
    Some(definition)
}
