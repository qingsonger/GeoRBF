//! Independent truth and policy tests for dense equality solving.

use std::error::Error;
use std::io;
use std::num::NonZeroUsize;

use georbf::{
    CenterRepresenter, ConditionPolicy, DenseEqualitySystem, DenseFactorization, DenseRankDecision,
    DenseSolveError, DenseSolveOptions, Enforcement, ExecutionOptions, FieldProblem,
    FunctionalAtom, FunctionalExpr, FunctionalProvenance, FunctionalTerm, Gaussian,
    ObservationFunctional, ObservationId, Point, RadialSeparation, Regularization,
    SemanticConstraint, SemanticExpression, SemanticProblemIr, SemanticProvenance,
    SemanticRelation, SourceLocation, SpatialKernelJet, try_solve_field,
};

fn options(
    factorization: DenseFactorization,
    regularization: Regularization,
    condition_policy: ConditionPolicy,
) -> Result<DenseSolveOptions, Box<dyn Error>> {
    Ok(DenseSolveOptions::try_new(
        factorization,
        regularization,
        condition_policy,
        4,
    )?)
}

fn assert_close(actual: &[f64], expected: &[f64], tolerance: f64) {
    assert_eq!(actual.len(), expected.len());
    for (index, (actual, expected)) in actual.iter().zip(expected).enumerate() {
        assert!(
            (actual - expected).abs() <= tolerance * expected.abs().max(1.0),
            "solution[{index}] expected {expected:.17e}, got {actual:.17e}"
        );
    }
}

#[test]
fn checked_cholesky_matches_independent_spd_truth() -> Result<(), Box<dyn Error>> {
    let system =
        DenseEqualitySystem::try_from_row_major(2, vec![4.0, 1.0, 1.0, 3.0], vec![6.0, 7.0])?;
    let solution = system.try_solve(options(
        DenseFactorization::Cholesky,
        Regularization::None,
        ConditionPolicy::default(),
    )?)?;
    assert_close(solution.values(), &[1.0, 2.0], 8.0 * f64::EPSILON);
    let diagnostics = solution.diagnostics();
    assert_eq!(
        diagnostics.requested_factorization,
        DenseFactorization::Cholesky
    );
    assert_eq!(
        diagnostics.actual_factorization,
        DenseFactorization::Cholesky
    );
    assert_eq!(
        diagnostics.original_rank.decision,
        DenseRankDecision::FullRank
    );
    assert_eq!(
        diagnostics.effective_rank.decision,
        DenseRankDecision::FullRank
    );
    assert!(!diagnostics.has_two_by_two_pivot);
    assert!(diagnostics.final_residual.original_backward_error <= diagnostics.residual_tolerance);
    assert!(
        diagnostics.final_residual.original_infinity
            <= diagnostics.initial_residual.original_infinity
    );
    Ok(())
}

#[test]
fn assembled_field_system_crosses_solver_boundary() -> Result<(), Box<dyn Error>> {
    let point = Point::try_new([0.0])?;
    let expression = FunctionalExpr::try_new([FunctionalTerm::try_new(
        1.0,
        FunctionalAtom::value(point, FunctionalProvenance::new(1)),
    )?])?;
    let provenance = SemanticProvenance::try_new(
        ObservationId::new(1),
        SourceLocation::try_new(
            "solver-test.csv".to_owned(),
            NonZeroUsize::new(1).ok_or("line")?,
        )?,
        "m".to_owned(),
        "field.equalities[0]".to_owned(),
        Some("solver integration".to_owned()),
    )?;
    let constraint = SemanticConstraint::try_new(
        provenance,
        SemanticRelation::Equality {
            expression: SemanticExpression::try_new(
                ObservationFunctional::new(expression.clone()),
                0.0,
            )?,
            target: 3.0,
        },
        Enforcement::Hard,
    )?;
    let problem = FieldProblem::try_new(
        SemanticProblemIr::try_new([constraint], ExecutionOptions::default())?,
        [CenterRepresenter::new(expression)],
    )?;
    let kernel = Gaussian::try_new(1.0)?;
    let assembled = problem.try_assemble(kernel.metadata(), |query, center, _| {
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
    let solution = try_solve_field(
        &assembled,
        options(
            DenseFactorization::Cholesky,
            Regularization::None,
            ConditionPolicy::default(),
        )?,
    )?;
    assert_close(solution.values(), &[3.0], f64::EPSILON);
    Ok(())
}

#[test]
fn pivoted_lblt_solves_mandatory_two_by_two_pivot_truth() -> Result<(), Box<dyn Error>> {
    let system =
        DenseEqualitySystem::try_from_row_major(2, vec![0.0, 2.0, 2.0, 0.0], vec![-4.0, 2.0])?;
    let solution = system.try_solve(options(
        DenseFactorization::PivotedLblt,
        Regularization::None,
        ConditionPolicy::default(),
    )?)?;
    assert_close(solution.values(), &[1.0, -2.0], 8.0 * f64::EPSILON);
    assert!(solution.diagnostics().has_two_by_two_pivot);

    let cholesky = system.try_solve(options(
        DenseFactorization::Cholesky,
        Regularization::None,
        ConditionPolicy::default(),
    )?);
    assert!(matches!(cholesky, Err(DenseSolveError::CholeskyRejected)));
    Ok(())
}

#[test]
fn exact_rank_failure_is_never_hidden() -> Result<(), Box<dyn Error>> {
    let system =
        DenseEqualitySystem::try_from_row_major(2, vec![1.0, 1.0, 1.0, 1.0], vec![2.0, 2.0])?;
    let result = system.try_solve(options(
        DenseFactorization::PivotedLblt,
        Regularization::None,
        ConditionPolicy::default(),
    )?);
    let diagnostics = match result {
        Err(DenseSolveError::RankDeficient { diagnostics }) => diagnostics,
        other => return Err(format!("expected structured rank failure, got {other:?}").into()),
    };
    assert_eq!(diagnostics.decision, DenseRankDecision::Deficient);
    assert_eq!(diagnostics.rrqr_rank, 1);
    assert_eq!(diagnostics.svd_rank, 1);
    assert!(!diagnostics.rank_disagreement);
    Ok(())
}

#[test]
fn power_of_two_unit_scaling_preserves_solution_and_rank() -> Result<(), Box<dyn Error>> {
    let matrix = vec![4.0, 1.0, 1.0, 3.0];
    let rhs = vec![6.0, 7.0];
    let scale = 2.0_f64.powi(500);
    let scaled_matrix = matrix.iter().map(|value| value * scale).collect();
    let scaled_rhs = rhs.iter().map(|value| value * scale).collect();
    let first = DenseEqualitySystem::try_from_row_major(2, matrix, rhs)?.try_solve(options(
        DenseFactorization::Cholesky,
        Regularization::None,
        ConditionPolicy::default(),
    )?)?;
    let second = DenseEqualitySystem::try_from_row_major(2, scaled_matrix, scaled_rhs)?.try_solve(
        options(
            DenseFactorization::Cholesky,
            Regularization::None,
            ConditionPolicy::default(),
        )?,
    )?;
    assert_close(first.values(), second.values(), 16.0 * f64::EPSILON);
    assert_eq!(
        first.diagnostics().effective_rank.decision,
        second.diagnostics().effective_rank.decision
    );
    assert_eq!(
        first.diagnostics().effective_rank.rrqr_rank,
        second.diagnostics().effective_rank.rrqr_rank
    );
    assert_eq!(
        first.diagnostics().effective_rank.svd_rank,
        second.diagnostics().effective_rank.svd_rank
    );
    Ok(())
}

#[test]
fn condition_warning_and_explicit_error_are_diagnostic_only() -> Result<(), Box<dyn Error>> {
    let coupling = 1.0 - 1.0e-10;
    let system = DenseEqualitySystem::try_from_row_major(
        2,
        vec![1.0, coupling, coupling, 1.0],
        vec![1.0 - coupling, coupling - 1.0],
    )?;
    let warning = system.try_solve(options(
        DenseFactorization::Cholesky,
        Regularization::None,
        ConditionPolicy::try_new(1.0e8, None)?,
    )?)?;
    assert_close(warning.values(), &[1.0, -1.0], 2.0e-6);
    assert!(warning.diagnostics().condition_warning);
    assert!(warning.diagnostics().effective_rank.condition_estimate > 1.0e8);

    let rejected = system.try_solve(options(
        DenseFactorization::Cholesky,
        Regularization::None,
        ConditionPolicy::try_new(1.0e6, Some(1.0e8))?,
    )?);
    assert!(matches!(
        rejected,
        Err(DenseSolveError::ConditionLimitExceeded { .. })
    ));
    Ok(())
}

#[test]
fn original_unit_residual_retains_exact_binary_roundoff() -> Result<(), Box<dyn Error>> {
    let system = DenseEqualitySystem::try_from_row_major(1, vec![10.0], vec![1.0])?;
    let solution = system.try_solve(options(
        DenseFactorization::Cholesky,
        Regularization::None,
        ConditionPolicy::default(),
    )?)?;
    assert_close(solution.values(), &[0.1], f64::EPSILON);
    assert!(solution.diagnostics().final_residual.original_infinity > 0.0);
    assert!(
        solution
            .diagnostics()
            .final_residual
            .original_backward_error
            <= solution.diagnostics().residual_tolerance
    );
    Ok(())
}

#[test]
fn explicit_regularization_records_original_and_effective_problems() -> Result<(), Box<dyn Error>> {
    let system =
        DenseEqualitySystem::try_from_row_major(2, vec![1.0, 1.0, 1.0, 1.0], vec![2.0, 2.0])?;
    let solution = system.try_solve(options(
        DenseFactorization::Cholesky,
        Regularization::Explicit(1.0),
        ConditionPolicy::default(),
    )?)?;
    assert_close(
        solution.values(),
        &[2.0 / 3.0, 2.0 / 3.0],
        16.0 * f64::EPSILON,
    );
    let diagnostics = solution.diagnostics();
    assert_eq!(
        diagnostics.original_rank.decision,
        DenseRankDecision::Deficient
    );
    assert_eq!(
        diagnostics.effective_rank.decision,
        DenseRankDecision::FullRank
    );
    assert_eq!(
        diagnostics.applied_regularization.to_bits(),
        1.0_f64.to_bits()
    );
    assert_eq!(
        diagnostics.requested_regularization,
        Regularization::Explicit(1.0)
    );
    assert!(
        diagnostics.unregularized_final_residual.original_infinity
            > diagnostics.final_residual.original_infinity
    );
    Ok(())
}

#[test]
fn malformed_nonfinite_and_asymmetric_inputs_are_rejected() {
    assert!(DenseEqualitySystem::try_from_row_major(0, Vec::new(), Vec::new()).is_err());
    assert!(DenseEqualitySystem::try_from_row_major(2, vec![1.0; 3], vec![1.0; 2]).is_err());
    assert!(
        DenseEqualitySystem::try_from_row_major(2, vec![1.0, 1.0, 0.0, 1.0], vec![1.0, 1.0])
            .is_err()
    );
    assert!(DenseEqualitySystem::try_from_row_major(1, vec![f64::NAN], vec![1.0]).is_err());
}
