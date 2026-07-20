//! Compile a scalar level gap and a distinct sampled local thickness cone.

use std::error::Error;
use std::num::NonZeroUsize;

use georbf::{
    AffineExpression, AffineTerm, FunctionalAtom, FunctionalExpr, FunctionalProvenance,
    FunctionalTerm, LevelDefinition, LevelId, LevelMembership, LevelOrder, LevelProblem,
    LevelValue, LocalNormalThickness, ObservationFunctional, ObservationId, Point, ProblemIrError,
    SemanticProvenance, SourceLocation, VariableBlock,
};

fn provenance(identifier: u64, path: &str) -> Result<SemanticProvenance, ProblemIrError> {
    SemanticProvenance::try_new(
        ObservationId::new(identifier),
        SourceLocation::try_new("thickness.csv".to_owned(), NonZeroUsize::MIN)?,
        "m".to_owned(),
        path.to_owned(),
        Some("example".to_owned()),
    )
}

fn value(identifier: u64, x: f64) -> Result<ObservationFunctional<2>, Box<dyn Error>> {
    Ok(ObservationFunctional::new(FunctionalExpr::try_new([
        FunctionalTerm::try_new(
            1.0,
            FunctionalAtom::value(
                Point::try_new([x, 0.0])?,
                FunctionalProvenance::new(identifier),
            ),
        )?,
    ])?))
}

fn main() -> Result<(), Box<dyn Error>> {
    let lower = LevelId::new(10);
    let upper = LevelId::new(20);
    let scalar_gap = LevelOrder::try_new(lower, upper, 2.0, provenance(5, "levels.scalar_gap")?)?;
    let scalar_diagnostics = scalar_gap.thickness_diagnostics();
    let levels = LevelProblem::try_new(
        [
            LevelDefinition::new(
                lower,
                LevelValue::try_fixed(0.0)?,
                provenance(1, "levels.lower")?,
            ),
            LevelDefinition::new(
                upper,
                LevelValue::try_fixed(10.0)?,
                provenance(2, "levels.upper")?,
            ),
        ],
        [
            LevelMembership::new(lower, value(3, 0.0)?, provenance(3, "memberships.lower")?),
            LevelMembership::new(upper, value(4, 10.0)?, provenance(4, "memberships.upper")?),
        ],
        [scalar_gap],
    )?;
    let local = LocalNormalThickness::try_new(
        lower,
        upper,
        Point::try_new([5.0, 0.0])?,
        2.0,
        provenance(6, "thickness.samples[0]")?,
    )?;
    let local_diagnostics = local.diagnostics();
    let mut membership_variable = 0_usize;
    let compiled = levels.try_compile(
        [VariableBlock::try_new(
            "field".to_owned(),
            NonZeroUsize::new(4).ok_or("field variables")?,
        )?],
        |_, _| {
            let variable = membership_variable;
            membership_variable += 1;
            AffineExpression::try_new([AffineTerm::try_new(variable, 1.0)?], 0.0)
        },
    )?;
    let compiled = compiled.try_compose_local_normal_thickness([local], |functional, _| {
        let FunctionalAtom::DirectionalDerivative { direction, .. } =
            functional.expression().terms()[0].atom()
        else {
            return Err(ProblemIrError::MemoryEstimateOverflow);
        };
        let axis = direction
            .components()
            .iter()
            .position(|component| component.to_bits() == 1.0_f64.to_bits())
            .ok_or(ProblemIrError::MemoryEstimateOverflow)?;
        AffineExpression::try_new([AffineTerm::try_new(axis + 2, 1.0)?], 0.0)
    })?;

    println!(
        "scalar={:?}/{:?} local={:?}/{:?} bounds={} cones={}",
        scalar_diagnostics.kind(),
        scalar_diagnostics.guarantee(),
        local_diagnostics.kind(),
        local_diagnostics.guarantee(),
        compiled.canonical_problem().linear_bounds().len(),
        compiled.canonical_problem().second_order_cones().len(),
    );
    Ok(())
}
