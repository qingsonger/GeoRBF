# Software Architecture

## Workspace boundaries

- `crates/georbf`: safe Rust mathematical core and all domain compilation.
- `crates/georbf-cli`: command and file adapter only.
- `crates/georbf-ffi`: smallest stable C ABI boundary and the only future
  location for narrowly justified unsafe code.
- `crates/georbf-python`: PyO3/NumPy adapter to the core.
- `xtask`: repository checks and release engineering.

The C++ API is a header-only RAII wrapper around the C ABI. No adapter
reimplements constraint compilation, assembly, solving, or evaluation.

## Core layers

The planned core modules are dimension, geometry, units, coordinates,
orientation, transform, kernel, kernel calculus, polynomial, functional,
observation, levels, anisotropy, problem, semantic and canonical IR, basis,
assembly, solver, model, contour, I/O, diagnostics, and structured errors. They
remain in one strong crate until evidence justifies a split.

Dependencies point inward: geometry and kernel calculus know no geology;
coordinate metadata and transforms depend only on validated geometry and units;
problem compilation depends on functionals and semantic concepts; assembly
depends on the canonical representation; solvers know only numerical forms;
models own immutable coefficients and transforms; adapters depend on public
core APIs.

The orientation layer depends only on dimension-safe geometry and angle units.
It keeps planar normals and linear directions as separate fixed-size D=2/D=3
types, applies explicit positive/negative/unknown polarity, and owns only
validated geological angle-to-direction conversion. It does not compile normal
or tangent constraints, infer gradient magnitude, perform coordinate
reprojection, or construct anisotropy.

The global-anisotropy layer depends on validated geometry, affine matrix
inversion, and kernel-calculus jets. It owns fixed D=1/D=2/D=3 distance
transforms, SPD metrics, explicit condition diagnostics, displacement mapping,
and the constant-map chain rule through third order in its caller's current
coordinate system. It does not identify that system with a fitted model's
external original coordinates; fitted fields call it in normalized model
coordinates and apply the affine normalization chain rule afterward. It
performs no axis estimation, local mixing, kernel-family selection, observation
construction, assembly, fitting, or solver work. Arbitrary location-dependent
metrics remain forbidden; the later local-trend layer uses the accepted SPD
mixture design.

The kernel-calculus layer accepts validated point separations and a
caller-supplied radial jet. D=2/D=3 away jets include cancellation-resistant
radial expansion coefficients computed by the concrete radial formula; D=1
uses no radial quotients, and center jets declare analytic Euclidean limits.
The layer owns stable radius construction,
center dispatch, fused Cartesian tensor expansion, and query/center signs. It
does not own concrete kernel formulas, parameters, definiteness or smoothness
metadata, anisotropy, geological concepts, functionals, assembly, or fitting
policy. Its D=1/D=2/D=3 outputs are fixed arrays with no allocation,
dependency, dynamic dispatch, or unsafe code.

The adjacent kernel-metadata layer is formula-free. It describes
definiteness/CPD order, supported dimensions, derivative and center limits,
global or parameterized compact support, and explicit unit-aware parameter
definitions. Static descriptions borrow their parameter slices and use only
scalar values and fixed flags; configured values remain separate. Concrete
kernel requirements will pair these descriptions with radial formulas and
stable expansion coefficients. Metadata does not implement polynomial spaces,
SPD/CPD proofs, anisotropy, functionals, assembly, fitting, schemas, or adapter
bindings.

Concrete kernel submodules sit beside, not inside, the calculus. The
polyharmonic submodule owns its signed radial formulas, dimension/order
validation, CPD metadata pairing, center capability, and direct stable
expansion coefficients. It delegates Cartesian tensor expansion and
query/center signs to kernel calculus. Concrete kernels remain independent of
geology, observations, polynomial construction, CPD rank enforcement,
assembly, solvers, and adapters.

The smooth-global submodule owns Gaussian, inverse multiquadric,
CPD-positive signed multiquadric, and the explicit Matérn half-integer catalog.
It validates one physical length scale, pairs each formula with exact SPD/CPD
and center-capability metadata, preserves representable exponential and
rational tails, and supplies direct D=2/D=3 expansion coefficients. Like the
polyharmonic submodule it delegates Cartesian tensors and argument signs to
kernel calculus and has no geology, polynomial construction, anisotropy,
assembly, fitting, solver, schema, or adapter dependency.

The Wendland submodule owns the normalized C2, C4, and C6 compact-support
formulas and their one physical support radius. It pairs exact SPD and center
capability metadata with an explicit zero branch at and beyond the boundary,
computes analytic radial derivatives and cancellation-resistant D=2/D=3
expansion coefficients, and preserves representable factored products near
the support edge. It delegates Cartesian tensors and argument signs to kernel
calculus. It does not select neighborhoods, sparse storage, or solver policy;
those remain blocked on the dedicated compact-sparse spike and ADR.

The field-assembly layer depends on semantic/canonical problem IR, distinct
observation and center functional wrappers, kernel metadata/calculus, complete
polynomial spaces, and CPD rank/null-space enforcement. `FieldProblem<D>` owns
only validated hard-equality problem inputs. Its evaluator callback keeps
concrete kernel family and optional global-anisotropy construction outside the
assembly module while receiving the exact point pair and derivative demand.
The layer preassembles only the upper kernel triangle, canonicalizes observation
rows over center-weight and polynomial variable blocks, appends CPD side rows,
and returns an immutable GeoRBF-owned row-major dense system with symmetry and
CPD diagnostics. It does not factor, solve, regularize, select centers, construct
geological semantics, or expose nalgebra types.

The fitted-model layer consumes one `FieldProblem<D>`, one concrete configured
kernel definition, optional constant global anisotropy, coordinate metadata,
normalization, and an explicit dense-solve policy. It uses that same retained
kernel definition for assembly and evaluation, then discards the semantic
builder, canonical problem, dense matrix, right-hand side, and factorization
workspace. `FittedField<D>` owns centers, center functionals, coefficients,
complete CPD polynomial space, capabilities, general assembly/solve
diagnostics, and the accepted CPD RRQR/SVD rank decision, verified null-space,
and projected-energy evidence when applicable.
Original-coordinate queries are normalized before evaluation; gradients use
`S^-T`, and Hessians use `S^-T H S^-1`. Directional-derivative centers retain
the kernel-calculus center-argument sign and require mixed second or third
derivatives for query gradients or Hessians. Exact center coincidences are
rejected when metadata declares only away-from-center support. The layer
performs no finite differences, hidden coefficient repair, persistence I/O,
schema migration, contouring, or adapter-side evaluation.

The diagnostics layer owns source-aware orchestration and adapter-boundary
failures. A `DiagnosticPath` can retain an input path and one-based line,
semantic field path, stable observation identifier, stable level identifier,
and optional constraint group without exposing a schema or language-specific
object. `GeoRbfError` distinguishes input, capability, rank, gauge, contrast,
infeasibility, conditioning, memory, cancellation, and version failures.
Every category has an explicit numeric and symbolic `ErrorCode`; these values
are stable public data, while Rust enum layout, `Debug` text, and memory layout
are not ABI or persistence formats. Backend-specific rank, residual, and
factorization records remain in their numerical layers and can be retained as
more detailed evidence beside this common boundary taxonomy.

## Runtime behavior

Long operations accept cancellation, progress, explicit thread count, and
determinism through interfaces. The core emits no stdout or stderr output.
User input returns structured errors rather than panicking. A fitted model is
immutable, `Send + Sync`, independent of its builder, and deterministic to
serialize.

Diagnostic display text is deterministic and begins with the symbolic error
code, but adapters branch on `ErrorCode` and typed evidence rather than parsing
display strings. CLI exit statuses, the stable C status ABI, the C++ exception
or result policy, Python exception classes, and persisted schema fields remain
separate later requirements that map to this one Rust source of truth.

Dense assembly computes only required symmetric work in blocks and reuses
per-thread storage. Compact-support paths use a neighborhood index and sparse
storage. Performance changes are accepted only with fixed-data baselines and
documented hardware and thread settings.
