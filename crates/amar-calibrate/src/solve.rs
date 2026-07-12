use crate::common::{CalError, Observation};
use amar_core::{
    ConstituentId, DatumId, Degrees, DegreesPerHour, HarmonicConstituent, HarmonicYearContext,
    Meters, PredictionMethod, TideModel, UtcDateTime, constituent_speed_degrees_per_hour,
    port_selection_constituent_names,
};
use amar_pack::{ConstituentPack, DegreesPerHourValue, DegreesValue, MetersValue};
use chrono::{DateTime, Datelike, Utc};
use clap::ValueEnum;
use nalgebra::{DMatrix, DVector};
use std::collections::BTreeMap;

const MIN_ANNUAL_WINDOW_DAYS: i64 = 365;
const RAYLEIGH_MIN: f64 = 1.0;
const SNR_MIN: f64 = 2.0;

pub(crate) const M2_BASE16: [ConstituentSpec; 16] = [
    ConstituentSpec::new("M2", 28.984_104),
    ConstituentSpec::new("S2", 30.0),
    ConstituentSpec::new("N2", 28.439_73),
    ConstituentSpec::new("K2", 30.082_138),
    ConstituentSpec::new("K1", 15.041_069),
    ConstituentSpec::new("O1", 13.943_035),
    ConstituentSpec::new("P1", 14.958_931),
    ConstituentSpec::new("Q1", 13.398_661),
    ConstituentSpec::new("M4", 57.968_21),
    ConstituentSpec::new("MS4", 58.984_104),
    ConstituentSpec::new("MN4", 57.423_832),
    ConstituentSpec::new("M6", 86.952_32),
    ConstituentSpec::new("MF", 1.098_033_1),
    ConstituentSpec::new("MM", 0.544_374_7),
    ConstituentSpec::new("SA", 0.041_068_6),
    ConstituentSpec::new("SSA", 0.082_137_3),
];

pub(crate) fn port_selection_catalog() -> Result<Vec<ConstituentSpec>, CalError> {
    port_selection_constituent_names()
        .iter()
        .map(|name| ConstituentSpec::from_core(name))
        .collect()
}

pub(crate) const M22_RAYLEIGH37: [ConstituentSpec; 37] = [
    ConstituentSpec::new("M2", 28.984_104),
    ConstituentSpec::new("S2", 30.0),
    ConstituentSpec::new("N2", 28.439_73),
    ConstituentSpec::new("K2", 30.082_138),
    ConstituentSpec::new("K1", 15.041_069),
    ConstituentSpec::new("O1", 13.943_035),
    ConstituentSpec::new("P1", 14.958_931),
    ConstituentSpec::new("Q1", 13.398_661),
    ConstituentSpec::new("M4", 57.968_21),
    ConstituentSpec::new("MS4", 58.984_104),
    ConstituentSpec::new("MN4", 57.423_832),
    ConstituentSpec::new("M6", 86.952_32),
    ConstituentSpec::new("MF", 1.098_033_1),
    ConstituentSpec::new("MM", 0.544_374_7),
    ConstituentSpec::new("SA", 0.041_068_6),
    ConstituentSpec::new("SSA", 0.082_137_3),
    ConstituentSpec::new("L2", 29.528_479),
    ConstituentSpec::new("NU2", 28.512_583),
    ConstituentSpec::new("MU2", 27.968_208),
    ConstituentSpec::new("2N2", 27.895_355),
    ConstituentSpec::new("LAM2", 29.455_626),
    ConstituentSpec::new("T2", 29.958_933),
    ConstituentSpec::new("R2", 30.041_067),
    ConstituentSpec::new("J1", 15.585_443_5),
    ConstituentSpec::new("OO1", 16.139_101),
    ConstituentSpec::new("RHO", 13.471_515),
    ConstituentSpec::new("2Q1", 12.854_286),
    ConstituentSpec::new("M1", 14.496_694),
    ConstituentSpec::new("S1", 15.0),
    ConstituentSpec::new("MK3", 44.025_173),
    ConstituentSpec::new("2MK3", 42.927_14),
    ConstituentSpec::new("M3", 43.476_16),
    ConstituentSpec::new("S4", 60.0),
    ConstituentSpec::new("S6", 90.0),
    ConstituentSpec::new("M8", 115.936_42),
    ConstituentSpec::new("MSF", 1.015_895_8),
    ConstituentSpec::new("2SM2", 31.015_896),
];

#[derive(Clone, Copy, Debug, ValueEnum)]
pub(crate) enum ConstituentSet {
    M2Base16,
    M22Rayleigh37,
    PortSelectionCatalog,
}

impl ConstituentSet {
    pub(crate) fn specs(self) -> Result<Vec<ConstituentSpec>, CalError> {
        match self {
            Self::M2Base16 => Ok(M2_BASE16.to_vec()),
            Self::M22Rayleigh37 => Ok(M22_RAYLEIGH37.to_vec()),
            Self::PortSelectionCatalog => port_selection_catalog(),
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) struct ConstituentSpec {
    pub(crate) name: &'static str,
    pub(crate) speed_deg_per_hour: f64,
}

impl ConstituentSpec {
    const fn new(name: &'static str, speed_deg_per_hour: f64) -> Self {
        Self {
            name,
            speed_deg_per_hour,
        }
    }

    fn from_core(name: &'static str) -> Result<Self, CalError> {
        let speed = constituent_speed_degrees_per_hour(name)
            .ok_or_else(|| CalError::QualityGate(format!("missing core speed for {name}")))?;
        Ok(Self::new(name, speed))
    }
}

#[derive(Debug)]
pub(crate) struct CalibrationResult {
    pub(crate) z0_m: f64,
    pub(crate) constituents: Vec<ConstituentPack>,
    pub(crate) model: TideModel,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct ConstituentSelection {
    pub(crate) rayleigh_count: usize,
    pub(crate) significant_count: usize,
}

#[derive(Clone)]
pub(crate) struct PreparedConstituent {
    pub(crate) spec: ConstituentSpec,
    pub(crate) id: ConstituentId,
    pub(crate) speed: DegreesPerHour,
}

pub(crate) struct DesignSystem {
    pub(crate) matrix: DMatrix<f64>,
    pub(crate) values: DVector<f64>,
}

struct FitResult {
    prepared_constituents: Vec<PreparedConstituent>,
    solution: DVector<f64>,
    covariance: DMatrix<f64>,
}

struct PreparedYearContext {
    context: HarmonicYearContext,
    constituents: Vec<PreparedYearConstituent>,
}

struct PreparedYearConstituent {
    argument0_degrees: f64,
    nodal_phase_degrees: f64,
    nodal_factor: f64,
    speed_degrees_per_hour: f64,
}

pub(crate) fn calibrate(
    samples: &[Observation],
    calibration_start: DateTime<Utc>,
    calibration_end: DateTime<Utc>,
    constituent_set: ConstituentSet,
) -> Result<CalibrationResult, CalError> {
    let constituent_specs = constituent_set.specs()?;
    calibrate_specs(
        samples,
        calibration_start,
        calibration_end,
        &constituent_specs,
    )
}

pub(crate) fn calibrate_with_port_selection(
    samples: &[Observation],
    calibration_start: DateTime<Utc>,
    calibration_end: DateTime<Utc>,
) -> Result<(CalibrationResult, ConstituentSelection), CalError> {
    let catalog = port_selection_catalog()?;
    let rayleigh = rayleigh_select(calibration_start, calibration_end, &catalog)?;
    let first_pass = fit_coefficients(samples, calibration_start, calibration_end, &rayleigh)?;
    let significant = significant_select(&rayleigh, &first_pass, SNR_MIN);
    if significant.is_empty() {
        return Err(CalError::QualityGate(
            "constituent selection retained no significant constituents".to_string(),
        ));
    }
    let selection = ConstituentSelection {
        rayleigh_count: rayleigh.len(),
        significant_count: significant.len(),
    };
    let calibration = calibrate_specs(samples, calibration_start, calibration_end, &significant)?;
    Ok((calibration, selection))
}

pub(crate) fn calibrate_specs(
    samples: &[Observation],
    calibration_start: DateTime<Utc>,
    calibration_end: DateTime<Utc>,
    constituent_specs: &[ConstituentSpec],
) -> Result<CalibrationResult, CalError> {
    if samples.is_empty() {
        return Err(CalError::EmptyObservations("calibration".to_string()));
    }
    enforce_resolvable_window(calibration_start, calibration_end, constituent_specs)?;

    let fit = fit_coefficients(
        samples,
        calibration_start,
        calibration_end,
        constituent_specs,
    )?;

    let z0_m = fit.solution[0];
    let mut pack_constituents = Vec::with_capacity(constituent_specs.len());
    let mut model_constituents = Vec::with_capacity(constituent_specs.len());
    for (index, constituent) in fit.prepared_constituents.iter().enumerate() {
        let cos_coefficient = fit.solution[1 + index * 2];
        let sin_coefficient = fit.solution[1 + index * 2 + 1];
        let amplitude_m = cos_coefficient.hypot(sin_coefficient);
        let phase_gmt_deg = sin_coefficient
            .atan2(cos_coefficient)
            .to_degrees()
            .rem_euclid(360.0);
        pack_constituents.push(ConstituentPack {
            name: constituent.spec.name.to_string(),
            amplitude_m: MetersValue::new(amplitude_m),
            phase_gmt_deg: DegreesValue::new(phase_gmt_deg),
            speed_deg_per_hour: DegreesPerHourValue::new(constituent.spec.speed_deg_per_hour),
        });
        model_constituents.push(HarmonicConstituent::new(
            constituent.id.clone(),
            Meters::new(amplitude_m)?,
            Degrees::new(phase_gmt_deg)?,
            constituent.speed,
        ));
    }
    let model = TideModel::new(
        DatumId::new("zero_hydrographique_brest")?,
        Meters::new(z0_m)?,
        model_constituents,
        PredictionMethod::StationHarmonicsV0,
    )?;

    Ok(CalibrationResult {
        z0_m,
        constituents: pack_constituents,
        model,
    })
}

fn fit_coefficients(
    samples: &[Observation],
    calibration_start: DateTime<Utc>,
    calibration_end: DateTime<Utc>,
    constituent_specs: &[ConstituentSpec],
) -> Result<FitResult, CalError> {
    enforce_resolvable_window(calibration_start, calibration_end, constituent_specs)?;
    let prepared_constituents = prepare_constituents(constituent_specs)?;
    let design_system = assemble_design_system(samples, &prepared_constituents)?;
    let matrix = design_system.matrix;
    let values = design_system.values;
    let solution = solve_svd(matrix.clone(), &values)?;
    let residuals = &matrix * &solution - values;
    let degrees_of_freedom = matrix.nrows().saturating_sub(matrix.ncols());
    if degrees_of_freedom == 0 {
        return Err(CalError::QualityGate(format!(
            "not enough samples for {} selected constituents",
            constituent_specs.len()
        )));
    }
    let residual_variance =
        residuals.iter().map(|value| value * value).sum::<f64>() / degrees_of_freedom as f64;
    let xtx = matrix.transpose() * matrix;
    let covariance = xtx.try_inverse().ok_or(CalError::SolveFailed)? * residual_variance;
    Ok(FitResult {
        prepared_constituents,
        solution,
        covariance,
    })
}

fn rayleigh_select(
    calibration_start: DateTime<Utc>,
    calibration_end: DateTime<Utc>,
    catalog: &[ConstituentSpec],
) -> Result<Vec<ConstituentSpec>, CalError> {
    let days = (calibration_end - calibration_start).num_seconds() as f64 / 86_400.0;
    if days <= 0.0 {
        return Err(CalError::InvalidTimestamp(
            "calibration_start must be before calibration_end".to_string(),
        ));
    }
    let min_separation_cpd = RAYLEIGH_MIN / days;
    let mut selected = Vec::new();
    for spec in catalog {
        let cpd = cycles_per_day(spec.speed_deg_per_hour);
        let separable = selected.iter().all(|selected_spec: &ConstituentSpec| {
            (cycles_per_day(selected_spec.speed_deg_per_hour) - cpd).abs() > min_separation_cpd
        });
        if separable {
            selected.push(*spec);
        }
    }
    Ok(selected)
}

fn significant_select(
    specs: &[ConstituentSpec],
    fit: &FitResult,
    min_snr: f64,
) -> Vec<ConstituentSpec> {
    specs
        .iter()
        .enumerate()
        .filter_map(|(index, spec)| {
            let cos_index = 1 + index * 2;
            let sin_index = cos_index + 1;
            let cos_coefficient = fit.solution[cos_index];
            let sin_coefficient = fit.solution[sin_index];
            let amplitude_squared =
                cos_coefficient * cos_coefficient + sin_coefficient * sin_coefficient;
            if amplitude_squared == 0.0 {
                return None;
            }
            let variance = (cos_coefficient
                * cos_coefficient
                * fit.covariance[(cos_index, cos_index)]
                + sin_coefficient * sin_coefficient * fit.covariance[(sin_index, sin_index)]
                + 2.0 * cos_coefficient * sin_coefficient * fit.covariance[(cos_index, sin_index)])
                / amplitude_squared;
            if variance <= 0.0 {
                return Some(*spec);
            }
            let snr = amplitude_squared / variance;
            (snr >= min_snr).then_some(*spec)
        })
        .collect()
}

fn cycles_per_day(speed_deg_per_hour: f64) -> f64 {
    speed_deg_per_hour * 24.0 / 360.0
}

pub(crate) fn prepare_constituents(
    constituent_specs: &[ConstituentSpec],
) -> Result<Vec<PreparedConstituent>, CalError> {
    constituent_specs
        .iter()
        .map(|spec| {
            Ok(PreparedConstituent {
                spec: *spec,
                id: ConstituentId::new(spec.name)?,
                speed: DegreesPerHour::new(spec.speed_deg_per_hour)?,
            })
        })
        .collect()
}

pub(crate) fn assemble_design_system(
    samples: &[Observation],
    constituents: &[PreparedConstituent],
) -> Result<DesignSystem, CalError> {
    let columns = 1 + constituents.len() * 2;
    let mut matrix = DMatrix::zeros(samples.len(), columns);
    let mut values = DVector::zeros(samples.len());
    let mut year_contexts = BTreeMap::new();

    for (row, sample) in samples.iter().enumerate() {
        matrix[(row, 0)] = 1.0;
        let at = UtcDateTime::from_utc(sample.at);
        let year_context = match year_contexts.entry(sample.at.year()) {
            std::collections::btree_map::Entry::Occupied(entry) => entry.into_mut(),
            std::collections::btree_map::Entry::Vacant(entry) => {
                entry.insert(prepare_year_context(at, constituents)?)
            }
        };
        let hours = year_context.context.hours_since_year_start(at)?;
        for (index, constituent) in year_context.constituents.iter().enumerate() {
            let radians = (constituent.argument0_degrees
                + constituent.speed_degrees_per_hour * hours
                + constituent.nodal_phase_degrees)
                .to_radians();
            let (sin, cos) = radians.sin_cos();
            let column = 1 + index * 2;
            matrix[(row, column)] = constituent.nodal_factor * cos;
            matrix[(row, column + 1)] = constituent.nodal_factor * sin;
        }
        values[row] = sample.value_m;
    }

    Ok(DesignSystem { matrix, values })
}

fn prepare_year_context(
    at: UtcDateTime,
    constituents: &[PreparedConstituent],
) -> Result<PreparedYearContext, CalError> {
    let context = HarmonicYearContext::new(at);
    let constituents = constituents
        .iter()
        .map(|constituent| {
            let basis = context.annual_basis(&constituent.id)?;
            Ok(PreparedYearConstituent {
                argument0_degrees: basis.argument0_degrees,
                nodal_phase_degrees: basis.nodal_phase_degrees,
                nodal_factor: basis.nodal_factor,
                speed_degrees_per_hour: constituent.speed.as_degrees_per_hour(),
            })
        })
        .collect::<Result<Vec<_>, CalError>>()?;
    Ok(PreparedYearContext {
        context,
        constituents,
    })
}

pub(crate) fn solve_svd(
    matrix: DMatrix<f64>,
    values: &DVector<f64>,
) -> Result<DVector<f64>, CalError> {
    matrix
        .svd(true, true)
        .solve(values, 1.0e-10)
        .map_err(|_| CalError::SolveFailed)
}

fn enforce_resolvable_window(
    calibration_start: DateTime<Utc>,
    calibration_end: DateTime<Utc>,
    constituent_specs: &[ConstituentSpec],
) -> Result<(), CalError> {
    let has_annual_terms = constituent_specs
        .iter()
        .any(|spec| matches!(spec.name, "SA" | "SSA"));
    let days = (calibration_end - calibration_start).num_days();
    if has_annual_terms && days < MIN_ANNUAL_WINDOW_DAYS {
        return Err(CalError::UnresolvableAnnualConstituents {
            days,
            required_days: MIN_ANNUAL_WINDOW_DAYS,
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(value: &str) -> DateTime<Utc> {
        match crate::common::parse_rfc3339(value) {
            Ok(value) => value,
            Err(error) => panic!("{error:?}"),
        }
    }

    #[test]
    fn annual_terms_require_one_year_of_calibration() {
        let samples = [Observation {
            at: at("2026-01-01T00:00:00Z"),
            value_m: 1.0,
            source: 4,
        }];

        let error = match calibrate(
            &samples,
            at("2026-01-01T00:00:00Z"),
            at("2026-06-01T00:00:00Z"),
            ConstituentSet::M2Base16,
        ) {
            Ok(_) => panic!("expected annual resolvability error"),
            Err(error) => error,
        };

        assert!(matches!(
            error,
            CalError::UnresolvableAnnualConstituents {
                days: 151,
                required_days: 365
            }
        ));
    }

    #[test]
    fn port_selection_picks_expected_constituents_on_synthetic_signal() {
        let calibration_start = at("2026-01-01T00:00:00Z");
        let calibration_end = at("2026-07-20T00:00:00Z");
        let catalog = [
            ConstituentSpec::new("M2", 28.984_104),
            ConstituentSpec::new("S2", 30.0),
            ConstituentSpec::new("T2", 29.958_933),
        ];
        let rayleigh = match rayleigh_select(calibration_start, calibration_end, &catalog) {
            Ok(value) => value,
            Err(error) => panic!("{error:?}"),
        };
        let rayleigh_names = rayleigh.iter().map(|spec| spec.name).collect::<Vec<_>>();

        assert_eq!(rayleigh_names, ["M2", "S2"]);

        let model = match TideModel::new(
            match DatumId::new("zero_hydrographique_synthetic") {
                Ok(value) => value,
                Err(error) => panic!("{error:?}"),
            },
            match Meters::new(2.0) {
                Ok(value) => value,
                Err(error) => panic!("{error:?}"),
            },
            vec![
                HarmonicConstituent::new(
                    match ConstituentId::new("M2") {
                        Ok(value) => value,
                        Err(error) => panic!("{error:?}"),
                    },
                    match Meters::new(0.8) {
                        Ok(value) => value,
                        Err(error) => panic!("{error:?}"),
                    },
                    match Degrees::new(40.0) {
                        Ok(value) => value,
                        Err(error) => panic!("{error:?}"),
                    },
                    match DegreesPerHour::new(28.984_104) {
                        Ok(value) => value,
                        Err(error) => panic!("{error:?}"),
                    },
                ),
                HarmonicConstituent::new(
                    match ConstituentId::new("S2") {
                        Ok(value) => value,
                        Err(error) => panic!("{error:?}"),
                    },
                    match Meters::new(0.4) {
                        Ok(value) => value,
                        Err(error) => panic!("{error:?}"),
                    },
                    match Degrees::new(120.0) {
                        Ok(value) => value,
                        Err(error) => panic!("{error:?}"),
                    },
                    match DegreesPerHour::new(30.0) {
                        Ok(value) => value,
                        Err(error) => panic!("{error:?}"),
                    },
                ),
            ],
            PredictionMethod::StationHarmonicsV0,
        ) {
            Ok(value) => value,
            Err(error) => panic!("{error:?}"),
        };
        let mut samples = Vec::new();
        let mut cursor = calibration_start;
        while cursor < calibration_end {
            let at = UtcDateTime::from_utc(cursor);
            samples.push(Observation {
                at: cursor,
                value_m: amar_core::predict_height(&model, at).height().as_meters(),
                source: 4,
            });
            cursor += chrono::Duration::hours(6);
        }

        let first_pass =
            match fit_coefficients(&samples, calibration_start, calibration_end, &rayleigh) {
                Ok(value) => value,
                Err(error) => panic!("{error:?}"),
            };
        let significant = significant_select(&rayleigh, &first_pass, SNR_MIN);
        let significant_names = significant.iter().map(|spec| spec.name).collect::<Vec<_>>();

        assert_eq!(significant_names, ["M2", "S2"]);
    }
}
