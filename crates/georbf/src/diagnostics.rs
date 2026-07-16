//! Stable structured errors and source-aware numerical diagnostics.
//!
//! The types in this module are owned by `GeoRBF` and expose neither
//! third-party numerical types nor Rust memory layout as a wire format.
//! [`ErrorCode`] supplies explicit numeric and symbolic values for later thin
//! adapter mappings. Those adapters must still define and snapshot their own
//! ABI or schema representation.

use std::error::Error;
use std::fmt;
use std::num::NonZeroUsize;

use crate::problem_ir::{ObservationId, SemanticProvenance};

/// Stable caller-controlled identifier for one semantic level.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[must_use]
pub struct LevelId(u64);

impl LevelId {
    /// Constructs an identifier from a caller-controlled stable value.
    pub const fn new(identifier: u64) -> Self {
        Self(identifier)
    }

    /// Returns the caller-controlled value.
    #[must_use]
    pub const fn identifier(self) -> u64 {
        self.0
    }
}

/// Owned source path retained by a structured diagnostic.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
#[must_use]
pub struct DiagnosticPath {
    source_path: Option<String>,
    source_line: Option<NonZeroUsize>,
    field_path: Option<String>,
    observation_id: Option<ObservationId>,
    level_id: Option<LevelId>,
    constraint_group: Option<String>,
}

impl DiagnosticPath {
    /// Constructs a diagnostic path without a specific input source.
    pub const fn global() -> Self {
        Self {
            source_path: None,
            source_line: None,
            field_path: None,
            observation_id: None,
            level_id: None,
            constraint_group: None,
        }
    }

    /// Fallibly copies an observation's complete source identity.
    ///
    /// # Errors
    ///
    /// Returns [`DiagnosticPathError::AllocationFailed`] when owned source
    /// text cannot be reserved.
    pub fn try_observation(provenance: &SemanticProvenance) -> Result<Self, DiagnosticPathError> {
        Self::try_observation_level(provenance, None)
    }

    /// Fallibly copies an observation's source identity and attaches a level.
    ///
    /// # Errors
    ///
    /// Returns [`DiagnosticPathError::AllocationFailed`] when owned source
    /// text cannot be reserved.
    pub fn try_observation_at_level(
        provenance: &SemanticProvenance,
        level_id: LevelId,
    ) -> Result<Self, DiagnosticPathError> {
        Self::try_observation_level(provenance, Some(level_id))
    }

    /// Constructs a level-only path within one validated semantic field path.
    ///
    /// # Errors
    ///
    /// Returns [`DiagnosticPathError::EmptyFieldPath`] for an empty field path.
    pub fn try_level(field_path: String, level_id: LevelId) -> Result<Self, DiagnosticPathError> {
        if field_path.trim().is_empty() {
            return Err(DiagnosticPathError::EmptyFieldPath);
        }
        Ok(Self {
            source_path: None,
            source_line: None,
            field_path: Some(field_path),
            observation_id: None,
            level_id: Some(level_id),
            constraint_group: None,
        })
    }

    /// Borrows the original source path, when available.
    #[must_use]
    pub fn source_path(&self) -> Option<&str> {
        self.source_path.as_deref()
    }

    /// Returns the original one-based source line, when available.
    #[must_use]
    pub const fn source_line(&self) -> Option<NonZeroUsize> {
        self.source_line
    }

    /// Borrows the semantic field path, when available.
    #[must_use]
    pub fn field_path(&self) -> Option<&str> {
        self.field_path.as_deref()
    }

    /// Returns the stable observation identifier, when available.
    #[must_use]
    pub const fn observation_id(&self) -> Option<ObservationId> {
        self.observation_id
    }

    /// Returns the stable level identifier, when available.
    #[must_use]
    pub const fn level_id(&self) -> Option<LevelId> {
        self.level_id
    }

    /// Borrows the optional constraint group.
    #[must_use]
    pub fn constraint_group(&self) -> Option<&str> {
        self.constraint_group.as_deref()
    }

    fn try_observation_level(
        provenance: &SemanticProvenance,
        level_id: Option<LevelId>,
    ) -> Result<Self, DiagnosticPathError> {
        Ok(Self {
            source_path: Some(try_copy_path_text(
                provenance.source().path(),
                DiagnosticPathField::SourcePath,
            )?),
            source_line: Some(provenance.source().line()),
            field_path: Some(try_copy_path_text(
                provenance.field_path(),
                DiagnosticPathField::FieldPath,
            )?),
            observation_id: Some(provenance.observation_id()),
            level_id,
            constraint_group: provenance
                .constraint_group()
                .map(|group| try_copy_path_text(group, DiagnosticPathField::ConstraintGroup))
                .transpose()?,
        })
    }
}

impl fmt::Display for DiagnosticPath {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut has_component = false;
        if let (Some(path), Some(line)) = (&self.source_path, self.source_line) {
            write!(formatter, "{path}:{}", line.get())?;
            has_component = true;
        }
        if let Some(field_path) = &self.field_path {
            write_path_separator(formatter, &mut has_component)?;
            write!(formatter, "field={field_path}")?;
        }
        if let Some(observation_id) = self.observation_id {
            write_path_separator(formatter, &mut has_component)?;
            write!(formatter, "observation={}", observation_id.identifier())?;
        }
        if let Some(level_id) = self.level_id {
            write_path_separator(formatter, &mut has_component)?;
            write!(formatter, "level={}", level_id.identifier())?;
        }
        if let Some(group) = &self.constraint_group {
            write_path_separator(formatter, &mut has_component)?;
            write!(formatter, "group={group}")?;
        }
        if !has_component {
            formatter.write_str("<global>")?;
        }
        Ok(())
    }
}

fn write_path_separator(
    formatter: &mut fmt::Formatter<'_>,
    has_component: &mut bool,
) -> fmt::Result {
    if *has_component {
        formatter.write_str(" | ")?;
    } else {
        *has_component = true;
    }
    Ok(())
}

/// Owned source-path field whose allocation failed.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DiagnosticPathField {
    /// Original input path.
    SourcePath,
    /// Semantic field path.
    FieldPath,
    /// Constraint group.
    ConstraintGroup,
}

/// Error returned while constructing an owned diagnostic source path.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum DiagnosticPathError {
    /// A level-only path used an empty semantic field path.
    EmptyFieldPath,
    /// Owned source text could not be reserved.
    AllocationFailed {
        /// Source field being copied.
        field: DiagnosticPathField,
        /// Exact UTF-8 byte count requested.
        requested: usize,
    },
}

impl fmt::Display for DiagnosticPathError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyFieldPath => formatter.write_str("diagnostic field path is empty"),
            Self::AllocationFailed { field, requested } => write!(
                formatter,
                "could not reserve {requested} bytes for diagnostic path field {field:?}"
            ),
        }
    }
}

impl Error for DiagnosticPathError {}

fn try_copy_path_text(
    value: &str,
    field: DiagnosticPathField,
) -> Result<String, DiagnosticPathError> {
    let mut copied = String::new();
    copied
        .try_reserve_exact(value.len())
        .map_err(|_| DiagnosticPathError::AllocationFailed {
            field,
            requested: value.len(),
        })?;
    copied.push_str(value);
    Ok(copied)
}

/// Broad failure category shared by all public structured errors.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum ErrorCategory {
    /// Invalid or inconsistent caller input.
    Input,
    /// A requested operation exceeds an explicit capability.
    Capability,
    /// A rank decision prevents an accepted result.
    Rank,
    /// A problem has unresolved additive gauge freedom.
    Gauge,
    /// A problem has no usable level contrast.
    Contrast,
    /// Hard constraints are mutually infeasible.
    Infeasibility,
    /// A condition policy rejected an ill-conditioned system.
    Conditioning,
    /// A memory estimate, limit, or allocation prevented completion.
    Memory,
    /// A caller cancellation request stopped an operation.
    Cancellation,
    /// A versioned input or component is incompatible.
    Version,
}

/// Stable machine-readable code for one structured failure category.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum ErrorCode {
    /// Invalid or inconsistent caller input.
    InvalidInput,
    /// A requested capability is unavailable.
    CapabilityUnavailable,
    /// A rank decision rejected the problem.
    RankRejected,
    /// An additive gauge is missing.
    MissingGauge,
    /// A usable level contrast is missing.
    MissingContrast,
    /// Hard constraints are infeasible.
    Infeasible,
    /// A condition limit was exceeded.
    IllConditioned,
    /// A memory policy or allocation prevented completion.
    MemoryUnavailable,
    /// The caller cancelled the operation.
    Cancelled,
    /// A version is incompatible.
    VersionMismatch,
}

impl ErrorCode {
    /// Returns the broad failure category.
    #[must_use]
    pub const fn category(self) -> ErrorCategory {
        match self {
            Self::InvalidInput => ErrorCategory::Input,
            Self::CapabilityUnavailable => ErrorCategory::Capability,
            Self::RankRejected => ErrorCategory::Rank,
            Self::MissingGauge => ErrorCategory::Gauge,
            Self::MissingContrast => ErrorCategory::Contrast,
            Self::Infeasible => ErrorCategory::Infeasibility,
            Self::IllConditioned => ErrorCategory::Conditioning,
            Self::MemoryUnavailable => ErrorCategory::Memory,
            Self::Cancelled => ErrorCategory::Cancellation,
            Self::VersionMismatch => ErrorCategory::Version,
        }
    }

    /// Returns the stable numeric code for thin adapters and persisted records.
    #[must_use]
    pub const fn number(self) -> u32 {
        match self {
            Self::InvalidInput => 1001,
            Self::CapabilityUnavailable => 2001,
            Self::RankRejected => 3001,
            Self::MissingGauge => 4001,
            Self::MissingContrast => 5001,
            Self::Infeasible => 6001,
            Self::IllConditioned => 7001,
            Self::MemoryUnavailable => 8001,
            Self::Cancelled => 9001,
            Self::VersionMismatch => 10_001,
        }
    }

    /// Returns the stable symbolic code for logs and human-facing adapters.
    #[must_use]
    pub const fn identifier(self) -> &'static str {
        match self {
            Self::InvalidInput => "GEORBF-E1001",
            Self::CapabilityUnavailable => "GEORBF-E2001",
            Self::RankRejected => "GEORBF-E3001",
            Self::MissingGauge => "GEORBF-E4001",
            Self::MissingContrast => "GEORBF-E5001",
            Self::Infeasible => "GEORBF-E6001",
            Self::IllConditioned => "GEORBF-E7001",
            Self::MemoryUnavailable => "GEORBF-E8001",
            Self::Cancelled => "GEORBF-E9001",
            Self::VersionMismatch => "GEORBF-E10001",
        }
    }
}

/// Text field rejected while constructing structured diagnostic evidence.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DiagnosticTextField {
    /// Input field name or path.
    InputField,
    /// Input rejection reason.
    InputReason,
    /// Capability name.
    Capability,
    /// Infeasibility explanation.
    InfeasibilityReason,
    /// Cancelled operation name.
    CancellationOperation,
    /// Versioned component name.
    VersionComponent,
    /// Expected version.
    ExpectedVersion,
    /// Found version.
    FoundVersion,
}

/// Error returned when structured diagnostic evidence is internally invalid.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum DiagnosticValueError {
    /// Required text was empty or whitespace-only.
    EmptyText {
        /// Rejected text field.
        field: DiagnosticTextField,
    },
    /// Rank dimensions or estimates were inconsistent.
    InvalidRank {
        /// Matrix row count.
        rows: usize,
        /// Matrix column count.
        columns: usize,
        /// Estimated rank.
        estimated_rank: usize,
        /// Required rank.
        required_rank: usize,
    },
    /// A missing-gauge diagnostic reported no connected component.
    InvalidGaugeComponentCount {
        /// Rejected component count.
        component_count: usize,
    },
    /// A contrast diagnostic used the same level on both sides.
    SameContrastLevel {
        /// Repeated level.
        level_id: LevelId,
    },
    /// Infeasibility evidence contained no source.
    EmptyInfeasibilitySources,
    /// Conditioning evidence was non-finite, nonpositive, or not beyond its limit.
    InvalidConditioning {
        /// Condition estimate.
        estimate: f64,
        /// Effective condition limit.
        limit: f64,
    },
    /// A bounded memory failure did not exceed its stated limit.
    MemoryWithinLimit {
        /// Requested bytes.
        requested_bytes: usize,
        /// Effective limit.
        limit_bytes: usize,
    },
    /// A version mismatch reported identical versions.
    IdenticalVersions,
}

impl fmt::Display for DiagnosticValueError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "invalid structured diagnostic evidence: {self:?}"
        )
    }
}

impl Error for DiagnosticValueError {}

/// Structured evidence for invalid caller input.
#[derive(Clone, Debug, Eq, PartialEq)]
#[must_use]
pub struct InputDiagnostic {
    field: String,
    reason: String,
}

impl InputDiagnostic {
    /// Validates and constructs input evidence.
    ///
    /// # Errors
    ///
    /// Returns [`DiagnosticValueError::EmptyText`] for an empty field or reason.
    pub fn try_new(field: String, reason: String) -> Result<Self, DiagnosticValueError> {
        validate_text(&field, DiagnosticTextField::InputField)?;
        validate_text(&reason, DiagnosticTextField::InputReason)?;
        Ok(Self { field, reason })
    }

    /// Borrows the rejected input field or path.
    #[must_use]
    pub fn field(&self) -> &str {
        &self.field
    }

    /// Borrows the rejection reason.
    #[must_use]
    pub fn reason(&self) -> &str {
        &self.reason
    }
}

/// Structured evidence for an unavailable capability.
#[derive(Clone, Debug, Eq, PartialEq)]
#[must_use]
pub struct CapabilityDiagnostic {
    capability: String,
}

impl CapabilityDiagnostic {
    /// Validates and constructs capability evidence.
    ///
    /// # Errors
    ///
    /// Returns [`DiagnosticValueError::EmptyText`] for an empty capability.
    pub fn try_new(capability: String) -> Result<Self, DiagnosticValueError> {
        validate_text(&capability, DiagnosticTextField::Capability)?;
        Ok(Self { capability })
    }

    /// Borrows the unavailable capability name.
    #[must_use]
    pub fn capability(&self) -> &str {
        &self.capability
    }
}

/// Generic rank evidence independent of a numerical backend.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[must_use]
pub struct RankDiagnostic {
    rows: usize,
    columns: usize,
    estimated_rank: usize,
    required_rank: usize,
}

impl RankDiagnostic {
    /// Validates and constructs rank evidence.
    ///
    /// # Errors
    ///
    /// Returns [`DiagnosticValueError::InvalidRank`] when either rank exceeds
    /// the matrix dimension or the estimate already satisfies the requirement.
    pub fn try_new(
        rows: usize,
        columns: usize,
        estimated_rank: usize,
        required_rank: usize,
    ) -> Result<Self, DiagnosticValueError> {
        let maximum_rank = rows.min(columns);
        if estimated_rank > maximum_rank
            || required_rank > maximum_rank
            || estimated_rank >= required_rank
        {
            return Err(DiagnosticValueError::InvalidRank {
                rows,
                columns,
                estimated_rank,
                required_rank,
            });
        }
        Ok(Self {
            rows,
            columns,
            estimated_rank,
            required_rank,
        })
    }

    /// Returns the matrix row count.
    #[must_use]
    pub const fn rows(self) -> usize {
        self.rows
    }

    /// Returns the matrix column count.
    #[must_use]
    pub const fn columns(self) -> usize {
        self.columns
    }

    /// Returns the accepted rank estimate.
    #[must_use]
    pub const fn estimated_rank(self) -> usize {
        self.estimated_rank
    }

    /// Returns the rank required by the rejected operation.
    #[must_use]
    pub const fn required_rank(self) -> usize {
        self.required_rank
    }
}

/// Structured evidence for unresolved additive gauge freedom.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[must_use]
pub struct GaugeDiagnostic {
    component_count: usize,
}

impl GaugeDiagnostic {
    /// Validates and constructs missing-gauge evidence.
    ///
    /// # Errors
    ///
    /// Returns [`DiagnosticValueError::InvalidGaugeComponentCount`] for zero
    /// connected components.
    pub fn try_new(component_count: usize) -> Result<Self, DiagnosticValueError> {
        if component_count == 0 {
            return Err(DiagnosticValueError::InvalidGaugeComponentCount { component_count });
        }
        Ok(Self { component_count })
    }

    /// Returns the number of ungauged connected components.
    #[must_use]
    pub const fn component_count(self) -> usize {
        self.component_count
    }
}

/// Structured evidence for a missing level contrast.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[must_use]
pub struct ContrastDiagnostic {
    lower: LevelId,
    upper: LevelId,
}

impl ContrastDiagnostic {
    /// Validates and constructs contrast evidence.
    ///
    /// # Errors
    ///
    /// Returns [`DiagnosticValueError::SameContrastLevel`] when both sides use
    /// the same stable level identifier.
    pub fn try_new(lower: LevelId, upper: LevelId) -> Result<Self, DiagnosticValueError> {
        if lower == upper {
            return Err(DiagnosticValueError::SameContrastLevel { level_id: lower });
        }
        Ok(Self { lower, upper })
    }

    /// Returns the lower level identifier.
    pub const fn lower(self) -> LevelId {
        self.lower
    }

    /// Returns the upper level identifier.
    pub const fn upper(self) -> LevelId {
        self.upper
    }
}

/// Structured evidence for mutually infeasible hard constraints.
#[derive(Clone, Debug, Eq, PartialEq)]
#[must_use]
pub struct InfeasibilityDiagnostic {
    sources: Vec<DiagnosticPath>,
    reason: String,
}

impl InfeasibilityDiagnostic {
    /// Validates and constructs infeasibility evidence.
    ///
    /// # Errors
    ///
    /// Returns [`DiagnosticValueError::EmptyInfeasibilitySources`] when no
    /// source participates, or [`DiagnosticValueError::EmptyText`] for an empty
    /// reason.
    pub fn try_new(
        sources: Vec<DiagnosticPath>,
        reason: String,
    ) -> Result<Self, DiagnosticValueError> {
        if sources.is_empty() {
            return Err(DiagnosticValueError::EmptyInfeasibilitySources);
        }
        validate_text(&reason, DiagnosticTextField::InfeasibilityReason)?;
        Ok(Self { sources, reason })
    }

    /// Borrows every source participating in the conflict.
    pub fn sources(&self) -> &[DiagnosticPath] {
        &self.sources
    }

    /// Borrows the structured conflict explanation.
    #[must_use]
    pub fn reason(&self) -> &str {
        &self.reason
    }
}

/// Structured evidence for a rejected condition estimate.
#[derive(Clone, Copy, Debug, PartialEq)]
#[must_use]
pub struct ConditioningDiagnostic {
    estimate: f64,
    limit: f64,
}

impl ConditioningDiagnostic {
    /// Validates and constructs conditioning evidence.
    ///
    /// # Errors
    ///
    /// Returns [`DiagnosticValueError::InvalidConditioning`] unless both values
    /// are finite and positive and the estimate exceeds the effective limit.
    pub fn try_new(estimate: f64, limit: f64) -> Result<Self, DiagnosticValueError> {
        if !estimate.is_finite()
            || !limit.is_finite()
            || estimate <= 0.0
            || limit <= 0.0
            || estimate <= limit
        {
            return Err(DiagnosticValueError::InvalidConditioning { estimate, limit });
        }
        Ok(Self { estimate, limit })
    }

    /// Returns the rejected condition estimate.
    #[must_use]
    pub const fn estimate(self) -> f64 {
        self.estimate
    }

    /// Returns the effective condition limit.
    #[must_use]
    pub const fn limit(self) -> f64 {
        self.limit
    }
}

/// Structured evidence for a memory failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[must_use]
pub struct MemoryDiagnostic {
    requested_bytes: usize,
    limit_bytes: Option<usize>,
}

impl MemoryDiagnostic {
    /// Constructs unbounded allocation-failure evidence.
    pub const fn allocation_failed(requested_bytes: usize) -> Self {
        Self {
            requested_bytes,
            limit_bytes: None,
        }
    }

    /// Validates and constructs explicit memory-limit evidence.
    ///
    /// # Errors
    ///
    /// Returns [`DiagnosticValueError::MemoryWithinLimit`] unless the request
    /// exceeds the effective limit.
    pub fn try_limit_exceeded(
        requested_bytes: usize,
        limit_bytes: usize,
    ) -> Result<Self, DiagnosticValueError> {
        if requested_bytes <= limit_bytes {
            return Err(DiagnosticValueError::MemoryWithinLimit {
                requested_bytes,
                limit_bytes,
            });
        }
        Ok(Self {
            requested_bytes,
            limit_bytes: Some(limit_bytes),
        })
    }

    /// Returns the requested byte count.
    #[must_use]
    pub const fn requested_bytes(self) -> usize {
        self.requested_bytes
    }

    /// Returns the effective byte limit, when one was configured.
    #[must_use]
    pub const fn limit_bytes(self) -> Option<usize> {
        self.limit_bytes
    }
}

/// Structured evidence for caller cancellation.
#[derive(Clone, Debug, Eq, PartialEq)]
#[must_use]
pub struct CancellationDiagnostic {
    operation: String,
}

impl CancellationDiagnostic {
    /// Validates and constructs cancellation evidence.
    ///
    /// # Errors
    ///
    /// Returns [`DiagnosticValueError::EmptyText`] for an empty operation.
    pub fn try_new(operation: String) -> Result<Self, DiagnosticValueError> {
        validate_text(&operation, DiagnosticTextField::CancellationOperation)?;
        Ok(Self { operation })
    }

    /// Borrows the cancelled operation name.
    #[must_use]
    pub fn operation(&self) -> &str {
        &self.operation
    }
}

/// Structured evidence for an incompatible version.
#[derive(Clone, Debug, Eq, PartialEq)]
#[must_use]
pub struct VersionDiagnostic {
    component: String,
    expected: String,
    found: String,
}

impl VersionDiagnostic {
    /// Validates and constructs version-mismatch evidence.
    ///
    /// # Errors
    ///
    /// Returns [`DiagnosticValueError::EmptyText`] for empty text or
    /// [`DiagnosticValueError::IdenticalVersions`] when the versions match.
    pub fn try_new(
        component: String,
        expected: String,
        found: String,
    ) -> Result<Self, DiagnosticValueError> {
        validate_text(&component, DiagnosticTextField::VersionComponent)?;
        validate_text(&expected, DiagnosticTextField::ExpectedVersion)?;
        validate_text(&found, DiagnosticTextField::FoundVersion)?;
        if expected == found {
            return Err(DiagnosticValueError::IdenticalVersions);
        }
        Ok(Self {
            component,
            expected,
            found,
        })
    }

    /// Borrows the versioned component name.
    #[must_use]
    pub fn component(&self) -> &str {
        &self.component
    }

    /// Borrows the expected version.
    #[must_use]
    pub fn expected(&self) -> &str {
        &self.expected
    }

    /// Borrows the incompatible found version.
    #[must_use]
    pub fn found(&self) -> &str {
        &self.found
    }
}

/// Unified source-aware error for public orchestration and adapter boundaries.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum GeoRbfError {
    /// Invalid or inconsistent caller input.
    Input {
        /// Most specific known source.
        source: Option<DiagnosticPath>,
        /// Structured input evidence.
        diagnostic: InputDiagnostic,
    },
    /// A requested capability is unavailable.
    Capability {
        /// Most specific known source.
        source: Option<DiagnosticPath>,
        /// Structured capability evidence.
        diagnostic: CapabilityDiagnostic,
    },
    /// A rank decision rejected the problem.
    Rank {
        /// Most specific known source.
        source: Option<DiagnosticPath>,
        /// Backend-independent rank evidence.
        diagnostic: RankDiagnostic,
    },
    /// An additive gauge is missing.
    Gauge {
        /// Most specific known source.
        source: Option<DiagnosticPath>,
        /// Structured gauge evidence.
        diagnostic: GaugeDiagnostic,
    },
    /// A usable level contrast is missing.
    Contrast {
        /// Most specific known source.
        source: Option<DiagnosticPath>,
        /// Structured contrast evidence.
        diagnostic: ContrastDiagnostic,
    },
    /// Hard constraints are mutually infeasible.
    Infeasible {
        /// Structured conflict sources and explanation.
        diagnostic: InfeasibilityDiagnostic,
    },
    /// A condition policy rejected the problem.
    Conditioning {
        /// Most specific known source.
        source: Option<DiagnosticPath>,
        /// Structured condition evidence.
        diagnostic: ConditioningDiagnostic,
    },
    /// A memory estimate, policy, or allocation prevented completion.
    Memory {
        /// Most specific known source.
        source: Option<DiagnosticPath>,
        /// Structured memory evidence.
        diagnostic: MemoryDiagnostic,
    },
    /// The caller cancelled the operation.
    Cancelled {
        /// Most specific known source.
        source: Option<DiagnosticPath>,
        /// Structured cancellation evidence.
        diagnostic: CancellationDiagnostic,
    },
    /// A versioned input or component is incompatible.
    Version {
        /// Most specific known source.
        source: Option<DiagnosticPath>,
        /// Structured version evidence.
        diagnostic: VersionDiagnostic,
    },
}

impl GeoRbfError {
    /// Returns the stable machine-readable error code.
    #[must_use]
    pub const fn code(&self) -> ErrorCode {
        match self {
            Self::Input { .. } => ErrorCode::InvalidInput,
            Self::Capability { .. } => ErrorCode::CapabilityUnavailable,
            Self::Rank { .. } => ErrorCode::RankRejected,
            Self::Gauge { .. } => ErrorCode::MissingGauge,
            Self::Contrast { .. } => ErrorCode::MissingContrast,
            Self::Infeasible { .. } => ErrorCode::Infeasible,
            Self::Conditioning { .. } => ErrorCode::IllConditioned,
            Self::Memory { .. } => ErrorCode::MemoryUnavailable,
            Self::Cancelled { .. } => ErrorCode::Cancelled,
            Self::Version { .. } => ErrorCode::VersionMismatch,
        }
    }

    /// Borrows the primary source path, when one is available.
    #[must_use]
    pub fn primary_source(&self) -> Option<&DiagnosticPath> {
        match self {
            Self::Input { source, .. }
            | Self::Capability { source, .. }
            | Self::Rank { source, .. }
            | Self::Gauge { source, .. }
            | Self::Contrast { source, .. }
            | Self::Conditioning { source, .. }
            | Self::Memory { source, .. }
            | Self::Cancelled { source, .. }
            | Self::Version { source, .. } => source.as_ref(),
            Self::Infeasible { diagnostic } => diagnostic.sources().first(),
        }
    }

    /// Borrows additional source paths participating in the failure.
    pub fn related_sources(&self) -> &[DiagnosticPath] {
        match self {
            Self::Infeasible { diagnostic } => diagnostic.sources().get(1..).unwrap_or_default(),
            _ => &[],
        }
    }
}

impl fmt::Display for GeoRbfError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} ", self.code().identifier())?;
        match self {
            Self::Input { source, diagnostic } => fmt_input(formatter, source.as_ref(), diagnostic),
            Self::Capability { source, diagnostic } => {
                fmt_capability(formatter, source.as_ref(), diagnostic)
            }
            Self::Rank { source, diagnostic } => fmt_rank(formatter, source.as_ref(), *diagnostic),
            Self::Gauge { source, diagnostic } => {
                fmt_gauge(formatter, source.as_ref(), *diagnostic)
            }
            Self::Contrast { source, diagnostic } => {
                fmt_contrast(formatter, source.as_ref(), *diagnostic)
            }
            Self::Infeasible { diagnostic } => fmt_infeasible(formatter, diagnostic),
            Self::Conditioning { source, diagnostic } => {
                fmt_conditioning(formatter, source.as_ref(), *diagnostic)
            }
            Self::Memory { source, diagnostic } => {
                fmt_memory(formatter, source.as_ref(), *diagnostic)
            }
            Self::Cancelled { source, diagnostic } => {
                fmt_cancelled(formatter, source.as_ref(), diagnostic)
            }
            Self::Version { source, diagnostic } => {
                fmt_version(formatter, source.as_ref(), diagnostic)
            }
        }
    }
}

impl Error for GeoRbfError {}

fn fmt_input(
    formatter: &mut fmt::Formatter<'_>,
    source: Option<&DiagnosticPath>,
    diagnostic: &InputDiagnostic,
) -> fmt::Result {
    formatter.write_str("invalid input")?;
    write_optional_source(formatter, source)?;
    write!(
        formatter,
        ": {}: {}",
        diagnostic.field(),
        diagnostic.reason()
    )
}

fn fmt_capability(
    formatter: &mut fmt::Formatter<'_>,
    source: Option<&DiagnosticPath>,
    diagnostic: &CapabilityDiagnostic,
) -> fmt::Result {
    formatter.write_str("capability unavailable")?;
    write_optional_source(formatter, source)?;
    write!(formatter, ": {}", diagnostic.capability())
}

fn fmt_rank(
    formatter: &mut fmt::Formatter<'_>,
    source: Option<&DiagnosticPath>,
    diagnostic: RankDiagnostic,
) -> fmt::Result {
    formatter.write_str("rank rejected")?;
    write_optional_source(formatter, source)?;
    write!(
        formatter,
        ": estimated rank {} is below required rank {} for {}x{} system",
        diagnostic.estimated_rank(),
        diagnostic.required_rank(),
        diagnostic.rows(),
        diagnostic.columns()
    )
}

fn fmt_gauge(
    formatter: &mut fmt::Formatter<'_>,
    source: Option<&DiagnosticPath>,
    diagnostic: GaugeDiagnostic,
) -> fmt::Result {
    formatter.write_str("missing gauge")?;
    write_optional_source(formatter, source)?;
    write!(
        formatter,
        ": {} connected component(s) remain ungauged",
        diagnostic.component_count()
    )
}

fn fmt_contrast(
    formatter: &mut fmt::Formatter<'_>,
    source: Option<&DiagnosticPath>,
    diagnostic: ContrastDiagnostic,
) -> fmt::Result {
    formatter.write_str("missing contrast")?;
    write_optional_source(formatter, source)?;
    write!(
        formatter,
        ": levels {} and {} have no usable contrast",
        diagnostic.lower().identifier(),
        diagnostic.upper().identifier()
    )
}

fn fmt_infeasible(
    formatter: &mut fmt::Formatter<'_>,
    diagnostic: &InfeasibilityDiagnostic,
) -> fmt::Result {
    write!(
        formatter,
        "infeasible hard constraints: {}",
        diagnostic.reason()
    )?;
    formatter.write_str(" [sources: ")?;
    for (index, source) in diagnostic.sources().iter().enumerate() {
        if index != 0 {
            formatter.write_str(", ")?;
        }
        write!(formatter, "{source}")?;
    }
    formatter.write_str("]")
}

fn fmt_conditioning(
    formatter: &mut fmt::Formatter<'_>,
    source: Option<&DiagnosticPath>,
    diagnostic: ConditioningDiagnostic,
) -> fmt::Result {
    formatter.write_str("condition limit exceeded")?;
    write_optional_source(formatter, source)?;
    write!(
        formatter,
        ": estimate {} exceeds limit {}",
        diagnostic.estimate(),
        diagnostic.limit()
    )
}

fn fmt_memory(
    formatter: &mut fmt::Formatter<'_>,
    source: Option<&DiagnosticPath>,
    diagnostic: MemoryDiagnostic,
) -> fmt::Result {
    formatter.write_str("memory unavailable")?;
    write_optional_source(formatter, source)?;
    if let Some(limit) = diagnostic.limit_bytes() {
        write!(
            formatter,
            ": requested {} bytes exceeds limit {limit}",
            diagnostic.requested_bytes()
        )
    } else {
        write!(
            formatter,
            ": allocation of {} bytes failed",
            diagnostic.requested_bytes()
        )
    }
}

fn fmt_cancelled(
    formatter: &mut fmt::Formatter<'_>,
    source: Option<&DiagnosticPath>,
    diagnostic: &CancellationDiagnostic,
) -> fmt::Result {
    formatter.write_str("operation cancelled")?;
    write_optional_source(formatter, source)?;
    write!(formatter, ": {}", diagnostic.operation())
}

fn fmt_version(
    formatter: &mut fmt::Formatter<'_>,
    source: Option<&DiagnosticPath>,
    diagnostic: &VersionDiagnostic,
) -> fmt::Result {
    formatter.write_str("version mismatch")?;
    write_optional_source(formatter, source)?;
    write!(
        formatter,
        ": {} expected {}, found {}",
        diagnostic.component(),
        diagnostic.expected(),
        diagnostic.found()
    )
}

fn write_optional_source(
    formatter: &mut fmt::Formatter<'_>,
    source: Option<&DiagnosticPath>,
) -> fmt::Result {
    if let Some(source) = source {
        write!(formatter, " at {source}")?;
    }
    Ok(())
}

fn validate_text(value: &str, field: DiagnosticTextField) -> Result<(), DiagnosticValueError> {
    if value.trim().is_empty() {
        Err(DiagnosticValueError::EmptyText { field })
    } else {
        Ok(())
    }
}
