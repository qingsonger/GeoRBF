//! Assemble a mixed hard-equality field system without solving it.

use std::error::Error;
use std::io;
use std::num::NonZeroUsize;

use georbf::{
    CenterRepresenter, Enforcement, ExecutionOptions, FieldProblem, FunctionalAtom, FunctionalExpr,
    FunctionalProvenance, FunctionalTerm, Gaussian, ObservationFunctional, ObservationId, Point,
    RadialSeparation, SemanticConstraint, SemanticExpression, SemanticProblemIr,
    SemanticProvenance, SemanticRelation, SourceLocation, SpatialKernelJet, UnitDirection,
};

fn expression(
    point: [f64; 2],
    direction: [f64; 2],
    identifier: u64,
) -> Result<FunctionalExpr<2>, Box<dyn Error>> {
    let point = Point::try_new(point)?;
    Ok(FunctionalExpr::try_new([
        FunctionalTerm::try_new(
            1.0,
            FunctionalAtom::value(point, FunctionalProvenance::new(identifier)),
        )?,
        FunctionalTerm::try_new(
            0.2,
            FunctionalAtom::directional_derivative(
                point,
                UnitDirection::try_new(direction)?,
                FunctionalProvenance::new(identifier + 1),
            ),
        )?,
    ])?)
}

fn provenance(identifier: u64) -> Result<SemanticProvenance, Box<dyn Error>> {
    Ok(SemanticProvenance::try_new(
        ObservationId::new(identifier),
        SourceLocation::try_new(
            "example/field.csv".to_owned(),
            NonZeroUsize::new(usize::try_from(identifier)?.checked_add(1).ok_or("line")?)
                .ok_or("line")?,
        )?,
        "m".to_owned(),
        format!("field.equalities[{identifier}]"),
        Some("example".to_owned()),
    )?)
}

fn main() -> Result<(), Box<dyn Error>> {
    let expressions = [
        expression([-0.5, 0.25], [1.0, 1.0], 10)?,
        expression([0.75, -0.5], [-1.0, 2.0], 20)?,
    ];
    let mut constraints = Vec::new();
    let mut centers = Vec::new();
    for (index, expression) in expressions.into_iter().enumerate() {
        let index_u32 = u32::try_from(index)?;
        centers.push(CenterRepresenter::new(expression.clone()));
        constraints.push(SemanticConstraint::try_new(
            provenance(u64::from(index_u32))?,
            SemanticRelation::Equality {
                expression: SemanticExpression::try_new(
                    ObservationFunctional::new(expression),
                    0.0,
                )?,
                target: f64::from(index_u32),
            },
            Enforcement::Hard,
        )?);
    }
    let semantic = SemanticProblemIr::try_new(constraints, ExecutionOptions::default())?;
    let problem = FieldProblem::try_new(semantic, centers)?;
    let kernel = Gaussian::try_new(1.5)?;
    let system = problem.try_assemble(kernel.metadata(), |query, center, _| {
        let separation = RadialSeparation::try_new(query, center)
            .map_err(|error| io::Error::other(error.to_string()))?;
        let radial = kernel
            .radial_jet(separation)
            .map_err(|error| io::Error::other(error.to_string()))?;
        Ok::<_, io::Error>(
            SpatialKernelJet::try_new(separation, radial)
                .map_err(|error| io::Error::other(error.to_string()))?
                .into(),
        )
    })?;

    println!(
        "dimension={} rhs={:?} upper_evaluations={} normalized_asymmetry={}",
        system.matrix().dimension(),
        system.rhs(),
        system.diagnostics().kernel_entry_evaluations,
        system.diagnostics().normalized_asymmetry,
    );
    Ok(())
}
