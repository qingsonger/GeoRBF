//! Compile two exact tangents at one point with an explicit value gauge.

use std::error::Error;
use std::num::NonZeroUsize;

use georbf::{
    DerivativeGaugeAnchor, Enforcement, ExecutionOptions, ObservationId, Point, ProblemIrError,
    SemanticProvenance, SourceLocation, TangentObservation, TangentProblem, UnitDirection,
};

fn provenance(identifier: u64, units: &str) -> Result<SemanticProvenance, ProblemIrError> {
    SemanticProvenance::try_new(
        ObservationId::new(identifier),
        SourceLocation::try_new("tangents.csv".to_owned(), NonZeroUsize::MIN)?,
        units.to_owned(),
        "fields.stratigraphy.tangents".to_owned(),
        Some("example".to_owned()),
    )
}

fn main() -> Result<(), Box<dyn Error>> {
    let point = Point::try_new([1.0, 2.0, 3.0])?;
    let strike = TangentObservation::try_new(
        provenance(1, "1/m")?,
        point,
        UnitDirection::try_new([1.0, 0.0, 0.0])?,
        Enforcement::Hard,
    )?;
    let dip = TangentObservation::try_new(
        provenance(2, "1/m")?,
        point,
        UnitDirection::try_new([0.0, 1.0, 0.0])?,
        Enforcement::Hard,
    )?;
    let gauge = DerivativeGaugeAnchor::try_new(
        provenance(3, "m")?,
        Point::try_new([0.0, 0.0, 0.0])?,
        125.0,
    )?;
    let problem = TangentProblem::try_new([strike, dip], Some(gauge), ExecutionOptions::default())?;

    println!(
        "tangents={} gauge_id={} gauge_value={}",
        problem.tangent_count(),
        problem.gauge_observation_id().identifier(),
        problem.gauge_value()
    );
    Ok(())
}
