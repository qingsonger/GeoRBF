//! Independent truth and mathematical-property tests for CPD rank enforcement.

use std::error::Error;

use georbf::{
    CenterRepresenter, CpdError, CpdMatrix, CpdNullSpace, CpdRankDecision, CpdWeightOrigin, Dim,
    FunctionalAtom, FunctionalExpr, FunctionalProvenance, FunctionalTerm, Point, PolynomialSpace,
    SupportedDimension, UnitDirection,
};

fn value_center<const D: usize>(
    coordinates: [f64; D],
    coefficient: f64,
    provenance: u64,
) -> Result<CenterRepresenter<D>, Box<dyn Error>>
where
    Dim<D>: SupportedDimension,
{
    let atom = FunctionalAtom::value(
        Point::try_new(coordinates)?,
        FunctionalProvenance::new(provenance),
    );
    Ok(CenterRepresenter::new(FunctionalExpr::try_new([
        FunctionalTerm::try_new(coefficient, atom)?,
    ])?))
}

fn derivative_center<const D: usize>(
    coordinates: [f64; D],
    direction: [f64; D],
    provenance: u64,
) -> Result<CenterRepresenter<D>, Box<dyn Error>>
where
    Dim<D>: SupportedDimension,
{
    let atom = FunctionalAtom::directional_derivative(
        Point::try_new(coordinates)?,
        UnitDirection::try_new(direction)?,
        FunctionalProvenance::new(provenance),
    );
    Ok(CenterRepresenter::new(FunctionalExpr::try_new([
        FunctionalTerm::try_new(1.0, atom)?,
    ])?))
}

fn action_row_center(
    action: [f64; 3],
    provenance: u64,
) -> Result<CenterRepresenter<2>, Box<dyn Error>> {
    let point = Point::try_new([0.0, 0.0])?;
    let atoms = [
        FunctionalAtom::value(point, FunctionalProvenance::new(provenance)),
        FunctionalAtom::directional_derivative(
            point,
            UnitDirection::try_new([1.0, 0.0])?,
            FunctionalProvenance::new(provenance + 1),
        ),
        FunctionalAtom::directional_derivative(
            point,
            UnitDirection::try_new([0.0, 1.0])?,
            FunctionalProvenance::new(provenance + 2),
        ),
    ];
    let terms = action
        .into_iter()
        .zip(atoms)
        .filter(|(coefficient, _)| *coefficient != 0.0)
        .map(|(coefficient, atom)| FunctionalTerm::try_new(coefficient, atom))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(CenterRepresenter::new(FunctionalExpr::try_new(terms)?))
}

fn assert_polynomial_reproduction(system: &CpdNullSpace, coefficients: &[f64]) {
    assert_eq!(system.actions().columns(), coefficients.len());
    let samples = (0..system.actions().rows())
        .map(|row| {
            coefficients
                .iter()
                .enumerate()
                .map(|(column, coefficient)| {
                    system.actions().get(row, column).unwrap_or(f64::NAN) * coefficient
                })
                .sum::<f64>()
        })
        .collect::<Vec<_>>();
    for column in 0..system.basis().columns() {
        let projected = samples
            .iter()
            .enumerate()
            .map(|(row, sample)| {
                let basis_value = system.basis().get(row, column).unwrap_or(f64::NAN);
                basis_value * sample
            })
            .sum::<f64>();
        assert!(projected.abs() < 1.0e-11);
    }
}

#[test]
fn assembles_complete_polynomial_actions_in_all_dimensions() -> Result<(), Box<dyn Error>> {
    let one = [
        value_center([-1.0], 1.0, 10)?,
        value_center([0.0], 1.0, 11)?,
        value_center([2.0], 1.0, 12)?,
    ];
    let one_system = CpdNullSpace::try_from_centers(&one, &PolynomialSpace::<1>::try_new(2)?)?;
    assert_eq!(
        one_system.actions().values(),
        &[1.0, -1.0, 1.0, 0.0, 1.0, 2.0]
    );
    assert_polynomial_reproduction(&one_system, &[2.0, -0.5]);

    let two = [
        value_center([0.0, 0.0], 1.0, 20)?,
        value_center([1.0, 0.0], 1.0, 21)?,
        value_center([0.0, 1.0], 1.0, 22)?,
        value_center([1.0, 1.0], 1.0, 23)?,
    ];
    let two_system = CpdNullSpace::try_from_centers(&two, &PolynomialSpace::<2>::try_new(2)?)?;
    assert_eq!(two_system.actions().row(3), Some(&[1.0, 1.0, 1.0][..]));
    assert_polynomial_reproduction(&two_system, &[2.0, -0.5, 3.0]);

    let three = [
        value_center([0.0, 0.0, 0.0], 1.0, 30)?,
        value_center([1.0, 0.0, 0.0], 1.0, 31)?,
        value_center([0.0, 1.0, 0.0], 1.0, 32)?,
        value_center([0.0, 0.0, 1.0], 1.0, 33)?,
        value_center([1.0, 1.0, 1.0], 1.0, 34)?,
    ];
    let three_system = CpdNullSpace::try_from_centers(&three, &PolynomialSpace::<3>::try_new(2)?)?;
    assert_eq!(
        three_system.actions().row(4),
        Some(&[1.0, 1.0, 1.0, 1.0][..])
    );
    assert_polynomial_reproduction(&three_system, &[2.0, -0.5, 3.0, 1.25]);
    assert_eq!(
        three_system
            .center_provenance(4)
            .map(|terms| terms[0].identifier()),
        Some(34)
    );
    Ok(())
}

#[test]
fn directional_centers_act_on_polynomials_without_special_cases() -> Result<(), Box<dyn Error>> {
    let centers = [
        value_center([0.0, 0.0], 1.0, 1)?,
        derivative_center([4.0, 5.0], [1.0, 0.0], 2)?,
        derivative_center([4.0, 5.0], [0.0, 1.0], 3)?,
        value_center([1.0, 1.0], 1.0, 4)?,
    ];
    let system = CpdNullSpace::try_from_centers(&centers, &PolynomialSpace::<2>::try_new(2)?)?;
    assert_eq!(system.actions().row(1), Some(&[0.0, 1.0, 0.0][..]));
    assert_eq!(system.actions().row(2), Some(&[0.0, 0.0, 1.0][..]));
    assert_eq!(system.diagnostics().decision, CpdRankDecision::FullRank);

    let zero_row = [
        value_center([0.0, 0.0], 1.0, 10)?,
        derivative_center([0.0, 0.0], [1.0, 0.0], 11)?,
    ];
    let zero_row_system =
        CpdNullSpace::try_from_centers(&zero_row, &PolynomialSpace::<2>::try_new(1)?)?;
    assert_eq!(zero_row_system.diagnostics().zero_rows, [1]);
    Ok(())
}

#[test]
fn null_space_is_orthonormal_and_expanded_weights_retain_provenance() -> Result<(), Box<dyn Error>>
{
    let centers = [
        value_center([-2.0], 1.0, 1)?,
        value_center([-1.0], 1.0, 2)?,
        value_center([0.0], 1.0, 3)?,
        value_center([1.0], 1.0, 4)?,
        value_center([2.0], 1.0, 5)?,
    ];
    let system = CpdNullSpace::try_from_centers(&centers, &PolynomialSpace::<1>::try_new(2)?)?;
    assert_eq!((system.basis().rows(), system.basis().columns()), (5, 3));
    assert!(system.quality().side_condition_residual <= system.quality().tolerance);
    assert!(system.quality().original_side_condition_residual < 1.0e-12);
    assert!(system.quality().orthonormality_residual <= system.quality().tolerance);

    let weights = system.try_expand_weights(&[1.0, -2.0, 0.5])?;
    assert_eq!(weights.origin(), CpdWeightOrigin::PolynomialNullSpace);
    assert!(weights.side_condition_residual() <= weights.tolerance());
    assert!(weights.original_side_condition_residual() < 1.0e-12);
    let constant = weights.values().iter().sum::<f64>();
    let linear = weights
        .values()
        .iter()
        .zip([-2.0, -1.0, 0.0, 1.0, 2.0])
        .map(|(weight, x)| weight * x)
        .sum::<f64>();
    assert!(constant.abs() < 1.0e-12);
    assert!(linear.abs() < 1.0e-12);
    Ok(())
}

#[test]
fn exact_degeneracy_is_reported_in_d1_d2_and_d3() -> Result<(), Box<dyn Error>> {
    let one = [
        value_center([1.0], 1.0, 1)?,
        value_center([1.0], 1.0, 2)?,
        value_center([1.0], 1.0, 3)?,
    ];
    assert_deficient(CpdNullSpace::try_from_centers(
        &one,
        &PolynomialSpace::<1>::try_new(2)?,
    ))?;

    let two = [
        value_center([0.0, 0.0], 1.0, 1)?,
        value_center([1.0, 2.0], 1.0, 2)?,
        value_center([2.0, 4.0], 1.0, 3)?,
        value_center([3.0, 6.0], 1.0, 4)?,
    ];
    assert_deficient(CpdNullSpace::try_from_centers(
        &two,
        &PolynomialSpace::<2>::try_new(2)?,
    ))?;

    let three = [
        value_center([0.0, 0.0, 0.0], 1.0, 1)?,
        value_center([1.0, 0.0, 0.0], 1.0, 2)?,
        value_center([0.0, 1.0, 0.0], 1.0, 3)?,
        value_center([1.0, 1.0, 0.0], 1.0, 4)?,
        value_center([2.0, -1.0, 0.0], 1.0, 5)?,
    ];
    assert_deficient(CpdNullSpace::try_from_centers(
        &three,
        &PolynomialSpace::<3>::try_new(2)?,
    ))?;
    Ok(())
}

fn assert_deficient(result: Result<CpdNullSpace, CpdError>) -> Result<(), Box<dyn Error>> {
    match result {
        Err(CpdError::RankDeficient { diagnostics }) => {
            assert_eq!(diagnostics.decision, CpdRankDecision::Deficient);
            assert!(diagnostics.svd_rank < diagnostics.columns);
            Ok(())
        }
        other => Err(std::io::Error::other(format!(
            "expected clear rank deficiency, got {other:?}"
        ))
        .into()),
    }
}

#[test]
fn coordinate_units_and_nonzero_functional_scaling_preserve_rank() -> Result<(), Box<dyn Error>> {
    let baseline = [
        value_center([0.0, 0.0], 1.0, 1)?,
        value_center([1.0, 0.0], 1.0, 2)?,
        value_center([0.0, 1.0], 1.0, 3)?,
        value_center([1.0, 1.0], 1.0, 4)?,
    ];
    let rescaled = [
        value_center([0.0, 0.0], 2.0_f64.powi(-20), 1)?,
        value_center([1.0e6, 0.0], -2.0_f64.powi(18), 2)?,
        value_center([0.0, 1.0e6], 2.0_f64.powi(10), 3)?,
        value_center([1.0e6, 1.0e6], -2.0_f64.powi(-12), 4)?,
    ];
    let space = PolynomialSpace::<2>::try_new(2)?;
    let first = CpdNullSpace::try_from_centers(&baseline, &space)?;
    let second = CpdNullSpace::try_from_centers(&rescaled, &space)?;
    assert_eq!(first.diagnostics().decision, second.diagnostics().decision);
    assert_eq!(first.diagnostics().svd_rank, second.diagnostics().svd_rank);
    assert_eq!(second.diagnostics().equilibration_passes, 8);
    assert!(
        second
            .diagnostics()
            .row_scales
            .iter()
            .all(|scale| scale.is_finite() && *scale > 0.0)
    );
    Ok(())
}

#[test]
fn threshold_adjacent_analytic_determinant_is_rejected_as_ambiguous() -> Result<(), Box<dyn Error>>
{
    // These already-equilibrated action matrices have one singular value 1.
    // The other two satisfy s_max*s_min=|1-2a| and
    // s_max^2+s_min^2=4+2a^2. The binary coefficients make the determinant an
    // exact integer multiple of epsilon, so this closed-form truth is
    // independent of nalgebra's RRQR and SVD implementations.
    for (ulps, expected_rank) in [(12_u64, 2_usize), (15, 3)] {
        let coefficient = f64::from_bits(0.5_f64.to_bits() + ulps);
        let centers = [
            action_row_center([1.0, 0.0, 1.0], 10)?,
            action_row_center([0.0, 1.0, 1.0], 20)?,
            action_row_center([coefficient, coefficient, 1.0], 30)?,
        ];
        let (analytic_rank, analytic_smallest, analytic_threshold) =
            analytic_rank_truth(coefficient);
        assert_eq!(analytic_rank, expected_rank);
        assert!((analytic_smallest / analytic_threshold - 1.0).abs() < 0.15);

        let result = CpdNullSpace::try_from_centers(&centers, &PolynomialSpace::<2>::try_new(2)?);
        let diagnostics = match result {
            Err(CpdError::AmbiguousRank { diagnostics }) => diagnostics,
            other => {
                return Err(std::io::Error::other(format!(
                    "expected guarded ambiguous rank, got {other:?}"
                ))
                .into());
            }
        };
        assert!(diagnostics.threshold_adjacent || diagnostics.rank_disagreement);
        assert_eq!(diagnostics.decision, CpdRankDecision::Ambiguous);
        assert_eq!(diagnostics.svd_rank, expected_rank);
    }
    Ok(())
}

fn analytic_rank_truth(coefficient: f64) -> (usize, f64, f64) {
    let determinant = (1.0 - 2.0 * coefficient).abs();
    let frobenius_squared = 4.0 + 2.0 * coefficient * coefficient;
    let discriminant =
        (frobenius_squared * frobenius_squared - 4.0 * determinant * determinant).sqrt();
    let largest = frobenius_squared.midpoint(discriminant).sqrt();
    let smallest = determinant / largest;
    let threshold = 3.0 * f64::EPSILON * largest;
    (2 + usize::from(smallest > threshold), smallest, threshold)
}

#[test]
fn polynomial_reproduction_and_projected_kkt_paths_agree() -> Result<(), Box<dyn Error>> {
    let centers = [
        value_center([-1.0], 1.0, 1)?,
        value_center([0.0], 1.0, 2)?,
        value_center([1.0], 1.0, 3)?,
    ];
    let system = CpdNullSpace::try_from_centers(&centers, &PolynomialSpace::<1>::try_new(2)?)?;
    let z = system.basis().values();
    assert_eq!(z.len(), 3);

    // Any polynomial sample Q*a is orthogonal to Z, so the projected solve
    // has zero RBF weights and the polynomial coefficients reproduce it.
    let polynomial_samples = [2.0, 3.0, 4.0];
    let projected_polynomial = z
        .iter()
        .zip(polynomial_samples)
        .map(|(basis, sample)| basis * sample)
        .sum::<f64>();
    assert!(projected_polynomial.abs() < 1.0e-12);

    let identity =
        CpdMatrix::try_from_row_major(3, 3, vec![1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0])?;
    let projected = system.try_project_symmetric_energy(&identity)?;
    assert!((projected.values()[0] - 1.0).abs() < 1.0e-12);

    // For K=I and b=[1,-2,4], the projected solution equals the analytic KKT
    // projection b - Q(Q^T Q)^-1 Q^T b.
    let b = [1.0, -2.0, 4.0];
    let reduced_rhs = z.iter().zip(b).map(|(basis, rhs)| basis * rhs).sum::<f64>();
    let projected_weights = system.try_expand_weights(&[reduced_rhs / projected.values()[0]])?;
    let mean = b.iter().sum::<f64>() / 3.0;
    let linear = (-b[0]).midpoint(b[2]);
    let kkt_weights = [b[0] - (mean - linear), b[1] - mean, b[2] - (mean + linear)];
    for (projected_weight, kkt_weight) in projected_weights.values().iter().zip(kkt_weights) {
        assert!((projected_weight - kkt_weight).abs() < 1.0e-12);
    }
    Ok(())
}

#[test]
fn invalid_inputs_and_projection_fail_without_fallback() -> Result<(), Box<dyn Error>> {
    let space = PolynomialSpace::<1>::try_new(1)?;
    assert!(matches!(
        CpdNullSpace::try_from_centers(&[], &space),
        Err(CpdError::EmptyCenters)
    ));
    assert!(matches!(
        CpdMatrix::try_from_row_major(1, 2, vec![1.0]),
        Err(CpdError::MatrixLengthMismatch { .. })
    ));
    assert!(matches!(
        CpdMatrix::try_from_row_major(1, 1, vec![f64::NAN]),
        Err(CpdError::NonFiniteMatrixEntry { .. })
    ));

    let centers = [value_center([0.0], 1.0, 1)?, value_center([1.0], 1.0, 2)?];
    let system = CpdNullSpace::try_from_centers(&centers, &space)?;
    assert!(matches!(
        system.try_expand_weights(&[]),
        Err(CpdError::ReducedLengthMismatch { .. })
    ));
    assert!(matches!(
        system.try_expand_weights(&[f64::INFINITY]),
        Err(CpdError::NonFiniteReducedCoordinate { .. })
    ));
    let nonsymmetric = CpdMatrix::try_from_row_major(2, 2, vec![1.0, 1.0, 0.0, 1.0])?;
    assert!(matches!(
        system.try_project_symmetric_energy(&nonsymmetric),
        Err(CpdError::EnergyNotSymmetric { .. })
    ));
    Ok(())
}

#[test]
fn diagnostics_and_basis_are_deterministic_and_thread_safe() -> Result<(), Box<dyn Error>> {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<CpdNullSpace>();

    let centers = [
        value_center([-1.0], 1.0, 1)?,
        value_center([0.0], 1.0, 2)?,
        value_center([1.0], 1.0, 3)?,
        value_center([2.0], 1.0, 4)?,
    ];
    let space = PolynomialSpace::<1>::try_new(2)?;
    let first = CpdNullSpace::try_from_centers(&centers, &space)?;
    let second = CpdNullSpace::try_from_centers(&centers, &space)?;
    assert_eq!(first, second);
    Ok(())
}
