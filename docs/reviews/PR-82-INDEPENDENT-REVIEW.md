# PR #82 Independent Review

- Requirement: REQ-CONVEX-001
- Issue: https://github.com/qingsonger/GeoRBF/issues/81
- Pull request: https://github.com/qingsonger/GeoRBF/pull/82
- Branch: `codex/req-convex-001-canonical-solver`
- Reviewed head: `29ca2a1682d93ba48a47605624bdba1453866f72`
- Base head: `a4d7d3631bb30203ffa464bea32050a5a12caf67`
- Review date: 2026-07-19
- Result: five P1 and two P2 findings; no P0 finding

## Scope and independence

A fresh read-only project `math_reviewer` received only the bounded
REQ-CONVEX-001 summary and integrated dependency closure, Issue #81 acceptance
criteria and exclusions, the M4 plan, `SOLVER_POLICY`, ADR-0011, the complete
exact PR diff, tests, example, benchmark, dependency evidence, registry and
handoff state, and exact-head CI evidence. It inherited no implementation
reasoning and made no repository or remote change.

The reviewer checked formulae, signs, dimensions, objective conventions, exact
L2/L1/Huber epigraph equivalence, cone ordering and dual conventions, hard
constraints, original-unit KKT and certificate checks, solver settings, hidden
regularization, memory policy, allocations, provenance, determinism, public
type isolation, interfaces, dependency and benchmark claims, and registry
truth. SPD/CPD kernel classification, center limits, polynomial rank, fitted-
field Hessian capability, and geological cone construction are not exercised
by this canonical adapter. Lorentz rotational invariance is applicable and
algebraically preserved, but lacks a regression.

## Findings

### P1 R82-001: certificate stationarity can accept a false certificate

`crates/georbf/src/convex.rs:1280-1322` infinity-normalizes the certificate but
then compares `||A^T z||_inf` with the absolute threshold `64 * tolerance`,
without a scale derived from `A`.

For the feasible one-variable hard upper bound `epsilon * x <= -1`, with
`epsilon = 1e-12`, the compiled standard form is `A = [epsilon]`, `b = [-1]`,
and `K = R_+`. The bogus normalized vector `z = [1]` has zero dual-cone
violation, stationarity residual `1e-12`, separating value `-1`, and separator
scale `1`. At tolerance `1e-9` the current reviewer accepts it even though
`A^T z != 0` and the primal problem is feasible at `x = -1e12`.

Impact: a backend scaling failure or misclassification can be promoted to a
reviewed GeoRBF infeasibility result.

Required repair: normalize stationarity componentwise by a homogeneous
original-data absolute-product scale, with representability guards but no
dimensioned absolute floor. Retain the cone and strict-separator checks. Add a
private regression rejecting the example above and accepting a genuinely
infeasible two-row certificate and every positive rescaling of it.

### P1 R82-002: solved acceptance is row-scale dependent and looser than requested

`crates/georbf/src/convex.rs:31`, `1094-1149`, `1184-1241`, and `1358-1379`
apply an unrecorded `REVIEW_FACTOR = 64`. Several normalizations insert a raw
`1.0` into potentially dimensioned original-unit scales, while zero-cone slack
is checked absolutely.

For equivalent hard equalities `x = 0` and `alpha * x = 0`, take
`x = 1e-5`, requested tolerance `1e-9`, and `alpha = 1e-6`. The first review
rejects residual `1e-5`; the scaled row accepts residual `1e-11`. At the
loosest allowed requested tolerance, the hidden threshold is `6.4e-5`.

Impact: a nonzero equivalent row or unit scaling can change hard-constraint
acceptance, and diagnostics omit the effective review threshold.

Required repair: use recorded, dimensionally homogeneous scales for primal,
cone, hard-relation, complementarity, and gap review. Any roundoff guard beyond
the requested tolerance must be named, justified, and recorded. Add synthetic
private-review regressions for equivalent row scalings `1e-12`, `1`, and
`1e12`, requiring invariant decisions and normalized residuals.

### P1 R82-003: objective and duality-gap checks are not independent

`crates/georbf/src/convex.rs:1093-1099`, `1134-1149`, `1249-1263`, and
`1544-1553` calculate `original_objective` from compiler-owned auxiliaries and
the same compiled `P` and `q` sent to Clarabel. They do not evaluate
`sum rho_i(v_i(x_original) / s_i)` from the original canonical relations.
Thus an epigraph sign, scale, Huber coefficient, or quadratic-factor error is
reproduced by the alleged independent review. The reported duality gap is only
`abs(backend_primal - backend_dual)`; no dual objective is reconstructed.

With stationarity convention `P x + q + A^T z = 0`, the independent dual value
is `-0.5 * x^T P x - b^T z`, and the exact primal-dual gap equals `s^T z`.

Impact: correlated compiler or backend-reporting errors can pass checks that
the solver policy and change fragment describe as independent.

Required repair: recompute every soft loss directly from original variables,
relations, scales, and `SoftLoss`, compare semantic, compiled, and backend
primal objectives, and reconstruct the dual objective from `P`, `x`, `b`, and
`z`. Add fixed-nonzero-residual analytic L2, L1, inner-Huber, and outer-Huber
tests with a nonunit scale, plus internal perturbation rejection tests.

### P1 R82-004: the execution memory limit is ignored

`crates/georbf/src/convex.rs:836-854` and `906-918` accept
`ExecutionOptions`, but use and report only
`ConvexSolveOptions::memory_limit_bytes`. The execution-level limit defined at
`crates/georbf/src/problem_ir.rs:227-263` does not participate.

Impact: a caller can request a one-byte execution limit and the solver can
proceed under a larger convex-option limit.

Required repair: use and record the smaller nonzero limit, matching the dense
solver policy. Add a pre-dispatch regression with a large convex limit and a
one-byte execution limit that must return `MemoryLimitExceeded` with limit
one.

### P1 R82-005: the peak-memory limit is late and not a conservative bound

Compilation and provenance cloning occur before the check at
`crates/georbf/src/convex.rs:848-855`; relevant allocations occur at
`630-779`, `936-1017`, `1102`, `1231-1237`, and `1286-1300`. The estimate at
`1499-1525` is a fixed numeric-storage multiple. It omits arbitrarily sized
owned provenance and cannot bound sparse QDLDL symbolic or numeric fill.
Several post-check GeoRBF paths and the backend also allocate infallibly despite
the public `AllocationFailed` error.

Impact: work can exceed the explicit limit or fail allocation before returning
the promised structured memory error; the documented conservative peak-working-
set claim is unsupported.

Required repair: perform a nonallocating checked preflight before cloning,
include owned metadata and every auxiliary allocation, and include a symbolic
backend fill/workspace bound or narrow the API and documentation to an input-
storage estimate. Make GeoRBF-owned post-check allocations fallible. Add large-
provenance pre-allocation and adversarial sparse-fill regressions with allocation
accounting.

### P2 R82-006: material Clarabel settings remain implicit

`crates/georbf/src/convex.rs:219-249` and `1020-1060` rely on
`..DefaultSettings::default()` for material Clarabel 0.11.1 policy, including
`tol_ktratio`, equilibration bounds, maximum step and termination lengths,
direct-KKT enablement, iterative-refinement tolerances and stop ratio, sparse-
zero handling, and reduced-status tolerances. The selected direct solver
`qdldl` is absent from `ConvexSettingsDiagnostics`.

Static and dynamic KKT regularization and presolve are genuinely disabled; no
hidden fallback or hard-to-soft conversion was found. The defect is incomplete
explicitness and reproducibility.

Required repair: assign every relevant pinned-backend setting explicitly and
record it in GeoRBF-owned diagnostics, including the direct solver and effective
independent-review policy. Add an internal exact settings snapshot that fails
when a material backend default lacks a recorded disposition.

### P2 R82-007: regressions do not establish central epigraph and review claims

`crates/georbf/tests/convex_solver.rs:90-172`, `239-322`, and `372-415`, plus
`crates/georbf/examples/convex_solver.rs:44-83`, do not independently check the
analytic objectives or rejection boundaries claimed in
`changes/REQ-CONVEX-001.md:32-36`.

The QP test checks only an optimizer unchanged by a uniform quadratic-factor
error. The mixed test permits zero Huber, interval, and cone violations, and
the Huber example also reaches objective zero. Huber transition behavior,
outer constant, residual scale, violated soft bounds and cones, bogus
certificates, normalization, strict separator, non-`Solved` routing, equivalent
row scaling, and Lorentz rotation are not exercised.

Impact: current tests cannot detect several central sign, factor, scaling, and
review-oracle regressions.

Required repair: add table-driven fixed-residual analytic objective cases for
L2, L1, both Huber branches, a nonunit scale, a violated soft bound, and a
violated soft cone. Add a bogus-certificate case, forced non-`Solved` routing,
equivalent-row scaling, and orthogonal Lorentz-rotation tests.

No other P0, P1, P2, or P3 finding was reported.

## Independent mathematical review

Writing each stored slack as `s(x) = c + Cx`, the adapter emits `A = -C` and
`b = c`, so `A x + s = b`. Equality, lower-bound, upper-bound, and ordered SOC
rows therefore have the correct signs. The Lorentz and nonnegative cones are
self-dual; the zero cone has the full dual space, and the implemented dual
signs and cone ordering are correct.

Squared L2 sets `P_vv = 2`, so `0.5 * 2 * v^2 = v^2`. L1 sets `q_v = 1`.
Huber minimizes `0.5 * q^2 + delta * l` subject to nonnegative `q,l` and
`q + l >= v`; its minimizer gives `0.5 * v^2` below the transition and
`delta * (v - delta / 2)` above it. These epigraph formulae are correct. The
diagonal nonnegative Hessian is PSD, not necessarily PD.

A primal-infeasibility certificate requires nonzero `z in K*`, exact
`A^T z = 0`, and `b^T z < 0`; positive rescaling is immaterial after infinity
normalization. R82-001 concerns the numerical acceptance oracle, not the
algebraic certificate convention.

## Validation and disposition

- Local and remote branch heads matched the exact reviewed head; the worktree
  was clean before this evidence-only Review change.
- Draft CI run 29683566407 passed the complete configured Ubuntu Draft job on
  the exact head. The Ready-only Windows, Ubuntu, and macOS workspace and
  benchmark-smoke matrix correctly did not run.
- The supplied stable-head standard gate and focused test, example, benchmark,
  and dependency evidence were treated as execution evidence, not as an
  independent mathematical oracle.
- Exact dependency pin and feature isolation, public backend-type isolation,
  hard/soft algebra, PSD classification, status mapping, disabled presolve and
  KKT regularization, serial QDLDL selection, interface N/A dispositions,
  benchmark qualification, and `implemented` registry state are truthful.

At the original reviewed head, PR #82 remained Draft and REQ-CONVEX-001
remained `implemented`, not `integrated`. The review required a fresh Repair
limited to R82-001 through R82-007, followed by another independent re-review.
That Review task made no production-code change and did not begin
REQ-INFEAS-001; the Repair evidence follows.

## Repair evidence pending fresh re-review

Repair code/test head `55f339c5d80666b089d2e2bdfae03a8b2029ae12`
implements only R82-001 through R82-007:

- R82-001: certificate stationarity is reviewed componentwise against scaled
  original-data absolute products, while the separator uses the same
  homogeneous construction and rejects unrepresentable nonzero products. The
  feasible `1e-12 * x <= -1` bogus certificate is rejected; a true two-row
  certificate is accepted at positive row scales `1e-12`, `1`, and `1e12`.
- R82-002: the factor-64 threshold and dimensioned raw floors are removed.
  Primal, dual, cone, hard-relation, complementarity, and semantic gap checks
  use homogeneous scales and exactly the requested tolerance. The only unit
  reference is the documented dimensionless count of unit-weight soft losses.
  Synthetic equality review decisions are invariant at row scales `1e-12`,
  `1`, and `1e12`.
- R82-003: the semantic objective is evaluated from original variables,
  relations, scales, and loss definitions; compiled and backend primal values
  are separate comparisons. The dual value is reconstructed as
  `-0.5 * x^T P x - b^T z`, and the recorded gap uses the semantic primal and
  reconstructed dual. A private compiler perturbation is rejected.
- R82-004: the effective limit is the smaller nonzero convex-option and
  execution limit, and all three values are recorded. A one-byte execution
  limit rejects before dispatch even with a larger convex limit.
- R82-005: a nonallocating preflight now runs before compiler/provenance
  cloning, includes owned provenance and auxiliary storage, and bounds QDLDL
  fill by the dense lower triangle of the full KKT dimension. Subsequent
  GeoRBF-owned vectors and provenance copies use fallible reservation. Large
  provenance and a 128-variable, one-coefficient adversarial-fill case exercise
  the accounting.
- R82-006: every material field in Clarabel 0.11.1 `DefaultSettings` is assigned
  without a default tail and mirrored by GeoRBF diagnostics, including direct
  QDLDL, reduced tolerances, equilibration bounds, step lengths, refinement
  thresholds, disabled-setting constants, and sparse-zero handling. An exact
  snapshot test covers every available field and the independent-review
  tolerance policy.
- R82-007: five private tests and nine end-to-end tests now cover the requested
  certificate, scale, objective, effective-memory, fill, settings, status,
  nonzero L2/L1/inner-Huber/outer-Huber, nonunit-scale, violated soft-bound and
  soft-cone, and Lorentz-rotation cases.

Focused warning-denying all-target/all-feature Clippy, all five private repair
tests, all nine convex integration tests, the runnable example, and the 8/16
release smoke workload passed. The smoke checksums remain
`4.00000000000000444` and `7.99999999999999911`; this repair run measured
1.0105 and 0.8482 ms and is not a performance promise.

After the last production/test change, exact code/test head
`55f339c5d80666b089d2e2bdfae03a8b2029ae12` passed the complete standard gate:
workspace format, warning-denying all-target/all-feature Clippy, all-feature
workspace tests, workspace Rustdoc, all 58 requirement checks, and
`git diff --check`. The following repair-record, solver-policy, change-fragment,
and bounded-handoff edits change only documentation; they do not change
production, test, manifest, schema, CI, build, registry, API, numerical, or
dependency inputs. A fresh independent re-review is still required before any
finding is considered closed or PR #82 is marked ready.
