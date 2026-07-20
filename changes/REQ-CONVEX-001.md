# REQ-CONVEX-001

Added a private Clarabel 0.11.1 production adapter for solver-neutral canonical
problems. Hard equality rows map to zero cones, lower and upper bound sides map
with explicit signs to nonnegative cones, and each ordered `||lhs|| <= rhs`
relation maps directly to one Lorentz cone. Soft equality, interval, and cone
violations receive nonnegative epigraph variables. Squared L2 uses a checked
diagonal PSD term, absolute L1 a linear epigraph cost, and Huber an exact
quadratic-plus-linear infimal representation. No hard row enters an objective;
no pseudoinverse, constraint relaxation, presolve, KKT regularization, hidden
fallback, warm start, or in-repository interior-point method was added.

The Rust API requires a finite tolerance no looser than `1e-6`, nonzero
iteration and memory limits, and an optional positive finite time limit. It
returns only immutable GeoRBF-owned values, statuses, diagnostics, and reviewed
certificates. Diagnostics record the backend version, requested tolerances,
limits, serial thread count, equilibration and refinement policy, disabled
presolve/regularization, original and auxiliary dimensions, sparse rows and
coefficients, product-cone count, iterations, solve time, objective, KKT and
cone evidence, peak-memory estimate and limit, and complete original semantic
provenance with raw and normalized residuals.

Only exact `Solved` is accepted, after independent reconstruction of the
objective, primal and dual equations, cone membership, complementarity,
duality gap, and every hard residual in original canonical units. A backend
`PrimalInfeasible` status is returned only with an infinity-normalized
GeoRBF-owned certificate whose original compiled `A^T z`, dual-cone membership,
nonzero magnitude, and strict scale-aware `b^T z < 0` separator all pass.
Reduced-accuracy, dual-infeasible, limit, numerical-error, insufficient-progress,
callback, and unsolved statuses are structured failures.

Seven independent tests cover the analytic QP solution `(0.5, 1.5)`, analytic
SOCP solution `(5, 3, 4)`, hard and soft cones mixed with L1 and Huber epigraphs,
hard-bound preservation, deterministic repeated values/objective/iterations,
reviewed conic infeasibility, invalid policy, pre-dispatch memory rejection,
and `Send + Sync`. A runnable Huber example and deterministic sparse QP
benchmark are included. Ready/main CI gains the production benchmark smoke on
Windows, Ubuntu, and macOS.

The production dependency review retained current non-yanked Clarabel 0.11.1,
Apache-2.0, published 2025-06-11 with declared Rust 1.70. Its repository was
active, unarchived, and pushed 2026-04-13. The exact active Windows graph has 34
packages; the all-target lock graph has 48, 2,971,926 bytes of cached crate
archives, only permissive declared licenses, and a highest declared MSRV of
Rust 1.77, with some omissions. A conservative all-target scan counted 1,552
Rust source lines containing `unsafe` across 30 packages; this is exposure, not
an unsafe-block or defect count. The selected path remains Rust-only and adds
no native C/Fortran, BLAS/LAPACK, SDP, Python, Julia, or Pardiso delivery. Exact
OSV batch queries for all 48 packages and the repository security-advisory API
returned no finding. `cargo-audit` and `cargo-deny` remain unavailable, so they
are not claimed.

Rust is implemented. CLI is N/A because the stage-0 command exposes only help
and version and complete project/schema commands belong to M8. C, C++, and
Python are N/A because their M9 requirements follow Rust API and schema freeze;
no adapter may expose Clarabel types or reimplement solver semantics. The
benchmark obligation is implemented. Geological angular/thickness compilation,
general near-duplicate/conflict diagnostics, fitted-field integration, sparse
field solving, schemas, bindings, and release work remain later requirements.

Repair of independent-review findings R82-001 through R82-007 removes the
absolute factor-64 acceptance policy and replaces it with exact requested-
tolerance, dimensionally homogeneous row, component, cone, complementarity,
gap, and certificate reviews. Certificate stationarity and separation now use
scaled original-data products with explicit representability rejection, so the
feasible `1e-12 * x <= -1` counterexample cannot be accepted as infeasible.

The semantic primal objective is now evaluated directly from original
relations, scales, and `SoftLoss`; compiled and backend primal values are
separate comparisons, and the dual objective is independently reconstructed as
`-0.5 * x^T P x - b^T z`. Diagnostics expose the semantic and compiled primal,
both backend-reported values, the reconstructed dual, normalized
complementarity, and the semantic primal-dual gap.

The effective memory limit is the smaller of the convex and optional execution
limits. A nonallocating preflight precedes provenance cloning, includes owned
metadata and auxiliary storage, and adds a dense full-KKT upper bound for
QDLDL symbolic/numeric fill; later GeoRBF-owned vectors and provenance copies
reserve fallibly. All material Clarabel 0.11.1 settings are assigned explicitly
and mirrored by an exact internal diagnostic snapshot, including direct QDLDL,
reduced-status thresholds, equilibration bounds, step lengths, refinement
tolerances, inactive regularization constants, and sparse-zero handling.

Five private regressions cover false and positively rescaled certificates,
equivalent row scales, semantic-objective perturbation rejection, metadata and
adversarial-fill memory accounting, the effective execution limit, exact
settings, and forced status routing. Nine end-to-end tests additionally cover
fixed nonzero L2, L1, inner/outer Huber, nonunit scale, violated soft-bound and
soft-cone objectives, and Lorentz rotation invariance.

Repair of R82-008 adds an explicit hard-only feasibility policy instead of
disabling stationarity review. Structurally zero objectives use a recorded
dimensionless objective-unit reference of one; original row values convert it
to componentwise objective-gradient units, so no raw dimensioned floor is
introduced. A synthetic nonstationary dual still fails. Before backend
dispatch, independent zero/nonnegative rows and whole Lorentz blocks receive
positive infinity normalization, with the complete scaling recorded and
backend slacks and duals mapped back before every original-unit review. The
public `x >= 1` feasibility solve now succeeds for equivalent row scales
`1e-12`, `1`, and `1e12`, with every normalized KKT and hard-relation diagnostic
within the exact requested tolerance.
