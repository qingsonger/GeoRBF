# Solver and Numerical Policy

Production backends are selected only after the recorded spikes:

- SPIKE-001: dense SPD and symmetric-indefinite factorization;
- SPIKE-002: rank-revealing QR and SVD;
- SPIKE-003: compact sparse assembly and solution; and
- SPIKE-004: canonical QP and SOCP mapping.

REQ-SPIKE-001 selects the nalgebra 0.35 release line as the preferred later
production backend for checked dense Cholesky and Bunch--Kaufman LBLT with
symmetric 1-by-1 and 2-by-2 pivot blocks. The spike itself remains isolated;
production adoption must stay behind the private numerical adapter, repeat the
dependency and advisory audit, and keep nalgebra matrix types out of public
geometry and model APIs. Checked Cholesky must never use the substitute-
diagonal entry point. A zero pivot, nonfinite result, failed original-unit
residual review, or nondecreasing refinement correction is an explicit failure.

REQ-CPD-001 adopts nalgebra 0.35.0 behind the private CPD numerical adapter for
column-pivoted QR screening, bounded SVD rank review, and QR-supported
null-space construction. GeoRBF-owned row-major matrices and diagnostics are
the only public types. The adapter does not expose or call pseudoinverse or
minimum-norm solve APIs.

REQ-SPIKE-004 selects the Clarabel 0.11 release line as the preferred later
production backend for canonical QP and SOCP dispatch. The spike pins 0.11.1
only in an excluded harness. Production adoption must remain behind one private
GeoRBF-owned convex adapter, repeat the exact dependency and advisory audit,
preserve row and cone provenance, and independently review solutions and
infeasibility certificates in original units. Presolve, KKT regularization,
equilibration, refinement, tolerances, limits, and thread count must be explicit
and recorded; no backend setting may relax a hard constraint or change the
objective silently. ADR-0011 records the mapping and selection evidence.

REQ-SPIKE-003 selects the rstar 0.13 release line for later immutable
support-neighborhood indexing, canonical CSC at the private sparse-solver
boundary, and the faer 0.24 release line for later checked sparse LLT. The
spike pins exact candidates only in an excluded harness. Production adoption
must independently recompute the strict compact-support predicate, sort and
deduplicate stable center pairs, assemble without densification, validate
symmetry and finiteness, enforce explicit memory and serial-execution policy,
and review every solution in original units. A failed pivot, factorization, or
residual is explicit; no jitter, regularization, pseudoinverse, densification,
or backend fallback is authorized. ADR-0012 records the selection and the
rejected valid-input panic and LGPL alternatives.

`REQ-SPARSE-001` adopts rstar 0.13.0 and faer 0.24.4 with the exact
default-disabled feature sets reviewed by ADR-0012. GeoRBF lifts D=1/D=2
locations into a private three-coordinate index with zero padding, because
rstar rejects one-coordinate trees, while exact support truth remains in the
original compile-time dimension. Global anisotropy uses a conservative
Euclidean candidate radius derived from the retained inverse transform; every
candidate is then accepted only after the existing stable transformed-radius
calculation proves `radius < support_radius`.

Sparse assembly evaluates only sorted unique support-neighbor representer
pairs, reflects one finite value exactly, compiles the same hard equalities,
and materializes sorted-unique full symmetric CSC without a dense intermediate.
Support coverage is derived from that exact-support pair graph, not from
whether a particular functional action happens to evaluate to zero.
Sparse solving copies that validated storage once into the private faer
boundary, selects lower-triangle LLT with AMD ordering, and independently
reviews the solution with the fixed exact-binary accumulator in original
units. The residual tolerance is `128*n*epsilon`. A conservative peak payload
checks and records the sum at each assembly stage, including the retained
index, accepted-pair and reflected-entry capacities, all reserved canonical
relation-vector capacities, equality row work and provenance strings, scaling,
CSC, and right-hand side. Solve review adds the complete borrowed retained
system, one backend CSC copy, vectors, exact residual accumulators, and a dense
fill bound before backend dispatch. The pinned faer 0.24.4 adapter reproduces
its AMD and complete symbolic-analysis `StackReq`, bounds both possible
retained symbolic representations by two dense index patterns plus
dimension-sized metadata, bounds numeric supernodal panel storage by a dense
square, and bounds numeric scratch by the permuted CSC copy, two dense value
workspaces, and dimension-sized indices. It checks separate
symbolic-factorization, numeric-factorization, and solve-and-review peaks and
enforces their maximum before constructing backend storage. Failure is
explicit: no equilibration, refinement, regularization, jitter, diagonal
substitution, pseudoinverse, densification, constraint relaxation, or
factorization fallback is enabled.

Each spike reviews correctness, scaling, maintenance, license, MSRV, unsafe
use, platforms, binary size, alternatives, and deterministic behavior, then
produces an ADR before dependency lock-in.

Target dispatch is checked Cholesky for SPD equality systems, pivoted
Bunch--Kaufman LBLT for symmetric-indefinite KKT systems, RRQR for rank risk,
SVD for rank review or justified minimum-norm work, projected weighted least
squares for L2 losses, a mature convex QP backend for linear inequalities and
epigraph losses, and a mature SOCP backend for angular and thickness cones.

Every solve performs finite and unit checks, coordinate and direction
normalization, explicit constraint scaling, duplicate/conflict review,
polynomial rank and kernel capability checks, anisotropy validation, memory
estimation, symmetry checks, conditioning estimation, iterative refinement, and
residual review in original units.

Iterative refinement is bounded and explicit. It reuses the unchanged
mathematical matrix and requested factorization, records the initial and final
original-unit residuals and accepted correction count, and accepts a correction
only when that residual strictly decreases. It never authorizes a factorization
switch, diagonal substitution, jitter, pseudoinverse, constraint relaxation,
or other implicit problem change.

Rank and condition decisions use an explicitly recorded dimensionless
equilibration. Diagnostics retain row and column scales, matrix norms, rank
thresholds, effective rank, condition estimates, and both scaled and
original-unit residuals. Unit changes and equivalent nonzero row scaling must
not change feasibility or rank classification. A large condition estimate may
trigger a warning or an explicit policy error, but never an unrequested change
to the mathematical problem.

The CPD rank policy uses eight alternating infinity-norm equilibration passes,
the dimension-times-epsilon RRQR and SVD thresholds recorded in
`docs/math/CPD_AND_POLYNOMIALS.md`, and a factor-16 SVD ambiguity guard band.
RRQR/SVD disagreement or threshold adjacency is an explicit error, not an
automatic solver adjustment. Equilibration rejects a cumulative multiplier or
individual nonzero entry that becomes unrepresentable. Bounded SVD
non-convergence retains completed equilibration and RRQR evidence while
marking every SVD-derived field and the final rank decision unavailable.
Null-space QR uses that same safely equilibrated matrix, maps the basis back
through exponent-aware products with the recorded row scaling, and
reorthogonalizes before binding matrix-infinity residual checks. Original-unit
residuals use a fixed stack exact-binary superaccumulator, round only the final
signed sum, and fail explicitly when a nonzero result would round to zero or
overflow instead of fabricating a finite diagnostic.

Regularization is None, Explicit(value), or AutomaticWithin(maximum). Any
automatic choice records requested and actual solver, amount added, original
and effective rank, condition estimates, and trigger. No pseudoinverse, jitter,
or fallback may hide failure.

## Canonical convex implementation

`REQ-CONVEX-001` adopts Clarabel 0.11.1 behind the private `convex` adapter.
The public boundary contains only GeoRBF-owned options, solutions, statuses,
provenance, certificates, and diagnostics. The adapter maps hard equalities to
zero cones, each finite bound side to a nonnegative-cone row, and every ordered
canonical `||lhs|| <= rhs` relation to one Lorentz cone. It introduces explicit
violation epigraphs for soft relations: squared L2 contributes a nonnegative
diagonal PSD term, L1 a linear epigraph cost, and Huber an exact quadratic-plus-
linear infimal representation. Hard relations are never included in an
objective or relaxed.

The production settings select direct serial QDLDL and assign every material
Clarabel 0.11.1 setting explicitly: exact and reduced-status tolerances,
kappa/tau tolerance, maximum step and line-search lengths, equilibration bounds
and ten passes, iterative-refinement tolerances, stop ratio and ten steps,
sparse-zero handling, iteration/time limits, and one thread. Reduced statuses
remain rejected even though their backend thresholds are explicit. Presolve
plus static and dynamic KKT regularization are disabled, with their inactive
constants recorded. No backend fallback or warm start exists. Diagnostics
mirror the complete settings snapshot, the exact independent-review tolerance
with multiplier one, backend version, terminal status, iterations, effective
memory policy, sparse shape, original-variable and auxiliary-variable counts,
and complete relation provenance.

An exact `Solved` status is necessary but insufficient. GeoRBF evaluates each
`rho(v(x_original) / scale)` directly from the original canonical relations,
separately reconstructs the compiled and backend primal objectives, and
reconstructs the dual objective as `-0.5 * x^T P x - b^T z`. Primal and dual
equations, product-cone membership, complementarity, semantic primal-dual gap,
and every hard residual use dimensionally homogeneous component or row scales
and the exact requested tolerance; there is no hidden multiplier or raw
dimensioned unit floor. The unit soft-loss count supplies a natural
dimensionless objective scale at a zero-loss optimum. For a structurally zero
objective, diagnostics instead record an explicit dimensionless objective-unit
reference of one. Stationarity converts that reference to each variable's
gradient units with `max_i |A_ij| / max(|b_i|, |s_i|, |A_ik x_k|)` over nonzero
original row references. The construction is invariant under positive row
scaling and variable-unit changes; a zero row supplies no artificial reference,
and the synthetic nonstationary dual remains rejected.

Before dispatch, the adapter applies a positive infinity normalization to each
zero-cone and nonnegative-cone row and one common normalization to every
Lorentz-cone block. The latter restriction preserves the Lorentz cone exactly.
Backend slack and dual values are mapped back as `s = D^-1 s_backend` and
`z = D z_backend` before every original-unit KKT or certificate review. The
complete row-scaling vector is retained in solution and certificate diagnostics;
it is not a hidden relaxation. This adapter normalization makes equivalent
hard-only rows at scales `1e-12`, `1`, and `1e12` reach the same exact requested-
tolerance review policy without depending on the backend's bounded internal
equilibration factors.

A reported primal-infeasibility vector is infinity-normalized and accepted
only after componentwise homogeneous original-data `A^T z`, dual-cone,
nonzero, representability, and strict scale-aware `b^T z < 0` reviews. The
convex option and optional execution memory limits combine by taking the
smaller nonzero value. A nonallocating preflight runs before compiler-owned
cloning, accounts for provenance and adapter auxiliaries, and bounds QDLDL
symbolic/numeric fill by the dense lower triangle of the complete KKT
dimension. GeoRBF-owned allocations after that check use fallible reservation.
Reduced-accuracy, dual-infeasible, limit, numerical-error,
insufficient-progress, callback, and unsolved statuses remain structured
failures.

The production re-audit retained the exact 0.11.1 patch because it remained the
current non-yanked crates.io release on 2026-07-19. The selected default-disabled
`serde` feature is still required by an unconditional upstream error variant;
no BLAS, LAPACK, SDP, Python, Julia, Pardiso, native C, or OSQP path is enabled.
The 34-package active Windows graph and 48-package all-target lock graph are
permissively licensed, build on Rust 1.96.1, and have no OSV or repository
security-advisory finding in the recorded audit. The highest declared all-target
MSRV is Rust 1.77; some transitive crates omit an MSRV. Source lines containing
the word `unsafe` are dependency-exposure evidence rather than proven defects;
the GeoRBF core continues to forbid unsafe code.

## Dense equality implementation

`REQ-SOLVE-001` implements the square dense equality path behind GeoRBF-owned
row-major types. The caller must select checked Cholesky or symmetric-pivoted
Bunch--Kaufman LBLT; the recorded requested and actual choices are identical
because factorization fallback is forbidden. Every effective matrix receives
the fixed eight-pass RRQR screen and bounded SVD review above before one
factorization. SVD is diagnostic in this requirement: no pseudoinverse,
minimum-norm solution, or SVD-based solve is exposed.

The factorization uses a symmetric congruence scaling derived from the recorded
row and column equilibration multipliers. The right-hand side and solution are
mapped consistently, and every accepted result is reviewed against the matrix
in original units with the fixed-stack exact-binary accumulator. Refinement
reuses that one factorization, is bounded at eight requested steps, and accepts
a candidate only when its exact original-unit infinity residual strictly
decreases. The final backward-error tolerance is `128*n*epsilon`.

Every dense solve also requires a nonzero explicit memory limit. Before any
nalgebra dispatch, checked `usize` arithmetic computes a conservative peak
payload covering six simultaneous `n*n` scalar matrix buffers, 32 `n`-scalar
vectors, two `n`-entry backend pivot/index-pair buffers, retained diagnostic
objects, and the fixed exact-dot workspace. These terms cover equilibration,
the source and effective matrices, the nalgebra matrix and RRQR clone, the
materialized `R` matrix, bounded-SVD bidiagonal work, Cholesky or LBLT storage,
the LBLT `D` inspection, solves, residuals, and refinement candidates. A field
solve additionally counts the still-live assembled matrix and right-hand side
while the solver-owned copy exists.

Estimate overflow and a peak above the effective limit are structured errors
returned before backend dispatch. `try_solve_field` applies the smaller of the
solver limit and `ExecutionOptions::memory_limit_bytes`, and performs this
check before copying the assembled field matrix. It is the only public field-
to-solver boundary: the internal solver-owned conversion is private, so a
caller cannot copy a field system while discarding its retained execution
limit. Accepted diagnostics retain both the estimate and effective limit. The
estimate is deliberately conservative payload accounting rather than a
promise about allocator or OS resident-set overhead.

This requirement exposes `None` and `Explicit(value)` regularization only.
Explicit regularization is validated before use and records both the original
and effective rank decisions, the exact amount applied, and the final residual
against the unmodified matrix. `AutomaticWithin(maximum)` remains a normative
future policy, not a placeholder success path in the current API.

## Rank-safe center selection

`REQ-CENTER-001` adds no solver or numerical dependency. It accepts an explicit
GeoRBF-owned symmetric candidate Gram matrix plus a kernel-definiteness
declaration and supports all-representer, ordered user-provided, seeded
farthest-point, seeded residual-greedy, and seeded power-greedy selection for
strictly positive-definite input. CPD input is rejected at this typed boundary
with its declared order before pivot, generic rank, or factorization work. A
future CPD selector must instead supply complete polynomial actions `Q`, verify
their rank and null space, and review positivity of `Z^T K Z`; raw-`K`
Cholesky is not a CPD rank test. The selection layer is side-effect free: it
returns indices and evidence but does not rewrite a semantic problem, relax a
hard equality, or fit a model.

Residual- and power-greedy selection share a deterministic incremental
Newton--Cholesky construction. At step `j`, the squared basis pivot is
`K_ii - sum_l L_il^2`. Residual-greedy selects the largest absolute current
interpolation residual and applies its normalized Newton column; power-greedy
selects the largest squared pivot. A pivot is accepted only when it is finite
and strictly greater than the selected candidate's local threshold
`candidate_count * epsilon * abs(K_ii)`. This relative rule scales by
`D_ii^2` together with the Schur pivot under an equivalent nonzero congruence
`K -> D K D`; a differently scaled unrelated candidate cannot reject an
independent basis member. Farthest-point selection uses stable Euclidean
`hypot` accumulation. All exact score ties use deterministic seeded keys;
there is no global random state.

The pivot check is not the final rank decision. Every selected principal Gram
matrix is independently reviewed by the existing eight-pass equilibration,
RRQR, bounded SVD, factor-16 ambiguity band, and checked Cholesky path under
the caller's explicit memory limit. Deficiency, ambiguity, non-convergence,
factorization rejection, nonfinite work, or an insufficient limit fails
explicitly. No jitter, substitute diagonal, hidden regularization,
pseudoinverse, candidate skipping, or factorization fallback is authorized.

## Deterministic parameter tuning

`REQ-TUNE-001` adds no solver or numerical dependency and does not fit a field
implicitly. It accepts only an explicit, finite candidate set whose active
kernel length, compact-support radius, explicit regularization, axis ratio, and
local-control influence radius all have inclusive physical-domain bounds. A
candidate must supply exactly the active bounded parameters. Duplicate and
out-of-bound candidates fail before evaluation; a failed candidate evaluation
rejects the complete search instead of being skipped.

Fixed selection returns one exact caller index without evaluating alternatives.
The distance heuristic computes each observation's stable Euclidean nearest-
neighbor distance, takes their median, and minimizes the mean squared
log-ratio for every active length, support-radius, and influence-radius value.
Duplicate locations, nonrepresentable distances, or the absence of a
length-like candidate parameter are explicit failures.

Cross-validation constructs nonempty folds by sorting observation indices with
an explicit-seed SplitMix64 key and assigning that order round-robin. It
minimizes the total caller-reviewed weighted squared error divided by total
weight while retaining each fold mean. Generalized cross-validation minimizes
`RSS / (n - effective_dof)^2` only when `0 <= effective_dof < n`.
Power-function tuning minimizes the caller-reviewed maximum nonnegative squared
power over a positive sample count. These formulas, finite evidence, and
observation counts are reviewed in the Rust core; the caller-owned evaluator
performs the actual candidate fit or factorization and cannot request a
criterion fallback.

Candidate traversal and score reduction are stable. Exact minimum-score ties
use keys derived only from the explicit seed and candidate index; no global
random state exists. Diagnostics retain the criterion, seed, bounds, fold
membership, every candidate score and criterion evidence, the selected index,
and exact tie count. Tuning never mutates a `FieldProblem`, changes a hard
constraint, invents regularization, adds jitter, uses a pseudoinverse, or
changes solver policy.
