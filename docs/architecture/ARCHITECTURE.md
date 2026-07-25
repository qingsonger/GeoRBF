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

The level layer depends on compiled scalar-field functionals, semantic
provenance, structured diagnostics, and the solver-neutral canonical IR. It
owns stable fixed, unknown, and prior level definitions; hard memberships;
minimum-gap order DAGs; connected-component gauge review; field contrast; and
source-aware membership and fixed conflicts. Memberships admit only one
unit-weight Value atom. A deterministic spanning forest represents the
transitive level-equality closure induced by shared mathematical Value
evaluations and retains a proving membership chain without quadratic edge
storage. A positive order path or distinct fixed values cannot connect levels
in one equality component, cycle evidence contains only cycle-participating
edges, fixed-path comparison is overflow safe and invariant under positive
scalar-unit rescaling, and contrast must reach two membership-coupled levels.
It accepts distinct fixed/prior anchors as contrast
only when their levels belong to different equality components, and its
missing-contrast evidence can represent a one-level field component without an
unrelated anchor. It appends one explicit level-variable block to caller field
variables and emits only canonical equality and linear-bound rows.
Soft priors compile to the same provenance-bearing canonical soft-equality
objectives as other independently scaled L2/L1/Huber observations, while the
level wrapper retains a stable level-identity view. The canonical IR also
retains soft bound and cone relation shapes without geological vocabulary.
No current dense equality path claims to optimize those objectives; lowering
them to quadratic or epigraph forms waits for an approved convex backend. The
level layer does not solve inequalities, eliminate levels into a separate model
type, select kernels or centers, or introduce geological semantics into a
solver.

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

The linear-constraint semantic layer sits above the shared problem IR and
below any future schema or adapter. It gives lower, upper, interval,
inside/outside, scalar-gap, and directional-monotonicity inputs explicit signs
before lowering them to ordinary semantic linear bounds. Inside/outside always
requires a caller-selected scalar orientation. Level order continues through
the explicit-level layer. Canonical hard-affine review detects constant
infeasibility and exact proportional equality/bound conflicts with full
sources. A separate immutable review reports exact and scale-aware
near-duplicate affine functionals without rewriting them; general multi-row or
cone infeasibility remains on the approved convex backend's independently
reviewed certificate path.
Field-only bound problems can compose with a compiled explicit-level problem
only when their named field variable spaces match exactly; the composition
preserves every existing row and rechecks stable provenance identity across
hard and soft records, canonical capabilities, memory, and hard conflicts.

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

The center-selection layer is a separate, pre-assembly numerical primitive.
`CenterSelectionProblem<D>` is constructed from an explicit
kernel-definiteness declaration and owns finite candidate locations, a finite
exact-symmetric row-major candidate Gram matrix, and aligned initial target
residuals. The current atomic capability accepts only strictly
positive-definite input. A CPD declaration is rejected with its positive order
at the typed construction boundary, before any greedy pivot, generic rank
review, or factorization, because a valid CPD selector additionally requires
complete polynomial actions `Q`, a verified null space, and projected-positive
review of `Z^T K Z`. It returns stable candidate indices and diagnostics; it
does not consume or mutate `FieldProblem<D>`, remove semantic observations,
convert hard relations to soft ones, fit coefficients, or select a solver.
This separation keeps center placement from becoming an implicit constraint
rewrite.

All-representer selection preserves input order. User-provided selection
preserves an explicitly validated unique index order. Farthest-point traversal
starts from `seed mod candidate_count`, then maximizes the minimum stable
Euclidean separation. Residual-greedy and power-greedy use the same
incremental Newton--Cholesky columns: residual-greedy maximizes the current
absolute interpolation residual, while power-greedy maximizes the current
Schur-complement diagonal. Seeded SplitMix64 keys break exact score ties
without mutable global randomness. A pivot must be strictly greater than
the selected candidate's local threshold
`candidate_count * epsilon * abs(K_ii)`. Under an equivalent nonzero basis
scaling `K -> D K D`, both that candidate's Schur pivot and threshold scale by
`D_ii^2`, preserving the rank classification. The implementation never adds
jitter, substitutes a diagonal, or skips a deficient requested step.

Every proposed selection, including all-representer, user-provided, and
farthest-point results, is materialized as its selected principal Gram matrix
and passed through the existing eight-pass equilibration, RRQR screen, bounded
SVD review, and checked Cholesky path under an explicit memory limit. A
deficient or ambiguous rank decision, SVD non-convergence, failed checked
factorization, or memory rejection is a structured selection error with the
underlying numerical evidence. Thus a greedy pivot screen is development of
the basis, not a substitute for the repository's final rank policy.

For strictly positive-definite Wendland kernels, the same `FieldProblem<D>`
also supports a compact sparse path. Atomic center locations are bulk-loaded
under stable `(center, term)` identities; D=1 and D=2 locations are embedded
with zero padding into the private three-coordinate rstar index because rstar's
tree requires at least two coordinates. Candidate hits are never mathematical
truth: GeoRBF recomputes the exact isotropic or globally anisotropic separation,
applies the strict `radius < support_radius` rule, then sorts and deduplicates
representer pairs. The path reflects each evaluated upper entry exactly,
compiles the same canonical hard equalities, validates finite exact symmetry,
and materializes GeoRBF-owned sorted-unique CSC once without a dense
intermediate. CPD kernels and polynomial side rows remain on the dense
rank-safe path until a separately reviewed sparse CPD formulation exists.

The fitted-model layer consumes one `FieldProblem<D>`, one concrete configured
kernel definition, optional constant global anisotropy, coordinate metadata,
normalization, and an explicit dense or compact-sparse solve policy. It uses
that same retained kernel definition for assembly and evaluation, then discards the semantic
builder, canonical problem, numerical matrix, right-hand side, and factorization
workspace. `FittedField<D>` owns centers, center functionals, coefficients,
complete CPD polynomial space, capabilities, general assembly/solve
diagnostics, and the accepted CPD RRQR/SVD rank decision, verified null-space,
and projected-energy evidence when applicable.
Compact Wendland models retain a private immutable support index and query only
exactly filtered local centers; dense models retain the existing all-center
evaluation. Returned fused evaluations report visited and total center counts
without adding mutable counters to the model.
Original-coordinate queries are normalized before evaluation; gradients use
`S^-T`, and Hessians use `S^-T H S^-1`. Directional-derivative centers retain
the kernel-calculus center-argument sign and require mixed second or third
derivatives for query gradients or Hessians. Exact center coincidences are
rejected when metadata declares only away-from-center support. The layer
performs no finite differences, hidden coefficient repair, persistence I/O,
schema migration, contouring, or adapter-side evaluation.

The one-dimensional contour layer consumes only an immutable
`FittedField<1>`, a finite target level and original-coordinate domain, and
explicit scan/refinement tolerances. It evaluates the fitted field's analytic
value and original-coordinate gradient only when the gradient capability is
supported everywhere; away-from-centers-only gradients are rejected before
evaluation. It midpoint-splits the requested uniform scan, retains value
brackets and only derivative brackets with an exact-zero endpoint or an actual
endpoint sign change, and applies bracket-preserving bisection. Merely
tolerance-small derivative samples remain separately inspectable stationary
candidate evidence, not fabricated sign brackets. Isolated points are sorted
and tolerance-deduped, and adjacent segments satisfying both value and
derivative tolerances are merged as non-isolated degenerate intervals. The
reported scan resolution is not a proof against arbitrarily many unseen
oscillations. This layer performs no finite differences, fitting, coefficient
repair, topology reconstruction, schema I/O, or adapter-side mathematics; D=2
and D=3 level sets remain separate atomic requirements.

The two-dimensional isoline layer consumes only an immutable `FittedField<2>`,
a finite target level, a finite original-coordinate rectangle, and explicit
grid and refinement settings. Its reference path splits every rectangular cell
from lower-left to upper-right and marches the two simplices. Its regular-grid
path uses marching squares; alternating-sign cells use the scale-normalized
bilinear saddle value when the saddle is interior, the bilinear center
otherwise, and a deterministic positive-connectivity tie when that decider is
exactly zero. Both paths retain only exact endpoint hits or true value-sign
brackets and refine the latter with bracket-preserving bisection. Shared
intersections are deduplicated by canonical grid-edge or grid-vertex identity,
not by an implicit spatial merge radius. The resulting unique undirected
segment graph must have degree at most two, and every degree-one vertex must
touch the requested rectangle within the explicit coordinate tolerance.
Returned topology consists only of deterministic open and closed polylines,
with boundary-side and ambiguity evidence. An exact grid edge on the target
level, or an edge whose two endpoint samples are exact but whose interior is
unknown, is diagnosed as underdetermined at the requested grid resolution
rather than converted into an arbitrary segment. The finite grid is reported
evidence rather than a proof against unseen sub-cell components. This layer
performs no finite differences, fitting, coefficient repair, schema I/O, mesh
export, or adapter-side mathematics; D=3 isosurfaces remain a separate atomic
requirement.

The three-dimensional isosurface layer consumes only an immutable
`FittedField<3>`, a finite target level, a finite original-coordinate box, and
explicit grid and refinement settings. Its marching-simplices reference path
uses the globally conforming Freudenthal split into six tetrahedra per cube.
Its regular-grid path intersects only the twelve cube edges, constructs
surface loops from the six cube faces, and applies the same scale-normalized
bilinear asymptotic decision on either side of every alternating-sign shared
face. Exact endpoint hits and true sign brackets are retained; brackets use
bounded bisection. Canonical global grid-vertex and grid-edge identities, not a
spatial merge radius, deduplicate intersections. Analytic
original-coordinate fitted gradients supply every unit vertex normal, and
triangle winding is selected toward the positive-gradient side. Edge
incidence must be one or two, adjacent triangle winding must oppose on shared
edges, and every incidence-one edge must lie on the requested box. Returned
components record closed/open status and requested-box faces. Exact
target-level sampled edges, unsupported exact-vertex patterns, zero-gradient
surface vertices, multiple disjoint cube-boundary loops with underdetermined
interior connectivity, collapsed triangles, non-manifold edges, inconsistent
winding, and interior boundary edges are structured failures. The finite grid
is evidence, not a completeness proof against unseen sub-cell components.
This layer performs no finite differences, refitting, coefficient repair,
schema I/O, file export, mesh repair, or adapter-side mathematics.

The project layer owns one or more independently fitted `FittedField<D>` values
behind stable caller-controlled `FieldId` values. `GeoProject<D>` preserves
insertion order, rejects duplicate identifiers, and performs deterministic
borrowed lookup without creating a second field core or a joint numerical
problem. Each retained field keeps its own coordinate metadata, normalization,
kernel, coefficients, capabilities, and diagnostics; project construction does
not reconcile coordinate systems or couple fitting and evaluation. A validated
`ReferenceFieldInput` resolves only an existing identifier and delegates
value, gradient, or Hessian evaluation to that immutable field in its own
original-coordinate convention. It reserves a typed input boundary for the
accepted future SPD local-mixture design but defines no weight function, local
anisotropy, cross-field constraint, topology, persistence schema, or adapter
mathematics.

The diagnostics layer owns source-aware orchestration and adapter-boundary
failures. A `DiagnosticPath` can retain an input path and one-based line,
semantic field path, stable observation identifier, stable level identifier,
and optional constraint group without exposing a schema or language-specific
object. Its fallible source-bound constructor accepts a validated source
location and field path while keeping observation, level, and constraint-group
identifiers independently optional. `GeoRbfError` distinguishes input,
capability, rank, gauge, contrast, infeasibility, conditioning, memory,
cancellation, and version failures. Every category has an explicit numeric and
symbolic `ErrorCode`; these values are stable public data, while Rust enum
layout, `Debug` text, and memory layout are not ABI or persistence formats.
Backend-specific rank, residual, and factorization records remain in their
numerical layers and can be retained as more detailed evidence beside this
common boundary taxonomy.

## Runtime behavior

Long operations accept cancellation, progress, explicit thread count, and
determinism through interfaces. The core emits no stdout or stderr output.
User input returns structured errors rather than panicking. A fitted model is
immutable, `Send + Sync`, independent of its builder, and deterministic to
serialize.

`ExecutionControl` borrows an optional cloneable `CancellationToken` and an
optional `ProgressSink` for one synchronous call. The token shares a sticky
atomic cancellation state across threads. The core never stores either control
in a fitted model, global variable, or solver object. Progress callbacks receive
copyable typed events synchronously without a core lock held; adapters may copy
events into their own queue, but callbacks must return promptly and must not
panic. The core does not catch callback panics or translate them into a false
successful operation.

Field assembly checks cancellation after every evaluated upper-triangle kernel
entry and polynomial row, and around CPD construction, canonicalization,
symmetry review, and projected-energy construction. Dense solving checks around
memory review, rank reviews, factorization, every attempted refinement, and
residual review. Sampled geometric thickness validation checks before and after
every fitted-field evaluation in its caller-bounded location, bracketing, and
refinement loops. A backend SVD or factorization call is indivisible, so a
request made during that call is observed immediately afterward. Fallible work
is retained until this post-call checkpoint: cancellation observable there
takes priority over the concurrent numerical failure, and a failed stage
publishes no successful progress event. Cancellation returns a typed
`ExecutionError::Cancelled` through the owning operation error and never
returns a partial system, solution, fitted model, or sampled-thickness report.
`FittedField` propagates one borrowed control through fitting or validation
without retaining it.

Compact sparse assembly additionally checks around index construction, each
row's candidate filtering, every supported kernel action, canonicalization,
symmetry review, and CSC materialization. Sparse solving checks before backend
storage construction and after the indivisible symbolic/numeric LLT and solve
calls.

Progress totals are checked maximum work budgets. Completed counts report only
work actually performed, so early refinement termination can complete with a
count below the budget. `Completed` is the single successful terminal event:
cancellation is checked before it is published, while cancellation requested
synchronously by that callback is post-completion and applies only to a later
operation that reuses the sticky token.

The current dense assembly and solve algorithms are serial. An absent thread
count or an explicit count of one is accepted and progress truthfully reports
one effective worker; a larger explicit count returns
`ExecutionError::UnsupportedThreadCount` before numerical work or progress.
This is an explicit capability boundary, not a silent thread-count clamp. A
true deterministic request preserves the fixed ordering already required by
assembly, rank review, factorization, refinement, and diagnostics. False permits
future nondeterministic implementations but does not make the current serial
path nondeterministic. No global thread pool or runtime dependency is selected.

Diagnostic display text is deterministic and begins with the symbolic error
code, but adapters branch on `ErrorCode` and typed evidence rather than parsing
display strings. CLI exit statuses, the stable C status ABI, the C++ exception
or result policy, Python exception classes, and persisted schema fields remain
separate later requirements that map to this one Rust source of truth.

Dense assembly computes only required symmetric work in blocks and reuses
per-thread storage. Compact-support paths use a neighborhood index and sparse
storage, reject nonrepresentable candidate bounds, retain stored-nonzero,
density, exact-support coverage independent of numerically zero functional
actions, ordering, and original-unit residual evidence, and never densify or
fall back. Their explicit memory review sums every simultaneously live logical
component: index construction, accepted-pair and reflected-entry capacities,
all reserved canonical relation-vector capacities, equality row work and
provenance strings, scaling, retained CSC and right-hand side, the complete
borrowed solve system, backend CSC copy, pinned-backend AMD and symbolic
analysis scratch, retained symbolic structures, conservative numeric-factor
storage and factorization scratch, solve vectors, and exact residual work.
The solver checks the symbolic-factorization, numeric-factorization, and
solve-and-review peaks separately before backend dispatch. Performance changes
are accepted only with fixed-data baselines and documented hardware and thread
settings.
