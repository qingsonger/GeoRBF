//! Deterministic three-dimensional isosurface extraction.
//!
//! One immutable [`FittedField<3>`] is sampled on an explicit
//! original-coordinate box. The reference path uses a conforming Freudenthal
//! split into six tetrahedra per cell. The regular-grid path connects
//! crossings on cube faces, applies one shared bilinear asymptotic decision
//! to alternating-sign faces, and triangulates the resulting boundary loops.
//! Crossings are refined by fitted values, deduplicated by canonical grid
//! identities, and assigned analytic original-coordinate gradient normals.
//!
//! ```
//! use std::error::Error;
//! use std::num::NonZeroU32;
//!
//! use georbf::{
//!     FittedField, IsosurfaceMethod, IsosurfaceRequest, IsosurfaceSettings, Point,
//! };
//!
//! # fn extract(model: &FittedField<3>) -> Result<(), Box<dyn Error>> {
//! let cells = NonZeroU32::new(32).ok_or("cells")?;
//! let settings = IsosurfaceSettings::try_new(
//!     cells,
//!     cells,
//!     cells,
//!     NonZeroU32::new(64).ok_or("refinement iterations")?,
//!     1.0e-10,
//!     1.0e-9,
//! )?;
//! let request = IsosurfaceRequest::try_new(
//!     0.0,
//!     Point::try_new([-10.0, -10.0, -10.0])?,
//!     Point::try_new([10.0, 10.0, 10.0])?,
//!     IsosurfaceMethod::TopologyAwareMarchingCubes,
//!     settings,
//! )?;
//! let report = model.try_isosurface(&request)?;
//! for triangle in report.triangles() {
//!     assert_eq!(triangle.vertex_indices().len(), 3);
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
use crate::geometry::{Point, UnitDirection};
use crate::model::{FittedField, FittedFieldEvaluationError, FittedFieldOutput};
use crate::problem_ir::ExecutionOptions;

/// Grid-cell traversal used to extract a three-dimensional isosurface.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum IsosurfaceMethod {
    /// Split every cube into six conforming Freudenthal tetrahedra.
    MarchingSimplices,
    /// Connect cube-edge crossings through face-consistent asymptotic decisions.
    TopologyAwareMarchingCubes,
}

/// Explicit grid and bracket-refinement policy for three-dimensional isosurfaces.
#[derive(Clone, Copy, Debug, PartialEq)]
#[must_use]
pub struct IsosurfaceSettings {
    cells_x: NonZeroU32,
    cells_y: NonZeroU32,
    cells_z: NonZeroU32,
    refinement_iterations: NonZeroU32,
    value_tolerance: f64,
    coordinate_tolerance: f64,
}

impl IsosurfaceSettings {
    /// Constructs a finite deterministic grid and refinement policy.
    ///
    /// # Errors
    ///
    /// Both tolerances must be positive and finite.
    pub fn try_new(
        cells_x: NonZeroU32,
        cells_y: NonZeroU32,
        cells_z: NonZeroU32,
        refinement_iterations: NonZeroU32,
        value_tolerance: f64,
        coordinate_tolerance: f64,
    ) -> Result<Self, IsosurfaceSettingsError> {
        validate_tolerance(IsosurfaceTolerance::Value, value_tolerance)?;
        validate_tolerance(IsosurfaceTolerance::Coordinate, coordinate_tolerance)?;
        Ok(Self {
            cells_x,
            cells_y,
            cells_z,
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

    /// Returns the requested cell count along original-coordinate Z.
    #[must_use]
    pub const fn cells_z(self) -> NonZeroU32 {
        self.cells_z
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

/// Tolerance field rejected while constructing isosurface settings.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum IsosurfaceTolerance {
    /// Fitted value minus requested level.
    Value,
    /// Original-coordinate edge-bracket width.
    Coordinate,
}

impl fmt::Display for IsosurfaceTolerance {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Value => "value",
            Self::Coordinate => "coordinate",
        })
    }
}

/// Invalid three-dimensional isosurface settings.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum IsosurfaceSettingsError {
    /// A tolerance was zero, negative, infinite, or NaN.
    InvalidTolerance {
        /// Rejected tolerance field.
        tolerance: IsosurfaceTolerance,
        /// Rejected value.
        value: f64,
    },
}

impl fmt::Display for IsosurfaceSettingsError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidTolerance { tolerance, value } => write!(
                formatter,
                "isosurface {tolerance} tolerance must be positive and finite, got {value}"
            ),
        }
    }
}

impl Error for IsosurfaceSettingsError {}

/// Original-coordinate axis used by domain diagnostics.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum IsosurfaceDomainAxis {
    /// First coordinate.
    X,
    /// Second coordinate.
    Y,
    /// Third coordinate.
    Z,
}

impl fmt::Display for IsosurfaceDomainAxis {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::X => "x",
            Self::Y => "y",
            Self::Z => "z",
        })
    }
}

/// One target-level extraction request over a rectangular original-coordinate box.
#[derive(Clone, Copy, Debug, PartialEq)]
#[must_use]
pub struct IsosurfaceRequest {
    level: f64,
    lower: Point<3>,
    upper: Point<3>,
    method: IsosurfaceMethod,
    settings: IsosurfaceSettings,
}

impl IsosurfaceRequest {
    /// Constructs a validated three-dimensional isosurface request.
    ///
    /// # Errors
    ///
    /// The target must be finite. Every lower coordinate must be strictly
    /// smaller than its upper coordinate and every span must be representable.
    pub fn try_new(
        level: f64,
        lower: Point<3>,
        upper: Point<3>,
        method: IsosurfaceMethod,
        settings: IsosurfaceSettings,
    ) -> Result<Self, IsosurfaceRequestError> {
        if !level.is_finite() {
            return Err(IsosurfaceRequestError::NonFiniteLevel { level });
        }
        for (index, axis) in [
            IsosurfaceDomainAxis::X,
            IsosurfaceDomainAxis::Y,
            IsosurfaceDomainAxis::Z,
        ]
        .into_iter()
        .enumerate()
        {
            let lower_value = lower.components()[index];
            let upper_value = upper.components()[index];
            if lower_value >= upper_value {
                return Err(IsosurfaceRequestError::InvalidDomain {
                    axis,
                    lower: lower_value,
                    upper: upper_value,
                });
            }
            if !(upper_value - lower_value).is_finite() {
                return Err(IsosurfaceRequestError::UnrepresentableDomainSpan {
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

    /// Returns the inclusive lower original-coordinate corner.
    pub const fn lower(self) -> Point<3> {
        self.lower
    }

    /// Returns the inclusive upper original-coordinate corner.
    pub const fn upper(self) -> Point<3> {
        self.upper
    }

    /// Returns the selected marching method.
    #[must_use]
    pub const fn method(self) -> IsosurfaceMethod {
        self.method
    }

    /// Returns the explicit grid and refinement settings.
    pub const fn settings(self) -> IsosurfaceSettings {
        self.settings
    }
}

/// Invalid target level or rectangular isosurface domain.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum IsosurfaceRequestError {
    /// The target level was infinite or NaN.
    NonFiniteLevel {
        /// Rejected level.
        level: f64,
    },
    /// One domain interval was empty or reversed.
    InvalidDomain {
        /// Rejected axis.
        axis: IsosurfaceDomainAxis,
        /// Lower coordinate.
        lower: f64,
        /// Upper coordinate.
        upper: f64,
    },
    /// Subtracting two finite bounds did not produce a finite span.
    UnrepresentableDomainSpan {
        /// Rejected axis.
        axis: IsosurfaceDomainAxis,
        /// Lower coordinate.
        lower: f64,
        /// Upper coordinate.
        upper: f64,
    },
}

impl fmt::Display for IsosurfaceRequestError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonFiniteLevel { level } => {
                write!(
                    formatter,
                    "isosurface target level must be finite, got {level}"
                )
            }
            Self::InvalidDomain { axis, lower, upper } => write!(
                formatter,
                "isosurface {axis} domain must satisfy lower < upper, got [{lower}, {upper}]"
            ),
            Self::UnrepresentableDomainSpan { axis, lower, upper } => write!(
                formatter,
                "isosurface {axis} domain span is not representable for [{lower}, {upper}]"
            ),
        }
    }
}

impl Error for IsosurfaceRequestError {}

/// One deduplicated original-coordinate isosurface vertex.
#[derive(Clone, Copy, Debug, PartialEq)]
#[must_use]
pub struct IsosurfaceVertex {
    point: Point<3>,
    value: f64,
    residual: f64,
    normal: UnitDirection<3>,
}

impl IsosurfaceVertex {
    /// Returns the original-coordinate point.
    pub const fn point(self) -> Point<3> {
        self.point
    }

    /// Returns the fitted value evaluated with the reported gradient.
    #[must_use]
    pub const fn value(self) -> f64 {
        self.value
    }

    /// Returns fitted value minus requested level.
    #[must_use]
    pub const fn residual(self) -> f64 {
        self.residual
    }

    /// Returns the analytic unit gradient direction in original coordinates.
    pub const fn normal(self) -> UnitDirection<3> {
        self.normal
    }
}

/// One consistently oriented indexed triangle.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[must_use]
pub struct IsosurfaceTriangle {
    vertex_indices: [usize; 3],
}

impl IsosurfaceTriangle {
    /// Returns the three counter-clockwise vertex indices viewed from the
    /// positive fitted-gradient side.
    #[must_use]
    pub const fn vertex_indices(self) -> [usize; 3] {
        self.vertex_indices
    }
}

/// Requested-box face touched by an open mesh boundary.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum IsosurfaceBoundaryFace {
    /// Minimum X face.
    XMin,
    /// Maximum X face.
    XMax,
    /// Minimum Y face.
    YMin,
    /// Maximum Y face.
    YMax,
    /// Minimum Z face.
    ZMin,
    /// Maximum Z face.
    ZMax,
}

/// One connected triangle component.
#[derive(Clone, Debug, Eq, PartialEq)]
#[must_use]
pub struct IsosurfaceComponent {
    triangle_indices: Vec<usize>,
    closed: bool,
    boundary_faces: Vec<IsosurfaceBoundaryFace>,
}

impl IsosurfaceComponent {
    /// Borrows component triangle indices in deterministic order.
    #[must_use]
    pub fn triangle_indices(&self) -> &[usize] {
        &self.triangle_indices
    }

    /// Returns whether every component edge has incidence two.
    #[must_use]
    pub const fn is_closed(&self) -> bool {
        self.closed
    }

    /// Borrows sorted requested-box faces touched by boundary edges.
    #[must_use]
    pub fn boundary_faces(&self) -> &[IsosurfaceBoundaryFace] {
        &self.boundary_faces
    }
}

/// Face-local pairing selected for four alternating-sign crossings.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum IsosurfaceFacePairing {
    /// Connect edge pairs `(0, 1)` and `(2, 3)`.
    AdjacentForward,
    /// Connect edge pairs `(0, 3)` and `(1, 2)`.
    AdjacentBackward,
}

/// One topology-aware cube-face ambiguity decision.
#[derive(Clone, Copy, Debug, PartialEq)]
#[must_use]
pub struct IsosurfaceAmbiguousFace {
    cell: [u32; 3],
    face: IsosurfaceBoundaryFace,
    normalized_decider: f64,
    pairing: IsosurfaceFacePairing,
}

impl IsosurfaceAmbiguousFace {
    /// Returns the zero-based containing cell coordinate.
    #[must_use]
    pub const fn cell(self) -> [u32; 3] {
        self.cell
    }

    /// Returns the decided cell face.
    #[must_use]
    pub const fn face(self) -> IsosurfaceBoundaryFace {
        self.face
    }

    /// Returns the scale-normalized bilinear asymptotic decision value.
    #[must_use]
    pub const fn normalized_decider(self) -> f64 {
        self.normalized_decider
    }

    /// Returns the selected face-edge pairing.
    #[must_use]
    pub const fn pairing(self) -> IsosurfaceFacePairing {
        self.pairing
    }
}

/// Deterministic extraction and topology evidence.
#[derive(Clone, Debug, PartialEq)]
#[must_use]
pub struct IsosurfaceDiagnostics {
    method: IsosurfaceMethod,
    cells: [u32; 3],
    evaluations: usize,
    raw_triangles: usize,
    unique_triangles: usize,
    duplicate_triangles: usize,
    deduplicated_vertices: usize,
    ambiguous_faces: Vec<IsosurfaceAmbiguousFace>,
    boundary_edges: usize,
    open_components: usize,
    closed_components: usize,
}

impl IsosurfaceDiagnostics {
    /// Returns the selected marching method.
    #[must_use]
    pub const fn method(&self) -> IsosurfaceMethod {
        self.method
    }

    /// Returns requested X, Y, and Z cell counts.
    #[must_use]
    pub const fn cells(&self) -> [u32; 3] {
        self.cells
    }

    /// Returns actual fitted-field value or value-gradient evaluations.
    #[must_use]
    pub const fn evaluations(&self) -> usize {
        self.evaluations
    }

    /// Returns triangles emitted before exact identity deduplication.
    #[must_use]
    pub const fn raw_triangles(&self) -> usize {
        self.raw_triangles
    }

    /// Returns unique accepted triangles.
    #[must_use]
    pub const fn unique_triangles(&self) -> usize {
        self.unique_triangles
    }

    /// Returns exactly repeated undirected triangles removed.
    #[must_use]
    pub const fn duplicate_triangles(&self) -> usize {
        self.duplicate_triangles
    }

    /// Returns unique canonical grid intersections.
    #[must_use]
    pub const fn deduplicated_vertices(&self) -> usize {
        self.deduplicated_vertices
    }

    /// Borrows shared-face ambiguity decisions.
    pub fn ambiguous_faces(&self) -> &[IsosurfaceAmbiguousFace] {
        &self.ambiguous_faces
    }

    /// Returns mesh edges with incidence one.
    #[must_use]
    pub const fn boundary_edges(&self) -> usize {
        self.boundary_edges
    }

    /// Returns connected components with at least one boundary edge.
    #[must_use]
    pub const fn open_components(&self) -> usize {
        self.open_components
    }

    /// Returns connected components with edge incidence exactly two.
    #[must_use]
    pub const fn closed_components(&self) -> usize {
        self.closed_components
    }
}

/// Complete deterministic indexed isosurface report.
#[derive(Clone, Debug, PartialEq)]
#[must_use]
pub struct IsosurfaceReport {
    request: IsosurfaceRequest,
    vertices: Vec<IsosurfaceVertex>,
    triangles: Vec<IsosurfaceTriangle>,
    components: Vec<IsosurfaceComponent>,
    diagnostics: IsosurfaceDiagnostics,
}

impl IsosurfaceReport {
    /// Returns the validated request.
    pub const fn request(&self) -> IsosurfaceRequest {
        self.request
    }

    /// Borrows deduplicated original-coordinate vertices.
    pub fn vertices(&self) -> &[IsosurfaceVertex] {
        &self.vertices
    }

    /// Borrows consistently oriented triangles.
    pub fn triangles(&self) -> &[IsosurfaceTriangle] {
        &self.triangles
    }

    /// Borrows deterministic connected components.
    pub fn components(&self) -> &[IsosurfaceComponent] {
        &self.components
    }

    /// Borrows extraction and topology evidence.
    pub const fn diagnostics(&self) -> &IsosurfaceDiagnostics {
        &self.diagnostics
    }
}

/// Logical storage whose checked reservation failed.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum IsosurfaceStorage {
    /// Sampled grid nodes.
    Nodes,
    /// Raw triangles and their endpoint records.
    RawTriangles,
    /// Ambiguous face records.
    AmbiguousFaces,
    /// Per-cell face segments.
    FaceSegments,
    /// Per-cell loop vertices.
    LoopVertices,
    /// Global endpoint records.
    EndpointRecords,
    /// Canonical vertex keys.
    VertexKeys,
    /// Report vertices.
    Vertices,
    /// Indexed triangles.
    Triangles,
    /// Mesh edge incidence.
    EdgeIncidence,
    /// Triangle adjacency.
    TriangleAdjacency,
    /// Component traversal state.
    ComponentTraversal,
    /// Component triangle indices.
    ComponentTriangles,
    /// Component boundary faces.
    ComponentBoundaryFaces,
    /// Components.
    Components,
}

impl fmt::Display for IsosurfaceStorage {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Nodes => "grid nodes",
            Self::RawTriangles => "raw triangles",
            Self::AmbiguousFaces => "ambiguous faces",
            Self::FaceSegments => "face segments",
            Self::LoopVertices => "loop vertices",
            Self::EndpointRecords => "endpoint records",
            Self::VertexKeys => "vertex keys",
            Self::Vertices => "vertices",
            Self::Triangles => "triangles",
            Self::EdgeIncidence => "edge incidence",
            Self::TriangleAdjacency => "triangle adjacency",
            Self::ComponentTraversal => "component traversal",
            Self::ComponentTriangles => "component triangles",
            Self::ComponentBoundaryFaces => "component boundary faces",
            Self::Components => "components",
        })
    }
}

/// Structured three-dimensional isosurface extraction failure.
#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
pub enum IsosurfaceError {
    /// Grid, triangle, or refinement work arithmetic overflowed.
    WorkBudgetOverflow {
        /// Requested cell counts.
        cells: [u32; 3],
        /// Requested iterations per crossed edge.
        refinement_iterations: u32,
    },
    /// One logical vector could not reserve its requested capacity.
    AllocationFailed {
        /// Logical storage.
        storage: IsosurfaceStorage,
        /// Requested element capacity.
        requested: usize,
    },
    /// Reusable fitted value-and-gradient storage could not be prepared.
    Preparation {
        /// Fitted-field failure.
        source: FittedFieldEvaluationError<3>,
    },
    /// The fitted field could not be evaluated at one point.
    Evaluation {
        /// Original-coordinate query.
        point: Point<3>,
        /// Fitted-field failure.
        source: FittedFieldEvaluationError<3>,
    },
    /// A derived coordinate was not finite.
    NonFiniteCoordinate {
        /// Rejected components.
        components: [f64; 3],
    },
    /// A finite fitted value minus finite target was not representable.
    NonFiniteResidual {
        /// Original-coordinate query.
        point: Point<3>,
        /// Finite fitted value.
        value: f64,
        /// Finite target level.
        level: f64,
    },
    /// Both endpoints of one sampled edge were exactly on the target.
    DegenerateGridEdge {
        /// First endpoint.
        first: Point<3>,
        /// Second endpoint.
        second: Point<3>,
    },
    /// One crossed edge exhausted its explicit bisection limit.
    RefinementLimitReached {
        /// Remaining first endpoint.
        first: Point<3>,
        /// Remaining second endpoint.
        second: Point<3>,
        /// Smallest endpoint absolute residual.
        absolute_residual: f64,
    },
    /// A tetrahedron had an unsupported exact-vertex intersection pattern.
    DegenerateSimplexTopology {
        /// Zero-based grid cell.
        cell: [u32; 3],
        /// Zero-based simplex within the cell.
        simplex: u8,
        /// Unique edge intersections.
        intersections: usize,
    },
    /// A cube face had an unsupported exact-vertex intersection pattern.
    DegenerateFaceTopology {
        /// Zero-based grid cell.
        cell: [u32; 3],
        /// Cell face.
        face: IsosurfaceBoundaryFace,
        /// Unique edge intersections.
        intersections: usize,
    },
    /// A cube boundary crossing graph was not a disjoint union of cycles.
    DegenerateCellTopology {
        /// Zero-based grid cell.
        cell: [u32; 3],
        /// Rejected vertex degree or unclosed traversal length.
        evidence: usize,
    },
    /// A triangle collapsed after canonical vertex deduplication.
    CollapsedTriangle {
        /// Rejected indices.
        vertex_indices: [usize; 3],
    },
    /// An analytic gradient was zero at a retained mesh vertex.
    ZeroGradientNormal {
        /// Rejected original-coordinate point.
        point: Point<3>,
    },
    /// More than two triangles met along one mesh edge.
    NonManifoldEdge {
        /// First edge vertex.
        first: Point<3>,
        /// Second edge vertex.
        second: Point<3>,
        /// Observed incidence.
        incidence: usize,
    },
    /// One incidence-one mesh edge did not lie on the requested box.
    InteriorBoundaryEdge {
        /// First edge vertex.
        first: Point<3>,
        /// Second edge vertex.
        second: Point<3>,
    },
    /// Adjacent triangles traverse their shared edge in the same direction.
    InconsistentWinding {
        /// First edge vertex index.
        first: usize,
        /// Second edge vertex index.
        second: usize,
    },
    /// Caller execution policy or cancellation failure.
    Execution(ExecutionError),
}

impl From<ExecutionError> for IsosurfaceError {
    fn from(source: ExecutionError) -> Self {
        Self::Execution(source)
    }
}

impl fmt::Display for IsosurfaceError {
    #[allow(clippy::too_many_lines)]
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WorkBudgetOverflow {
                cells,
                refinement_iterations,
            } => write!(
                formatter,
                "isosurface work budget is not representable for {} by {} by {} cells and {refinement_iterations} refinement iterations",
                cells[0], cells[1], cells[2]
            ),
            Self::AllocationFailed { storage, requested } => write!(
                formatter,
                "isosurface {storage} could not reserve {requested} elements"
            ),
            Self::Preparation { source } => {
                write!(
                    formatter,
                    "isosurface evaluation preparation failed: {source}"
                )
            }
            Self::Evaluation { point, source } => write!(
                formatter,
                "isosurface fitted-field evaluation failed at {:?}: {source}",
                point.components()
            ),
            Self::NonFiniteCoordinate { components } => {
                write!(
                    formatter,
                    "isosurface derived coordinate is not finite: {components:?}"
                )
            }
            Self::NonFiniteResidual {
                point,
                value,
                level,
            } => write!(
                formatter,
                "isosurface residual is not finite at {:?}: {value} - {level}",
                point.components()
            ),
            Self::DegenerateGridEdge { first, second } => write!(
                formatter,
                "isosurface sampled edge from {:?} to {:?} has both endpoints exactly on the target",
                first.components(),
                second.components()
            ),
            Self::RefinementLimitReached {
                first,
                second,
                absolute_residual,
            } => write!(
                formatter,
                "isosurface edge refinement exhausted between {:?} and {:?} with absolute residual {absolute_residual}",
                first.components(),
                second.components()
            ),
            Self::DegenerateSimplexTopology {
                cell,
                simplex,
                intersections,
            } => write!(
                formatter,
                "isosurface cell {cell:?} simplex {simplex} has {intersections} edge intersections"
            ),
            Self::DegenerateFaceTopology {
                cell,
                face,
                intersections,
            } => write!(
                formatter,
                "isosurface cell {cell:?} face {face:?} has {intersections} edge intersections"
            ),
            Self::DegenerateCellTopology { cell, evidence } => write!(
                formatter,
                "isosurface cell {cell:?} has a non-cyclic boundary graph (evidence {evidence})"
            ),
            Self::CollapsedTriangle { vertex_indices } => {
                write!(
                    formatter,
                    "isosurface triangle collapsed at indices {vertex_indices:?}"
                )
            }
            Self::ZeroGradientNormal { point } => write!(
                formatter,
                "isosurface gradient is zero at retained vertex {:?}",
                point.components()
            ),
            Self::NonManifoldEdge {
                first,
                second,
                incidence,
            } => write!(
                formatter,
                "isosurface edge {:?}--{:?} has non-manifold incidence {incidence}",
                first.components(),
                second.components()
            ),
            Self::InteriorBoundaryEdge { first, second } => write!(
                formatter,
                "isosurface boundary edge {:?}--{:?} lies inside the requested box",
                first.components(),
                second.components()
            ),
            Self::InconsistentWinding { first, second } => write!(
                formatter,
                "isosurface adjacent triangles traverse shared edge ({first}, {second}) in the same direction"
            ),
            Self::Execution(source) => source.fmt(formatter),
        }
    }
}

impl Error for IsosurfaceError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Preparation { source } | Self::Evaluation { source, .. } => Some(source),
            Self::Execution(source) => Some(source),
            _ => None,
        }
    }
}

impl FittedField<3> {
    /// Extracts a deterministic indexed three-dimensional isosurface.
    ///
    /// # Errors
    ///
    /// Returns structured preparation, evaluation, allocation, arithmetic,
    /// refinement, degeneracy, normal, or topology failures. No partial mesh
    /// is returned.
    pub fn try_isosurface(
        &self,
        request: &IsosurfaceRequest,
    ) -> Result<IsosurfaceReport, IsosurfaceError> {
        self.try_isosurface_with_control(
            request,
            ExecutionOptions::default(),
            ExecutionControl::default(),
        )
    }

    /// Extracts an isosurface with explicit serial execution controls.
    ///
    /// # Errors
    ///
    /// Returns the same failures as [`Self::try_isosurface`], plus structured
    /// execution-policy and cancellation failures.
    pub fn try_isosurface_with_control(
        &self,
        request: &IsosurfaceRequest,
        execution: ExecutionOptions,
        control: ExecutionControl<'_>,
    ) -> Result<IsosurfaceReport, IsosurfaceError> {
        let work = GridWork::try_new(request)?;
        let mut progress = ProgressTracker::try_new(
            control,
            ExecutionOperation::IsosurfaceExtraction,
            execution,
            work.maximum_evaluations,
        )?;
        let scratch_result = self
            .try_evaluation_scratch(FittedFieldOutput::Gradient)
            .map_err(|source| IsosurfaceError::Preparation { source });
        let mut scratch = progress.observe_result(ExecutionStage::Started, scratch_result)?;
        let mut evaluations = 0_usize;
        let extraction = {
            let mut evaluate_value = |point: Point<3>| {
                progress.checkpoint(ExecutionStage::IsosurfaceEvaluation)?;
                let result = self
                    .try_value_with_scratch(point, &mut scratch)
                    .map_err(|source| IsosurfaceError::Evaluation { point, source })
                    .and_then(|value| sample(request, point, value));
                let result = progress.finish_work(ExecutionStage::IsosurfaceEvaluation, result);
                if result.is_ok() {
                    evaluations = evaluations.saturating_add(1);
                }
                result
            };
            extract_raw(request, work, &mut evaluate_value)?
        };
        let mut evaluate_vertex = |point: Point<3>| {
            progress.checkpoint(ExecutionStage::IsosurfaceEvaluation)?;
            let result = self
                .try_evaluate_with_scratch(point, &mut scratch)
                .map_err(|source| IsosurfaceError::Evaluation { point, source });
            let result = progress.finish_work(ExecutionStage::IsosurfaceEvaluation, result);
            if result.is_ok() {
                evaluations = evaluations.saturating_add(1);
            }
            result
        };
        let mut report = build_report(request, extraction, &mut evaluate_vertex)?;
        report.diagnostics.evaluations = evaluations;
        progress.complete()?;
        Ok(report)
    }
}

#[derive(Clone, Copy, Debug)]
struct Sample {
    point: Point<3>,
    residual: f64,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum IntersectionKey {
    Vertex(usize),
    Edge { first: usize, second: usize },
}

#[derive(Clone, Copy, Debug)]
struct Endpoint {
    key: IntersectionKey,
    sample: Sample,
}

#[derive(Clone, Copy, Debug)]
struct RawTriangle {
    vertices: [Endpoint; 3],
}

#[derive(Clone, Copy, Debug)]
struct FaceSegment {
    first: Endpoint,
    second: Endpoint,
}

struct RawExtraction {
    triangles: Vec<RawTriangle>,
    ambiguous_faces: Vec<IsosurfaceAmbiguousFace>,
}

#[derive(Clone, Copy, Debug)]
struct GridWork {
    columns: usize,
    rows: usize,
    layers: usize,
    node_columns: usize,
    node_rows: usize,
    node_count: usize,
    cell_count: usize,
    raw_triangle_capacity: usize,
    maximum_evaluations: usize,
}

impl GridWork {
    fn try_new(request: &IsosurfaceRequest) -> Result<Self, IsosurfaceError> {
        let columns =
            usize::try_from(request.settings.cells_x.get()).map_err(|_| work_overflow(request))?;
        let rows =
            usize::try_from(request.settings.cells_y.get()).map_err(|_| work_overflow(request))?;
        let layers =
            usize::try_from(request.settings.cells_z.get()).map_err(|_| work_overflow(request))?;
        let node_columns = columns
            .checked_add(1)
            .ok_or_else(|| work_overflow(request))?;
        let node_rows = rows.checked_add(1).ok_or_else(|| work_overflow(request))?;
        let node_layers = layers
            .checked_add(1)
            .ok_or_else(|| work_overflow(request))?;
        let node_count = node_columns
            .checked_mul(node_rows)
            .and_then(|count| count.checked_mul(node_layers))
            .ok_or_else(|| work_overflow(request))?;
        let cell_count = columns
            .checked_mul(rows)
            .and_then(|count| count.checked_mul(layers))
            .ok_or_else(|| work_overflow(request))?;
        let raw_triangle_capacity = cell_count
            .checked_mul(12)
            .ok_or_else(|| work_overflow(request))?;
        let edge_queries_per_cell = match request.method {
            IsosurfaceMethod::MarchingSimplices => 36_usize,
            IsosurfaceMethod::TopologyAwareMarchingCubes => 12_usize,
        };
        let iterations = usize::try_from(request.settings.refinement_iterations.get())
            .map_err(|_| work_overflow(request))?;
        let maximum_evaluations = cell_count
            .checked_mul(edge_queries_per_cell)
            .and_then(|count| count.checked_mul(iterations))
            .and_then(|count| count.checked_add(raw_triangle_capacity.checked_mul(3)?))
            .and_then(|count| count.checked_add(node_count))
            .ok_or_else(|| work_overflow(request))?;
        Ok(Self {
            columns,
            rows,
            layers,
            node_columns,
            node_rows,
            node_count,
            cell_count,
            raw_triangle_capacity,
            maximum_evaluations,
        })
    }
}

fn extract_raw(
    request: &IsosurfaceRequest,
    work: GridWork,
    evaluate: &mut impl FnMut(Point<3>) -> Result<Sample, IsosurfaceError>,
) -> Result<RawExtraction, IsosurfaceError> {
    let mut nodes = try_vec(work.node_count, IsosurfaceStorage::Nodes)?;
    for layer in 0..=work.layers {
        for row in 0..=work.rows {
            for column in 0..=work.columns {
                let components = [
                    interpolate(
                        request.lower.components()[0],
                        request.upper.components()[0],
                        u32::try_from(column).map_err(|_| work_overflow(request))?,
                        request.settings.cells_x.get(),
                    ),
                    interpolate(
                        request.lower.components()[1],
                        request.upper.components()[1],
                        u32::try_from(row).map_err(|_| work_overflow(request))?,
                        request.settings.cells_y.get(),
                    ),
                    interpolate(
                        request.lower.components()[2],
                        request.upper.components()[2],
                        u32::try_from(layer).map_err(|_| work_overflow(request))?,
                        request.settings.cells_z.get(),
                    ),
                ];
                let point = Point::try_new(components)
                    .map_err(|_| IsosurfaceError::NonFiniteCoordinate { components })?;
                nodes.push(evaluate(point)?);
            }
        }
    }

    let mut triangles = try_vec(work.raw_triangle_capacity, IsosurfaceStorage::RawTriangles)?;
    let ambiguous_capacity = work
        .cell_count
        .checked_mul(6)
        .ok_or_else(|| work_overflow(request))?;
    let mut ambiguous_faces = try_vec(ambiguous_capacity, IsosurfaceStorage::AmbiguousFaces)?;
    for layer in 0..work.layers {
        for row in 0..work.rows {
            for column in 0..work.columns {
                let indices = cube_indices(column, row, layer, work);
                let corners = indices.map(|index| nodes[index]);
                let cell = [
                    u32::try_from(column).map_err(|_| work_overflow(request))?,
                    u32::try_from(row).map_err(|_| work_overflow(request))?,
                    u32::try_from(layer).map_err(|_| work_overflow(request))?,
                ];
                match request.method {
                    IsosurfaceMethod::MarchingSimplices => {
                        emit_simplices(request, cell, indices, corners, evaluate, &mut triangles)?;
                    }
                    IsosurfaceMethod::TopologyAwareMarchingCubes => {
                        emit_cube(
                            request,
                            cell,
                            indices,
                            corners,
                            evaluate,
                            &mut triangles,
                            &mut ambiguous_faces,
                        )?;
                    }
                }
            }
        }
    }
    Ok(RawExtraction {
        triangles,
        ambiguous_faces,
    })
}

const TETRAHEDRA: [[usize; 4]; 6] = [
    [0, 1, 3, 7],
    [0, 3, 2, 7],
    [0, 2, 6, 7],
    [0, 6, 4, 7],
    [0, 4, 5, 7],
    [0, 5, 1, 7],
];

const TETRA_EDGES: [[usize; 2]; 6] = [[0, 1], [0, 2], [0, 3], [1, 2], [1, 3], [2, 3]];

fn emit_simplices(
    request: &IsosurfaceRequest,
    cell: [u32; 3],
    indices: [usize; 8],
    corners: [Sample; 8],
    evaluate: &mut impl FnMut(Point<3>) -> Result<Sample, IsosurfaceError>,
    triangles: &mut Vec<RawTriangle>,
) -> Result<(), IsosurfaceError> {
    for (simplex, tetra) in TETRAHEDRA.into_iter().enumerate() {
        let tetra_indices = tetra.map(|corner| indices[corner]);
        let tetra_corners = tetra.map(|corner| corners[corner]);
        let mut endpoints = [None; 6];
        for (edge_index, [first, second]) in TETRA_EDGES.into_iter().enumerate() {
            endpoints[edge_index] = edge_intersection(
                request,
                tetra_corners[first],
                tetra_corners[second],
                tetra_indices[first],
                tetra_indices[second],
                evaluate,
            )?;
        }
        let mut unique = [None; 4];
        let count = collect_unique(endpoints, &mut unique);
        match count {
            0 => {}
            3 => push_triangle(
                triangles,
                RawTriangle {
                    vertices: [
                        unique[0].ok_or_else(|| work_overflow(request))?,
                        unique[1].ok_or_else(|| work_overflow(request))?,
                        unique[2].ok_or_else(|| work_overflow(request))?,
                    ],
                },
            )?,
            4 => emit_tetra_quad(request, tetra_corners, &endpoints, triangles)?,
            _ => {
                return Err(IsosurfaceError::DegenerateSimplexTopology {
                    cell,
                    simplex: u8::try_from(simplex).unwrap_or(u8::MAX),
                    intersections: count,
                });
            }
        }
    }
    Ok(())
}

fn emit_tetra_quad(
    request: &IsosurfaceRequest,
    corners: [Sample; 4],
    endpoints: &[Option<Endpoint>; 6],
    triangles: &mut Vec<RawTriangle>,
) -> Result<(), IsosurfaceError> {
    if corners.iter().any(|corner| corner.residual == 0.0) {
        return Err(IsosurfaceError::DegenerateGridEdge {
            first: corners[0].point,
            second: corners[0].point,
        });
    }
    let mut positive = [usize::MAX; 2];
    let mut negative = [usize::MAX; 2];
    let mut positive_count = 0;
    let mut negative_count = 0;
    for (index, corner) in corners.into_iter().enumerate() {
        if corner.residual.is_sign_positive() {
            if positive_count < 2 {
                positive[positive_count] = index;
            }
            positive_count += 1;
        } else {
            if negative_count < 2 {
                negative[negative_count] = index;
            }
            negative_count += 1;
        }
    }
    if positive_count != 2 || negative_count != 2 {
        return Err(work_overflow(request));
    }
    let endpoint = |first: usize, second: usize| -> Result<Endpoint, IsosurfaceError> {
        let edge = TETRA_EDGES
            .iter()
            .position(|pair| {
                (pair[0] == first && pair[1] == second) || (pair[0] == second && pair[1] == first)
            })
            .ok_or_else(|| work_overflow(request))?;
        endpoints[edge].ok_or_else(|| work_overflow(request))
    };
    let a = endpoint(positive[0], negative[0])?;
    let b = endpoint(positive[0], negative[1])?;
    let c = endpoint(positive[1], negative[0])?;
    let d = endpoint(positive[1], negative[1])?;
    push_triangle(
        triangles,
        RawTriangle {
            vertices: [a, b, d],
        },
    )?;
    push_triangle(
        triangles,
        RawTriangle {
            vertices: [a, d, c],
        },
    )
}

const CUBE_EDGES: [[usize; 2]; 12] = [
    [0, 1],
    [1, 3],
    [2, 3],
    [0, 2],
    [4, 5],
    [5, 7],
    [6, 7],
    [4, 6],
    [0, 4],
    [1, 5],
    [2, 6],
    [3, 7],
];

const CUBE_FACES: [([usize; 4], [usize; 4], IsosurfaceBoundaryFace); 6] = [
    ([0, 1, 3, 2], [0, 1, 2, 3], IsosurfaceBoundaryFace::ZMin),
    ([4, 5, 7, 6], [4, 5, 6, 7], IsosurfaceBoundaryFace::ZMax),
    ([0, 1, 5, 4], [0, 9, 4, 8], IsosurfaceBoundaryFace::YMin),
    ([2, 3, 7, 6], [2, 11, 6, 10], IsosurfaceBoundaryFace::YMax),
    ([0, 2, 6, 4], [3, 10, 7, 8], IsosurfaceBoundaryFace::XMin),
    ([1, 3, 7, 5], [1, 11, 5, 9], IsosurfaceBoundaryFace::XMax),
];

fn emit_cube(
    request: &IsosurfaceRequest,
    cell: [u32; 3],
    indices: [usize; 8],
    corners: [Sample; 8],
    evaluate: &mut impl FnMut(Point<3>) -> Result<Sample, IsosurfaceError>,
    triangles: &mut Vec<RawTriangle>,
    ambiguous_faces: &mut Vec<IsosurfaceAmbiguousFace>,
) -> Result<(), IsosurfaceError> {
    let mut edge_endpoints = [None; 12];
    for (edge, [first, second]) in CUBE_EDGES.into_iter().enumerate() {
        edge_endpoints[edge] = edge_intersection(
            request,
            corners[first],
            corners[second],
            indices[first],
            indices[second],
            evaluate,
        )?;
    }
    let mut segments = try_vec(12, IsosurfaceStorage::FaceSegments)?;
    for (face_corners, face_edges, face) in CUBE_FACES {
        emit_face(
            request,
            cell,
            face,
            face_corners.map(|corner| corners[corner]),
            face_edges.map(|edge| edge_endpoints[edge]),
            &mut segments,
            ambiguous_faces,
        )?;
    }
    triangulate_face_loops(cell, &segments, triangles)
}

fn emit_face(
    request: &IsosurfaceRequest,
    cell: [u32; 3],
    face: IsosurfaceBoundaryFace,
    corners: [Sample; 4],
    edges: [Option<Endpoint>; 4],
    segments: &mut Vec<FaceSegment>,
    ambiguous_faces: &mut Vec<IsosurfaceAmbiguousFace>,
) -> Result<(), IsosurfaceError> {
    let mut unique = [None; 4];
    let count = collect_unique(edges, &mut unique);
    match count {
        0 => Ok(()),
        2 => try_push(
            segments,
            FaceSegment {
                first: unique[0].ok_or_else(|| work_overflow(request))?,
                second: unique[1].ok_or_else(|| work_overflow(request))?,
            },
            IsosurfaceStorage::FaceSegments,
        ),
        4 => {
            if corners.iter().any(|corner| corner.residual == 0.0) {
                return Err(IsosurfaceError::DegenerateFaceTopology {
                    cell,
                    face,
                    intersections: count,
                });
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
                return Err(IsosurfaceError::DegenerateFaceTopology {
                    cell,
                    face,
                    intersections: count,
                });
            }
            let normalized_decider = face_decider(corners);
            let positive_connected = normalized_decider >= 0.0;
            let pairing = if (pattern == 0b0101 && positive_connected)
                || (pattern == 0b1010 && !positive_connected)
            {
                IsosurfaceFacePairing::AdjacentForward
            } else {
                IsosurfaceFacePairing::AdjacentBackward
            };
            let endpoint = |index: usize| {
                edges[index].ok_or(IsosurfaceError::DegenerateFaceTopology {
                    cell,
                    face,
                    intersections: count,
                })
            };
            let pairs = match pairing {
                IsosurfaceFacePairing::AdjacentForward => [(0, 1), (2, 3)],
                IsosurfaceFacePairing::AdjacentBackward => [(0, 3), (1, 2)],
            };
            for (first, second) in pairs {
                try_push(
                    segments,
                    FaceSegment {
                        first: endpoint(first)?,
                        second: endpoint(second)?,
                    },
                    IsosurfaceStorage::FaceSegments,
                )?;
            }
            try_push(
                ambiguous_faces,
                IsosurfaceAmbiguousFace {
                    cell,
                    face,
                    normalized_decider,
                    pairing,
                },
                IsosurfaceStorage::AmbiguousFaces,
            )
        }
        _ => Err(IsosurfaceError::DegenerateFaceTopology {
            cell,
            face,
            intersections: count,
        }),
    }
}

fn triangulate_face_loops(
    cell: [u32; 3],
    segments: &[FaceSegment],
    triangles: &mut Vec<RawTriangle>,
) -> Result<(), IsosurfaceError> {
    let mut visited = [false; 12];
    let mut loop_count = 0_usize;
    for start_segment in 0..segments.len() {
        if visited[start_segment] {
            continue;
        }
        loop_count += 1;
        if loop_count > 1 {
            // Multiple disjoint boundary loops leave the cube-interior
            // connection underdetermined by face samples alone. A finite
            // center sample is not a general trilinear-topology proof.
            return Err(IsosurfaceError::DegenerateCellTopology {
                cell,
                evidence: loop_count,
            });
        }
        let mut loop_vertices = try_vec(12, IsosurfaceStorage::LoopVertices)?;
        let start = segments[start_segment].first;
        let mut current = start;
        let mut segment_index = start_segment;
        loop {
            if visited[segment_index] {
                return Err(IsosurfaceError::DegenerateCellTopology {
                    cell,
                    evidence: loop_vertices.len(),
                });
            }
            visited[segment_index] = true;
            try_push(&mut loop_vertices, current, IsosurfaceStorage::LoopVertices)?;
            let segment = segments[segment_index];
            let next = if segment.first.key == current.key {
                segment.second
            } else if segment.second.key == current.key {
                segment.first
            } else {
                return Err(IsosurfaceError::DegenerateCellTopology {
                    cell,
                    evidence: loop_vertices.len(),
                });
            };
            if next.key == start.key {
                break;
            }
            let mut matches = segments.iter().enumerate().filter(|(index, candidate)| {
                !visited[*index]
                    && (candidate.first.key == next.key || candidate.second.key == next.key)
            });
            let Some((next_segment, _)) = matches.next() else {
                return Err(IsosurfaceError::DegenerateCellTopology {
                    cell,
                    evidence: loop_vertices.len(),
                });
            };
            if matches.next().is_some() {
                return Err(IsosurfaceError::DegenerateCellTopology { cell, evidence: 3 });
            }
            current = next;
            segment_index = next_segment;
        }
        if loop_vertices.len() < 3 {
            return Err(IsosurfaceError::DegenerateCellTopology {
                cell,
                evidence: loop_vertices.len(),
            });
        }
        canonicalize_loop(&mut loop_vertices);
        for index in 1..loop_vertices.len() - 1 {
            push_triangle(
                triangles,
                RawTriangle {
                    vertices: [
                        loop_vertices[0],
                        loop_vertices[index],
                        loop_vertices[index + 1],
                    ],
                },
            )?;
        }
    }
    Ok(())
}

fn canonicalize_loop(vertices: &mut [Endpoint]) {
    let start = vertices
        .iter()
        .enumerate()
        .min_by_key(|(_, vertex)| vertex.key)
        .map_or(0, |(index, _)| index);
    vertices.rotate_left(start);
    if vertices.len() > 2 && vertices[vertices.len() - 1].key < vertices[1].key {
        vertices[1..].reverse();
    }
}

fn edge_intersection(
    request: &IsosurfaceRequest,
    first: Sample,
    second: Sample,
    first_index: usize,
    second_index: usize,
    evaluate: &mut impl FnMut(Point<3>) -> Result<Sample, IsosurfaceError>,
) -> Result<Option<Endpoint>, IsosurfaceError> {
    if first.residual == 0.0 && second.residual == 0.0 {
        return Err(IsosurfaceError::DegenerateGridEdge {
            first: first.point,
            second: second.point,
        });
    }
    if first.residual == 0.0 {
        return Ok(Some(Endpoint {
            key: IntersectionKey::Vertex(first_index),
            sample: first,
        }));
    }
    if second.residual == 0.0 {
        return Ok(Some(Endpoint {
            key: IntersectionKey::Vertex(second_index),
            sample: second,
        }));
    }
    if same_sign(first.residual, second.residual) {
        return Ok(None);
    }
    let key = if first_index < second_index {
        IntersectionKey::Edge {
            first: first_index,
            second: second_index,
        }
    } else {
        IntersectionKey::Edge {
            first: second_index,
            second: first_index,
        }
    };
    let first_absolute = first.residual.abs();
    let second_absolute = second.residual.abs();
    if first_absolute <= request.settings.value_tolerance
        || second_absolute <= request.settings.value_tolerance
    {
        return Ok(Some(Endpoint {
            key,
            sample: if first_absolute <= second_absolute {
                first
            } else {
                second
            },
        }));
    }
    Ok(Some(Endpoint {
        key,
        sample: refine_edge(request, first, second, evaluate)?,
    }))
}

fn refine_edge(
    request: &IsosurfaceRequest,
    mut first: Sample,
    mut second: Sample,
    evaluate: &mut impl FnMut(Point<3>) -> Result<Sample, IsosurfaceError>,
) -> Result<Sample, IsosurfaceError> {
    for _ in 0..request.settings.refinement_iterations.get() {
        let a = first.point.components();
        let b = second.point.components();
        let components = [
            a[0] + (b[0] - a[0]) * 0.5,
            a[1] + (b[1] - a[1]) * 0.5,
            a[2] + (b[2] - a[2]) * 0.5,
        ];
        let point = Point::try_new(components)
            .map_err(|_| IsosurfaceError::NonFiniteCoordinate { components })?;
        let middle = evaluate(point)?;
        if middle.residual.abs() <= request.settings.value_tolerance {
            return Ok(middle);
        }
        if same_sign(middle.residual, first.residual) {
            first = middle;
        } else {
            second = middle;
        }
        if bracket_width(first.point, second.point) <= request.settings.coordinate_tolerance {
            return Ok(middle);
        }
    }
    Err(IsosurfaceError::RefinementLimitReached {
        first: first.point,
        second: second.point,
        absolute_residual: first.residual.abs().min(second.residual.abs()),
    })
}

#[allow(clippy::too_many_lines)]
fn build_report(
    request: &IsosurfaceRequest,
    extraction: RawExtraction,
    evaluate: &mut impl FnMut(
        Point<3>,
    ) -> Result<crate::model::FittedFieldEvaluation<3>, IsosurfaceError>,
) -> Result<IsosurfaceReport, IsosurfaceError> {
    let endpoint_capacity = extraction
        .triangles
        .len()
        .checked_mul(3)
        .ok_or_else(|| work_overflow(request))?;
    let mut records = try_vec(endpoint_capacity, IsosurfaceStorage::EndpointRecords)?;
    for triangle in &extraction.triangles {
        for endpoint in triangle.vertices {
            records.push((endpoint.key, endpoint.sample));
        }
    }
    records.sort_unstable_by(|left, right| {
        left.0
            .cmp(&right.0)
            .then_with(|| left.1.residual.abs().total_cmp(&right.1.residual.abs()))
            .then_with(|| compare_point(left.1.point, right.1.point))
    });
    records.dedup_by(|later, earlier| {
        if later.0 != earlier.0 {
            return false;
        }
        if later.1.residual.abs() < earlier.1.residual.abs() {
            earlier.1 = later.1;
        }
        true
    });

    let mut keys = try_vec(records.len(), IsosurfaceStorage::VertexKeys)?;
    let mut vertices = try_vec(records.len(), IsosurfaceStorage::Vertices)?;
    for (key, retained) in records {
        let evaluation = evaluate(retained.point)?;
        let value = evaluation.value();
        let residual = value - request.level;
        if !residual.is_finite() {
            return Err(IsosurfaceError::NonFiniteResidual {
                point: retained.point,
                value,
                level: request.level,
            });
        }
        let normal =
            UnitDirection::try_new(evaluation.gradient().into_components()).map_err(|_| {
                IsosurfaceError::ZeroGradientNormal {
                    point: retained.point,
                }
            })?;
        keys.push(key);
        vertices.push(IsosurfaceVertex {
            point: retained.point,
            value,
            residual,
            normal,
        });
    }

    let mut indexed = try_vec(extraction.triangles.len(), IsosurfaceStorage::Triangles)?;
    for triangle in extraction.triangles {
        let mut indices = [0; 3];
        for (slot, endpoint) in triangle.vertices.into_iter().enumerate() {
            indices[slot] = keys
                .binary_search(&endpoint.key)
                .map_err(|_| work_overflow(request))?;
        }
        if indices[0] == indices[1] || indices[1] == indices[2] || indices[2] == indices[0] {
            return Err(IsosurfaceError::CollapsedTriangle {
                vertex_indices: indices,
            });
        }
        orient_triangle(&vertices, &mut indices)?;
        indexed.push((
            canonical_triangle(indices),
            IsosurfaceTriangle {
                vertex_indices: indices,
            },
        ));
    }
    indexed.sort_unstable_by_key(|entry| entry.0);
    indexed.dedup_by_key(|entry| entry.0);
    let duplicate_triangles = endpoint_capacity
        .checked_div(3)
        .unwrap_or(0)
        .saturating_sub(indexed.len());
    let mut triangles = try_vec(indexed.len(), IsosurfaceStorage::Triangles)?;
    for (_, triangle) in indexed {
        triangles.push(triangle);
    }

    let topology = build_topology(request, &vertices, &triangles)?;
    let open_components = topology
        .components
        .iter()
        .filter(|component| !component.closed)
        .count();
    let closed_components = topology.components.len().saturating_sub(open_components);
    Ok(IsosurfaceReport {
        request: *request,
        diagnostics: IsosurfaceDiagnostics {
            method: request.method,
            cells: [
                request.settings.cells_x.get(),
                request.settings.cells_y.get(),
                request.settings.cells_z.get(),
            ],
            evaluations: 0,
            raw_triangles: endpoint_capacity / 3,
            unique_triangles: triangles.len(),
            duplicate_triangles,
            deduplicated_vertices: vertices.len(),
            ambiguous_faces: extraction.ambiguous_faces,
            boundary_edges: topology.boundary_edges,
            open_components,
            closed_components,
        },
        vertices,
        triangles,
        components: topology.components,
    })
}

struct Topology {
    components: Vec<IsosurfaceComponent>,
    boundary_edges: usize,
}

#[derive(Clone, Copy, Debug)]
struct EdgeRecord {
    vertices: [usize; 2],
    triangle: usize,
    forward: bool,
}

#[allow(clippy::too_many_lines)]
fn build_topology(
    request: &IsosurfaceRequest,
    vertices: &[IsosurfaceVertex],
    triangles: &[IsosurfaceTriangle],
) -> Result<Topology, IsosurfaceError> {
    let edge_capacity = triangles
        .len()
        .checked_mul(3)
        .ok_or_else(|| work_overflow(request))?;
    let mut edges = try_vec(edge_capacity, IsosurfaceStorage::EdgeIncidence)?;
    for (triangle, item) in triangles.iter().enumerate() {
        let indices = item.vertex_indices;
        for [from, to] in [
            [indices[0], indices[1]],
            [indices[1], indices[2]],
            [indices[2], indices[0]],
        ] {
            edges.push(EdgeRecord {
                vertices: [from.min(to), from.max(to)],
                triangle,
                forward: from < to,
            });
        }
    }
    edges.sort_unstable_by_key(|edge| (edge.vertices, edge.triangle));
    let mut adjacency = try_vec(triangles.len(), IsosurfaceStorage::TriangleAdjacency)?;
    adjacency.resize_with(triangles.len(), Vec::new);
    let mut triangle_boundary_faces =
        try_vec(triangles.len(), IsosurfaceStorage::TriangleAdjacency)?;
    triangle_boundary_faces.resize_with(triangles.len(), Vec::new);
    let mut boundary_edges = 0;
    let mut start = 0;
    while start < edges.len() {
        let mut end = start + 1;
        while end < edges.len() && edges[end].vertices == edges[start].vertices {
            end += 1;
        }
        let incidence = end - start;
        let [first, second] = edges[start].vertices;
        match incidence {
            1 => {
                let faces =
                    common_boundary_faces(request, vertices[first].point, vertices[second].point)?;
                if faces.is_empty() {
                    return Err(IsosurfaceError::InteriorBoundaryEdge {
                        first: vertices[first].point,
                        second: vertices[second].point,
                    });
                }
                boundary_edges += 1;
                for face in faces {
                    try_push(
                        &mut triangle_boundary_faces[edges[start].triangle],
                        face,
                        IsosurfaceStorage::ComponentBoundaryFaces,
                    )?;
                }
            }
            2 => {
                if edges[start].forward == edges[start + 1].forward {
                    return Err(IsosurfaceError::InconsistentWinding { first, second });
                }
                let first_triangle = edges[start].triangle;
                let second_triangle = edges[start + 1].triangle;
                try_push(
                    &mut adjacency[first_triangle],
                    second_triangle,
                    IsosurfaceStorage::TriangleAdjacency,
                )?;
                try_push(
                    &mut adjacency[second_triangle],
                    first_triangle,
                    IsosurfaceStorage::TriangleAdjacency,
                )?;
            }
            _ => {
                return Err(IsosurfaceError::NonManifoldEdge {
                    first: vertices[first].point,
                    second: vertices[second].point,
                    incidence,
                });
            }
        }
        start = end;
    }

    let mut visited = try_vec(triangles.len(), IsosurfaceStorage::ComponentTraversal)?;
    visited.resize(triangles.len(), false);
    let mut components = try_vec(triangles.len(), IsosurfaceStorage::Components)?;
    for seed in 0..triangles.len() {
        if visited[seed] {
            continue;
        }
        let mut stack = try_vec(8, IsosurfaceStorage::ComponentTraversal)?;
        let mut triangle_indices = try_vec(8, IsosurfaceStorage::ComponentTriangles)?;
        let mut faces = try_vec(6, IsosurfaceStorage::ComponentBoundaryFaces)?;
        try_push(&mut stack, seed, IsosurfaceStorage::ComponentTraversal)?;
        visited[seed] = true;
        while let Some(triangle) = stack.pop() {
            try_push(
                &mut triangle_indices,
                triangle,
                IsosurfaceStorage::ComponentTriangles,
            )?;
            for &face in &triangle_boundary_faces[triangle] {
                try_push(&mut faces, face, IsosurfaceStorage::ComponentBoundaryFaces)?;
            }
            for &neighbor in &adjacency[triangle] {
                if !visited[neighbor] {
                    visited[neighbor] = true;
                    try_push(&mut stack, neighbor, IsosurfaceStorage::ComponentTraversal)?;
                }
            }
        }
        triangle_indices.sort_unstable();
        faces.sort_unstable();
        faces.dedup();
        try_push(
            &mut components,
            IsosurfaceComponent {
                closed: faces.is_empty(),
                triangle_indices,
                boundary_faces: faces,
            },
            IsosurfaceStorage::Components,
        )?;
    }
    components.sort_unstable_by(|left, right| left.triangle_indices.cmp(&right.triangle_indices));
    Ok(Topology {
        components,
        boundary_edges,
    })
}

fn orient_triangle(
    vertices: &[IsosurfaceVertex],
    indices: &mut [usize; 3],
) -> Result<(), IsosurfaceError> {
    let a = vertices[indices[0]].point.components();
    let b = vertices[indices[1]].point.components();
    let c = vertices[indices[2]].point.components();
    let first = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let second = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let cross = [
        first[1] * second[2] - first[2] * second[1],
        first[2] * second[0] - first[0] * second[2],
        first[0] * second[1] - first[1] * second[0],
    ];
    if cross.iter().all(|component| *component == 0.0)
        || cross.iter().any(|component| !component.is_finite())
    {
        return Err(IsosurfaceError::CollapsedTriangle {
            vertex_indices: *indices,
        });
    }
    let mut normal_sum = [0.0; 3];
    for &index in indices.iter() {
        for (axis, component) in vertices[index].normal.components().iter().enumerate() {
            normal_sum[axis] += component;
        }
    }
    let alignment = cross[0] * normal_sum[0] + cross[1] * normal_sum[1] + cross[2] * normal_sum[2];
    if !alignment.is_finite() || alignment == 0.0 {
        return Err(IsosurfaceError::CollapsedTriangle {
            vertex_indices: *indices,
        });
    }
    if alignment < 0.0 {
        indices.swap(1, 2);
    }
    Ok(())
}

fn common_boundary_faces(
    request: &IsosurfaceRequest,
    first: Point<3>,
    second: Point<3>,
) -> Result<Vec<IsosurfaceBoundaryFace>, IsosurfaceError> {
    let mut faces = try_vec(6, IsosurfaceStorage::ComponentBoundaryFaces)?;
    let first = first.components();
    let second = second.components();
    let lower = request.lower.components();
    let upper = request.upper.components();
    let tolerance = request.settings.coordinate_tolerance;
    for (axis, minimum, maximum) in [
        (
            0,
            IsosurfaceBoundaryFace::XMin,
            IsosurfaceBoundaryFace::XMax,
        ),
        (
            1,
            IsosurfaceBoundaryFace::YMin,
            IsosurfaceBoundaryFace::YMax,
        ),
        (
            2,
            IsosurfaceBoundaryFace::ZMin,
            IsosurfaceBoundaryFace::ZMax,
        ),
    ] {
        if (first[axis] - lower[axis]).abs() <= tolerance
            && (second[axis] - lower[axis]).abs() <= tolerance
        {
            faces.push(minimum);
        }
        if (first[axis] - upper[axis]).abs() <= tolerance
            && (second[axis] - upper[axis]).abs() <= tolerance
        {
            faces.push(maximum);
        }
    }
    Ok(faces)
}

fn collect_unique<const N: usize, const M: usize>(
    endpoints: [Option<Endpoint>; N],
    unique: &mut [Option<Endpoint>; M],
) -> usize {
    let mut count = 0;
    for endpoint in endpoints.into_iter().flatten() {
        if unique[..count]
            .iter()
            .flatten()
            .any(|retained| retained.key == endpoint.key)
        {
            continue;
        }
        if count < M {
            unique[count] = Some(endpoint);
        }
        count += 1;
    }
    count
}

fn face_decider(corners: [Sample; 4]) -> f64 {
    let scale = corners
        .iter()
        .map(|corner| corner.residual.abs())
        .fold(0.0_f64, f64::max);
    if scale == 0.0 {
        return 0.0;
    }
    let values = corners.map(|corner| corner.residual / scale);
    let horizontal = values[1] - values[0];
    let vertical = values[3] - values[0];
    let mixed = values[0] - values[1] - values[3] + values[2];
    if mixed != 0.0 {
        let saddle_x = -vertical / mixed;
        let saddle_y = -horizontal / mixed;
        if saddle_x > 0.0 && saddle_x < 1.0 && saddle_y > 0.0 && saddle_y < 1.0 {
            return values[0] - horizontal * vertical / mixed;
        }
    }
    (values[0] + values[1] + values[2] + values[3]) * 0.25
}

fn sample(
    request: &IsosurfaceRequest,
    point: Point<3>,
    value: f64,
) -> Result<Sample, IsosurfaceError> {
    let residual = value - request.level;
    if residual.is_finite() {
        Ok(Sample { point, residual })
    } else {
        Err(IsosurfaceError::NonFiniteResidual {
            point,
            value,
            level: request.level,
        })
    }
}

fn cube_indices(column: usize, row: usize, layer: usize, work: GridWork) -> [usize; 8] {
    [
        node_index(column, row, layer, work),
        node_index(column + 1, row, layer, work),
        node_index(column, row + 1, layer, work),
        node_index(column + 1, row + 1, layer, work),
        node_index(column, row, layer + 1, work),
        node_index(column + 1, row, layer + 1, work),
        node_index(column, row + 1, layer + 1, work),
        node_index(column + 1, row + 1, layer + 1, work),
    ]
}

fn node_index(column: usize, row: usize, layer: usize, work: GridWork) -> usize {
    (layer * work.node_rows + row) * work.node_columns + column
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

fn bracket_width(first: Point<3>, second: Point<3>) -> f64 {
    let first = first.components();
    let second = second.components();
    (second[0] - first[0])
        .abs()
        .max((second[1] - first[1]).abs())
        .max((second[2] - first[2]).abs())
}

fn compare_point(first: Point<3>, second: Point<3>) -> std::cmp::Ordering {
    first.components()[0]
        .total_cmp(&second.components()[0])
        .then_with(|| first.components()[1].total_cmp(&second.components()[1]))
        .then_with(|| first.components()[2].total_cmp(&second.components()[2]))
}

fn canonical_triangle(mut indices: [usize; 3]) -> [usize; 3] {
    indices.sort_unstable();
    indices
}

fn same_sign(first: f64, second: f64) -> bool {
    first.is_sign_positive() == second.is_sign_positive()
}

fn push_triangle(
    triangles: &mut Vec<RawTriangle>,
    triangle: RawTriangle,
) -> Result<(), IsosurfaceError> {
    try_push(triangles, triangle, IsosurfaceStorage::RawTriangles)
}

fn work_overflow(request: &IsosurfaceRequest) -> IsosurfaceError {
    IsosurfaceError::WorkBudgetOverflow {
        cells: [
            request.settings.cells_x.get(),
            request.settings.cells_y.get(),
            request.settings.cells_z.get(),
        ],
        refinement_iterations: request.settings.refinement_iterations.get(),
    }
}

fn try_vec<T>(requested: usize, storage: IsosurfaceStorage) -> Result<Vec<T>, IsosurfaceError> {
    let mut values = Vec::new();
    values
        .try_reserve_exact(requested)
        .map_err(|_| IsosurfaceError::AllocationFailed { storage, requested })?;
    Ok(values)
}

fn try_push<T>(
    values: &mut Vec<T>,
    value: T,
    storage: IsosurfaceStorage,
) -> Result<(), IsosurfaceError> {
    if values.len() == values.capacity() {
        values
            .try_reserve(1)
            .map_err(|_| IsosurfaceError::AllocationFailed {
                storage,
                requested: values.len().saturating_add(1),
            })?;
    }
    values.push(value);
    Ok(())
}

fn validate_tolerance(
    tolerance: IsosurfaceTolerance,
    value: f64,
) -> Result<(), IsosurfaceSettingsError> {
    if value.is_finite() && value > 0.0 {
        Ok(())
    } else {
        Err(IsosurfaceSettingsError::InvalidTolerance { tolerance, value })
    }
}
