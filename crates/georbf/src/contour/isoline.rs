//! Deterministic two-dimensional isoline extraction.
//!
//! The extractor samples one immutable [`FittedField<2>`] on an explicit
//! original-coordinate rectangular grid. It offers a fixed-diagonal
//! marching-simplices reference path and a marching-squares path whose
//! alternating-sign cells use a bilinear asymptotic decider. Every retained
//! edge crossing is refined with fitted-field values, shared grid-edge
//! identities are deduplicated before topology reconstruction, and open
//! components retain explicit requested-domain boundary evidence.
//!
//! ```
//! use std::error::Error;
//! use std::num::NonZeroU32;
//!
//! use georbf::{
//!     FittedField, IsolineMethod, IsolineRequest, IsolineSettings, Point,
//! };
//!
//! # fn extract(model: &FittedField<2>) -> Result<(), Box<dyn Error>> {
//! let settings = IsolineSettings::try_new(
//!     NonZeroU32::new(64).ok_or("x cells")?,
//!     NonZeroU32::new(64).ok_or("y cells")?,
//!     NonZeroU32::new(64).ok_or("refinement iterations")?,
//!     1.0e-10,
//!     1.0e-9,
//! )?;
//! let request = IsolineRequest::try_new(
//!     0.0,
//!     Point::try_new([-10.0, -10.0])?,
//!     Point::try_new([10.0, 10.0])?,
//!     IsolineMethod::DisambiguatedMarchingSquares,
//!     settings,
//! )?;
//! let report = model.try_isolines(&request)?;
//! for polyline in report.polylines() {
//!     assert!(polyline.vertex_indices().len() >= 2);
//! }
//! # Ok(())
//! # }
//! ```

use std::error::Error;
use std::fmt;
use std::num::NonZeroU32;

use crate::execution::{
    ExecutionControl, ExecutionError, ExecutionOperation, ExecutionStage, ProgressTracker,
};
use crate::geometry::Point;
use crate::model::{FittedField, FittedFieldEvaluationError, FittedFieldOutput};
use crate::problem_ir::ExecutionOptions;

/// Grid-cell traversal used to extract a two-dimensional isoline.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum IsolineMethod {
    /// Split every cell from its lower-left to upper-right corner.
    MarchingSimplices,
    /// Use marching squares and disambiguate alternating-sign cells.
    DisambiguatedMarchingSquares,
}

/// Explicit grid and bracket-refinement policy for two-dimensional isolines.
#[derive(Clone, Copy, Debug, PartialEq)]
#[must_use]
pub struct IsolineSettings {
    cells_x: NonZeroU32,
    cells_y: NonZeroU32,
    refinement_iterations: NonZeroU32,
    value_tolerance: f64,
    coordinate_tolerance: f64,
}

impl IsolineSettings {
    /// Constructs a finite deterministic grid and refinement policy.
    ///
    /// # Errors
    ///
    /// Both tolerances must be positive and finite.
    pub fn try_new(
        cells_x: NonZeroU32,
        cells_y: NonZeroU32,
        refinement_iterations: NonZeroU32,
        value_tolerance: f64,
        coordinate_tolerance: f64,
    ) -> Result<Self, IsolineSettingsError> {
        validate_tolerance(IsolineTolerance::Value, value_tolerance)?;
        validate_tolerance(IsolineTolerance::Coordinate, coordinate_tolerance)?;
        Ok(Self {
            cells_x,
            cells_y,
            refinement_iterations,
            value_tolerance,
            coordinate_tolerance,
        })
    }

    /// Returns the requested cell count along original-coordinate X.
    #[must_use]
    pub const fn cells_x(self) -> NonZeroU32 {
        self.cells_x
    }

    /// Returns the requested cell count along original-coordinate Y.
    #[must_use]
    pub const fn cells_y(self) -> NonZeroU32 {
        self.cells_y
    }

    /// Returns the maximum bisection count for one crossed edge.
    #[must_use]
    pub const fn refinement_iterations(self) -> NonZeroU32 {
        self.refinement_iterations
    }

    /// Returns the accepted absolute fitted-value residual.
    #[must_use]
    pub const fn value_tolerance(self) -> f64 {
        self.value_tolerance
    }

    /// Returns the accepted maximum-coordinate bracket width.
    #[must_use]
    pub const fn coordinate_tolerance(self) -> f64 {
        self.coordinate_tolerance
    }
}

/// Tolerance field rejected while constructing isoline settings.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum IsolineTolerance {
    /// Fitted value minus requested level.
    Value,
    /// Original-coordinate edge-bracket width.
    Coordinate,
}

impl fmt::Display for IsolineTolerance {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Value => "value",
            Self::Coordinate => "coordinate",
        })
    }
}

/// Invalid two-dimensional isoline settings.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum IsolineSettingsError {
    /// A tolerance was zero, negative, infinite, or NaN.
    InvalidTolerance {
        /// Rejected tolerance field.
        tolerance: IsolineTolerance,
        /// Rejected value.
        value: f64,
    },
}

impl fmt::Display for IsolineSettingsError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidTolerance { tolerance, value } => write!(
                formatter,
                "isoline {tolerance} tolerance must be positive and finite, got {value}"
            ),
        }
    }
}

impl Error for IsolineSettingsError {}

/// Original-coordinate axis used by domain diagnostics.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum IsolineDomainAxis {
    /// First coordinate.
    X,
    /// Second coordinate.
    Y,
}

impl fmt::Display for IsolineDomainAxis {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::X => "x",
            Self::Y => "y",
        })
    }
}

/// One target-level extraction request over a rectangular original-coordinate domain.
#[derive(Clone, Copy, Debug, PartialEq)]
#[must_use]
pub struct IsolineRequest {
    level: f64,
    lower: Point<2>,
    upper: Point<2>,
    method: IsolineMethod,
    settings: IsolineSettings,
}

impl IsolineRequest {
    /// Constructs a validated two-dimensional isoline request.
    ///
    /// # Errors
    ///
    /// The target level must be finite. Each lower coordinate must be
    /// strictly less than its upper coordinate, and each finite span must
    /// remain representable.
    pub fn try_new(
        level: f64,
        lower: Point<2>,
        upper: Point<2>,
        method: IsolineMethod,
        settings: IsolineSettings,
    ) -> Result<Self, IsolineRequestError> {
        if !level.is_finite() {
            return Err(IsolineRequestError::NonFiniteLevel { level });
        }
        for (index, axis) in [IsolineDomainAxis::X, IsolineDomainAxis::Y]
            .into_iter()
            .enumerate()
        {
            let lower_value = lower.components()[index];
            let upper_value = upper.components()[index];
            if lower_value >= upper_value {
                return Err(IsolineRequestError::InvalidDomain {
                    axis,
                    lower: lower_value,
                    upper: upper_value,
                });
            }
            if !(upper_value - lower_value).is_finite() {
                return Err(IsolineRequestError::UnrepresentableDomainSpan {
                    axis,
                    lower: lower_value,
                    upper: upper_value,
                });
            }
        }
        Ok(Self {
            level,
            lower,
            upper,
            method,
            settings,
        })
    }

    /// Returns the requested fitted scalar value.
    #[must_use]
    pub const fn level(self) -> f64 {
        self.level
    }

    /// Returns the inclusive lower-left original-coordinate corner.
    pub const fn lower(self) -> Point<2> {
        self.lower
    }

    /// Returns the inclusive upper-right original-coordinate corner.
    pub const fn upper(self) -> Point<2> {
        self.upper
    }

    /// Returns the selected marching method.
    #[must_use]
    pub const fn method(self) -> IsolineMethod {
        self.method
    }

    /// Returns the explicit grid and refinement settings.
    pub const fn settings(self) -> IsolineSettings {
        self.settings
    }
}

/// Invalid target level or rectangular isoline domain.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum IsolineRequestError {
    /// The target level was infinite or NaN.
    NonFiniteLevel {
        /// Rejected level.
        level: f64,
    },
    /// One domain interval was empty or reversed.
    InvalidDomain {
        /// Rejected axis.
        axis: IsolineDomainAxis,
        /// Lower coordinate.
        lower: f64,
        /// Upper coordinate.
        upper: f64,
    },
    /// Subtracting two finite bounds did not produce a finite span.
    UnrepresentableDomainSpan {
        /// Rejected axis.
        axis: IsolineDomainAxis,
        /// Lower coordinate.
        lower: f64,
        /// Upper coordinate.
        upper: f64,
    },
}

impl fmt::Display for IsolineRequestError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonFiniteLevel { level } => {
                write!(
                    formatter,
                    "isoline target level must be finite, got {level}"
                )
            }
            Self::InvalidDomain { axis, lower, upper } => write!(
                formatter,
                "isoline {axis} domain must satisfy lower < upper, got [{lower}, {upper}]"
            ),
            Self::UnrepresentableDomainSpan { axis, lower, upper } => write!(
                formatter,
                "isoline {axis} domain span is not representable for [{lower}, {upper}]"
            ),
        }
    }
}

impl Error for IsolineRequestError {}

/// One deduplicated original-coordinate isoline vertex.
#[derive(Clone, Copy, Debug, PartialEq)]
#[must_use]
pub struct IsolineVertex {
    point: Point<2>,
    value: f64,
    residual: f64,
}

impl IsolineVertex {
    /// Returns the vertex in the fitted field's original coordinates.
    pub const fn point(self) -> Point<2> {
        self.point
    }

    /// Returns the analytic fitted-field value.
    #[must_use]
    pub const fn value(self) -> f64 {
        self.value
    }

    /// Returns `value - requested_level`.
    #[must_use]
    pub const fn residual(self) -> f64 {
        self.residual
    }
}

/// One connected open or closed isoline component.
#[derive(Clone, Debug, Eq, PartialEq)]
#[must_use]
pub struct IsolinePolyline {
    vertex_indices: Vec<usize>,
    closed: bool,
}

impl IsolinePolyline {
    /// Borrows indices into [`IsolineReport::vertices`].
    #[must_use]
    pub fn vertex_indices(&self) -> &[usize] {
        &self.vertex_indices
    }

    /// Returns whether the last segment reconnects the last vertex to the first.
    #[must_use]
    pub const fn is_closed(&self) -> bool {
        self.closed
    }
}

/// Side of the requested rectangle touched by an open endpoint.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum IsolineBoundarySide {
    /// Minimum X.
    Left,
    /// Maximum X.
    Right,
    /// Minimum Y.
    Bottom,
    /// Maximum Y.
    Top,
}

/// Boundary evidence for one open-polyline endpoint.
#[derive(Clone, Debug, Eq, PartialEq)]
#[must_use]
pub struct IsolineBoundaryEndpoint {
    vertex_index: usize,
    sides: Vec<IsolineBoundarySide>,
}

impl IsolineBoundaryEndpoint {
    /// Returns the endpoint's index into [`IsolineReport::vertices`].
    #[must_use]
    pub const fn vertex_index(&self) -> usize {
        self.vertex_index
    }

    /// Borrows every touched side; a requested-domain corner touches two.
    #[must_use]
    pub fn sides(&self) -> &[IsolineBoundarySide] {
        &self.sides
    }
}

/// Alternating-sign marching-squares segment pairing.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum IsolineCellPairing {
    /// Connect bottom-to-right and top-to-left.
    BottomRightAndTopLeft,
    /// Connect bottom-to-left and right-to-top.
    BottomLeftAndRightTop,
}

/// Evidence used to choose an alternating-sign cell pairing.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum IsolineAmbiguityDecider {
    /// The bilinear saddle lies inside the cell and has a nonzero value.
    BilinearSaddle,
    /// The bilinear center value was used because its saddle was not interior.
    BilinearCenter,
    /// The normalized decider was exactly zero and positive connectivity won.
    PositiveConnectivityTie,
}

/// One disambiguated alternating-sign marching-squares cell.
#[derive(Clone, Copy, Debug, PartialEq)]
#[must_use]
pub struct IsolineAmbiguousCell {
    column: u32,
    row: u32,
    normalized_decider: f64,
    decider: IsolineAmbiguityDecider,
    pairing: IsolineCellPairing,
}

impl IsolineAmbiguousCell {
    /// Returns the zero-based cell column.
    #[must_use]
    pub const fn column(self) -> u32 {
        self.column
    }

    /// Returns the zero-based cell row.
    #[must_use]
    pub const fn row(self) -> u32 {
        self.row
    }

    /// Returns the scale-normalized bilinear decider value.
    #[must_use]
    pub const fn normalized_decider(self) -> f64 {
        self.normalized_decider
    }

    /// Returns how the decider value was obtained.
    #[must_use]
    pub const fn decider(self) -> IsolineAmbiguityDecider {
        self.decider
    }

    /// Returns the selected segment pairing.
    #[must_use]
    pub const fn pairing(self) -> IsolineCellPairing {
        self.pairing
    }
}

/// Deterministic grid, refinement, deduplication, and topology evidence.
#[derive(Clone, Debug, PartialEq)]
#[must_use]
pub struct IsolineDiagnostics {
    method: IsolineMethod,
    cells_x: u32,
    cells_y: u32,
    evaluations: usize,
    raw_segments: usize,
    unique_segments: usize,
    duplicate_segments: usize,
    deduplicated_vertices: usize,
    ambiguous_cells: Vec<IsolineAmbiguousCell>,
    boundary_endpoints: Vec<IsolineBoundaryEndpoint>,
    open_polylines: usize,
    closed_polylines: usize,
}

impl IsolineDiagnostics {
    /// Returns the selected marching method.
    #[must_use]
    pub const fn method(&self) -> IsolineMethod {
        self.method
    }

    /// Returns the requested X cell count.
    #[must_use]
    pub const fn cells_x(&self) -> u32 {
        self.cells_x
    }

    /// Returns the requested Y cell count.
    #[must_use]
    pub const fn cells_y(&self) -> u32 {
        self.cells_y
    }

    /// Returns the actual fitted-field value evaluation count.
    #[must_use]
    pub const fn evaluations(&self) -> usize {
        self.evaluations
    }

    /// Returns the segment count before undirected duplicate removal.
    #[must_use]
    pub const fn raw_segments(&self) -> usize {
        self.raw_segments
    }

    /// Returns the segment count retained for topology construction.
    #[must_use]
    pub const fn unique_segments(&self) -> usize {
        self.unique_segments
    }

    /// Returns the exact canonical duplicates removed from the segment list.
    #[must_use]
    pub const fn duplicate_segments(&self) -> usize {
        self.duplicate_segments
    }

    /// Returns the number of canonical grid-edge or grid-vertex intersections.
    #[must_use]
    pub const fn deduplicated_vertices(&self) -> usize {
        self.deduplicated_vertices
    }

    /// Borrows alternating-sign marching-squares decisions in cell order.
    pub fn ambiguous_cells(&self) -> &[IsolineAmbiguousCell] {
        &self.ambiguous_cells
    }

    /// Borrows requested-domain evidence for every open endpoint.
    pub fn boundary_endpoints(&self) -> &[IsolineBoundaryEndpoint] {
        &self.boundary_endpoints
    }

    /// Returns the number of open connected components.
    #[must_use]
    pub const fn open_polylines(&self) -> usize {
        self.open_polylines
    }

    /// Returns the number of closed connected components.
    #[must_use]
    pub const fn closed_polylines(&self) -> usize {
        self.closed_polylines
    }
}

/// Complete topology and numerical evidence for one isoline request.
#[derive(Clone, Debug, PartialEq)]
#[must_use]
pub struct IsolineReport {
    request: IsolineRequest,
    vertices: Vec<IsolineVertex>,
    polylines: Vec<IsolinePolyline>,
    diagnostics: IsolineDiagnostics,
}

impl IsolineReport {
    /// Returns the validated request.
    pub const fn request(&self) -> IsolineRequest {
        self.request
    }

    /// Borrows deduplicated vertices in canonical grid-identity order.
    pub fn vertices(&self) -> &[IsolineVertex] {
        &self.vertices
    }

    /// Borrows deterministic connected components.
    pub fn polylines(&self) -> &[IsolinePolyline] {
        &self.polylines
    }

    /// Borrows extraction and topology evidence.
    pub const fn diagnostics(&self) -> &IsolineDiagnostics {
        &self.diagnostics
    }
}

/// Logical allocation owned by two-dimensional isoline extraction.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum IsolineStorage {
    /// Sampled grid nodes.
    GridSamples,
    /// Refined horizontal-edge intersections.
    HorizontalIntersections,
    /// Refined vertical-edge intersections.
    VerticalIntersections,
    /// Refined fixed-diagonal intersections.
    DiagonalIntersections,
    /// Cell-local segments before deduplication.
    RawSegments,
    /// Alternating-sign cell decisions.
    AmbiguousCells,
    /// Segment endpoint records.
    EndpointRecords,
    /// Canonical intersection keys.
    VertexKeys,
    /// Returned isoline vertices.
    Vertices,
    /// Unique undirected segment connectivity.
    Segments,
    /// Fixed-degree topology adjacency.
    Adjacency,
    /// Segment visitation flags.
    VisitedSegments,
    /// Returned connected polylines.
    Polylines,
    /// Vertex indices for one connected polyline.
    PolylineVertices,
    /// Open-boundary endpoint records.
    BoundaryEndpoints,
    /// Boundary sides for one endpoint.
    BoundarySides,
}

impl fmt::Display for IsolineStorage {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::GridSamples => "grid samples",
            Self::HorizontalIntersections => "horizontal intersections",
            Self::VerticalIntersections => "vertical intersections",
            Self::DiagonalIntersections => "diagonal intersections",
            Self::RawSegments => "raw segments",
            Self::AmbiguousCells => "ambiguous cells",
            Self::EndpointRecords => "endpoint records",
            Self::VertexKeys => "vertex keys",
            Self::Vertices => "vertices",
            Self::Segments => "segments",
            Self::Adjacency => "adjacency",
            Self::VisitedSegments => "visited segments",
            Self::Polylines => "polylines",
            Self::PolylineVertices => "polyline vertices",
            Self::BoundaryEndpoints => "boundary endpoints",
            Self::BoundarySides => "boundary sides",
        })
    }
}

/// Structured two-dimensional isoline extraction failure.
// The fitted-field source is retained inline so constructing a diagnostic
// cannot introduce an infallible allocation on an error path.
#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
pub enum IsolineError {
    /// Grid, edge, segment, or refinement work arithmetic overflowed.
    WorkBudgetOverflow {
        /// Requested X cells.
        cells_x: u32,
        /// Requested Y cells.
        cells_y: u32,
        /// Requested iterations per crossed edge.
        refinement_iterations: u32,
    },
    /// One logical vector could not reserve its complete requested capacity.
    AllocationFailed {
        /// Logical storage.
        storage: IsolineStorage,
        /// Requested element capacity.
        requested: usize,
    },
    /// Reusable fitted-value evaluation storage could not be prepared.
    Preparation {
        /// Fitted-field failure.
        source: FittedFieldEvaluationError<2>,
    },
    /// The fitted field could not be evaluated at one original-coordinate point.
    Evaluation {
        /// Original-coordinate query.
        point: Point<2>,
        /// Fitted-field failure.
        source: FittedFieldEvaluationError<2>,
    },
    /// A derived grid or refinement point was not finite.
    NonFiniteCoordinate {
        /// Rejected components.
        components: [f64; 2],
    },
    /// Subtracting the finite fitted value and target level was not representable.
    NonFiniteResidual {
        /// Original-coordinate query.
        point: Point<2>,
        /// Finite fitted value.
        value: f64,
        /// Finite target level.
        level: f64,
    },
    /// Both endpoints of one complete grid edge are exactly on the target level.
    ///
    /// The finite samples do not prove that the complete nonlinear edge lies
    /// on the level; they make ordinary one-crossing marching topology
    /// underdetermined at the requested grid resolution.
    DegenerateGridEdge {
        /// First edge endpoint.
        first: Point<2>,
        /// Second edge endpoint.
        second: Point<2>,
    },
    /// A crossed edge exhausted its explicit bisection limit.
    RefinementLimitReached {
        /// Remaining first bracket endpoint.
        first: Point<2>,
        /// Remaining second bracket endpoint.
        second: Point<2>,
        /// Smallest endpoint absolute level residual.
        absolute_residual: f64,
    },
    /// One square had an unsupported exact-vertex intersection pattern.
    DegenerateCellTopology {
        /// Zero-based cell column.
        column: u32,
        /// Zero-based cell row.
        row: u32,
        /// Number of crossed cell edges.
        intersections: usize,
    },
    /// One reference triangle had an unsupported exact-vertex intersection pattern.
    DegenerateSimplexTopology {
        /// Zero-based cell column.
        column: u32,
        /// Zero-based cell row.
        row: u32,
        /// Zero for the lower-right simplex or one for the upper-left simplex.
        simplex: u8,
        /// Number of crossed simplex edges.
        intersections: usize,
    },
    /// A segment's two endpoint identities collapsed to one vertex.
    CollapsedSegment {
        /// Collapsed vertex.
        point: Point<2>,
    },
    /// More than two unique segments met at one vertex.
    NonManifoldVertex {
        /// Rejected vertex.
        point: Point<2>,
        /// Observed degree after adding the rejected segment.
        degree: usize,
    },
    /// An open component terminated away from the requested rectangle.
    InteriorOpenEndpoint {
        /// Rejected endpoint.
        point: Point<2>,
    },
    /// Caller execution policy or cancellation failure.
    Execution(ExecutionError),
}

impl From<ExecutionError> for IsolineError {
    fn from(source: ExecutionError) -> Self {
        Self::Execution(source)
    }
}

impl fmt::Display for IsolineError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WorkBudgetOverflow {
                cells_x,
                cells_y,
                refinement_iterations,
            } => write!(
                formatter,
                "isoline work budget is not representable for {cells_x} by {cells_y} cells and {refinement_iterations} refinement iterations"
            ),
            Self::AllocationFailed { storage, requested } => {
                write!(
                    formatter,
                    "isoline {storage} could not reserve {requested} elements"
                )
            }
            Self::Preparation { source } => {
                write!(formatter, "isoline evaluation preparation failed: {source}")
            }
            Self::Evaluation { point, source } => write!(
                formatter,
                "isoline fitted-field evaluation failed at {:?}: {source}",
                point.components()
            ),
            Self::NonFiniteCoordinate { components } => {
                write!(
                    formatter,
                    "isoline derived coordinate is not finite: {components:?}"
                )
            }
            Self::NonFiniteResidual {
                point,
                value,
                level,
            } => write!(
                formatter,
                "isoline residual is not finite at {:?}: {value} - {level}",
                point.components()
            ),
            Self::DegenerateGridEdge { first, second } => write!(
                formatter,
                "isoline grid edge from {:?} to {:?} has both endpoints exactly on the target level",
                first.components(),
                second.components()
            ),
            Self::RefinementLimitReached {
                first,
                second,
                absolute_residual,
            } => write!(
                formatter,
                "isoline edge refinement exhausted between {:?} and {:?} with absolute residual {absolute_residual}",
                first.components(),
                second.components()
            ),
            Self::DegenerateCellTopology {
                column,
                row,
                intersections,
            } => write!(
                formatter,
                "isoline cell ({column}, {row}) has {intersections} exact/crossing edge intersections"
            ),
            Self::DegenerateSimplexTopology {
                column,
                row,
                simplex,
                intersections,
            } => write!(
                formatter,
                "isoline cell ({column}, {row}) simplex {simplex} has {intersections} exact/crossing edge intersections"
            ),
            Self::CollapsedSegment { point } => write!(
                formatter,
                "isoline segment collapsed to one vertex at {:?}",
                point.components()
            ),
            Self::NonManifoldVertex { point, degree } => write!(
                formatter,
                "isoline vertex at {:?} has non-manifold degree {degree}",
                point.components()
            ),
            Self::InteriorOpenEndpoint { point } => write!(
                formatter,
                "isoline component terminates inside the requested domain at {:?}",
                point.components()
            ),
            Self::Execution(source) => source.fmt(formatter),
        }
    }
}

impl Error for IsolineError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Preparation { source } | Self::Evaluation { source, .. } => Some(source),
            Self::Execution(source) => Some(source),
            Self::WorkBudgetOverflow { .. }
            | Self::AllocationFailed { .. }
            | Self::NonFiniteCoordinate { .. }
            | Self::NonFiniteResidual { .. }
            | Self::DegenerateGridEdge { .. }
            | Self::RefinementLimitReached { .. }
            | Self::DegenerateCellTopology { .. }
            | Self::DegenerateSimplexTopology { .. }
            | Self::CollapsedSegment { .. }
            | Self::NonManifoldVertex { .. }
            | Self::InteriorOpenEndpoint { .. } => None,
        }
    }
}

fn preparation_error(source: FittedFieldEvaluationError<2>) -> IsolineError {
    IsolineError::Preparation { source }
}

fn evaluation_error(point: Point<2>, source: FittedFieldEvaluationError<2>) -> IsolineError {
    IsolineError::Evaluation { point, source }
}

impl FittedField<2> {
    /// Extracts deterministic two-dimensional isoline topology.
    ///
    /// # Errors
    ///
    /// Returns structured preparation, evaluation, allocation, arithmetic,
    /// refinement, degeneracy, or topology failures. No partial report is
    /// returned.
    pub fn try_isolines(&self, request: &IsolineRequest) -> Result<IsolineReport, IsolineError> {
        self.try_isolines_with_control(
            request,
            ExecutionOptions::default(),
            ExecutionControl::default(),
        )
    }

    /// Extracts isolines with explicit serial execution metadata and controls.
    ///
    /// The current algorithm rejects an explicit thread count above one before
    /// preparing fitted-field scratch or evaluating a point. Cancellation is
    /// checked before and after every fitted-field value query and before the
    /// successful terminal progress event.
    ///
    /// # Errors
    ///
    /// Returns the same failures as [`Self::try_isolines`], plus structured
    /// execution-policy and cancellation failures.
    pub fn try_isolines_with_control(
        &self,
        request: &IsolineRequest,
        execution: ExecutionOptions,
        control: ExecutionControl<'_>,
    ) -> Result<IsolineReport, IsolineError> {
        let work = GridWork::try_new(request)?;
        let mut progress = ProgressTracker::try_new(
            control,
            ExecutionOperation::IsolineExtraction,
            execution,
            work.maximum_evaluations,
        )?;
        let scratch_result = self
            .try_evaluation_scratch(FittedFieldOutput::Value)
            .map_err(preparation_error);
        let mut scratch = progress.observe_result(ExecutionStage::Started, scratch_result)?;
        let mut evaluations = 0_usize;
        let mut report = extract_isolines(request, work, |point| {
            let sample = finish_value_query(&mut progress, || {
                self.try_value_with_scratch(point, &mut scratch)
                    .map_err(|source| evaluation_error(point, source))
                    .and_then(|value| {
                        let residual = value - request.level;
                        if residual.is_finite() {
                            Ok(Sample {
                                point,
                                value,
                                residual,
                            })
                        } else {
                            Err(IsolineError::NonFiniteResidual {
                                point,
                                value,
                                level: request.level,
                            })
                        }
                    })
            })?;
            evaluations = evaluations.saturating_add(1);
            Ok(sample)
        })?;
        report.diagnostics.evaluations = evaluations;
        progress.complete()?;
        Ok(report)
    }
}

fn finish_value_query<T>(
    progress: &mut ProgressTracker<'_>,
    query: impl FnOnce() -> Result<T, IsolineError>,
) -> Result<T, IsolineError> {
    progress.checkpoint(ExecutionStage::IsolineEvaluation)?;
    let result = query();
    progress.finish_work(ExecutionStage::IsolineEvaluation, result)
}

#[derive(Clone, Copy, Debug)]
struct Sample {
    point: Point<2>,
    value: f64,
    residual: f64,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum IntersectionKey {
    Vertex(usize),
    Horizontal { column: usize, row: usize },
    Vertical { column: usize, row: usize },
    Diagonal { column: usize, row: usize },
}

#[derive(Clone, Copy, Debug)]
struct Endpoint {
    key: IntersectionKey,
    sample: Sample,
}

#[derive(Clone, Copy, Debug)]
struct RawSegment {
    first: Endpoint,
    second: Endpoint,
}

#[derive(Clone, Copy, Debug)]
struct GridWork {
    columns: usize,
    rows: usize,
    node_columns: usize,
    node_count: usize,
    horizontal_count: usize,
    vertical_count: usize,
    diagonal_count: usize,
    cell_count: usize,
    raw_segment_capacity: usize,
    maximum_evaluations: usize,
}

impl GridWork {
    fn try_new(request: &IsolineRequest) -> Result<Self, IsolineError> {
        let columns =
            usize::try_from(request.settings.cells_x.get()).map_err(|_| work_overflow(request))?;
        let rows =
            usize::try_from(request.settings.cells_y.get()).map_err(|_| work_overflow(request))?;
        let node_columns = columns
            .checked_add(1)
            .ok_or_else(|| work_overflow(request))?;
        let node_rows = rows.checked_add(1).ok_or_else(|| work_overflow(request))?;
        let node_count = node_columns
            .checked_mul(node_rows)
            .ok_or_else(|| work_overflow(request))?;
        let horizontal_count = columns
            .checked_mul(node_rows)
            .ok_or_else(|| work_overflow(request))?;
        let vertical_count = node_columns
            .checked_mul(rows)
            .ok_or_else(|| work_overflow(request))?;
        let cell_count = columns
            .checked_mul(rows)
            .ok_or_else(|| work_overflow(request))?;
        let diagonal_count = if request.method == IsolineMethod::MarchingSimplices {
            cell_count
        } else {
            0
        };
        let edge_count = horizontal_count
            .checked_add(vertical_count)
            .and_then(|count| count.checked_add(diagonal_count))
            .ok_or_else(|| work_overflow(request))?;
        let iterations = usize::try_from(request.settings.refinement_iterations.get())
            .map_err(|_| work_overflow(request))?;
        let maximum_evaluations = edge_count
            .checked_mul(iterations)
            .and_then(|refinement| refinement.checked_add(node_count))
            .ok_or_else(|| work_overflow(request))?;
        let raw_segment_capacity = cell_count
            .checked_mul(2)
            .ok_or_else(|| work_overflow(request))?;
        Ok(Self {
            columns,
            rows,
            node_columns,
            node_count,
            horizontal_count,
            vertical_count,
            diagonal_count,
            cell_count,
            raw_segment_capacity,
            maximum_evaluations,
        })
    }
}

#[allow(clippy::too_many_lines)]
fn extract_isolines(
    request: &IsolineRequest,
    work: GridWork,
    mut evaluate: impl FnMut(Point<2>) -> Result<Sample, IsolineError>,
) -> Result<IsolineReport, IsolineError> {
    let mut nodes = try_vec(work.node_count, IsolineStorage::GridSamples)?;
    for row in 0..=request.settings.cells_y.get() {
        let row_index = usize::try_from(row).map_err(|_| work_overflow(request))?;
        let y = interpolate(
            request.lower.components()[1],
            request.upper.components()[1],
            row,
            request.settings.cells_y.get(),
        );
        for column in 0..=request.settings.cells_x.get() {
            let column_index = usize::try_from(column).map_err(|_| work_overflow(request))?;
            let x = interpolate(
                request.lower.components()[0],
                request.upper.components()[0],
                column,
                request.settings.cells_x.get(),
            );
            let point = Point::try_new([x, y])
                .map_err(|_| IsolineError::NonFiniteCoordinate { components: [x, y] })?;
            debug_assert_eq!(
                nodes.len(),
                node_index(column_index, row_index, work.node_columns)
            );
            nodes.push(evaluate(point)?);
        }
    }

    let mut horizontal = try_vec(
        work.horizontal_count,
        IsolineStorage::HorizontalIntersections,
    )?;
    for row in 0..=work.rows {
        for column in 0..work.columns {
            let first_index = node_index(column, row, work.node_columns);
            let second_index = node_index(column + 1, row, work.node_columns);
            horizontal.push(edge_intersection(
                request,
                nodes[first_index],
                nodes[second_index],
                IntersectionKey::Vertex(first_index),
                IntersectionKey::Vertex(second_index),
                IntersectionKey::Horizontal { column, row },
                &mut evaluate,
            )?);
        }
    }

    let mut vertical = try_vec(work.vertical_count, IsolineStorage::VerticalIntersections)?;
    for row in 0..work.rows {
        for column in 0..=work.columns {
            let first_index = node_index(column, row, work.node_columns);
            let second_index = node_index(column, row + 1, work.node_columns);
            vertical.push(edge_intersection(
                request,
                nodes[first_index],
                nodes[second_index],
                IntersectionKey::Vertex(first_index),
                IntersectionKey::Vertex(second_index),
                IntersectionKey::Vertical { column, row },
                &mut evaluate,
            )?);
        }
    }

    let mut diagonal = try_vec(work.diagonal_count, IsolineStorage::DiagonalIntersections)?;
    if request.method == IsolineMethod::MarchingSimplices {
        for row in 0..work.rows {
            for column in 0..work.columns {
                let first_index = node_index(column, row, work.node_columns);
                let second_index = node_index(column + 1, row + 1, work.node_columns);
                diagonal.push(edge_intersection(
                    request,
                    nodes[first_index],
                    nodes[second_index],
                    IntersectionKey::Vertex(first_index),
                    IntersectionKey::Vertex(second_index),
                    IntersectionKey::Diagonal { column, row },
                    &mut evaluate,
                )?);
            }
        }
    }

    let mut raw_segments = try_vec(work.raw_segment_capacity, IsolineStorage::RawSegments)?;
    let mut ambiguous_cells = try_vec(work.cell_count, IsolineStorage::AmbiguousCells)?;
    for row in 0..work.rows {
        for column in 0..work.columns {
            let bottom = horizontal[horizontal_index(column, row, work.columns)];
            let top = horizontal[horizontal_index(column, row + 1, work.columns)];
            let left = vertical[vertical_index(column, row, work.node_columns)];
            let right = vertical[vertical_index(column + 1, row, work.node_columns)];
            match request.method {
                IsolineMethod::DisambiguatedMarchingSquares => {
                    let lower_left = nodes[node_index(column, row, work.node_columns)];
                    let lower_right = nodes[node_index(column + 1, row, work.node_columns)];
                    let upper_right = nodes[node_index(column + 1, row + 1, work.node_columns)];
                    let upper_left = nodes[node_index(column, row + 1, work.node_columns)];
                    emit_square(
                        request,
                        column,
                        row,
                        [bottom, right, top, left],
                        [lower_left, lower_right, upper_right, upper_left],
                        &mut raw_segments,
                        &mut ambiguous_cells,
                    )?;
                }
                IsolineMethod::MarchingSimplices => {
                    let cell_index = row * work.columns + column;
                    let shared_diagonal = diagonal[cell_index];
                    emit_simplex(
                        request,
                        column,
                        row,
                        0,
                        [bottom, right, shared_diagonal],
                        &mut raw_segments,
                    )?;
                    emit_simplex(
                        request,
                        column,
                        row,
                        1,
                        [shared_diagonal, top, left],
                        &mut raw_segments,
                    )?;
                }
            }
        }
    }

    build_report(request, &raw_segments, ambiguous_cells)
}

fn emit_square(
    request: &IsolineRequest,
    column: usize,
    row: usize,
    edges: [Option<Endpoint>; 4],
    corners: [Sample; 4],
    raw_segments: &mut Vec<RawSegment>,
    ambiguous_cells: &mut Vec<IsolineAmbiguousCell>,
) -> Result<(), IsolineError> {
    let (intersections, count) = unique_endpoints(edges);
    match count {
        0 => Ok(()),
        2 => {
            raw_segments.push(RawSegment {
                first: intersections[0].ok_or_else(|| cell_error(request, column, row, count))?,
                second: intersections[1].ok_or_else(|| cell_error(request, column, row, count))?,
            });
            Ok(())
        }
        4 => {
            if corners.iter().any(|corner| corner.residual == 0.0) {
                return Err(cell_error(request, column, row, count));
            }
            let pattern = corners
                .iter()
                .enumerate()
                .fold(0_u8, |bits, (index, corner)| {
                    if corner.residual.is_sign_positive() {
                        bits | (1_u8 << index)
                    } else {
                        bits
                    }
                });
            if pattern != 0b0101 && pattern != 0b1010 {
                return Err(cell_error(request, column, row, count));
            }
            let (normalized_decider, decider) = ambiguity_decider(corners);
            let positive_connected = normalized_decider >= 0.0;
            let pairing = if (pattern == 0b0101 && positive_connected)
                || (pattern == 0b1010 && !positive_connected)
            {
                IsolineCellPairing::BottomRightAndTopLeft
            } else {
                IsolineCellPairing::BottomLeftAndRightTop
            };
            let endpoint =
                |index: usize| edges[index].ok_or_else(|| cell_error(request, column, row, count));
            match pairing {
                IsolineCellPairing::BottomRightAndTopLeft => {
                    raw_segments.push(RawSegment {
                        first: endpoint(0)?,
                        second: endpoint(1)?,
                    });
                    raw_segments.push(RawSegment {
                        first: endpoint(2)?,
                        second: endpoint(3)?,
                    });
                }
                IsolineCellPairing::BottomLeftAndRightTop => {
                    raw_segments.push(RawSegment {
                        first: endpoint(0)?,
                        second: endpoint(3)?,
                    });
                    raw_segments.push(RawSegment {
                        first: endpoint(1)?,
                        second: endpoint(2)?,
                    });
                }
            }
            ambiguous_cells.push(IsolineAmbiguousCell {
                column: u32::try_from(column).map_err(|_| work_overflow(request))?,
                row: u32::try_from(row).map_err(|_| work_overflow(request))?,
                normalized_decider,
                decider,
                pairing,
            });
            Ok(())
        }
        _ => Err(cell_error(request, column, row, count)),
    }
}

fn emit_simplex(
    request: &IsolineRequest,
    column: usize,
    row: usize,
    simplex: u8,
    edges: [Option<Endpoint>; 3],
    raw_segments: &mut Vec<RawSegment>,
) -> Result<(), IsolineError> {
    let (intersections, count) = unique_endpoints(edges);
    match count {
        0 => Ok(()),
        2 => {
            let column = u32::try_from(column).map_err(|_| work_overflow(request))?;
            let row = u32::try_from(row).map_err(|_| work_overflow(request))?;
            let first = intersections[0].ok_or(IsolineError::DegenerateSimplexTopology {
                column,
                row,
                simplex,
                intersections: count,
            })?;
            let second = intersections[1].ok_or(IsolineError::DegenerateSimplexTopology {
                column,
                row,
                simplex,
                intersections: count,
            })?;
            raw_segments.push(RawSegment { first, second });
            Ok(())
        }
        _ => Err(IsolineError::DegenerateSimplexTopology {
            column: u32::try_from(column).map_err(|_| work_overflow(request))?,
            row: u32::try_from(row).map_err(|_| work_overflow(request))?,
            simplex,
            intersections: count,
        }),
    }
}

fn unique_endpoints<const N: usize>(
    edges: [Option<Endpoint>; N],
) -> ([Option<Endpoint>; N], usize) {
    let mut intersections: [Option<Endpoint>; N] = [None; N];
    let mut count = 0_usize;
    for endpoint in edges.into_iter().flatten() {
        if intersections[..count]
            .iter()
            .flatten()
            .any(|existing| existing.key == endpoint.key)
        {
            continue;
        }
        intersections[count] = Some(endpoint);
        count += 1;
    }
    (intersections, count)
}

fn ambiguity_decider(corners: [Sample; 4]) -> (f64, IsolineAmbiguityDecider) {
    let scale = corners
        .iter()
        .map(|corner| corner.residual.abs())
        .fold(0.0_f64, f64::max);
    let values = corners.map(|corner| corner.residual / scale);
    let lower_left = values[0];
    let lower_right = values[1];
    let upper_right = values[2];
    let upper_left = values[3];
    let b = lower_right - lower_left;
    let c = upper_left - lower_left;
    let d = lower_left - lower_right - upper_left + upper_right;
    if d != 0.0 {
        let saddle_x = -c / d;
        let saddle_y = -b / d;
        if saddle_x > 0.0 && saddle_x < 1.0 && saddle_y > 0.0 && saddle_y < 1.0 {
            let value = lower_left - b * c / d;
            if value == 0.0 {
                return (value, IsolineAmbiguityDecider::PositiveConnectivityTie);
            }
            return (value, IsolineAmbiguityDecider::BilinearSaddle);
        }
    }
    let center = 0.25 * (lower_left + lower_right + upper_right + upper_left);
    if center == 0.0 {
        (center, IsolineAmbiguityDecider::PositiveConnectivityTie)
    } else {
        (center, IsolineAmbiguityDecider::BilinearCenter)
    }
}

fn cell_error(
    request: &IsolineRequest,
    column: usize,
    row: usize,
    intersections: usize,
) -> IsolineError {
    IsolineError::DegenerateCellTopology {
        column: u32::try_from(column).unwrap_or(request.settings.cells_x.get()),
        row: u32::try_from(row).unwrap_or(request.settings.cells_y.get()),
        intersections,
    }
}

fn edge_intersection(
    request: &IsolineRequest,
    first: Sample,
    second: Sample,
    first_key: IntersectionKey,
    second_key: IntersectionKey,
    edge_key: IntersectionKey,
    evaluate: &mut impl FnMut(Point<2>) -> Result<Sample, IsolineError>,
) -> Result<Option<Endpoint>, IsolineError> {
    if first.residual == 0.0 && second.residual == 0.0 {
        return Err(IsolineError::DegenerateGridEdge {
            first: first.point,
            second: second.point,
        });
    }
    if first.residual == 0.0 {
        return Ok(Some(Endpoint {
            key: first_key,
            sample: first,
        }));
    }
    if second.residual == 0.0 {
        return Ok(Some(Endpoint {
            key: second_key,
            sample: second,
        }));
    }
    if same_sign(first.residual, second.residual) {
        return Ok(None);
    }
    let first_absolute = first.residual.abs();
    let second_absolute = second.residual.abs();
    if first_absolute <= request.settings.value_tolerance
        || second_absolute <= request.settings.value_tolerance
    {
        return Ok(Some(if first_absolute <= second_absolute {
            Endpoint {
                key: edge_key,
                sample: first,
            }
        } else {
            Endpoint {
                key: edge_key,
                sample: second,
            }
        }));
    }
    Ok(Some(Endpoint {
        key: edge_key,
        sample: refine_edge(request, first, second, evaluate)?,
    }))
}

fn refine_edge(
    request: &IsolineRequest,
    mut first: Sample,
    mut second: Sample,
    evaluate: &mut impl FnMut(Point<2>) -> Result<Sample, IsolineError>,
) -> Result<Sample, IsolineError> {
    for _ in 0..request.settings.refinement_iterations.get() {
        let first_components = first.point.components();
        let second_components = second.point.components();
        let middle_components = [
            first_components[0] + (second_components[0] - first_components[0]) * 0.5,
            first_components[1] + (second_components[1] - first_components[1]) * 0.5,
        ];
        let middle =
            Point::try_new(middle_components).map_err(|_| IsolineError::NonFiniteCoordinate {
                components: middle_components,
            })?;
        let sample = evaluate(middle)?;
        if sample.residual.abs() <= request.settings.value_tolerance {
            return Ok(sample);
        }
        if same_sign(sample.residual, first.residual) {
            first = sample;
        } else {
            second = sample;
        }
        if bracket_width(first.point, second.point) <= request.settings.coordinate_tolerance {
            return Ok(sample);
        }
    }
    Err(IsolineError::RefinementLimitReached {
        first: first.point,
        second: second.point,
        absolute_residual: first.residual.abs().min(second.residual.abs()),
    })
}

#[allow(clippy::too_many_lines)]
fn build_report(
    request: &IsolineRequest,
    raw_segments: &[RawSegment],
    ambiguous_cells: Vec<IsolineAmbiguousCell>,
) -> Result<IsolineReport, IsolineError> {
    let endpoint_capacity = raw_segments
        .len()
        .checked_mul(2)
        .ok_or_else(|| work_overflow(request))?;
    let mut endpoint_records = try_vec(endpoint_capacity, IsolineStorage::EndpointRecords)?;
    for segment in raw_segments {
        endpoint_records.push((segment.first.key, segment.first.sample));
        endpoint_records.push((segment.second.key, segment.second.sample));
    }
    sort_endpoint_records(&mut endpoint_records);
    endpoint_records.dedup_by(|later, earlier| {
        if later.0 != earlier.0 {
            return false;
        }
        if later.1.residual.abs() < earlier.1.residual.abs() {
            earlier.1 = later.1;
        }
        true
    });

    let mut vertex_keys = try_vec(endpoint_records.len(), IsolineStorage::VertexKeys)?;
    let mut vertices = try_vec(endpoint_records.len(), IsolineStorage::Vertices)?;
    for (key, sample) in endpoint_records {
        vertex_keys.push(key);
        vertices.push(IsolineVertex {
            point: sample.point,
            value: sample.value,
            residual: sample.residual,
        });
    }

    let mut segments = try_vec(raw_segments.len(), IsolineStorage::Segments)?;
    for segment in raw_segments {
        let first = vertex_keys
            .binary_search(&segment.first.key)
            .map_err(|_| work_overflow(request))?;
        let second = vertex_keys
            .binary_search(&segment.second.key)
            .map_err(|_| work_overflow(request))?;
        if first == second {
            return Err(IsolineError::CollapsedSegment {
                point: vertices[first].point,
            });
        }
        segments.push(if first < second {
            (first, second)
        } else {
            (second, first)
        });
    }
    segments.sort_unstable();
    segments.dedup();
    let duplicate_segments = raw_segments.len().saturating_sub(segments.len());

    let mut adjacency = try_vec(vertices.len(), IsolineStorage::Adjacency)?;
    adjacency.resize(vertices.len(), Adjacency::default());
    for (edge_index, &(first, second)) in segments.iter().enumerate() {
        add_link(
            &mut adjacency[first],
            second,
            edge_index,
            vertices[first].point,
        )?;
        add_link(
            &mut adjacency[second],
            first,
            edge_index,
            vertices[second].point,
        )?;
    }

    let mut boundary_endpoints = try_vec(vertices.len(), IsolineStorage::BoundaryEndpoints)?;
    for (index, adjacent) in adjacency.iter().enumerate() {
        if adjacent.degree != 1 {
            continue;
        }
        let sides = boundary_sides(request, vertices[index].point)?;
        if sides.is_empty() {
            return Err(IsolineError::InteriorOpenEndpoint {
                point: vertices[index].point,
            });
        }
        boundary_endpoints.push(IsolineBoundaryEndpoint {
            vertex_index: index,
            sides,
        });
    }

    let mut visited = try_vec(segments.len(), IsolineStorage::VisitedSegments)?;
    visited.resize(segments.len(), false);
    let mut polylines = try_vec(segments.len(), IsolineStorage::Polylines)?;
    for start in 0..vertices.len() {
        if adjacency[start].degree == 1
            && adjacency[start]
                .links
                .iter()
                .flatten()
                .any(|link| !visited[link.edge])
        {
            polylines.push(trace_polyline(
                request,
                start,
                &vertices,
                &adjacency,
                &mut visited,
                segments.len(),
            )?);
        }
    }
    for (edge_index, &(first, _)) in segments.iter().enumerate() {
        if !visited[edge_index] {
            polylines.push(trace_polyline(
                request,
                first,
                &vertices,
                &adjacency,
                &mut visited,
                segments.len(),
            )?);
        }
    }
    sort_polylines(&mut polylines);
    let open_polylines = polylines.iter().filter(|line| !line.closed).count();
    let closed_polylines = polylines.len().saturating_sub(open_polylines);

    Ok(IsolineReport {
        request: *request,
        diagnostics: IsolineDiagnostics {
            method: request.method,
            cells_x: request.settings.cells_x.get(),
            cells_y: request.settings.cells_y.get(),
            evaluations: 0,
            raw_segments: raw_segments.len(),
            unique_segments: segments.len(),
            duplicate_segments,
            deduplicated_vertices: vertices.len(),
            ambiguous_cells,
            boundary_endpoints,
            open_polylines,
            closed_polylines,
        },
        vertices,
        polylines,
    })
}

fn sort_endpoint_records(records: &mut [(IntersectionKey, Sample)]) {
    records.sort_unstable_by(|left, right| {
        left.0
            .cmp(&right.0)
            .then_with(|| left.1.residual.abs().total_cmp(&right.1.residual.abs()))
            .then_with(|| left.1.point.components()[0].total_cmp(&right.1.point.components()[0]))
            .then_with(|| left.1.point.components()[1].total_cmp(&right.1.point.components()[1]))
            .then_with(|| left.1.value.total_cmp(&right.1.value))
            .then_with(|| left.1.residual.total_cmp(&right.1.residual))
    });
}

fn sort_polylines(polylines: &mut [IsolinePolyline]) {
    polylines.sort_unstable_by(|left, right| {
        left.vertex_indices
            .cmp(&right.vertex_indices)
            .then_with(|| left.closed.cmp(&right.closed))
    });
}

#[derive(Clone, Copy, Debug)]
struct Link {
    vertex: usize,
    edge: usize,
}

#[derive(Clone, Copy, Debug, Default)]
struct Adjacency {
    links: [Option<Link>; 2],
    degree: usize,
}

fn add_link(
    adjacency: &mut Adjacency,
    vertex: usize,
    edge: usize,
    point: Point<2>,
) -> Result<(), IsolineError> {
    if adjacency.degree >= 2 {
        return Err(IsolineError::NonManifoldVertex {
            point,
            degree: adjacency.degree.saturating_add(1),
        });
    }
    adjacency.links[adjacency.degree] = Some(Link { vertex, edge });
    adjacency.degree += 1;
    Ok(())
}

fn trace_polyline(
    request: &IsolineRequest,
    start: usize,
    vertices: &[IsolineVertex],
    adjacency: &[Adjacency],
    visited: &mut [bool],
    segment_count: usize,
) -> Result<IsolinePolyline, IsolineError> {
    let capacity = segment_count
        .checked_add(1)
        .ok_or_else(|| work_overflow(request))?;
    let mut vertex_indices = try_vec(capacity.min(16), IsolineStorage::PolylineVertices)?;
    let mut current = start;
    loop {
        try_push(
            &mut vertex_indices,
            current,
            IsolineStorage::PolylineVertices,
        )?;
        let next = adjacency[current]
            .links
            .iter()
            .flatten()
            .filter(|link| !visited[link.edge])
            .min_by_key(|link| link.vertex)
            .copied();
        let Some(link) = next else {
            return Ok(IsolinePolyline {
                vertex_indices,
                closed: false,
            });
        };
        visited[link.edge] = true;
        current = link.vertex;
        if current == start {
            return Ok(IsolinePolyline {
                vertex_indices,
                closed: true,
            });
        }
        if vertex_indices.len() > segment_count {
            return Err(IsolineError::NonManifoldVertex {
                point: vertices[current].point,
                degree: 3,
            });
        }
    }
}

fn boundary_sides(
    request: &IsolineRequest,
    point: Point<2>,
) -> Result<Vec<IsolineBoundarySide>, IsolineError> {
    let mut sides = try_vec(4, IsolineStorage::BoundarySides)?;
    let coordinates = point.components();
    let lower = request.lower.components();
    let upper = request.upper.components();
    let tolerance = request.settings.coordinate_tolerance;
    if (coordinates[0] - lower[0]).abs() <= tolerance {
        sides.push(IsolineBoundarySide::Left);
    }
    if (coordinates[0] - upper[0]).abs() <= tolerance {
        sides.push(IsolineBoundarySide::Right);
    }
    if (coordinates[1] - lower[1]).abs() <= tolerance {
        sides.push(IsolineBoundarySide::Bottom);
    }
    if (coordinates[1] - upper[1]).abs() <= tolerance {
        sides.push(IsolineBoundarySide::Top);
    }
    Ok(sides)
}

fn node_index(column: usize, row: usize, node_columns: usize) -> usize {
    row * node_columns + column
}

fn horizontal_index(column: usize, row: usize, columns: usize) -> usize {
    row * columns + column
}

fn vertical_index(column: usize, row: usize, node_columns: usize) -> usize {
    row * node_columns + column
}

fn interpolate(lower: f64, upper: f64, index: u32, intervals: u32) -> f64 {
    if index == 0 {
        lower
    } else if index == intervals {
        upper
    } else {
        let ratio = f64::from(index) / f64::from(intervals);
        lower.mul_add(1.0 - ratio, upper * ratio)
    }
}

fn bracket_width(first: Point<2>, second: Point<2>) -> f64 {
    let first = first.components();
    let second = second.components();
    (second[0] - first[0])
        .abs()
        .max((second[1] - first[1]).abs())
}

fn same_sign(first: f64, second: f64) -> bool {
    first.is_sign_positive() == second.is_sign_positive()
}

fn work_overflow(request: &IsolineRequest) -> IsolineError {
    IsolineError::WorkBudgetOverflow {
        cells_x: request.settings.cells_x.get(),
        cells_y: request.settings.cells_y.get(),
        refinement_iterations: request.settings.refinement_iterations.get(),
    }
}

fn try_vec<T>(requested: usize, storage: IsolineStorage) -> Result<Vec<T>, IsolineError> {
    let mut values = Vec::new();
    values
        .try_reserve_exact(requested)
        .map_err(|_| IsolineError::AllocationFailed { storage, requested })?;
    Ok(values)
}

fn try_push<T>(values: &mut Vec<T>, value: T, storage: IsolineStorage) -> Result<(), IsolineError> {
    if values.len() == values.capacity() {
        let requested = values.len().saturating_add(1);
        values
            .try_reserve(1)
            .map_err(|_| IsolineError::AllocationFailed { storage, requested })?;
    }
    values.push(value);
    Ok(())
}

fn validate_tolerance(tolerance: IsolineTolerance, value: f64) -> Result<(), IsolineSettingsError> {
    if !value.is_finite() || value <= 0.0 {
        return Err(IsolineSettingsError::InvalidTolerance { tolerance, value });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::error::Error;

    use super::{
        Endpoint, IntersectionKey, IsolineError, IsolinePolyline, Sample, evaluation_error,
        finish_value_query, preparation_error, sort_endpoint_records, sort_polylines,
    };
    use crate::execution::{
        CancellationToken, ExecutionControl, ExecutionError, ExecutionOperation, ExecutionStage,
        ProgressTracker,
    };
    use crate::geometry::Point;
    use crate::model::{FittedFieldComponent, FittedFieldEvaluationError, FittedFieldStorage};
    use crate::problem_ir::ExecutionOptions;

    #[test]
    fn fitted_field_source_conversions_use_no_allocation() -> Result<(), Box<dyn Error>> {
        let preparation_source = FittedFieldEvaluationError::AllocationFailed {
            storage: FittedFieldStorage::PolynomialValues,
            requested: 7,
        };
        let mut preparation = None;
        let preparation_allocations = allocation_counter::measure(|| {
            preparation = Some(preparation_error(preparation_source));
        });
        assert_eq!(preparation_allocations.count_total, 0);
        let preparation = preparation
            .ok_or_else(|| std::io::Error::other("preparation conversion did not run"))?;
        let preparation_source = preparation
            .source()
            .and_then(|source| source.downcast_ref::<FittedFieldEvaluationError<2>>())
            .ok_or_else(|| std::io::Error::other("preparation source was not retained"))?;
        assert!(matches!(
            preparation_source,
            FittedFieldEvaluationError::AllocationFailed {
                storage: FittedFieldStorage::PolynomialValues,
                requested: 7,
            }
        ));

        let point = Point::try_new([1.0, -2.0])?;
        let evaluation_source = FittedFieldEvaluationError::NonFiniteOutput {
            component: FittedFieldComponent::Value,
        };
        let mut evaluation = None;
        let evaluation_allocations = allocation_counter::measure(|| {
            evaluation = Some(evaluation_error(point, evaluation_source));
        });
        assert_eq!(evaluation_allocations.count_total, 0);
        let evaluation =
            evaluation.ok_or_else(|| std::io::Error::other("evaluation conversion did not run"))?;
        assert!(matches!(
            &evaluation,
            IsolineError::Evaluation {
                point: retained_point,
                ..
            } if retained_point == &point
        ));
        let evaluation_source = evaluation
            .source()
            .and_then(|source| source.downcast_ref::<FittedFieldEvaluationError<2>>())
            .ok_or_else(|| std::io::Error::other("evaluation source was not retained"))?;
        assert!(matches!(
            evaluation_source,
            FittedFieldEvaluationError::NonFiniteOutput {
                component: FittedFieldComponent::Value,
            }
        ));
        Ok(())
    }

    #[test]
    fn topology_ordering_uses_no_auxiliary_allocation() -> Result<(), Box<dyn Error>> {
        const COUNT: usize = 4096;

        let mut records = Vec::with_capacity(COUNT);
        for index in (0..COUNT).rev() {
            let index_value = f64::from(u32::try_from(index)?);
            let row_value = f64::from(u32::try_from(index % 17)?);
            records.push((
                IntersectionKey::Horizontal {
                    column: index,
                    row: index % 17,
                },
                Sample {
                    point: Point::try_new([index_value, row_value])?,
                    value: index_value,
                    residual: index_value.mul_add(0.5, -1000.0),
                },
            ));
        }
        let record_allocations =
            allocation_counter::measure(|| sort_endpoint_records(&mut records));
        assert_eq!(record_allocations.count_total, 0);

        let mut polylines = Vec::with_capacity(COUNT);
        for index in (0..COUNT).rev() {
            polylines.push(IsolinePolyline {
                vertex_indices: vec![index, index + 1],
                closed: index % 2 == 0,
            });
        }
        let polyline_allocations = allocation_counter::measure(|| sort_polylines(&mut polylines));
        assert_eq!(polyline_allocations.count_total, 0);
        Ok(())
    }

    #[test]
    fn value_query_checks_cancellation_before_and_after_evaluator() -> Result<(), Box<dyn Error>> {
        let token = CancellationToken::new();
        let mut progress = ProgressTracker::try_new(
            ExecutionControl::with_cancellation(&token),
            ExecutionOperation::IsolineExtraction,
            ExecutionOptions::default(),
            1,
        )?;
        progress.checkpoint(ExecutionStage::IsolineEvaluation)?;
        token.cancel();
        let calls = Cell::new(0_usize);
        let cancelled_before: Result<(), IsolineError> = finish_value_query(&mut progress, || {
            calls.set(calls.get() + 1);
            Ok(())
        });
        assert!(matches!(
            cancelled_before,
            Err(IsolineError::Execution(ExecutionError::Cancelled {
                operation: ExecutionOperation::IsolineExtraction,
                stage: ExecutionStage::IsolineEvaluation,
            }))
        ));
        assert_eq!(calls.get(), 0);

        let token = CancellationToken::new();
        let mut progress = ProgressTracker::try_new(
            ExecutionControl::with_cancellation(&token),
            ExecutionOperation::IsolineExtraction,
            ExecutionOptions::default(),
            1,
        )?;
        let calls = Cell::new(0_usize);
        let cancelled_after: Result<(), IsolineError> = finish_value_query(&mut progress, || {
            calls.set(calls.get() + 1);
            token.cancel();
            Err(IsolineError::NonFiniteCoordinate {
                components: [f64::NAN, 0.0],
            })
        });
        assert!(matches!(
            cancelled_after,
            Err(IsolineError::Execution(ExecutionError::Cancelled {
                operation: ExecutionOperation::IsolineExtraction,
                stage: ExecutionStage::IsolineEvaluation,
            }))
        ));
        assert_eq!(calls.get(), 1);
        Ok(())
    }

    #[test]
    fn unique_endpoint_identity_is_key_based() -> Result<(), Box<dyn Error>> {
        let sample = Sample {
            point: Point::try_new([0.0, 0.0])?,
            value: 0.0,
            residual: 0.0,
        };
        let endpoint = Endpoint {
            key: IntersectionKey::Vertex(0),
            sample,
        };
        let (unique, count) = super::unique_endpoints([Some(endpoint), Some(endpoint)]);
        assert_eq!(count, 1);
        assert!(unique[0].is_some());
        Ok(())
    }
}
