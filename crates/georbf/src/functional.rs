//! Atomic scalar-field functionals and their finite linear expressions.
//!
//! `GeoRBF` has exactly two atomic functionals: a value at a point and a
//! directional derivative along a validated unit direction. Expressions keep
//! caller provenance on every term and preserve insertion order. Observation
//! functionals and center representers are deliberately different types even
//! when they wrap equal expressions.
//!
//! Functionals are available only in one, two, or three dimensions:
//!
//! ```compile_fail
//! use georbf::{FunctionalAtom, FunctionalProvenance, Point};
//!
//! let point = Point::<4>::try_new([0.0; 4])?;
//! let _ = FunctionalAtom::value(point, FunctionalProvenance::new(1));
//! # Ok::<(), Box<dyn std::error::Error>>(() )
//! ```

use std::error::Error;
use std::fmt;

use crate::dimension::{Dim, SupportedDimension};
use crate::geometry::{Point, UnitDirection};
use crate::kernel::KernelDerivativeOrder;
use crate::kernel_calculus::{KernelArgument, SpatialKernelJetPrefix};
use crate::polynomial::{PolynomialSpace, PolynomialSpaceError};

/// Stable opaque caller provenance attached to one atomic functional.
///
/// The core preserves this identifier exactly but assigns it no semantic
/// meaning. A later semantic IR may map it to source files, rows, fields, or
/// observation identifiers.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[must_use]
pub struct FunctionalProvenance(u64);

impl FunctionalProvenance {
    /// Constructs provenance from a caller-controlled stable identifier.
    pub const fn new(identifier: u64) -> Self {
        Self(identifier)
    }

    /// Returns the preserved caller identifier.
    #[must_use]
    pub const fn identifier(self) -> u64 {
        self.0
    }
}

/// One of the two v1 atomic scalar-field functionals.
#[derive(Clone, Copy, Debug, PartialEq)]
#[must_use]
pub enum FunctionalAtom<const D: usize>
where
    Dim<D>: SupportedDimension,
{
    /// Evaluate the scalar field at a finite point.
    Value {
        /// Evaluation point.
        point: Point<D>,
        /// Opaque caller provenance.
        provenance: FunctionalProvenance,
    },
    /// Evaluate `u^T grad f(x)` for a validated unit direction `u`.
    DirectionalDerivative {
        /// Derivative point.
        point: Point<D>,
        /// Unit derivative direction.
        direction: UnitDirection<D>,
        /// Opaque caller provenance.
        provenance: FunctionalProvenance,
    },
}

impl<const D: usize> FunctionalAtom<D>
where
    Dim<D>: SupportedDimension,
{
    /// Constructs a value functional.
    pub const fn value(point: Point<D>, provenance: FunctionalProvenance) -> Self {
        Self::Value { point, provenance }
    }

    /// Constructs a directional-derivative functional.
    pub const fn directional_derivative(
        point: Point<D>,
        direction: UnitDirection<D>,
        provenance: FunctionalProvenance,
    ) -> Self {
        Self::DirectionalDerivative {
            point,
            direction,
            provenance,
        }
    }

    /// Returns the point at which this atom acts.
    pub const fn point(self) -> Point<D> {
        match self {
            Self::Value { point, .. } | Self::DirectionalDerivative { point, .. } => point,
        }
    }

    /// Returns the preserved caller provenance.
    pub const fn provenance(self) -> FunctionalProvenance {
        match self {
            Self::Value { provenance, .. } | Self::DirectionalDerivative { provenance, .. } => {
                provenance
            }
        }
    }

    /// Returns the unit direction for a directional derivative.
    #[must_use]
    pub const fn direction(self) -> Option<UnitDirection<D>> {
        match self {
            Self::Value { .. } => None,
            Self::DirectionalDerivative { direction, .. } => Some(direction),
        }
    }

    /// Returns the Cartesian derivative order demanded by this atom.
    #[must_use]
    pub const fn derivative_order(self) -> KernelDerivativeOrder {
        match self {
            Self::Value { .. } => KernelDerivativeOrder::Value,
            Self::DirectionalDerivative { .. } => KernelDerivativeOrder::First,
        }
    }
}

/// A finite scalar coefficient paired with one atomic functional.
#[derive(Clone, Copy, Debug, PartialEq)]
#[must_use]
pub struct FunctionalTerm<const D: usize>
where
    Dim<D>: SupportedDimension,
{
    coefficient: f64,
    atom: FunctionalAtom<D>,
}

impl<const D: usize> FunctionalTerm<D>
where
    Dim<D>: SupportedDimension,
{
    /// Constructs a term after validating its scalar coefficient.
    ///
    /// # Errors
    ///
    /// Returns [`FunctionalError::NonFiniteCoefficient`] for NaN or infinity.
    pub fn try_new(coefficient: f64, atom: FunctionalAtom<D>) -> Result<Self, FunctionalError> {
        if !coefficient.is_finite() {
            return Err(FunctionalError::NonFiniteCoefficient { value: coefficient });
        }
        Ok(Self { coefficient, atom })
    }

    /// Returns the finite scalar coefficient.
    #[must_use]
    pub const fn coefficient(self) -> f64 {
        self.coefficient
    }

    /// Returns the atomic functional.
    pub const fn atom(self) -> FunctionalAtom<D> {
        self.atom
    }
}

/// Validated scalar-field value and Cartesian gradient at one point.
#[derive(Clone, Copy, Debug, PartialEq)]
#[must_use]
pub struct ScalarFieldSample<const D: usize>
where
    Dim<D>: SupportedDimension,
{
    value: f64,
    gradient: [f64; D],
}

impl<const D: usize> ScalarFieldSample<D>
where
    Dim<D>: SupportedDimension,
{
    /// Constructs a sample after validating every supplied scalar.
    ///
    /// # Errors
    ///
    /// Returns a structured error for a non-finite value or gradient component.
    pub fn try_new(value: f64, gradient: [f64; D]) -> Result<Self, FunctionalError> {
        if !value.is_finite() {
            return Err(FunctionalError::NonFiniteSampleValue { value });
        }
        for (axis, component) in gradient.iter().copied().enumerate() {
            if !component.is_finite() {
                return Err(FunctionalError::NonFiniteSampleGradient {
                    axis,
                    value: component,
                });
            }
        }
        Ok(Self { value, gradient })
    }

    /// Returns the finite scalar-field value.
    #[must_use]
    pub const fn value(self) -> f64 {
        self.value
    }

    /// Returns the finite Cartesian gradient.
    #[must_use]
    pub const fn gradient(self) -> [f64; D] {
        self.gradient
    }
}

/// Storage whose allocation failed during functional construction or action.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum FunctionalStorage {
    /// Expression terms.
    Terms,
    /// Polynomial action output.
    PolynomialOutput,
    /// Scratch values for polynomial evaluation.
    PolynomialValues,
    /// Scratch gradients for polynomial evaluation.
    PolynomialGradients,
}

/// Error returned while constructing or applying a functional expression.
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub enum FunctionalError {
    /// An expression contained no terms.
    EmptyExpression,
    /// A term coefficient was NaN or infinite.
    NonFiniteCoefficient {
        /// Rejected coefficient.
        value: f64,
    },
    /// A scalar-field sample value was NaN or infinite.
    NonFiniteSampleValue {
        /// Rejected value.
        value: f64,
    },
    /// A scalar-field sample gradient component was NaN or infinite.
    NonFiniteSampleGradient {
        /// Cartesian axis.
        axis: usize,
        /// Rejected component.
        value: f64,
    },
    /// Storage could not be reserved.
    AllocationFailed {
        /// Kind of requested storage.
        storage: FunctionalStorage,
        /// Exact or minimum number of entries requested.
        requested: usize,
    },
    /// Caller-provided samples did not match the expression term count.
    SampleLengthMismatch {
        /// Required sample count.
        expected: usize,
        /// Supplied sample count.
        actual: usize,
    },
    /// Applying one term or accumulating the expression produced a non-finite result.
    NonFiniteAction {
        /// Term index in deterministic insertion order.
        term_index: usize,
        /// Preserved caller provenance for that term.
        provenance: FunctionalProvenance,
    },
    /// Evaluating the polynomial basis for one term failed.
    PolynomialEvaluation {
        /// Term index in deterministic insertion order.
        term_index: usize,
        /// Preserved caller provenance for that term.
        provenance: FunctionalProvenance,
        /// Polynomial evaluation diagnostic.
        source: PolynomialSpaceError,
    },
    /// Applying a term to one polynomial basis member produced a non-finite result.
    NonFinitePolynomialAction {
        /// Term index in deterministic insertion order.
        term_index: usize,
        /// Preserved caller provenance for that term.
        provenance: FunctionalProvenance,
        /// Polynomial basis index in deterministic basis order.
        basis_index: usize,
    },
}

impl fmt::Display for FunctionalError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyExpression => formatter.write_str("functional expression must not be empty"),
            Self::NonFiniteCoefficient { value } => {
                write!(
                    formatter,
                    "functional coefficient must be finite, got {value}"
                )
            }
            Self::NonFiniteSampleValue { value } => {
                write!(
                    formatter,
                    "scalar-field sample value must be finite, got {value}"
                )
            }
            Self::NonFiniteSampleGradient { axis, value } => write!(
                formatter,
                "scalar-field sample gradient on axis {axis} must be finite, got {value}"
            ),
            Self::AllocationFailed { storage, requested } => write!(
                formatter,
                "could not reserve {storage:?} storage for {requested} entries"
            ),
            Self::SampleLengthMismatch { expected, actual } => write!(
                formatter,
                "functional expression requires {expected} samples, got {actual}"
            ),
            Self::NonFiniteAction {
                term_index,
                provenance,
            } => write!(
                formatter,
                "functional term {term_index} (provenance {}) produced a non-finite action",
                provenance.identifier()
            ),
            Self::PolynomialEvaluation {
                term_index,
                provenance,
                source,
            } => write!(
                formatter,
                "functional term {term_index} (provenance {}) could not evaluate the polynomial basis: {source}",
                provenance.identifier()
            ),
            Self::NonFinitePolynomialAction {
                term_index,
                provenance,
                basis_index,
            } => write!(
                formatter,
                "functional term {term_index} (provenance {}) produced a non-finite action for polynomial basis term {basis_index}",
                provenance.identifier()
            ),
        }
    }
}

impl Error for FunctionalError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::PolynomialEvaluation { source, .. } => Some(source),
            _ => None,
        }
    }
}

/// A nonempty finite linear expression of atomic functionals.
///
/// Term insertion order and provenance are preserved exactly. Construction
/// and actions return structured errors instead of partially successful
/// values.
#[derive(Clone, Debug, PartialEq)]
#[must_use]
pub struct FunctionalExpr<const D: usize>
where
    Dim<D>: SupportedDimension,
{
    terms: Vec<FunctionalTerm<D>>,
}

impl<const D: usize> FunctionalExpr<D>
where
    Dim<D>: SupportedDimension,
{
    /// Constructs a nonempty expression from terms in iterator order.
    ///
    /// # Errors
    ///
    /// Returns [`FunctionalError::EmptyExpression`] when no term is supplied,
    /// or [`FunctionalError::AllocationFailed`] when term storage cannot be
    /// reserved. Any partial local construction is discarded on error.
    pub fn try_new(
        terms: impl IntoIterator<Item = FunctionalTerm<D>>,
    ) -> Result<Self, FunctionalError> {
        let iterator = terms.into_iter();
        let minimum = iterator.size_hint().0;
        let mut stored = Vec::new();
        if minimum > 0 {
            stored
                .try_reserve_exact(minimum)
                .map_err(|_| FunctionalError::AllocationFailed {
                    storage: FunctionalStorage::Terms,
                    requested: minimum,
                })?;
        }
        for term in iterator {
            if stored.len() == stored.capacity() {
                let requested = stored.len().saturating_add(1);
                stored
                    .try_reserve(1)
                    .map_err(|_| FunctionalError::AllocationFailed {
                        storage: FunctionalStorage::Terms,
                        requested,
                    })?;
            }
            stored.push(term);
        }
        if stored.is_empty() {
            return Err(FunctionalError::EmptyExpression);
        }
        Ok(Self { terms: stored })
    }

    /// Borrows terms in deterministic insertion order.
    pub fn terms(&self) -> &[FunctionalTerm<D>] {
        &self.terms
    }

    /// Returns the number of terms.
    #[must_use]
    pub fn term_count(&self) -> usize {
        self.terms.len()
    }

    /// Returns the maximum derivative order demanded by any term.
    #[must_use]
    pub fn maximum_derivative_order(&self) -> KernelDerivativeOrder {
        self.terms
            .iter()
            .map(|term| term.atom.derivative_order())
            .max()
            .unwrap_or(KernelDerivativeOrder::Value)
    }

    /// Applies each term to its corresponding validated scalar-field sample.
    ///
    /// Sample `i` must describe the field at the point of term `i`. This
    /// explicit alignment avoids hidden point lookup or caching policy.
    ///
    /// # Errors
    ///
    /// Returns a length mismatch or a provenance-bearing non-finite action.
    pub fn try_apply_samples(
        &self,
        samples: &[ScalarFieldSample<D>],
    ) -> Result<f64, FunctionalError> {
        if samples.len() != self.terms.len() {
            return Err(FunctionalError::SampleLengthMismatch {
                expected: self.terms.len(),
                actual: samples.len(),
            });
        }

        let mut result = 0.0;
        for (term_index, (term, sample)) in self.terms.iter().zip(samples).enumerate() {
            let action = atom_sample_action(term.atom, *sample);
            let next = result + term.coefficient * action;
            if !action.is_finite() || !next.is_finite() {
                return Err(FunctionalError::NonFiniteAction {
                    term_index,
                    provenance: term.atom.provenance(),
                });
            }
            result = next;
        }
        Ok(result)
    }

    /// Applies the expression to every basis member of a polynomial space.
    ///
    /// The returned vector follows [`PolynomialSpace::terms`] order. Scratch
    /// storage is allocated once per call and reused across all expression
    /// terms; no allocation occurs per basis member or expression term.
    ///
    /// # Errors
    ///
    /// Returns structured allocation, polynomial-evaluation, or non-finite
    /// action diagnostics with the originating term provenance.
    pub fn try_apply_polynomial(
        &self,
        space: &PolynomialSpace<D>,
    ) -> Result<Vec<f64>, FunctionalError> {
        let count = space.term_count();
        let mut output = try_zeroed_vec(count, 0.0, FunctionalStorage::PolynomialOutput)?;
        let mut values = try_zeroed_vec(count, 0.0, FunctionalStorage::PolynomialValues)?;
        let mut gradients =
            try_zeroed_vec(count, [0.0; D], FunctionalStorage::PolynomialGradients)?;

        for (term_index, term) in self.terms.iter().enumerate() {
            let provenance = term.atom.provenance();
            match term.atom {
                FunctionalAtom::Value { point, .. } => space
                    .try_evaluate_values(point, &mut values)
                    .map_err(|source| FunctionalError::PolynomialEvaluation {
                        term_index,
                        provenance,
                        source,
                    })?,
                FunctionalAtom::DirectionalDerivative { point, .. } => space
                    .try_evaluate_gradients(point, &mut gradients)
                    .map_err(|source| FunctionalError::PolynomialEvaluation {
                        term_index,
                        provenance,
                        source,
                    })?,
            }

            for basis_index in 0..count {
                let action = match term.atom {
                    FunctionalAtom::Value { .. } => values[basis_index],
                    FunctionalAtom::DirectionalDerivative { direction, .. } => {
                        dot(*direction.components(), gradients[basis_index])
                    }
                };
                let next = output[basis_index] + term.coefficient * action;
                if !action.is_finite() || !next.is_finite() {
                    return Err(FunctionalError::NonFinitePolynomialAction {
                        term_index,
                        provenance,
                        basis_index,
                    });
                }
                output[basis_index] = next;
            }
        }
        Ok(output)
    }
}

/// An observation-side functional expression.
///
/// This is intentionally not interchangeable with [`CenterRepresenter`]. It
/// contains no observation relation, enforcement, geological semantics, or
/// solver row.
#[derive(Clone, Debug, PartialEq)]
#[must_use]
pub struct ObservationFunctional<const D: usize>
where
    Dim<D>: SupportedDimension,
{
    expression: FunctionalExpr<D>,
}

impl<const D: usize> ObservationFunctional<D>
where
    Dim<D>: SupportedDimension,
{
    /// Wraps an expression for use on the observation/query side.
    pub const fn new(expression: FunctionalExpr<D>) -> Self {
        Self { expression }
    }

    /// Borrows the underlying expression.
    pub const fn expression(&self) -> &FunctionalExpr<D> {
        &self.expression
    }

    /// Consumes the wrapper and returns its expression.
    pub fn into_expression(self) -> FunctionalExpr<D> {
        self.expression
    }

    /// Applies this observation expression to a distinct center representer.
    ///
    /// The evaluator is called once for every observation-term/center-term
    /// pair with their validated points and exact combined derivative demand.
    /// It must return a Cartesian kernel-jet prefix for exactly that
    /// query-center ordering. The shared kernel calculus then supplies the
    /// query/center signs used here.
    ///
    /// # Errors
    ///
    /// Returns [`KernelActionError::Evaluation`] with both term provenances
    /// when the evaluator fails, [`KernelActionError::InsufficientDerivativeOrder`]
    /// when its jet prefix does not satisfy the stated demand, or
    /// [`KernelActionError::NonFiniteAction`] when contraction, coefficient
    /// multiplication, or accumulation is not finitely representable.
    pub fn try_apply_kernel<E>(
        &self,
        center: &CenterRepresenter<D>,
        mut evaluator: impl FnMut(
            Point<D>,
            Point<D>,
            KernelDerivativeOrder,
        ) -> Result<SpatialKernelJetPrefix<D>, E>,
    ) -> Result<f64, KernelActionError<E>> {
        let mut result = 0.0;
        for (observation_term_index, observation_term) in self.expression.terms.iter().enumerate() {
            for (center_term_index, center_term) in center.expression.terms.iter().enumerate() {
                let observation_provenance = observation_term.atom.provenance();
                let center_provenance = center_term.atom.provenance();
                let demanded = atom_pair_derivative_order(observation_term.atom, center_term.atom);
                let jet = evaluator(
                    observation_term.atom.point(),
                    center_term.atom.point(),
                    demanded,
                )
                .map_err(|source| KernelActionError::Evaluation {
                    observation_term_index,
                    observation_provenance,
                    center_term_index,
                    center_provenance,
                    source,
                })?;
                let action = atom_kernel_action(observation_term.atom, center_term.atom, &jet)
                    .ok_or(KernelActionError::InsufficientDerivativeOrder {
                        observation_term_index,
                        observation_provenance,
                        center_term_index,
                        center_provenance,
                        demanded,
                        available_through: jet.available_through(),
                    })?;
                let weighted = observation_term.coefficient * center_term.coefficient * action;
                let next = result + weighted;
                if !action.is_finite() || !weighted.is_finite() || !next.is_finite() {
                    return Err(KernelActionError::NonFiniteAction {
                        observation_term_index,
                        observation_provenance,
                        center_term_index,
                        center_provenance,
                    });
                }
                result = next;
            }
        }
        Ok(result)
    }
}

/// A center-side representer expression.
///
/// This type remains distinct from [`ObservationFunctional`] even when both
/// contain equal atoms and points.
#[derive(Clone, Debug, PartialEq)]
#[must_use]
pub struct CenterRepresenter<const D: usize>
where
    Dim<D>: SupportedDimension,
{
    expression: FunctionalExpr<D>,
}

impl<const D: usize> CenterRepresenter<D>
where
    Dim<D>: SupportedDimension,
{
    /// Wraps an expression for use on the center argument.
    pub const fn new(expression: FunctionalExpr<D>) -> Self {
        Self { expression }
    }

    /// Borrows the underlying expression.
    pub const fn expression(&self) -> &FunctionalExpr<D> {
        &self.expression
    }

    /// Consumes the wrapper and returns its expression.
    pub fn into_expression(self) -> FunctionalExpr<D> {
        self.expression
    }
}

/// Error returned while applying an observation expression to a center representer.
#[derive(Debug)]
pub enum KernelActionError<E> {
    /// Cartesian kernel-jet evaluation failed for one term pair.
    Evaluation {
        /// Observation term index in insertion order.
        observation_term_index: usize,
        /// Observation term provenance.
        observation_provenance: FunctionalProvenance,
        /// Center term index in insertion order.
        center_term_index: usize,
        /// Center term provenance.
        center_provenance: FunctionalProvenance,
        /// Evaluator diagnostic.
        source: E,
    },
    /// The evaluator returned fewer Cartesian derivatives than the atom pair demands.
    InsufficientDerivativeOrder {
        /// Observation term index in insertion order.
        observation_term_index: usize,
        /// Observation term provenance.
        observation_provenance: FunctionalProvenance,
        /// Center term index in insertion order.
        center_term_index: usize,
        /// Center term provenance.
        center_provenance: FunctionalProvenance,
        /// Exact derivative order demanded by the atom pair.
        demanded: KernelDerivativeOrder,
        /// Highest derivative order carried by the returned jet prefix.
        available_through: KernelDerivativeOrder,
    },
    /// Contraction, weighting, or accumulation was not finitely representable.
    NonFiniteAction {
        /// Observation term index in insertion order.
        observation_term_index: usize,
        /// Observation term provenance.
        observation_provenance: FunctionalProvenance,
        /// Center term index in insertion order.
        center_term_index: usize,
        /// Center term provenance.
        center_provenance: FunctionalProvenance,
    },
}

impl<E> fmt::Display for KernelActionError<E>
where
    E: fmt::Display,
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Evaluation {
                observation_term_index,
                observation_provenance,
                center_term_index,
                center_provenance,
                source,
            } => write!(
                formatter,
                "kernel evaluation failed for observation term {observation_term_index} (provenance {}) and center term {center_term_index} (provenance {}): {source}",
                observation_provenance.identifier(),
                center_provenance.identifier()
            ),
            Self::InsufficientDerivativeOrder {
                observation_term_index,
                observation_provenance,
                center_term_index,
                center_provenance,
                demanded,
                available_through,
            } => write!(
                formatter,
                "kernel evaluation returned derivatives through {available_through:?}, but observation term {observation_term_index} (provenance {}) and center term {center_term_index} (provenance {}) demand {demanded:?}",
                observation_provenance.identifier(),
                center_provenance.identifier()
            ),
            Self::NonFiniteAction {
                observation_term_index,
                observation_provenance,
                center_term_index,
                center_provenance,
            } => write!(
                formatter,
                "kernel action is not finite for observation term {observation_term_index} (provenance {}) and center term {center_term_index} (provenance {})",
                observation_provenance.identifier(),
                center_provenance.identifier()
            ),
        }
    }
}

impl<E> Error for KernelActionError<E>
where
    E: Error + 'static,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Evaluation { source, .. } => Some(source),
            Self::InsufficientDerivativeOrder { .. } | Self::NonFiniteAction { .. } => None,
        }
    }
}

fn atom_sample_action<const D: usize>(atom: FunctionalAtom<D>, sample: ScalarFieldSample<D>) -> f64
where
    Dim<D>: SupportedDimension,
{
    match atom {
        FunctionalAtom::Value { .. } => sample.value,
        FunctionalAtom::DirectionalDerivative { direction, .. } => {
            dot(*direction.components(), sample.gradient)
        }
    }
}

fn atom_kernel_action<const D: usize>(
    observation: FunctionalAtom<D>,
    center: FunctionalAtom<D>,
    jet: &SpatialKernelJetPrefix<D>,
) -> Option<f64>
where
    Dim<D>: SupportedDimension,
{
    match (observation.direction(), center.direction()) {
        (None, None) => Some(jet.value()),
        (Some(direction), None) => Some(dot(
            *direction.components(),
            jet.first_derivative(KernelArgument::Query)?,
        )),
        (None, Some(direction)) => Some(dot(
            *direction.components(),
            jet.first_derivative(KernelArgument::Center)?,
        )),
        (Some(observation_direction), Some(center_direction)) => {
            let hessian = jet.second_derivative([KernelArgument::Query, KernelArgument::Center])?;
            let mut result = 0.0;
            for (row, observation_component) in observation_direction
                .components()
                .iter()
                .copied()
                .enumerate()
            {
                for (column, center_component) in
                    center_direction.components().iter().copied().enumerate()
                {
                    result += observation_component * hessian[row][column] * center_component;
                }
            }
            Some(result)
        }
    }
}

fn atom_pair_derivative_order<const D: usize>(
    observation: FunctionalAtom<D>,
    center: FunctionalAtom<D>,
) -> KernelDerivativeOrder
where
    Dim<D>: SupportedDimension,
{
    match (observation.direction(), center.direction()) {
        (None, None) => KernelDerivativeOrder::Value,
        (Some(_), None) | (None, Some(_)) => KernelDerivativeOrder::First,
        (Some(_), Some(_)) => KernelDerivativeOrder::Second,
    }
}

fn dot<const D: usize>(left: [f64; D], right: [f64; D]) -> f64 {
    left.into_iter()
        .zip(right)
        .map(|(left, right)| left * right)
        .sum()
}

fn try_zeroed_vec<T: Clone>(
    count: usize,
    zero: T,
    storage: FunctionalStorage,
) -> Result<Vec<T>, FunctionalError> {
    let mut values = Vec::new();
    values
        .try_reserve_exact(count)
        .map_err(|_| FunctionalError::AllocationFailed {
            storage,
            requested: count,
        })?;
    values.resize(count, zero);
    Ok(values)
}
