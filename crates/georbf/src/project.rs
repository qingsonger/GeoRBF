//! Deterministic ownership of multiple independent fitted scalar fields.
//!
//! Every project field is fitted through the same [`crate::FieldProblem`]
//! and [`FittedField`] core used outside a project. The project layer owns no
//! assembly, solver, or evaluation mathematics: it retains fields, validates
//! stable identifiers, and resolves typed reference-field inputs.
//!
//! Projects are available only in the supported dimensions:
//!
//! ```compile_fail
//! use georbf::GeoProject;
//!
//! fn unsupported(_: Option<GeoProject<4>>) {}
//! ```

use std::error::Error;
use std::fmt;

use crate::dimension::{Dim, SupportedDimension};
use crate::geometry::{Point, Vector};
use crate::model::{
    FittedField, FittedFieldEvaluation, FittedFieldEvaluationError,
    FittedFieldSecondOrderEvaluation,
};

/// Stable caller-controlled identifier for one scalar field in a project.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[must_use]
pub struct FieldId(u64);

impl FieldId {
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

/// One identified fitted scalar field owned by a [`GeoProject`].
#[derive(Clone, Debug, PartialEq)]
#[must_use]
pub struct ProjectField<const D: usize>
where
    Dim<D>: SupportedDimension,
{
    id: FieldId,
    field: FittedField<D>,
}

impl<const D: usize> ProjectField<D>
where
    Dim<D>: SupportedDimension,
{
    /// Associates a stable project identifier with an independently fitted field.
    pub fn new(id: FieldId, field: FittedField<D>) -> Self {
        Self { id, field }
    }

    /// Returns the stable field identifier.
    pub const fn id(&self) -> FieldId {
        self.id
    }

    /// Borrows the independently fitted scalar field.
    pub const fn field(&self) -> &FittedField<D> {
        &self.field
    }

    /// Consumes the entry and returns its identifier and fitted field.
    pub fn into_parts(self) -> (FieldId, FittedField<D>) {
        (self.id, self.field)
    }
}

/// Project allocation category retained by structured construction failures.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum GeoProjectStorage {
    /// Deterministic project-field entries.
    Fields,
}

/// Structured construction or reference-resolution failure for a project.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GeoProjectError {
    /// A project cannot own zero scalar fields.
    EmptyFields,
    /// Two entries supplied the same stable field identifier.
    DuplicateFieldId {
        /// Repeated caller-controlled identifier.
        field_id: FieldId,
        /// Zero-based index of the first entry.
        first_index: usize,
        /// Zero-based index of the repeated entry.
        duplicate_index: usize,
    },
    /// A typed reference requested an identifier not owned by the project.
    UnknownReferenceField {
        /// Missing caller-controlled identifier.
        field_id: FieldId,
    },
    /// Checked project storage allocation failed.
    AllocationFailed {
        /// Storage category that could not be allocated.
        storage: GeoProjectStorage,
        /// Requested minimum element count.
        requested: usize,
    },
}

impl fmt::Display for GeoProjectError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyFields => write!(formatter, "a GeoProject requires at least one field"),
            Self::DuplicateFieldId {
                field_id,
                first_index,
                duplicate_index,
            } => write!(
                formatter,
                "field identifier {} is duplicated at indices {first_index} and {duplicate_index}",
                field_id.identifier()
            ),
            Self::UnknownReferenceField { field_id } => write!(
                formatter,
                "reference field identifier {} is not present in the project",
                field_id.identifier()
            ),
            Self::AllocationFailed { storage, requested } => write!(
                formatter,
                "failed to allocate {requested} elements for project {storage:?} storage"
            ),
        }
    }
}

impl Error for GeoProjectError {}

/// Immutable deterministic owner of independent fitted scalar fields.
///
/// Entry order is the caller's insertion order. Fields may retain different
/// coordinate metadata, normalizations, kernels, and solver evidence; the
/// project neither reconciles those conventions nor introduces cross-field
/// constraints. A query through a reference input therefore uses the referenced
/// field's own original-coordinate convention.
#[derive(Clone, Debug, PartialEq)]
#[must_use]
pub struct GeoProject<const D: usize>
where
    Dim<D>: SupportedDimension,
{
    fields: Vec<ProjectField<D>>,
}

impl<const D: usize> GeoProject<D>
where
    Dim<D>: SupportedDimension,
{
    /// Validates and owns one or more independently fitted scalar fields.
    ///
    /// # Errors
    ///
    /// Returns a structured empty-project, duplicate-identifier, or allocation
    /// failure without returning a partial project.
    pub fn try_new(
        fields: impl IntoIterator<Item = ProjectField<D>>,
    ) -> Result<Self, GeoProjectError> {
        let iterator = fields.into_iter();
        let minimum = iterator.size_hint().0;
        let mut stored = Vec::new();
        stored
            .try_reserve_exact(minimum)
            .map_err(|_| GeoProjectError::AllocationFailed {
                storage: GeoProjectStorage::Fields,
                requested: minimum,
            })?;

        for field in iterator {
            let duplicate_index = stored.len();
            if let Some(first_index) = stored
                .iter()
                .position(|existing: &ProjectField<D>| existing.id() == field.id())
            {
                return Err(GeoProjectError::DuplicateFieldId {
                    field_id: field.id(),
                    first_index,
                    duplicate_index,
                });
            }
            if stored.len() == stored.capacity() {
                let requested = stored.len().saturating_add(1);
                stored
                    .try_reserve(1)
                    .map_err(|_| GeoProjectError::AllocationFailed {
                        storage: GeoProjectStorage::Fields,
                        requested,
                    })?;
            }
            stored.push(field);
        }

        if stored.is_empty() {
            return Err(GeoProjectError::EmptyFields);
        }
        Ok(Self { fields: stored })
    }

    /// Returns the number of independently owned fields.
    #[must_use]
    pub fn len(&self) -> usize {
        self.fields.len()
    }

    /// Reports whether the project contains no fields.
    ///
    /// A successfully constructed project always returns `false`; this method
    /// is supplied for ordinary collection-style inspection.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.fields.is_empty()
    }

    /// Borrows project entries in deterministic insertion order.
    pub fn fields(&self) -> &[ProjectField<D>] {
        &self.fields
    }

    /// Borrows an entry by stable identifier, or returns `None` when absent.
    #[must_use]
    pub fn entry(&self, field_id: FieldId) -> Option<&ProjectField<D>> {
        self.fields.iter().find(|entry| entry.id() == field_id)
    }

    /// Borrows an independently fitted field by stable identifier.
    pub fn field(&self, field_id: FieldId) -> Option<&FittedField<D>> {
        self.entry(field_id).map(ProjectField::field)
    }

    /// Resolves one typed input for future reference-controlled trend work.
    ///
    /// Resolution does not evaluate, couple, or refit any field and makes no
    /// geological-topology claim. The returned view delegates all requested
    /// scalar-field outputs to the retained immutable [`FittedField`].
    ///
    /// # Errors
    ///
    /// Returns [`GeoProjectError::UnknownReferenceField`] when the identifier
    /// is not present.
    pub fn try_reference_input(
        &self,
        field_id: FieldId,
    ) -> Result<ReferenceFieldInput<'_, D>, GeoProjectError> {
        self.field(field_id)
            .map(|field| ReferenceFieldInput { field_id, field })
            .ok_or(GeoProjectError::UnknownReferenceField { field_id })
    }

    /// Consumes the project and returns entries in deterministic insertion order.
    #[must_use]
    pub fn into_fields(self) -> Vec<ProjectField<D>> {
        self.fields
    }
}

/// Validated borrowed view of one project field for reference-controlled inputs.
///
/// This type is an identity and delegation boundary only. It does not define a
/// reference-field weighting function, local kernel mixture, topology, or
/// coordinate reprojection policy.
#[derive(Clone, Copy, Debug, PartialEq)]
#[must_use]
pub struct ReferenceFieldInput<'a, const D: usize>
where
    Dim<D>: SupportedDimension,
{
    field_id: FieldId,
    field: &'a FittedField<D>,
}

impl<'a, const D: usize> ReferenceFieldInput<'a, D>
where
    Dim<D>: SupportedDimension,
{
    /// Returns the resolved stable field identifier.
    pub const fn field_id(self) -> FieldId {
        self.field_id
    }

    /// Borrows the resolved immutable fitted field.
    pub const fn field(self) -> &'a FittedField<D> {
        self.field
    }

    /// Delegates scalar-value evaluation to the referenced field.
    ///
    /// # Errors
    ///
    /// Returns the referenced field's structured evaluation diagnostic.
    pub fn try_value(self, point: Point<D>) -> Result<f64, FittedFieldEvaluationError<D>> {
        self.field.try_value(point)
    }

    /// Delegates value-and-gradient evaluation to the referenced field.
    ///
    /// # Errors
    ///
    /// Returns the referenced field's structured evaluation diagnostic.
    pub fn try_evaluate(
        self,
        point: Point<D>,
    ) -> Result<FittedFieldEvaluation<D>, FittedFieldEvaluationError<D>> {
        self.field.try_evaluate(point)
    }

    /// Delegates value, gradient, and Hessian evaluation to the referenced field.
    ///
    /// # Errors
    ///
    /// Returns the referenced field's structured evaluation diagnostic.
    pub fn try_evaluate_with_hessian(
        self,
        point: Point<D>,
    ) -> Result<FittedFieldSecondOrderEvaluation<D>, FittedFieldEvaluationError<D>> {
        self.field.try_evaluate_with_hessian(point)
    }

    /// Delegates Cartesian-gradient evaluation to the referenced field.
    ///
    /// # Errors
    ///
    /// Returns the referenced field's structured evaluation diagnostic.
    pub fn try_gradient(self, point: Point<D>) -> Result<Vector<D>, FittedFieldEvaluationError<D>> {
        self.field.try_gradient(point)
    }

    /// Delegates Cartesian-Hessian evaluation to the referenced field.
    ///
    /// # Errors
    ///
    /// Returns the referenced field's structured evaluation diagnostic.
    pub fn try_hessian(
        self,
        point: Point<D>,
    ) -> Result<[[f64; D]; D], FittedFieldEvaluationError<D>> {
        self.field.try_hessian(point)
    }
}
