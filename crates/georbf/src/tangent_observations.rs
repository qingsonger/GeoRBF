//! Provenance-preserving tangent observations with explicit derivative gauges.
//!
//! A tangent direction `t` at `x` lowers to the scalar equality
//! `t^T grad f(x) = 0`. Each tangent is one solver-neutral semantic relation,
//! so hard enforcement and all existing scalar soft losses remain explicit.
//! Multiple independently identified tangents may share a point.
//!
//! A problem containing only derivative observations has additive constant
//! freedom. [`TangentProblem`] therefore requires one caller-supplied hard
//! [`DerivativeGaugeAnchor`]. It never selects or inserts an automatic anchor.
//!
//! Tangent observations are available only in one, two, or three dimensions:
//!
//! ```compile_fail
//! use georbf::TangentObservation;
//!
//! let _ = std::mem::size_of::<TangentObservation<4>>();
//! ```

use std::error::Error;
use std::fmt;

#[cfg(test)]
use std::cell::Cell;

use crate::diagnostics::{
    DiagnosticPath, DiagnosticPathError, DiagnosticValueError, GaugeDiagnostic, GeoRbfError,
};
use crate::dimension::{Dim, SupportedDimension};
use crate::functional::{
    FunctionalAtom, FunctionalError, FunctionalExpr, FunctionalProvenance, FunctionalTerm,
    ObservationFunctional,
};
use crate::geometry::{Point, UnitDirection};
use crate::problem_ir::{
    Enforcement, ExecutionOptions, ObservationId, ProblemIrError, SemanticConstraint,
    SemanticExpression, SemanticProblemIr, SemanticProvenance, SemanticRelation,
};

/// One validated scalar tangent relation.
#[derive(Clone, Debug, PartialEq)]
#[must_use]
pub struct TangentObservation<const D: usize>
where
    Dim<D>: SupportedDimension,
{
    point: Point<D>,
    tangent: UnitDirection<D>,
    constraint: SemanticConstraint<D>,
}

impl<const D: usize> TangentObservation<D>
where
    Dim<D>: SupportedDimension,
{
    /// Constructs `t^T grad f(x) = 0` with explicit hard or soft enforcement.
    ///
    /// The tangent magnitude is intentionally absent: a validated unit
    /// direction carries the geometric line while the enforcement scale carries
    /// residual units. Distinct provenances allow several tangents at one point.
    ///
    /// # Errors
    ///
    /// Returns shared functional or semantic-IR validation errors, including an
    /// invalid soft scale or Huber transition.
    pub fn try_new(
        provenance: SemanticProvenance,
        point: Point<D>,
        tangent: UnitDirection<D>,
        enforcement: Enforcement,
    ) -> Result<Self, TangentObservationError> {
        let expression = derivative_expression(point, tangent, &provenance)?;
        let constraint = SemanticConstraint::try_new(
            provenance,
            SemanticRelation::Equality {
                expression,
                target: 0.0,
            },
            enforcement,
        )?;
        Ok(Self {
            point,
            tangent,
            constraint,
        })
    }

    /// Returns the point shared by this scalar tangent relation.
    pub const fn point(&self) -> Point<D> {
        self.point
    }

    /// Returns the validated unit tangent direction.
    pub const fn tangent(&self) -> UnitDirection<D> {
        self.tangent
    }

    /// Borrows the lowered scalar semantic equality.
    pub const fn constraint(&self) -> &SemanticConstraint<D> {
        &self.constraint
    }

    fn into_constraint(self) -> SemanticConstraint<D> {
        self.constraint
    }
}

/// One explicit hard scalar-value anchor for a derivative-only field.
#[derive(Clone, Debug, PartialEq)]
#[must_use]
pub struct DerivativeGaugeAnchor<const D: usize>
where
    Dim<D>: SupportedDimension,
{
    point: Point<D>,
    value: f64,
    constraint: SemanticConstraint<D>,
}

impl<const D: usize> DerivativeGaugeAnchor<D>
where
    Dim<D>: SupportedDimension,
{
    /// Constructs the hard equality `f(x) = value` used as the additive gauge.
    ///
    /// # Errors
    ///
    /// Rejects a non-finite value or shared functional/IR validation failure.
    pub fn try_new(
        provenance: SemanticProvenance,
        point: Point<D>,
        value: f64,
    ) -> Result<Self, TangentObservationError> {
        let expression = value_expression(point, &provenance)?;
        let constraint = SemanticConstraint::try_new(
            provenance,
            SemanticRelation::Equality {
                expression,
                target: value,
            },
            Enforcement::Hard,
        )?;
        Ok(Self {
            point,
            value,
            constraint,
        })
    }

    /// Returns the anchored point.
    pub const fn point(&self) -> Point<D> {
        self.point
    }

    /// Returns the finite anchored scalar value.
    #[must_use]
    pub const fn value(&self) -> f64 {
        self.value
    }

    /// Borrows the hard value equality.
    pub const fn constraint(&self) -> &SemanticConstraint<D> {
        &self.constraint
    }

    fn into_parts(self) -> (Point<D>, f64, ObservationId, SemanticConstraint<D>) {
        let identifier = self.constraint.provenance().observation_id();
        (self.point, self.value, identifier, self.constraint)
    }
}

/// A nonempty tangent-only problem with one explicit hard value gauge.
#[derive(Clone, Debug, PartialEq)]
#[must_use]
pub struct TangentProblem<const D: usize>
where
    Dim<D>: SupportedDimension,
{
    tangent_count: usize,
    gauge_point: Point<D>,
    gauge_value: f64,
    gauge_observation_id: ObservationId,
    semantic: SemanticProblemIr<D>,
}

impl<const D: usize> TangentProblem<D>
where
    Dim<D>: SupportedDimension,
{
    /// Collects tangents in input order and appends one explicit hard gauge.
    ///
    /// The first tangent supplies the source for a missing-gauge diagnostic.
    /// No default point or value is inferred. The returned semantic IR retains
    /// every tangent followed by the gauge anchor in deterministic order.
    ///
    /// # Errors
    ///
    /// Rejects an empty tangent collection, a missing anchor, duplicate stable
    /// identifiers, count overflow, allocation failure, or shared IR failure.
    #[allow(clippy::result_large_err)]
    pub fn try_new(
        observations: impl IntoIterator<Item = TangentObservation<D>>,
        gauge_anchor: Option<DerivativeGaugeAnchor<D>>,
        execution: ExecutionOptions,
    ) -> Result<Self, TangentProblemError> {
        let iterator = observations.into_iter();
        let minimum = iterator.size_hint().0;
        let requested = minimum
            .checked_add(1)
            .ok_or(TangentProblemError::CountOverflow)?;
        let mut constraints = reserve(requested).map_err(TangentProblemError::from)?;
        for observation in iterator {
            if constraints.len() == constraints.capacity() {
                let requested = constraints
                    .len()
                    .checked_add(1)
                    .ok_or(TangentProblemError::CountOverflow)?;
                try_reserve_one(&mut constraints, requested).map_err(TangentProblemError::from)?;
            }
            constraints.push(observation.into_constraint());
        }
        if constraints.is_empty() {
            return Err(TangentProblemError::EmptyTangentProblem);
        }
        let tangent_count = constraints.len();
        let Some(gauge_anchor) = gauge_anchor else {
            let source = DiagnosticPath::try_observation(constraints[0].provenance())?;
            let diagnostic = GaugeDiagnostic::try_new(1)?;
            return Err(TangentProblemError::MissingGauge(GeoRbfError::Gauge {
                source: Some(source),
                diagnostic,
            }));
        };
        if constraints.len() == constraints.capacity() {
            let requested = constraints
                .len()
                .checked_add(1)
                .ok_or(TangentProblemError::CountOverflow)?;
            try_reserve_one(&mut constraints, requested).map_err(TangentProblemError::from)?;
        }
        let (gauge_point, gauge_value, gauge_observation_id, gauge_constraint) =
            gauge_anchor.into_parts();
        constraints.push(gauge_constraint);
        let semantic = SemanticProblemIr::try_new(constraints, execution)?;
        Ok(Self {
            tangent_count,
            gauge_point,
            gauge_value,
            gauge_observation_id,
            semantic,
        })
    }

    /// Returns the number of tangent rows before the final gauge row.
    #[must_use]
    pub const fn tangent_count(&self) -> usize {
        self.tangent_count
    }

    /// Returns the explicitly selected gauge point.
    pub const fn gauge_point(&self) -> Point<D> {
        self.gauge_point
    }

    /// Returns the explicitly selected finite gauge value.
    #[must_use]
    pub const fn gauge_value(&self) -> f64 {
        self.gauge_value
    }

    /// Returns the stable identifier of the appended gauge equality.
    pub const fn gauge_observation_id(&self) -> ObservationId {
        self.gauge_observation_id
    }

    /// Borrows the complete semantic problem: tangents first, gauge last.
    pub const fn semantic_problem(&self) -> &SemanticProblemIr<D> {
        &self.semantic
    }

    /// Consumes the wrapper and returns the complete solver-neutral problem.
    pub fn into_semantic_problem(self) -> SemanticProblemIr<D> {
        self.semantic
    }
}

fn derivative_expression<const D: usize>(
    point: Point<D>,
    tangent: UnitDirection<D>,
    provenance: &SemanticProvenance,
) -> Result<SemanticExpression<D>, TangentObservationError>
where
    Dim<D>: SupportedDimension,
{
    let atom = FunctionalAtom::directional_derivative(
        point,
        tangent,
        FunctionalProvenance::new(provenance.observation_id().identifier()),
    );
    one_term_expression(atom)
}

fn value_expression<const D: usize>(
    point: Point<D>,
    provenance: &SemanticProvenance,
) -> Result<SemanticExpression<D>, TangentObservationError>
where
    Dim<D>: SupportedDimension,
{
    let atom = FunctionalAtom::value(
        point,
        FunctionalProvenance::new(provenance.observation_id().identifier()),
    );
    one_term_expression(atom)
}

fn one_term_expression<const D: usize>(
    atom: FunctionalAtom<D>,
) -> Result<SemanticExpression<D>, TangentObservationError>
where
    Dim<D>: SupportedDimension,
{
    let term = FunctionalTerm::try_new(1.0, atom)?;
    let functional = ObservationFunctional::new(FunctionalExpr::try_new([term])?);
    Ok(SemanticExpression::try_new(functional, 0.0)?)
}

#[cfg(test)]
std::thread_local! {
    static FORCE_ALLOCATION_FAILURE: Cell<bool> = const { Cell::new(false) };
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct TangentAllocationError {
    requested: usize,
}

fn reserve<T>(requested: usize) -> Result<Vec<T>, TangentAllocationError> {
    #[cfg(test)]
    if FORCE_ALLOCATION_FAILURE.with(|forced| forced.replace(false)) {
        return Err(TangentAllocationError { requested });
    }
    let mut values = Vec::new();
    values
        .try_reserve_exact(requested)
        .map_err(|_| TangentAllocationError { requested })?;
    Ok(values)
}

fn try_reserve_one<T>(values: &mut Vec<T>, requested: usize) -> Result<(), TangentAllocationError> {
    values
        .try_reserve(1)
        .map_err(|_| TangentAllocationError { requested })
}

/// Error returned while constructing a tangent observation or gauge anchor.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum TangentObservationError {
    /// Atomic functional construction failed.
    Functional(FunctionalError),
    /// Shared semantic IR validation failed.
    Ir(ProblemIrError),
}

impl fmt::Display for TangentObservationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "invalid tangent observation: {self:?}")
    }
}

impl Error for TangentObservationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Functional(source) => Some(source),
            Self::Ir(source) => Some(source),
        }
    }
}

impl From<FunctionalError> for TangentObservationError {
    fn from(source: FunctionalError) -> Self {
        Self::Functional(source)
    }
}

impl From<ProblemIrError> for TangentObservationError {
    fn from(source: ProblemIrError) -> Self {
        Self::Ir(source)
    }
}

/// Error returned while collecting tangents with their explicit gauge.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum TangentProblemError {
    /// No tangent was supplied to a tangent-only problem.
    EmptyTangentProblem,
    /// The derivative-only problem omitted its explicit hard value anchor.
    MissingGauge(GeoRbfError),
    /// A checked tangent or gauge count overflowed `usize`.
    CountOverflow,
    /// Fallible tangent/gauge storage reservation failed.
    AllocationFailed {
        /// Exact or minimum requested capacity.
        requested: usize,
    },
    /// Shared semantic IR validation failed.
    Ir(ProblemIrError),
    /// A source-aware diagnostic path could not be copied.
    DiagnosticPath(DiagnosticPathError),
    /// Structured gauge evidence was invalid.
    DiagnosticValue(DiagnosticValueError),
}

impl TangentProblemError {
    /// Borrows the stable public diagnostic for a missing gauge, when present.
    #[must_use]
    pub const fn gauge_diagnostic(&self) -> Option<&GeoRbfError> {
        match self {
            Self::MissingGauge(diagnostic) => Some(diagnostic),
            _ => None,
        }
    }
}

impl fmt::Display for TangentProblemError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingGauge(diagnostic) => diagnostic.fmt(formatter),
            _ => write!(formatter, "invalid tangent observation: {self:?}"),
        }
    }
}

impl Error for TangentProblemError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::MissingGauge(source) => Some(source),
            Self::Ir(source) => Some(source),
            Self::DiagnosticPath(source) => Some(source),
            Self::DiagnosticValue(source) => Some(source),
            _ => None,
        }
    }
}

impl From<ProblemIrError> for TangentProblemError {
    fn from(source: ProblemIrError) -> Self {
        Self::Ir(source)
    }
}

impl From<DiagnosticPathError> for TangentProblemError {
    fn from(source: DiagnosticPathError) -> Self {
        Self::DiagnosticPath(source)
    }
}

impl From<DiagnosticValueError> for TangentProblemError {
    fn from(source: DiagnosticValueError) -> Self {
        Self::DiagnosticValue(source)
    }
}

impl From<TangentAllocationError> for TangentProblemError {
    fn from(source: TangentAllocationError) -> Self {
        Self::AllocationFailed {
            requested: source.requested,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroUsize;

    use super::*;
    use crate::diagnostics::ErrorCode;
    use crate::problem_ir::{ObservationId, SoftLoss, SourceLocation};

    fn provenance(identifier: u64) -> Result<SemanticProvenance, ProblemIrError> {
        SemanticProvenance::try_new(
            ObservationId::new(identifier),
            SourceLocation::try_new("tests/tangents.csv".to_owned(), NonZeroUsize::MIN)?,
            "1/m".to_owned(),
            "fields.stratigraphy.tangents".to_owned(),
            Some("tangent".to_owned()),
        )
    }

    fn tangent(identifier: u64) -> Result<TangentObservation<2>, Box<dyn Error>> {
        Ok(TangentObservation::try_new(
            provenance(identifier)?,
            Point::try_new([0.0, 0.0])?,
            UnitDirection::try_new([1.0, 0.0])?,
            Enforcement::Soft {
                scale: 2.0,
                loss: SoftLoss::SquaredL2,
            },
        )?)
    }

    #[test]
    fn missing_gauge_uses_stable_source_aware_diagnostic() -> Result<(), Box<dyn Error>> {
        let Err(error) = TangentProblem::try_new([tangent(7)?], None, ExecutionOptions::default())
        else {
            return Err("a derivative-only problem accepted no gauge".into());
        };
        let diagnostic = error
            .gauge_diagnostic()
            .ok_or("missing stable gauge diagnostic")?;
        assert_eq!(diagnostic.code(), ErrorCode::MissingGauge);
        assert_eq!(
            diagnostic
                .primary_source()
                .and_then(DiagnosticPath::observation_id),
            Some(ObservationId::new(7))
        );
        assert_eq!(
            diagnostic.to_string().split_whitespace().next(),
            Some("GEORBF-E4001")
        );
        Ok(())
    }

    #[test]
    fn final_problem_storage_failure_is_structural() -> Result<(), Box<dyn Error>> {
        FORCE_ALLOCATION_FAILURE.with(|forced| forced.set(true));
        let anchor =
            DerivativeGaugeAnchor::try_new(provenance(9)?, Point::try_new([0.0, 0.0])?, 0.0)?;
        let Err(error) =
            TangentProblem::try_new([tangent(8)?], Some(anchor), ExecutionOptions::default())
        else {
            return Err("forced allocation failure was accepted".into());
        };
        assert!(matches!(
            error,
            TangentProblemError::AllocationFailed { requested: 2 }
        ));
        Ok(())
    }
}
