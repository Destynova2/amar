use crate::{CalError, Observation};
use amar_core::{
    ConstituentId, DatumId, Degrees, DegreesPerHour, HarmonicConstituent, HarmonicYearContext,
    Meters, PredictionMethod, TideModel, UtcDateTime,
};
use amar_pack::{ConstituentPack, DegreesPerHourValue, DegreesValue, MetersValue};
use chrono::{DateTime, Datelike, Utc};
use clap::ValueEnum;
use nalgebra::{DMatrix, DVector};
use std::collections::BTreeMap;

const MIN_ANNUAL_WINDOW_DAYS: i64 = 365;

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
}

impl ConstituentSet {
    pub(crate) fn specs(self) -> &'static [ConstituentSpec] {
        match self {
            Self::M2Base16 => &M2_BASE16,
            Self::M22Rayleigh37 => &M22_RAYLEIGH37,
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
}

#[derive(Debug)]
pub(crate) struct CalibrationResult {
    pub(crate) z0_m: f64,
    pub(crate) constituents: Vec<ConstituentPack>,
    pub(crate) model: TideModel,
}

pub(crate) fn calibrate(
    samples: &[Observation],
    calibration_start: DateTime<Utc>,
    calibration_end: DateTime<Utc>,
    constituent_set: ConstituentSet,
) -> Result<CalibrationResult, CalError> {
    if samples.is_empty() {
        return Err(CalError::EmptyObservations("calibration".to_string()));
    }
    let constituent_specs = constituent_set.specs();
    enforce_resolvable_window(calibration_start, calibration_end, constituent_specs)?;

    let columns = 1 + constituent_specs.len() * 2;
    let mut matrix = Vec::with_capacity(samples.len() * columns);
    let mut values = Vec::with_capacity(samples.len());
    let mut year_contexts = BTreeMap::new();
    let ids = constituent_specs
        .iter()
        .map(|spec| {
            Ok((
                spec,
                ConstituentId::new(spec.name)?,
                DegreesPerHour::new(spec.speed_deg_per_hour)?,
            ))
        })
        .collect::<Result<Vec<_>, CalError>>()?;

    for sample in samples {
        matrix.push(1.0);
        let at = UtcDateTime::from_utc(sample.at);
        let year_context = year_contexts
            .entry(sample.at.year())
            .or_insert_with(|| HarmonicYearContext::new(at));
        for (_, id, speed) in &ids {
            let basis = year_context.basis(id, *speed, at)?;
            let radians = basis.argument_degrees.to_radians();
            matrix.push(basis.nodal_factor * radians.cos());
            matrix.push(basis.nodal_factor * radians.sin());
        }
        values.push(sample.value_m);
    }

    let a = DMatrix::from_row_slice(samples.len(), columns, &matrix);
    let y = DVector::from_row_slice(&values);
    let solution = a
        .svd(true, true)
        .solve(&y, 1.0e-10)
        .map_err(|_| CalError::SolveFailed)?;

    let z0_m = solution[0];
    let mut constituents = Vec::with_capacity(constituent_specs.len());
    let mut model_constituents = Vec::with_capacity(constituent_specs.len());
    for (index, (spec, id, speed)) in ids.iter().enumerate() {
        let cos_coefficient = solution[1 + index * 2];
        let sin_coefficient = solution[1 + index * 2 + 1];
        let amplitude_m = cos_coefficient.hypot(sin_coefficient);
        let phase_gmt_deg = sin_coefficient
            .atan2(cos_coefficient)
            .to_degrees()
            .rem_euclid(360.0);
        constituents.push(ConstituentPack {
            name: spec.name.to_string(),
            amplitude_m: MetersValue::new(amplitude_m),
            phase_gmt_deg: DegreesValue::new(phase_gmt_deg),
            speed_deg_per_hour: DegreesPerHourValue::new(spec.speed_deg_per_hour),
        });
        model_constituents.push(HarmonicConstituent::new(
            id.clone(),
            Meters::new(amplitude_m)?,
            Degrees::new(phase_gmt_deg)?,
            *speed,
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
        constituents,
        model,
    })
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
        match crate::parse_rfc3339(value) {
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
}
