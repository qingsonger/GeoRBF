//! Independent polynomial truth and error-path tests for three-dimensional isosurfaces.

use std::error::Error;
use std::num::{NonZeroU32, NonZeroUsize};

use georbf::{
    AffineNormalization, AngleUnit, AxisOrder, CancellationToken, CenterRepresenter,
    ConditionPolicy, CoordinateMetadata, CrsMetadata, DenseFactorization, DenseSolveOptions,
    Enforcement, ExecutionControl, ExecutionError, ExecutionOptions, FieldProblem, FittedField,
    FunctionalAtom, FunctionalExpr, FunctionalProvenance, FunctionalTerm, Handedness,
    IsosurfaceError, IsosurfaceMethod, IsosurfaceRequest, IsosurfaceRequestError,
    IsosurfaceSettings, IsosurfaceSettingsError, IsosurfaceTolerance, KernelDefinition, LengthUnit,
    ObservationFunctional, ObservationId, Point, PolyharmonicSpline, Regularization,
    SemanticConstraint, SemanticExpression, SemanticProblemIr, SemanticProvenance,
    SemanticRelation, SourceLocation, VerticalDirection,
};

const TEST_MEMORY_LIMIT_BYTES: usize = 128 * 1024 * 1024;

fn metadata() -> Result<CoordinateMetadata<3>, Box<dyn Error>> {
    Ok(CoordinateMetadata::new(
        LengthUnit::try_new("m")?,
        CrsMetadata::unspecified(),
        AxisOrder::identity(),
        VerticalDirection::Up,
        Handedness::Right,
        AngleUnit::Radians,
    ))
}

fn value_expression(point: [f64; 3], identifier: u64) -> Result<FunctionalExpr<3>, Box<dyn Error>> {
    Ok(FunctionalExpr::try_new([FunctionalTerm::try_new(
        1.0,
        FunctionalAtom::value(
            Point::try_new(point)?,
            FunctionalProvenance::new(identifier),
        ),
    )?])?)
}

fn polynomial_model(
    polynomial: impl Fn(f64, f64, f64) -> f64,
    normalization: AffineNormalization<3>,
) -> Result<FittedField<3>, Box<dyn Error>> {
    let mut constraints = Vec::new();
    let mut centers = Vec::new();
    let coordinates = [-2.0, 0.0, 2.0];
    let mut index = 0_usize;
    for z in coordinates {
        for y in coordinates {
            for x in coordinates {
                index += 1;
                let identifier = u64::try_from(index)?;
                let site = [x, y, z];
                let expression = value_expression(site, identifier)?;
                centers.push(CenterRepresenter::new(expression.clone()));
                constraints.push(SemanticConstraint::try_new(
                    SemanticProvenance::try_new(
                        ObservationId::new(identifier),
                        SourceLocation::try_new(
                            "isosurface-test.csv".to_owned(),
                            NonZeroUsize::new(index).ok_or("line")?,
                        )?,
                        "m".to_owned(),
                        format!("field.equalities[{}]", index - 1),
                        Some("isosurface polynomial truth".to_owned()),
                    )?,
                    SemanticRelation::Equality {
                        expression: SemanticExpression::try_new(
                            ObservationFunctional::new(expression),
                            0.0,
                        )?,
                        target: polynomial(x, y, z),
                    },
                    Enforcement::Hard,
                )?);
            }
        }
    }
    let problem = FieldProblem::try_new(
        SemanticProblemIr::try_new(constraints, ExecutionOptions::default())?,
        centers,
    )?;
    let options = DenseSolveOptions::try_new(
        DenseFactorization::PivotedLblt,
        Regularization::None,
        ConditionPolicy::default(),
        4,
        NonZeroUsize::new(TEST_MEMORY_LIMIT_BYTES).ok_or("memory limit")?,
    )?;
    Ok(FittedField::try_fit(
        problem,
        metadata()?,
        normalization,
        KernelDefinition::from(PolyharmonicSpline::try_new(5)?),
        None,
        options,
    )?)
}

fn identity_normalization() -> Result<AffineNormalization<3>, Box<dyn Error>> {
    Ok(AffineNormalization::try_new(
        Point::try_new([0.0, 0.0, 0.0])?,
        [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
    )?)
}

fn settings(cells: u32) -> Result<IsosurfaceSettings, Box<dyn Error>> {
    Ok(IsosurfaceSettings::try_new(
        NonZeroU32::new(cells).ok_or("x cells")?,
        NonZeroU32::new(cells).ok_or("y cells")?,
        NonZeroU32::new(cells).ok_or("z cells")?,
        NonZeroU32::new(64).ok_or("refinement iterations")?,
        1.0e-10,
        1.0e-10,
    )?)
}

fn request(
    method: IsosurfaceMethod,
    lower: [f64; 3],
    upper: [f64; 3],
    settings: IsosurfaceSettings,
) -> Result<IsosurfaceRequest, Box<dyn Error>> {
    Ok(IsosurfaceRequest::try_new(
        0.0,
        Point::try_new(lower)?,
        Point::try_new(upper)?,
        method,
        settings,
    )?)
}

fn cross_alignment(report: &georbf::IsosurfaceReport, indices: [usize; 3]) -> f64 {
    let vertices = report.vertices();
    let a = vertices[indices[0]].point().into_components();
    let b = vertices[indices[1]].point().into_components();
    let c = vertices[indices[2]].point().into_components();
    let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let ac = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let cross = [
        ab[1] * ac[2] - ab[2] * ac[1],
        ab[2] * ac[0] - ab[0] * ac[2],
        ab[0] * ac[1] - ab[1] * ac[0],
    ];
    let normal = vertices[indices[0]].normal().into_components();
    cross[0] * normal[0] + cross[1] * normal[1] + cross[2] * normal[2]
}

#[test]
fn regular_grid_plane_is_one_oriented_open_component() -> Result<(), Box<dyn Error>> {
    let model = polynomial_model(
        |x, _, _| x - 0.17,
        AffineNormalization::try_new(
            Point::try_new([3.0, -4.0, 2.0])?,
            [[2.0, 0.0, 0.0], [0.0, 0.5, 0.0], [0.0, 0.0, 1.5]],
        )?,
    )?;
    let report = model.try_isosurface(&request(
        IsosurfaceMethod::TopologyAwareMarchingCubes,
        [2.0, -5.0, 1.0],
        [4.0, -3.0, 3.0],
        settings(4)?,
    )?)?;
    let repeated = model.try_isosurface(&report.request())?;
    assert_eq!(report, repeated);
    assert_eq!(report.components().len(), 1);
    assert!(!report.components()[0].is_closed());
    assert_eq!(report.diagnostics().open_components(), 1);
    assert_eq!(report.diagnostics().closed_components(), 0);
    assert!(report.diagnostics().boundary_edges() > 0);
    for vertex in report.vertices() {
        assert!((vertex.point().components()[0] - 3.34).abs() <= 2.0e-9);
        assert!(vertex.residual().abs() <= 2.0e-9);
        assert!(vertex.normal().components()[0] > 0.999_999_999);
    }
    for triangle in report.triangles() {
        assert!(cross_alignment(&report, triangle.vertex_indices()) > 0.0);
    }
    Ok(())
}

#[test]
fn sphere_is_one_closed_regular_grid_component() -> Result<(), Box<dyn Error>> {
    let radius = 0.73_f64;
    let model = polynomial_model(
        |x, y, z| x * x + y * y + z * z - radius * radius,
        identity_normalization()?,
    )?;
    let report = model.try_isosurface(&request(
        IsosurfaceMethod::TopologyAwareMarchingCubes,
        [-1.0, -1.0, -1.0],
        [1.0, 1.0, 1.0],
        settings(6)?,
    )?)?;
    assert_eq!(report.components().len(), 1);
    assert!(report.components()[0].is_closed());
    assert_eq!(report.diagnostics().boundary_edges(), 0);
    assert_eq!(report.diagnostics().closed_components(), 1);
    for vertex in report.vertices() {
        let point = vertex.point().into_components();
        let norm = (point[0] * point[0] + point[1] * point[1] + point[2] * point[2]).sqrt();
        assert!((norm - radius).abs() <= 2.0e-8);
        let normal = vertex.normal().into_components();
        assert!(normal[0] * point[0] + normal[1] * point[1] + normal[2] * point[2] > 0.0);
    }
    Ok(())
}

#[test]
fn marching_simplices_plane_is_conforming_and_oriented() -> Result<(), Box<dyn Error>> {
    let model = polynomial_model(|x, _, _| x - 0.19, identity_normalization()?)?;
    let report = model.try_isosurface(&request(
        IsosurfaceMethod::MarchingSimplices,
        [-1.0, -1.0, -1.0],
        [1.0, 1.0, 1.0],
        settings(3)?,
    )?)?;
    assert_eq!(report.components().len(), 1);
    assert!(!report.components()[0].is_closed());
    assert!(report.diagnostics().unique_triangles() > 0);
    for triangle in report.triangles() {
        assert!(cross_alignment(&report, triangle.vertex_indices()) > 0.0);
    }
    Ok(())
}

#[test]
fn saddle_records_shared_face_ambiguity() -> Result<(), Box<dyn Error>> {
    let model = polynomial_model(|x, y, z| x * y - z, identity_normalization()?)?;
    let report = model.try_isosurface(&request(
        IsosurfaceMethod::TopologyAwareMarchingCubes,
        [-1.0, -1.0, -0.4],
        [1.0, 1.0, 0.4],
        settings(1)?,
    )?)?;
    assert!(!report.triangles().is_empty());
    assert!(!report.diagnostics().ambiguous_faces().is_empty());
    for decision in report.diagnostics().ambiguous_faces() {
        assert!(decision.normalized_decider().is_finite());
    }
    Ok(())
}

#[test]
fn multiple_cube_boundary_loops_are_conservatively_underdetermined() -> Result<(), Box<dyn Error>> {
    let model = polynomial_model(
        |x, y, z| x * y + y * z + z * x - 1.0,
        identity_normalization()?,
    )?;
    let result = model.try_isosurface(&request(
        IsosurfaceMethod::TopologyAwareMarchingCubes,
        [-1.0, -1.0, -1.0],
        [1.0, 1.0, 1.0],
        settings(1)?,
    )?);
    assert!(matches!(
        result,
        Err(IsosurfaceError::DegenerateCellTopology { evidence: 2, .. })
    ));
    Ok(())
}

#[test]
fn exact_target_grid_edge_is_reported_as_underdetermined() -> Result<(), Box<dyn Error>> {
    let model = polynomial_model(|_, _, _| 0.0, identity_normalization()?)?;
    let result = model.try_isosurface(&request(
        IsosurfaceMethod::TopologyAwareMarchingCubes,
        [-1.0, -1.0, -1.0],
        [1.0, 1.0, 1.0],
        settings(2)?,
    )?);
    assert!(matches!(
        result,
        Err(IsosurfaceError::DegenerateGridEdge { .. })
    ));
    Ok(())
}

#[test]
fn validates_settings_domains_and_work_budget() -> Result<(), Box<dyn Error>> {
    let one = NonZeroU32::MIN;
    assert!(matches!(
        IsosurfaceSettings::try_new(one, one, one, one, 0.0, 1.0),
        Err(IsosurfaceSettingsError::InvalidTolerance {
            tolerance: IsosurfaceTolerance::Value,
            ..
        })
    ));
    let valid = settings(1)?;
    assert!(matches!(
        IsosurfaceRequest::try_new(
            f64::NAN,
            Point::try_new([0.0, 0.0, 0.0])?,
            Point::try_new([1.0, 1.0, 1.0])?,
            IsosurfaceMethod::MarchingSimplices,
            valid,
        ),
        Err(IsosurfaceRequestError::NonFiniteLevel { .. })
    ));
    assert!(matches!(
        IsosurfaceRequest::try_new(
            0.0,
            Point::try_new([0.0, 0.0, 0.0])?,
            Point::try_new([0.0, 1.0, 1.0])?,
            IsosurfaceMethod::MarchingSimplices,
            valid,
        ),
        Err(IsosurfaceRequestError::InvalidDomain { .. })
    ));

    let huge = IsosurfaceSettings::try_new(
        NonZeroU32::new(u32::MAX).ok_or("x")?,
        NonZeroU32::new(u32::MAX).ok_or("y")?,
        NonZeroU32::new(u32::MAX).ok_or("z")?,
        one,
        1.0e-6,
        1.0e-6,
    )?;
    let model = polynomial_model(|x, _, _| x - 0.2, identity_normalization()?)?;
    let request = request(
        IsosurfaceMethod::MarchingSimplices,
        [-1.0, -1.0, -1.0],
        [1.0, 1.0, 1.0],
        huge,
    )?;
    assert!(matches!(
        model.try_isosurface(&request),
        Err(IsosurfaceError::WorkBudgetOverflow { .. })
    ));
    Ok(())
}

#[test]
fn refinement_exhaustion_and_execution_controls_return_no_mesh() -> Result<(), Box<dyn Error>> {
    let model = polynomial_model(|x, _, _| x * x - 0.2, identity_normalization()?)?;
    let limited = IsosurfaceSettings::try_new(
        NonZeroU32::MIN,
        NonZeroU32::MIN,
        NonZeroU32::MIN,
        NonZeroU32::MIN,
        1.0e-15,
        1.0e-15,
    )?;
    let request = request(
        IsosurfaceMethod::TopologyAwareMarchingCubes,
        [0.0, -1.0, -1.0],
        [1.0, 1.0, 1.0],
        limited,
    )?;
    assert!(matches!(
        model.try_isosurface(&request),
        Err(IsosurfaceError::RefinementLimitReached { .. })
    ));

    let token = CancellationToken::new();
    token.cancel();
    assert!(matches!(
        model.try_isosurface_with_control(
            &request,
            ExecutionOptions::default(),
            ExecutionControl::new(Some(&token), None),
        ),
        Err(IsosurfaceError::Execution(ExecutionError::Cancelled { .. }))
    ));
    let two_threads = ExecutionOptions::new(true, NonZeroUsize::new(2), None);
    assert!(matches!(
        model.try_isosurface_with_control(&request, two_threads, ExecutionControl::default()),
        Err(IsosurfaceError::Execution(
            ExecutionError::UnsupportedThreadCount { .. }
        ))
    ));
    Ok(())
}
