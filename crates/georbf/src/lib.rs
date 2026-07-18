//! The `GeoRBF` scalar-field core.
//!
//! The initial public API provides validated, dimension-safe geometry and
//! coordinate transforms, radial kernel derivative calculus, kernel metadata,
//! polyharmonic/surface-spline kernels, smooth global-support kernels, and
//! Wendland compact-support kernels for exactly one, two, or three dimensions,
//! validated geological orientation conversions in two and three dimensions,
//! fixed global anisotropy metrics with chain-rule derivatives, complete
//! dimension-generic polynomial spaces, atomic scalar-field functionals, and
//! scale-aware CPD polynomial rank enforcement with null-space projection,
//! plus provenance-preserving semantic and solver-neutral canonical problem
//! intermediate representations and symmetric dense hard-equality field
//! assembly, plus checked dense equality solving with explicit numerical
//! policy and original-unit diagnostics, and immutable fitted fields with
//! capability-gated original-coordinate value, gradient, and Hessian
//! evaluation, plus stable source-aware structured error codes for adapter and
//! orchestration boundaries, and caller-owned cancellation, deterministic
//! progress, and explicit serial-execution controls for assembly and solving,
//! plus provenance-preserving fixed, unknown, and prior level variables with
//! hard memberships, order-DAG validation, gauge and contrast checks, and
//! solver-neutral equality/linear-bound compilation.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

pub mod anisotropy;
pub mod coordinates;
pub mod cpd;
pub mod diagnostics;
pub mod dimension;
pub mod execution;
pub mod field;
pub mod functional;
pub mod geometry;
pub mod kernel;
pub mod kernel_calculus;
pub mod levels;
pub mod model;
pub mod orientation;
pub mod polynomial;
pub mod problem_ir;
pub mod solver;
pub mod transform;
pub mod units;

pub use anisotropy::{
    AnisotropyConditionPolicy, AnisotropyDiagnostics, AnisotropyError, GlobalAnisotropy,
};
pub use coordinates::{
    AxisOrder, CoordinateMetadata, CoordinateMetadataError, CoordinateMetadataField, CrsMetadata,
    Handedness, VerticalDirection,
};
pub use cpd::{
    CpdError, CpdIncompleteRankDiagnostics, CpdMatrix, CpdMatrixNorms, CpdNullSpace,
    CpdNullSpaceQuality, CpdRankDecision, CpdRankDiagnostics, CpdStorage, CpdWeightOrigin,
    CpdWeights,
};
pub use diagnostics::{
    CancellationDiagnostic, CapabilityDiagnostic, ConditioningDiagnostic, ContrastDiagnostic,
    DiagnosticPath, DiagnosticPathError, DiagnosticPathField, DiagnosticTextField,
    DiagnosticValueError, ErrorCategory, ErrorCode, GaugeDiagnostic, GeoRbfError,
    InfeasibilityDiagnostic, InputDiagnostic, LevelId, MemoryDiagnostic, RankDiagnostic,
    VersionDiagnostic,
};
pub use dimension::{Dim, SupportedDimension};
pub use execution::{
    CancellationToken, ExecutionControl, ExecutionError, ExecutionOperation, ExecutionStage,
    ProgressEvent, ProgressSink,
};
pub use field::{
    CpdFieldAssembly, DenseFieldMatrix, DenseFieldSystem, FieldAssemblyDiagnostics,
    FieldAssemblyError, FieldAssemblyStorage, FieldLinearizationError, FieldProblem,
    FieldProblemError,
};
pub use functional::{
    CenterRepresenter, FunctionalAtom, FunctionalError, FunctionalExpr, FunctionalProvenance,
    FunctionalStorage, FunctionalTerm, KernelActionError, ObservationFunctional, ScalarFieldSample,
};
pub use geometry::{Direction, GeometryError, Point, UnitDirection, Vector};
pub use kernel::{
    CpdOrder, CpdOrderError, Gaussian, InverseMultiquadric, KernelDefiniteness,
    KernelDerivativeCapabilities, KernelDerivativeCapabilitiesError, KernelDerivativeCapability,
    KernelDerivativeOrder, KernelDimensions, KernelDimensionsError, KernelMetadata,
    KernelMetadataError, KernelParameterConstraint, KernelParameterDefinition,
    KernelParameterDefinitionError, KernelParameterUnit, KernelParameterValueError, KernelSupport,
    Matern, MaternSmoothness, Multiquadric, PolyharmonicSpline,
    PolyharmonicSplineConstructionError, PolyharmonicSplineEvaluationError,
    SmoothKernelConstructionError, SmoothKernelEvaluationError, SmoothKernelFamily, SurfaceSpline,
    SurfaceSplineConstructionError, Wendland, WendlandConstructionError, WendlandEvaluationError,
    WendlandSmoothness,
};
pub use kernel_calculus::{
    KernelArgument, KernelCalculusError, RadialDerivativeOrder, RadialExpansionCoefficient,
    RadialExpansionCoefficients, RadialJet, RadialJetLocation, RadialSeparation, SpatialKernelJet,
    SpatialKernelJetPrefix,
};
pub use levels::{
    CanonicalLevelPrior, CompiledLevelProblem, LevelCanonicalizationError, LevelDefinition,
    LevelMembership, LevelOrder, LevelPrior, LevelProblem, LevelProblemDiagnostics,
    LevelProblemError, LevelStorage, LevelValue,
};
pub use model::{
    FittedField, FittedFieldCapabilities, FittedFieldComponent, FittedFieldDiagnostics,
    FittedFieldEvaluation, FittedFieldEvaluationError, FittedFieldFitError, FittedFieldOutput,
    FittedFieldRecord, FittedFieldSecondOrderEvaluation, FittedFieldStorage, KernelDefinition,
    KernelDefinitionEvaluationError,
};
pub use orientation::{
    LinearOrientation, OrientationAngleField, OrientationError, OrientationPolarity,
    PlanarOrientation, SupportedOrientationDimension,
};
pub use polynomial::{MultiIndex, PolynomialOutput, PolynomialSpace, PolynomialSpaceError};
pub use problem_ir::{
    AffineExpression, AffineTerm, CanonicalCapabilities, CanonicalEquality, CanonicalLinearBound,
    CanonicalMemoryEstimate, CanonicalProblem, CanonicalScaling, CanonicalSecondOrderCone,
    CanonicalSoftCapabilities, CanonicalSoftObjective, CanonicalSoftRelation,
    CanonicalizationError, Enforcement, ExecutionOptions, ObservationId, ProblemIrError,
    ProblemIrStorage, SemanticConstraint, SemanticExpression, SemanticMetadataField,
    SemanticProblemIr, SemanticProvenance, SemanticRelation, SoftLoss, SourceLocation,
    VariableBlock,
};
pub use solver::{
    ConditionPolicy, DenseEqualitySystem, DenseEqualitySystemError, DenseFactorization,
    DenseIncompleteRankDiagnostics, DenseMatrixNorms, DenseRankDecision, DenseRankDiagnostics,
    DenseResidualDiagnostics, DenseSolution, DenseSolveDiagnostics, DenseSolveError,
    DenseSolveOptions, DenseSolverConfigurationError, Regularization, try_solve_field,
    try_solve_field_with_control,
};
pub use transform::{AffineNormalization, TransformError, TransformOperation};
pub use units::{AngleUnit, LengthUnit, UnitError};
