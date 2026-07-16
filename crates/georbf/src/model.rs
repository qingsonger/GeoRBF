//! Immutable fitted scalar fields and original-coordinate evaluation.
//!
//! A fitted field is available only in the supported dimensions:
//!
//! ```compile_fail
//! use georbf::FittedField;
//!
//! fn unsupported(_: Option<FittedField<4>>) {}
//! ```

use std::error::Error;
use std::fmt;

use crate::anisotropy::{AnisotropyError, GlobalAnisotropy};
use crate::coordinates::CoordinateMetadata;
use crate::dimension::{Dim, SupportedDimension};
use crate::field::{FieldAssemblyDiagnostics, FieldAssemblyError, FieldProblem};
use crate::functional::{CenterRepresenter, FunctionalAtom, FunctionalProvenance};
use crate::geometry::{Point, Vector};
use crate::kernel::{
    Gaussian, InverseMultiquadric, KernelDerivativeCapability, KernelDerivativeOrder,
    KernelMetadata, Matern, Multiquadric, PolyharmonicSpline, PolyharmonicSplineEvaluationError,
    SmoothKernelEvaluationError, SurfaceSpline, Wendland, WendlandEvaluationError,
};
use crate::kernel_calculus::{
    KernelArgument, KernelCalculusError, RadialJet, RadialSeparation, SpatialKernelJet,
    SpatialKernelJetPrefix,
};
use crate::polynomial::{PolynomialSpace, PolynomialSpaceError};
use crate::solver::{DenseSolveDiagnostics, DenseSolveError, DenseSolveOptions, try_solve_field};
use crate::transform::{AffineNormalization, TransformError};

/// One concrete configured kernel that can be retained by a fitted model.
///
/// Variant order and configured scalar values are stable deterministic model
/// inputs. This enum is not a persistence schema; versioned schemas and
/// migrations remain deferred to `REQ-SCHEMA-001`.
#[derive(Clone, Copy, Debug, PartialEq)]
#[must_use]
pub enum KernelDefinition<const D: usize>
where
    Dim<D>: SupportedDimension,
{
    /// Gaussian kernel.
    Gaussian(Gaussian),
    /// Inverse multiquadric kernel.
    InverseMultiquadric(InverseMultiquadric),
    /// CPD-positive signed multiquadric kernel.
    Multiquadric(Multiquadric),
    /// Supported half-integer Matérn kernel.
    Matern(Matern),
    /// Integer-power polyharmonic spline.
    PolyharmonicSpline(PolyharmonicSpline),
    /// Dimension-specific surface spline.
    SurfaceSpline(SurfaceSpline<D>),
    /// Compactly supported Wendland kernel.
    Wendland(Wendland),
}

impl<const D: usize> KernelDefinition<D>
where
    Dim<D>: SupportedDimension,
{
    /// Returns the exact metadata paired with this configured kernel.
    pub const fn metadata(self) -> KernelMetadata<'static> {
        match self {
            Self::Gaussian(kernel) => kernel.metadata(),
            Self::InverseMultiquadric(kernel) => kernel.metadata(),
            Self::Multiquadric(kernel) => kernel.metadata(),
            Self::Matern(kernel) => kernel.metadata(),
            Self::PolyharmonicSpline(kernel) => kernel.metadata(),
            Self::SurfaceSpline(kernel) => kernel.metadata(),
            Self::Wendland(kernel) => kernel.metadata(),
        }
    }

    fn try_radial_derivative(
        self,
        radius: f64,
        order: KernelDerivativeOrder,
    ) -> Result<Option<f64>, KernelDefinitionEvaluationError<D>> {
        match self {
            Self::Gaussian(kernel) => kernel
                .radial_derivative(radius, order)
                .map_err(KernelDefinitionEvaluationError::Smooth),
            Self::InverseMultiquadric(kernel) => kernel
                .radial_derivative(radius, order)
                .map_err(KernelDefinitionEvaluationError::Smooth),
            Self::Multiquadric(kernel) => kernel
                .radial_derivative(radius, order)
                .map_err(KernelDefinitionEvaluationError::Smooth),
            Self::Matern(kernel) => kernel
                .radial_derivative(radius, order)
                .map_err(KernelDefinitionEvaluationError::Smooth),
            Self::PolyharmonicSpline(kernel) => kernel
                .radial_derivative(radius, order)
                .map_err(KernelDefinitionEvaluationError::Polyharmonic),
            Self::SurfaceSpline(kernel) => kernel
                .radial_derivative(radius, order)
                .map_err(KernelDefinitionEvaluationError::Polyharmonic),
            Self::Wendland(kernel) => kernel
                .radial_derivative(radius, order)
                .map_err(KernelDefinitionEvaluationError::Wendland),
        }
    }

    fn try_radial_jet(
        self,
        separation: RadialSeparation<D>,
    ) -> Result<RadialJet, KernelDefinitionEvaluationError<D>> {
        match self {
            Self::Gaussian(kernel) => kernel
                .radial_jet(separation)
                .map_err(KernelDefinitionEvaluationError::Smooth),
            Self::InverseMultiquadric(kernel) => kernel
                .radial_jet(separation)
                .map_err(KernelDefinitionEvaluationError::Smooth),
            Self::Multiquadric(kernel) => kernel
                .radial_jet(separation)
                .map_err(KernelDefinitionEvaluationError::Smooth),
            Self::Matern(kernel) => kernel
                .radial_jet(separation)
                .map_err(KernelDefinitionEvaluationError::Smooth),
            Self::PolyharmonicSpline(kernel) => kernel
                .radial_jet(separation)
                .map_err(KernelDefinitionEvaluationError::Polyharmonic),
            Self::SurfaceSpline(kernel) => kernel
                .radial_jet(separation)
                .map_err(KernelDefinitionEvaluationError::Polyharmonic),
            Self::Wendland(kernel) => kernel
                .radial_jet(separation)
                .map_err(KernelDefinitionEvaluationError::Wendland),
        }
    }

    fn try_spatial_jet(
        self,
        query: Point<D>,
        center: Point<D>,
        demanded: KernelDerivativeOrder,
        anisotropy: Option<&GlobalAnisotropy<D>>,
    ) -> Result<SpatialKernelJet<D>, KernelDefinitionEvaluationError<D>> {
        let separation = match anisotropy {
            Some(anisotropy) => anisotropy
                .try_transform_separation(query, center)
                .map_err(KernelDefinitionEvaluationError::Anisotropy)?,
            None => RadialSeparation::try_new(query, center)
                .map_err(KernelDefinitionEvaluationError::Calculus)?,
        };
        let capability = self.metadata().derivatives().capability(demanded);
        if capability == KernelDerivativeCapability::Unsupported
            || (separation.is_center()
                && capability == KernelDerivativeCapability::SupportedAwayFromCenters)
        {
            return Err(KernelDefinitionEvaluationError::UnavailableDerivative {
                demanded,
                at_center: separation.is_center(),
            });
        }

        let transformed = if separation.is_center() && demanded < KernelDerivativeOrder::Third {
            let value = self
                .try_radial_derivative(0.0, KernelDerivativeOrder::Value)?
                .ok_or(KernelDefinitionEvaluationError::UnavailableDerivative {
                    demanded: KernelDerivativeOrder::Value,
                    at_center: true,
                })?;
            let second = if demanded >= KernelDerivativeOrder::Second {
                self.try_radial_derivative(0.0, KernelDerivativeOrder::Second)?
                    .ok_or(KernelDefinitionEvaluationError::UnavailableDerivative {
                        demanded: KernelDerivativeOrder::Second,
                        at_center: true,
                    })?
            } else {
                0.0
            };
            SpatialKernelJet::from_query_derivatives(
                value,
                [0.0; D],
                std::array::from_fn(|row| {
                    std::array::from_fn(|column| if row == column { second } else { 0.0 })
                }),
                [[[0.0; D]; D]; D],
            )
        } else {
            let radial = self.try_radial_jet(separation)?;
            SpatialKernelJet::try_new(separation, radial)
                .map_err(KernelDefinitionEvaluationError::Calculus)?
        };

        match anisotropy {
            Some(anisotropy) => anisotropy
                .try_transform_spatial_jet(transformed)
                .map_err(KernelDefinitionEvaluationError::Anisotropy),
            None => Ok(transformed),
        }
    }

    fn try_assembly_prefix(
        self,
        query: Point<D>,
        center: Point<D>,
        demanded: KernelDerivativeOrder,
        anisotropy: Option<&GlobalAnisotropy<D>>,
    ) -> Result<SpatialKernelJetPrefix<D>, KernelDefinitionEvaluationError<D>> {
        if demanded == KernelDerivativeOrder::Third {
            return Err(KernelDefinitionEvaluationError::InvalidAssemblyDemand { demanded });
        }
        let jet = self.try_spatial_jet(query, center, demanded, anisotropy)?;
        let first = (demanded >= KernelDerivativeOrder::First)
            .then(|| jet.first_derivative(KernelArgument::Query));
        let second = (demanded >= KernelDerivativeOrder::Second)
            .then(|| jet.second_derivative([KernelArgument::Query, KernelArgument::Query]));
        Ok(SpatialKernelJetPrefix::from_query_derivatives(
            demanded,
            jet.value(),
            first,
            second,
        ))
    }
}

impl<const D: usize> From<Gaussian> for KernelDefinition<D>
where
    Dim<D>: SupportedDimension,
{
    fn from(kernel: Gaussian) -> Self {
        Self::Gaussian(kernel)
    }
}

impl<const D: usize> From<InverseMultiquadric> for KernelDefinition<D>
where
    Dim<D>: SupportedDimension,
{
    fn from(kernel: InverseMultiquadric) -> Self {
        Self::InverseMultiquadric(kernel)
    }
}

impl<const D: usize> From<Multiquadric> for KernelDefinition<D>
where
    Dim<D>: SupportedDimension,
{
    fn from(kernel: Multiquadric) -> Self {
        Self::Multiquadric(kernel)
    }
}

impl<const D: usize> From<Matern> for KernelDefinition<D>
where
    Dim<D>: SupportedDimension,
{
    fn from(kernel: Matern) -> Self {
        Self::Matern(kernel)
    }
}

impl<const D: usize> From<PolyharmonicSpline> for KernelDefinition<D>
where
    Dim<D>: SupportedDimension,
{
    fn from(kernel: PolyharmonicSpline) -> Self {
        Self::PolyharmonicSpline(kernel)
    }
}

impl<const D: usize> From<SurfaceSpline<D>> for KernelDefinition<D>
where
    Dim<D>: SupportedDimension,
{
    fn from(kernel: SurfaceSpline<D>) -> Self {
        Self::SurfaceSpline(kernel)
    }
}

impl<const D: usize> From<Wendland> for KernelDefinition<D>
where
    Dim<D>: SupportedDimension,
{
    fn from(kernel: Wendland) -> Self {
        Self::Wendland(kernel)
    }
}

/// Failure while evaluating one retained configured kernel.
#[derive(Clone, Debug, PartialEq)]
#[must_use]
pub enum KernelDefinitionEvaluationError<const D: usize>
where
    Dim<D>: SupportedDimension,
{
    /// Radial separation or Cartesian expansion failed.
    Calculus(KernelCalculusError),
    /// A smooth global kernel failed.
    Smooth(SmoothKernelEvaluationError),
    /// A polyharmonic or surface spline failed.
    Polyharmonic(PolyharmonicSplineEvaluationError),
    /// A Wendland kernel failed.
    Wendland(WendlandEvaluationError),
    /// Global anisotropy transformation failed.
    Anisotropy(AnisotropyError<D>),
    /// Metadata does not permit the demanded derivative at this location.
    UnavailableDerivative {
        /// Combined query and center derivative order.
        demanded: KernelDerivativeOrder,
        /// Whether the query and center coincide.
        at_center: bool,
    },
    /// Field assembly unexpectedly requested a third-order prefix.
    InvalidAssemblyDemand {
        /// Rejected demand.
        demanded: KernelDerivativeOrder,
    },
}

impl<const D: usize> fmt::Display for KernelDefinitionEvaluationError<D>
where
    Dim<D>: SupportedDimension,
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Calculus(source) => source.fmt(formatter),
            Self::Smooth(source) => source.fmt(formatter),
            Self::Polyharmonic(source) => source.fmt(formatter),
            Self::Wendland(source) => source.fmt(formatter),
            Self::Anisotropy(source) => source.fmt(formatter),
            Self::UnavailableDerivative {
                demanded,
                at_center,
            } => write!(
                formatter,
                "kernel derivative {demanded:?} is unavailable with at_center={at_center}"
            ),
            Self::InvalidAssemblyDemand { demanded } => write!(
                formatter,
                "field assembly cannot consume a {demanded:?} kernel-jet prefix"
            ),
        }
    }
}

impl<const D: usize> Error for KernelDefinitionEvaluationError<D>
where
    Dim<D>: SupportedDimension,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Calculus(source) => Some(source),
            Self::Smooth(source) => Some(source),
            Self::Polyharmonic(source) => Some(source),
            Self::Wendland(source) => Some(source),
            Self::Anisotropy(source) => Some(source),
            Self::UnavailableDerivative { .. } | Self::InvalidAssemblyDemand { .. } => None,
        }
    }
}

/// Fitted output whose availability is reported by model capabilities.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum FittedFieldOutput {
    /// Scalar value.
    Value,
    /// Cartesian gradient.
    Gradient,
    /// Cartesian Hessian.
    Hessian,
}

impl FittedFieldOutput {
    const fn derivative_order(self) -> KernelDerivativeOrder {
        match self {
            Self::Value => KernelDerivativeOrder::Value,
            Self::Gradient => KernelDerivativeOrder::First,
            Self::Hessian => KernelDerivativeOrder::Second,
        }
    }
}

/// Exact value, gradient, and Hessian capability classification.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[must_use]
pub struct FittedFieldCapabilities {
    value: KernelDerivativeCapability,
    gradient: KernelDerivativeCapability,
    hessian: KernelDerivativeCapability,
}

impl FittedFieldCapabilities {
    /// Returns scalar-value availability.
    #[must_use]
    pub const fn value(self) -> KernelDerivativeCapability {
        self.value
    }

    /// Returns gradient availability.
    #[must_use]
    pub const fn gradient(self) -> KernelDerivativeCapability {
        self.gradient
    }

    /// Returns Hessian availability.
    #[must_use]
    pub const fn hessian(self) -> KernelDerivativeCapability {
        self.hessian
    }

    /// Returns availability for one requested output.
    #[must_use]
    pub const fn for_output(self, output: FittedFieldOutput) -> KernelDerivativeCapability {
        match output {
            FittedFieldOutput::Value => self.value,
            FittedFieldOutput::Gradient => self.gradient,
            FittedFieldOutput::Hessian => self.hessian,
        }
    }
}

/// Immutable assembly and solve evidence retained by a fitted field.
#[derive(Clone, Debug, PartialEq)]
#[must_use]
pub struct FittedFieldDiagnostics {
    assembly: FieldAssemblyDiagnostics,
    solve: DenseSolveDiagnostics,
}

impl FittedFieldDiagnostics {
    /// Returns field-assembly symmetry and work-count evidence.
    pub const fn assembly(&self) -> FieldAssemblyDiagnostics {
        self.assembly
    }

    /// Borrows complete dense-solve numerical evidence.
    pub const fn solve(&self) -> &DenseSolveDiagnostics {
        &self.solve
    }
}

/// Original-coordinate scalar value and gradient.
#[derive(Clone, Copy, Debug, PartialEq)]
#[must_use]
pub struct FittedFieldEvaluation<const D: usize>
where
    Dim<D>: SupportedDimension,
{
    value: f64,
    gradient: Vector<D>,
}

impl<const D: usize> FittedFieldEvaluation<D>
where
    Dim<D>: SupportedDimension,
{
    /// Returns the scalar value.
    #[must_use]
    pub const fn value(self) -> f64 {
        self.value
    }

    /// Returns the Cartesian gradient in original coordinates.
    pub const fn gradient(self) -> Vector<D> {
        self.gradient
    }
}

/// Original-coordinate scalar value, gradient, and Hessian.
#[derive(Clone, Copy, Debug, PartialEq)]
#[must_use]
pub struct FittedFieldSecondOrderEvaluation<const D: usize>
where
    Dim<D>: SupportedDimension,
{
    value: f64,
    gradient: Vector<D>,
    hessian: [[f64; D]; D],
}

impl<const D: usize> FittedFieldSecondOrderEvaluation<D>
where
    Dim<D>: SupportedDimension,
{
    /// Returns the scalar value.
    #[must_use]
    pub const fn value(self) -> f64 {
        self.value
    }

    /// Returns the Cartesian gradient in original coordinates.
    pub const fn gradient(self) -> Vector<D> {
        self.gradient
    }

    /// Returns the Cartesian Hessian in original coordinates.
    #[must_use]
    pub const fn hessian(self) -> [[f64; D]; D] {
        self.hessian
    }
}

/// Immutable fitted scalar field.
///
/// The consumed [`FieldProblem`] must already use normalized model
/// coordinates for every functional point and direction. Public query points
/// are supplied in the retained original coordinate convention; the model
/// applies `x_tilde = S^-1 (x - mu)` and maps derivatives back through the
/// exact affine chain rule.
#[derive(Clone, Debug, PartialEq)]
#[must_use]
pub struct FittedField<const D: usize>
where
    Dim<D>: SupportedDimension,
{
    coordinate_metadata: CoordinateMetadata<D>,
    normalization: AffineNormalization<D>,
    kernel: KernelDefinition<D>,
    anisotropy: Option<GlobalAnisotropy<D>>,
    centers: Vec<CenterRepresenter<D>>,
    coefficients: Vec<f64>,
    center_count: usize,
    polynomial_space: Option<PolynomialSpace<D>>,
    capabilities: FittedFieldCapabilities,
    diagnostics: FittedFieldDiagnostics,
}

impl<const D: usize> FittedField<D>
where
    Dim<D>: SupportedDimension,
{
    /// Assembles, solves, and owns one immutable fitted field.
    ///
    /// The concrete kernel definition is used for both assembly and retained
    /// evaluation, preventing a callback/model mismatch. The problem is
    /// consumed, so the fitted model borrows no builder, semantic IR, dense
    /// system, or solver workspace.
    ///
    /// # Errors
    ///
    /// Returns structured assembly, kernel, solve, or coefficient-layout
    /// diagnostics. No hidden regularization, factorization switch,
    /// pseudoinverse, or derivative approximation is introduced.
    pub fn try_fit(
        problem: FieldProblem<D>,
        coordinate_metadata: CoordinateMetadata<D>,
        normalization: AffineNormalization<D>,
        kernel: KernelDefinition<D>,
        anisotropy: Option<GlobalAnisotropy<D>>,
        solve_options: DenseSolveOptions,
    ) -> Result<Self, FittedFieldFitError<D>> {
        let system = problem
            .try_assemble(kernel.metadata(), |query, center, demanded| {
                kernel.try_assembly_prefix(query, center, demanded, anisotropy.as_ref())
            })
            .map_err(|source| FittedFieldFitError::Assembly(Box::new(source)))?;
        let center_count = system.center_count();
        let polynomial_count = system.polynomial_count();
        let expected = center_count
            .checked_add(polynomial_count)
            .ok_or(FittedFieldFitError::CoefficientCountOverflow)?;
        let solution = try_solve_field(&system, solve_options)
            .map_err(|source| FittedFieldFitError::Solve(Box::new(source)))?;
        let (coefficients, solve) = solution.into_parts();
        if coefficients.len() != expected {
            return Err(FittedFieldFitError::CoefficientCountMismatch {
                expected,
                actual: coefficients.len(),
            });
        }
        let (assembly, polynomial_space) = system.into_model_parts();
        if polynomial_space
            .as_ref()
            .map_or(0, PolynomialSpace::term_count)
            != polynomial_count
        {
            return Err(FittedFieldFitError::PolynomialCountMismatch {
                expected: polynomial_count,
                actual: polynomial_space
                    .as_ref()
                    .map_or(0, PolynomialSpace::term_count),
            });
        }
        let centers = problem.into_centers();
        if centers.len() != center_count {
            return Err(FittedFieldFitError::CenterCountMismatch {
                expected: center_count,
                actual: centers.len(),
            });
        }
        let maximum_center_order = centers
            .iter()
            .flat_map(|center| center.expression().terms())
            .map(|term| term.atom().derivative_order())
            .max()
            .unwrap_or(KernelDerivativeOrder::Value);
        let derivatives = kernel.metadata().derivatives();
        let capabilities = FittedFieldCapabilities {
            value: derivatives.query_capability(KernelDerivativeOrder::Value, maximum_center_order),
            gradient: derivatives
                .query_capability(KernelDerivativeOrder::First, maximum_center_order),
            hessian: derivatives
                .query_capability(KernelDerivativeOrder::Second, maximum_center_order),
        };

        Ok(Self {
            coordinate_metadata,
            normalization,
            kernel,
            anisotropy,
            centers,
            coefficients,
            center_count,
            polynomial_space,
            capabilities,
            diagnostics: FittedFieldDiagnostics { assembly, solve },
        })
    }

    /// Borrows the original-coordinate convention metadata.
    pub const fn coordinate_metadata(&self) -> &CoordinateMetadata<D> {
        &self.coordinate_metadata
    }

    /// Returns the affine original-to-normalized coordinate map.
    pub const fn normalization(&self) -> AffineNormalization<D> {
        self.normalization
    }

    /// Returns the retained configured kernel.
    pub const fn kernel(&self) -> KernelDefinition<D> {
        self.kernel
    }

    /// Returns the optional constant global anisotropy definition.
    #[must_use]
    pub const fn anisotropy(&self) -> Option<GlobalAnisotropy<D>> {
        self.anisotropy
    }

    /// Borrows center representers in deterministic assembly order.
    pub fn centers(&self) -> &[CenterRepresenter<D>] {
        &self.centers
    }

    /// Borrows center weights in deterministic center order.
    #[must_use]
    pub fn center_weights(&self) -> &[f64] {
        &self.coefficients[..self.center_count]
    }

    /// Borrows the complete polynomial space when the kernel is CPD.
    #[must_use]
    pub const fn polynomial_space(&self) -> Option<&PolynomialSpace<D>> {
        self.polynomial_space.as_ref()
    }

    /// Borrows polynomial coefficients in deterministic basis order.
    #[must_use]
    pub fn polynomial_coefficients(&self) -> &[f64] {
        &self.coefficients[self.center_count..]
    }

    /// Returns exact value, gradient, and Hessian availability.
    pub const fn capabilities(&self) -> FittedFieldCapabilities {
        self.capabilities
    }

    /// Borrows retained assembly and solve evidence.
    pub const fn diagnostics(&self) -> &FittedFieldDiagnostics {
        &self.diagnostics
    }

    /// Returns the crate build version retained as a deterministic model input.
    #[must_use]
    pub const fn build_version(&self) -> &'static str {
        env!("CARGO_PKG_VERSION")
    }

    /// Returns a deterministic borrowed view for future schema encoders.
    ///
    /// The view exposes stable center, coefficient, and polynomial-basis order
    /// without claiming that a public file format, checksum, or migration
    /// policy exists.
    pub const fn record(&self) -> FittedFieldRecord<'_, D> {
        FittedFieldRecord { field: self }
    }

    /// Evaluates only the scalar value at one original-coordinate point.
    ///
    /// # Errors
    ///
    /// Returns structured transform, capability, kernel, polynomial,
    /// allocation, or finite-representation diagnostics.
    pub fn try_value(&self, point: Point<D>) -> Result<f64, FittedFieldEvaluationError<D>> {
        Ok(self
            .try_evaluate_normalized(point, FittedFieldOutput::Value)?
            .value)
    }

    /// Evaluates scalar value and Cartesian gradient in original coordinates.
    ///
    /// # Errors
    ///
    /// Returns structured transform, capability, kernel, polynomial,
    /// allocation, or finite-representation diagnostics.
    pub fn try_evaluate(
        &self,
        point: Point<D>,
    ) -> Result<FittedFieldEvaluation<D>, FittedFieldEvaluationError<D>> {
        let normalized = self.try_evaluate_normalized(point, FittedFieldOutput::Gradient)?;
        let gradient = self
            .normalization
            .gradient_to_original(Vector::try_new(normalized.gradient).map_err(|_| {
                FittedFieldEvaluationError::NonFiniteOutput {
                    component: FittedFieldComponent::Gradient { axis: 0 },
                }
            })?)
            .map_err(FittedFieldEvaluationError::Transform)?;
        Ok(FittedFieldEvaluation {
            value: normalized.value,
            gradient,
        })
    }

    /// Evaluates scalar value, gradient, and Hessian in original coordinates.
    ///
    /// # Errors
    ///
    /// Hessian requests additionally fail at exact query/center coincidences
    /// when the retained kernel or a directional-derivative center supplies
    /// only an away-from-center third derivative.
    pub fn try_evaluate_with_hessian(
        &self,
        point: Point<D>,
    ) -> Result<FittedFieldSecondOrderEvaluation<D>, FittedFieldEvaluationError<D>> {
        let normalized = self.try_evaluate_normalized(point, FittedFieldOutput::Hessian)?;
        let gradient = self
            .normalization
            .gradient_to_original(Vector::try_new(normalized.gradient).map_err(|_| {
                FittedFieldEvaluationError::NonFiniteOutput {
                    component: FittedFieldComponent::Gradient { axis: 0 },
                }
            })?)
            .map_err(FittedFieldEvaluationError::Transform)?;
        let hessian = self
            .normalization
            .hessian_to_original(normalized.hessian)
            .map_err(FittedFieldEvaluationError::Transform)?;
        Ok(FittedFieldSecondOrderEvaluation {
            value: normalized.value,
            gradient,
            hessian,
        })
    }

    /// Evaluates only the original-coordinate Cartesian gradient.
    ///
    /// # Errors
    ///
    /// Returns the same structured diagnostics as [`Self::try_evaluate`].
    pub fn try_gradient(
        &self,
        point: Point<D>,
    ) -> Result<Vector<D>, FittedFieldEvaluationError<D>> {
        self.try_evaluate(point)
            .map(FittedFieldEvaluation::gradient)
    }

    /// Evaluates only the original-coordinate Cartesian Hessian.
    ///
    /// # Errors
    ///
    /// Returns the same structured diagnostics as
    /// [`Self::try_evaluate_with_hessian`].
    pub fn try_hessian(
        &self,
        point: Point<D>,
    ) -> Result<[[f64; D]; D], FittedFieldEvaluationError<D>> {
        self.try_evaluate_with_hessian(point)
            .map(FittedFieldSecondOrderEvaluation::hessian)
    }

    #[allow(clippy::too_many_lines)]
    fn try_evaluate_normalized(
        &self,
        original_point: Point<D>,
        output: FittedFieldOutput,
    ) -> Result<NormalizedEvaluation<D>, FittedFieldEvaluationError<D>> {
        let overall_capability = self.capabilities.for_output(output);
        if overall_capability == KernelDerivativeCapability::Unsupported {
            return Err(FittedFieldEvaluationError::UnsupportedOutput {
                output,
                capability: overall_capability,
            });
        }
        let query = self
            .normalization
            .normalize_point(original_point)
            .map_err(FittedFieldEvaluationError::Transform)?;
        let mut evaluation = NormalizedEvaluation {
            value: 0.0,
            gradient: [0.0; D],
            hessian: [[0.0; D]; D],
        };

        for (center_index, (center, weight)) in
            self.centers.iter().zip(self.center_weights()).enumerate()
        {
            for (term_index, term) in center.expression().terms().iter().enumerate() {
                let atom = term.atom();
                let center_order = atom.derivative_order();
                let capability = self
                    .kernel
                    .metadata()
                    .derivatives()
                    .query_capability(output.derivative_order(), center_order);
                if capability == KernelDerivativeCapability::Unsupported {
                    return Err(FittedFieldEvaluationError::UnsupportedCenterTerm {
                        output,
                        center_index,
                        term_index,
                        provenance: atom.provenance(),
                    });
                }
                if capability == KernelDerivativeCapability::SupportedAwayFromCenters
                    && query == atom.point()
                {
                    return Err(FittedFieldEvaluationError::UnavailableAtCenter {
                        output,
                        center_index,
                        term_index,
                        provenance: atom.provenance(),
                    });
                }
                let demanded = combined_order(output.derivative_order(), center_order).ok_or(
                    FittedFieldEvaluationError::UnsupportedCenterTerm {
                        output,
                        center_index,
                        term_index,
                        provenance: atom.provenance(),
                    },
                )?;
                let jet = self
                    .kernel
                    .try_spatial_jet(query, atom.point(), demanded, self.anisotropy.as_ref())
                    .map_err(|source| FittedFieldEvaluationError::Kernel {
                        center_index,
                        term_index,
                        provenance: atom.provenance(),
                        source,
                    })?;
                let multiplier = *weight * term.coefficient();
                let value_action = center_value_action(atom, &jet);
                accumulate_center(
                    &mut evaluation.value,
                    multiplier,
                    value_action,
                    FittedFieldComponent::Value,
                    center_index,
                    term_index,
                )?;

                if output >= FittedFieldOutput::Gradient {
                    let gradient_action = center_gradient_action(atom, &jet);
                    for (axis, action) in gradient_action.into_iter().enumerate() {
                        accumulate_center(
                            &mut evaluation.gradient[axis],
                            multiplier,
                            action,
                            FittedFieldComponent::Gradient { axis },
                            center_index,
                            term_index,
                        )?;
                    }
                }
                if output >= FittedFieldOutput::Hessian {
                    let hessian_action = center_hessian_action(atom, &jet);
                    for (row, values) in hessian_action.into_iter().enumerate() {
                        for (column, action) in values.into_iter().enumerate() {
                            accumulate_center(
                                &mut evaluation.hessian[row][column],
                                multiplier,
                                action,
                                FittedFieldComponent::Hessian { row, column },
                                center_index,
                                term_index,
                            )?;
                        }
                    }
                }
            }
        }

        self.try_add_polynomial(query, output, &mut evaluation)?;
        Ok(evaluation)
    }

    fn try_add_polynomial(
        &self,
        query: Point<D>,
        output: FittedFieldOutput,
        evaluation: &mut NormalizedEvaluation<D>,
    ) -> Result<(), FittedFieldEvaluationError<D>> {
        let Some(space) = &self.polynomial_space else {
            return Ok(());
        };
        let count = space.term_count();
        let mut values = try_filled(count, 0.0, FittedFieldStorage::PolynomialValues)?;
        let mut gradients = if output >= FittedFieldOutput::Gradient {
            try_filled(count, [0.0; D], FittedFieldStorage::PolynomialGradients)?
        } else {
            Vec::new()
        };
        let mut hessians = if output >= FittedFieldOutput::Hessian {
            try_filled(count, [[0.0; D]; D], FittedFieldStorage::PolynomialHessians)?
        } else {
            Vec::new()
        };
        match output {
            FittedFieldOutput::Value => space
                .try_evaluate_values(query, &mut values)
                .map_err(FittedFieldEvaluationError::Polynomial)?,
            FittedFieldOutput::Gradient => space
                .try_evaluate(query, &mut values, &mut gradients)
                .map_err(FittedFieldEvaluationError::Polynomial)?,
            FittedFieldOutput::Hessian => space
                .try_evaluate_through_second(query, &mut values, &mut gradients, &mut hessians)
                .map_err(FittedFieldEvaluationError::Polynomial)?,
        }

        for (term_index, coefficient) in self.polynomial_coefficients().iter().copied().enumerate()
        {
            accumulate_polynomial(
                &mut evaluation.value,
                coefficient,
                values[term_index],
                FittedFieldComponent::Value,
                term_index,
            )?;
            if output >= FittedFieldOutput::Gradient {
                for (axis, action) in gradients[term_index].iter().copied().enumerate() {
                    accumulate_polynomial(
                        &mut evaluation.gradient[axis],
                        coefficient,
                        action,
                        FittedFieldComponent::Gradient { axis },
                        term_index,
                    )?;
                }
            }
            if output >= FittedFieldOutput::Hessian {
                for (row, values) in hessians[term_index].iter().enumerate() {
                    for (column, action) in values.iter().copied().enumerate() {
                        accumulate_polynomial(
                            &mut evaluation.hessian[row][column],
                            coefficient,
                            action,
                            FittedFieldComponent::Hessian { row, column },
                            term_index,
                        )?;
                    }
                }
            }
        }
        Ok(())
    }
}

/// Deterministic borrowed inputs for a future model-schema encoder.
#[derive(Clone, Copy, Debug, PartialEq)]
#[must_use]
pub struct FittedFieldRecord<'a, const D: usize>
where
    Dim<D>: SupportedDimension,
{
    field: &'a FittedField<D>,
}

impl<'a, const D: usize> FittedFieldRecord<'a, D>
where
    Dim<D>: SupportedDimension,
{
    /// Returns the crate build version.
    #[must_use]
    pub const fn build_version(self) -> &'static str {
        self.field.build_version()
    }

    /// Borrows coordinate metadata.
    pub const fn coordinate_metadata(self) -> &'a CoordinateMetadata<D> {
        self.field.coordinate_metadata()
    }

    /// Returns affine normalization.
    pub const fn normalization(self) -> AffineNormalization<D> {
        self.field.normalization()
    }

    /// Returns the configured kernel definition.
    pub const fn kernel(self) -> KernelDefinition<D> {
        self.field.kernel()
    }

    /// Returns optional global anisotropy.
    #[must_use]
    pub const fn anisotropy(self) -> Option<GlobalAnisotropy<D>> {
        self.field.anisotropy()
    }

    /// Borrows centers in deterministic order.
    pub fn centers(self) -> &'a [CenterRepresenter<D>] {
        self.field.centers()
    }

    /// Borrows center weights in deterministic order.
    #[must_use]
    pub fn center_weights(self) -> &'a [f64] {
        self.field.center_weights()
    }

    /// Borrows the complete polynomial space when present.
    #[must_use]
    pub const fn polynomial_space(self) -> Option<&'a PolynomialSpace<D>> {
        self.field.polynomial_space()
    }

    /// Borrows polynomial coefficients in deterministic basis order.
    #[must_use]
    pub fn polynomial_coefficients(self) -> &'a [f64] {
        self.field.polynomial_coefficients()
    }

    /// Returns retained output capabilities.
    pub const fn capabilities(self) -> FittedFieldCapabilities {
        self.field.capabilities()
    }

    /// Borrows assembly and solve diagnostics.
    pub const fn diagnostics(self) -> &'a FittedFieldDiagnostics {
        self.field.diagnostics()
    }
}

/// Failure while assembling and solving an immutable fitted field.
#[derive(Debug)]
#[must_use]
pub enum FittedFieldFitError<const D: usize>
where
    Dim<D>: SupportedDimension,
{
    /// Field assembly failed.
    Assembly(Box<FieldAssemblyError<KernelDefinitionEvaluationError<D>>>),
    /// Dense solve failed.
    Solve(Box<DenseSolveError>),
    /// Center plus polynomial count overflowed.
    CoefficientCountOverflow,
    /// Solver output length disagreed with assembled variables.
    CoefficientCountMismatch {
        /// Expected coefficient count.
        expected: usize,
        /// Actual solution count.
        actual: usize,
    },
    /// Retained polynomial space disagreed with assembly.
    PolynomialCountMismatch {
        /// Expected polynomial term count.
        expected: usize,
        /// Actual retained term count.
        actual: usize,
    },
    /// Consumed field problem disagreed with assembly.
    CenterCountMismatch {
        /// Expected center count.
        expected: usize,
        /// Actual retained center count.
        actual: usize,
    },
}

impl<const D: usize> fmt::Display for FittedFieldFitError<D>
where
    Dim<D>: SupportedDimension,
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Assembly(source) => source.fmt(formatter),
            Self::Solve(source) => source.fmt(formatter),
            Self::CoefficientCountOverflow => {
                formatter.write_str("fitted-field coefficient count overflowed")
            }
            Self::CoefficientCountMismatch { expected, actual } => write!(
                formatter,
                "fitted-field solution must contain {expected} coefficients, got {actual}"
            ),
            Self::PolynomialCountMismatch { expected, actual } => write!(
                formatter,
                "fitted-field polynomial space must contain {expected} terms, got {actual}"
            ),
            Self::CenterCountMismatch { expected, actual } => write!(
                formatter,
                "fitted-field problem must retain {expected} centers, got {actual}"
            ),
        }
    }
}

impl<const D: usize> Error for FittedFieldFitError<D>
where
    Dim<D>: SupportedDimension,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Assembly(source) => Some(source.as_ref()),
            Self::Solve(source) => Some(source.as_ref()),
            Self::CoefficientCountOverflow
            | Self::CoefficientCountMismatch { .. }
            | Self::PolynomialCountMismatch { .. }
            | Self::CenterCountMismatch { .. } => None,
        }
    }
}

/// Evaluation component whose finite representation failed.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum FittedFieldComponent {
    /// Scalar value.
    Value,
    /// One gradient axis.
    Gradient {
        /// Cartesian axis.
        axis: usize,
    },
    /// One Hessian component.
    Hessian {
        /// Hessian row.
        row: usize,
        /// Hessian column.
        column: usize,
    },
}

/// Temporary storage whose checked allocation failed during evaluation.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum FittedFieldStorage {
    /// Polynomial values.
    PolynomialValues,
    /// Polynomial gradients.
    PolynomialGradients,
    /// Polynomial Hessians.
    PolynomialHessians,
}

/// Structured fitted-field evaluation failure.
#[derive(Debug)]
#[must_use]
pub enum FittedFieldEvaluationError<const D: usize>
where
    Dim<D>: SupportedDimension,
{
    /// Coordinate normalization or derivative transformation failed.
    Transform(TransformError),
    /// The model supplies no requested output even away from centers.
    UnsupportedOutput {
        /// Requested output.
        output: FittedFieldOutput,
        /// Reported capability.
        capability: KernelDerivativeCapability,
    },
    /// One retained center term cannot supply the requested output.
    UnsupportedCenterTerm {
        /// Requested output.
        output: FittedFieldOutput,
        /// Center index.
        center_index: usize,
        /// Term index within the center expression.
        term_index: usize,
        /// Preserved functional provenance.
        provenance: FunctionalProvenance,
    },
    /// The output is defined away from centers but not at this coincidence.
    UnavailableAtCenter {
        /// Requested output.
        output: FittedFieldOutput,
        /// Center index.
        center_index: usize,
        /// Term index within the center expression.
        term_index: usize,
        /// Preserved functional provenance.
        provenance: FunctionalProvenance,
    },
    /// Concrete kernel evaluation failed.
    Kernel {
        /// Center index.
        center_index: usize,
        /// Term index within the center expression.
        term_index: usize,
        /// Preserved functional provenance.
        provenance: FunctionalProvenance,
        /// Concrete source.
        source: KernelDefinitionEvaluationError<D>,
    },
    /// Complete polynomial evaluation failed.
    Polynomial(PolynomialSpaceError),
    /// Checked temporary allocation failed.
    AllocationFailed {
        /// Storage role.
        storage: FittedFieldStorage,
        /// Requested entry count.
        requested: usize,
    },
    /// A weighted center contribution or accumulation was nonfinite.
    NonFiniteCenterContribution {
        /// Center index.
        center_index: usize,
        /// Term index within the center expression.
        term_index: usize,
        /// Output component.
        component: FittedFieldComponent,
    },
    /// A weighted polynomial contribution or accumulation was nonfinite.
    NonFinitePolynomialContribution {
        /// Polynomial basis index.
        term_index: usize,
        /// Output component.
        component: FittedFieldComponent,
    },
    /// A final output vector component was nonfinite.
    NonFiniteOutput {
        /// Output component.
        component: FittedFieldComponent,
    },
}

impl<const D: usize> fmt::Display for FittedFieldEvaluationError<D>
where
    Dim<D>: SupportedDimension,
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Transform(source) => source.fmt(formatter),
            Self::UnsupportedOutput { output, capability } => {
                write!(
                    formatter,
                    "fitted output {output:?} has capability {capability:?}"
                )
            }
            Self::UnsupportedCenterTerm {
                output,
                center_index,
                term_index,
                ..
            } => write!(
                formatter,
                "center {center_index} term {term_index} cannot supply fitted output {output:?}"
            ),
            Self::UnavailableAtCenter {
                output,
                center_index,
                term_index,
                ..
            } => write!(
                formatter,
                "fitted output {output:?} is unavailable at center {center_index} term {term_index}"
            ),
            Self::Kernel {
                center_index,
                term_index,
                source,
                ..
            } => write!(
                formatter,
                "kernel evaluation failed at center {center_index} term {term_index}: {source}"
            ),
            Self::Polynomial(source) => source.fmt(formatter),
            Self::AllocationFailed { storage, requested } => write!(
                formatter,
                "could not reserve {requested} fitted-field entries for {storage:?}"
            ),
            Self::NonFiniteCenterContribution {
                center_index,
                term_index,
                component,
            } => write!(
                formatter,
                "center {center_index} term {term_index} produced non-finite {component:?}"
            ),
            Self::NonFinitePolynomialContribution {
                term_index,
                component,
            } => write!(
                formatter,
                "polynomial term {term_index} produced non-finite {component:?}"
            ),
            Self::NonFiniteOutput { component } => {
                write!(formatter, "fitted output {component:?} is non-finite")
            }
        }
    }
}

impl<const D: usize> Error for FittedFieldEvaluationError<D>
where
    Dim<D>: SupportedDimension,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Transform(source) => Some(source),
            Self::Kernel { source, .. } => Some(source),
            Self::Polynomial(source) => Some(source),
            Self::UnsupportedOutput { .. }
            | Self::UnsupportedCenterTerm { .. }
            | Self::UnavailableAtCenter { .. }
            | Self::AllocationFailed { .. }
            | Self::NonFiniteCenterContribution { .. }
            | Self::NonFinitePolynomialContribution { .. }
            | Self::NonFiniteOutput { .. } => None,
        }
    }
}

#[derive(Clone, Copy)]
struct NormalizedEvaluation<const D: usize> {
    value: f64,
    gradient: [f64; D],
    hessian: [[f64; D]; D],
}

const fn combined_order(
    output: KernelDerivativeOrder,
    center: KernelDerivativeOrder,
) -> Option<KernelDerivativeOrder> {
    match (output, center) {
        (KernelDerivativeOrder::Value, KernelDerivativeOrder::Value) => {
            Some(KernelDerivativeOrder::Value)
        }
        (KernelDerivativeOrder::Value, KernelDerivativeOrder::First)
        | (KernelDerivativeOrder::First, KernelDerivativeOrder::Value) => {
            Some(KernelDerivativeOrder::First)
        }
        (KernelDerivativeOrder::First, KernelDerivativeOrder::First)
        | (KernelDerivativeOrder::Second, KernelDerivativeOrder::Value) => {
            Some(KernelDerivativeOrder::Second)
        }
        (KernelDerivativeOrder::Second, KernelDerivativeOrder::First) => {
            Some(KernelDerivativeOrder::Third)
        }
        _ => None,
    }
}

fn center_value_action<const D: usize>(atom: FunctionalAtom<D>, jet: &SpatialKernelJet<D>) -> f64
where
    Dim<D>: SupportedDimension,
{
    match atom {
        FunctionalAtom::Value { .. } => jet.value(),
        FunctionalAtom::DirectionalDerivative { direction, .. } => dot(
            jet.first_derivative(KernelArgument::Center),
            *direction.components(),
        ),
    }
}

fn center_gradient_action<const D: usize>(
    atom: FunctionalAtom<D>,
    jet: &SpatialKernelJet<D>,
) -> [f64; D]
where
    Dim<D>: SupportedDimension,
{
    match atom {
        FunctionalAtom::Value { .. } => jet.first_derivative(KernelArgument::Query),
        FunctionalAtom::DirectionalDerivative { direction, .. } => {
            let mixed = jet.second_derivative([KernelArgument::Query, KernelArgument::Center]);
            std::array::from_fn(|axis| dot(mixed[axis], *direction.components()))
        }
    }
}

fn center_hessian_action<const D: usize>(
    atom: FunctionalAtom<D>,
    jet: &SpatialKernelJet<D>,
) -> [[f64; D]; D]
where
    Dim<D>: SupportedDimension,
{
    match atom {
        FunctionalAtom::Value { .. } => {
            jet.second_derivative([KernelArgument::Query, KernelArgument::Query])
        }
        FunctionalAtom::DirectionalDerivative { direction, .. } => {
            let mixed = jet.third_derivative([
                KernelArgument::Query,
                KernelArgument::Query,
                KernelArgument::Center,
            ]);
            std::array::from_fn(|row| {
                std::array::from_fn(|column| dot(mixed[row][column], *direction.components()))
            })
        }
    }
}

fn dot<const D: usize>(left: [f64; D], right: [f64; D]) -> f64 {
    left.into_iter()
        .zip(right)
        .map(|(left, right)| left * right)
        .sum()
}

fn accumulate_center<const D: usize>(
    output: &mut f64,
    multiplier: f64,
    action: f64,
    component: FittedFieldComponent,
    center_index: usize,
    term_index: usize,
) -> Result<(), FittedFieldEvaluationError<D>>
where
    Dim<D>: SupportedDimension,
{
    let contribution = multiplier * action;
    let next = *output + contribution;
    if !multiplier.is_finite()
        || !action.is_finite()
        || !contribution.is_finite()
        || !next.is_finite()
    {
        return Err(FittedFieldEvaluationError::NonFiniteCenterContribution {
            center_index,
            term_index,
            component,
        });
    }
    *output = next;
    Ok(())
}

fn accumulate_polynomial<const D: usize>(
    output: &mut f64,
    coefficient: f64,
    action: f64,
    component: FittedFieldComponent,
    term_index: usize,
) -> Result<(), FittedFieldEvaluationError<D>>
where
    Dim<D>: SupportedDimension,
{
    let contribution = coefficient * action;
    let next = *output + contribution;
    if !coefficient.is_finite()
        || !action.is_finite()
        || !contribution.is_finite()
        || !next.is_finite()
    {
        return Err(
            FittedFieldEvaluationError::NonFinitePolynomialContribution {
                term_index,
                component,
            },
        );
    }
    *output = next;
    Ok(())
}

fn try_filled<T: Clone, const D: usize>(
    count: usize,
    value: T,
    storage: FittedFieldStorage,
) -> Result<Vec<T>, FittedFieldEvaluationError<D>>
where
    Dim<D>: SupportedDimension,
{
    let mut output = Vec::new();
    output
        .try_reserve_exact(count)
        .map_err(|_| FittedFieldEvaluationError::AllocationFailed {
            storage,
            requested: count,
        })?;
    output.resize(count, value);
    Ok(output)
}
