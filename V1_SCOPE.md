# GeoRBF v1 Scope

## Product statement

GeoRBF v1.0.0 is a new MIT-licensed, multi-language radial-basis scalar-field
system. One Rust mathematical core fits and evaluates fields
`f: R^D -> R` for exactly D=1, D=2, or D=3. CLI, stable C ABI, header-only C++
RAII, and PyO3/NumPy adapters delegate to that core.

## Included capabilities

- Dimension-safe geometry, coordinate metadata, normalization, orientation
  conversion, and global anisotropy.
- Value and directional-derivative functionals and finite linear expressions.
- Explicit fixed, unknown, and prior level variables with order DAGs.
- Hard equalities, soft L2/L1/Huber losses, linear bounds, intervals, QP, and
  SOCP constraints.
- Gradient-vector, direction-only, polarity, axial, angular-cone, tangent,
  monotonicity, scalar-gap, local-thickness, and sampled geometric validation.
- CPD polynomial spaces, rank diagnostics, side conditions, and no hidden
  regularization.
- Polyharmonic/surface-spline, Gaussian, inverse multiquadric, validated
  multiquadric, Matérn, and Wendland C2/C4/C6 kernels.
- Isotropic and global ellipsoidal metrics plus positive-definite local kernel
  mixtures with background, regional, and reference-field controls.
- Immutable fitted fields with value, gradient, and capability-gated Hessian.
- Dense and compact-support sparse paths, rank-safe center selection,
  deterministic parameter tuning, memory estimates, and performance baselines.
- 1D level points, 2D isolines, and 3D isosurfaces with reference simplicial and
  tested regular-grid paths.
- Versioned multi-field project and model formats, CSV/JSON/TOML inputs, model
  persistence, structured diagnostics, and contour data exports.
- Rust, CLI, C, C++, and Python parity; three-platform artifacts; complete
  verification, documentation, SBOM, licenses, checksums, RC rehearsal, and
  formal publication.

The atomic decomposition and authoritative completion status are maintained in
`requirements/v1.yaml`.

## Explicit exclusions

v1 does not include or expose placeholder APIs for:

- Surfe API or file compatibility, Surfe model types, or Surfe golden results;
- GUI, Qt, VTK, or vendor-specific desktop integration;
- complete geological-body topology, fault displacement restoration, or
  Boolean geological solids;
- arbitrary non-conservative vector fields or arbitrary high-order PDE
  functionals;
- GPU or distributed execution, FMM, H/H2 matrices, or out-of-core solvers;
- unproved arbitrary location-dependent distances;
- a general probabilistic Gaussian-process platform; or
- a continuous-domain proof of global minimum Euclidean thickness.

Internal extension traits may reserve design space, but unimplemented modes are
not public user capabilities.

## Compatibility and correctness

GeoRBF makes no compatibility or numerical-equivalence claim with Surfe or any
commercial package. Correctness comes from documented mathematics, independent
analytic or high-precision truth, invariance and property tests, and explicit
numerical diagnostics.
