# PR #91 Independent Review

- Requirement: REQ-TANGENT-001
- Issue: https://github.com/qingsonger/GeoRBF/issues/90
- Pull request: https://github.com/qingsonger/GeoRBF/pull/91
- Branch: `codex/req-tangent-001-tangent-constraints`
- Reviewed head: `86d1d3dcc948d70f6825822d1efe94b92b8b4f5b`
- Base head: `60952be9cd84c098c09482b5373ec2e7665d0e7e`
- Review date: 2026-07-20
- Result: clean independent re-review; Ready CI pending

## Scope and independence

A fresh read-only project `math_reviewer` received only the bounded
REQ-TANGENT-001 summary and integrated dependency closure, Issue #90 acceptance
criteria and exclusions, the M5 plan, relevant tangent, functional,
problem-IR, architecture, and ADR contracts, the exact PR diff, tests, example,
benchmark, registry hunk, handoff, and validation evidence. It inherited no
implementation reasoning and made no repository or remote change.

The reviewer checked formulae, signs, dimensions, units, tangent reversal,
derivative-gauge semantics, hard and soft enforcement, deterministic order,
provenance, finite input delegation, count and allocation failures, hidden
regularization, interfaces, benchmark and CI wiring, and registry truth. It
also classified kernel, center, polynomial, rank, SPD/CPD, anisotropy, and
Hessian concerns as outside this observation-side semantic-lowering diff.

## Findings

### P2 R91-001: missing-gauge diagnostics are delayed until after the entire iterator is consumed

`crates/georbf/src/tangent_observations.rs:206-225` reads the observation
iterator's lower size hint, computes and reserves its requested storage, and
then drains the entire iterator. The constructor does not inspect the already
known `gauge_anchor` absence until lines 226-232. Consequently a nonempty
derivative-only input without a gauge can return `CountOverflow` or
`AllocationFailed`, or never return for an unbounded iterator with a small
lower bound, instead of the documented source-aware `GEORBF-E4001` missing-
gauge diagnostic.

For the smallest deterministic case, `std::iter::repeat(valid_tangent)` has a
`usize::MAX` lower size hint. With `gauge_anchor = None`, line 209 overflows
before the constructor can use the first tangent's source at line 227. The
absence of the gauge is already known, and only the first tangent is
mathematically necessary to establish the derivative-only additive freedom and
identify the diagnostic source. This contradicts the public contract at
`crates/georbf/src/tangent_observations.rs:192-199`,
`docs/math/NORMAL_AND_TANGENT.md:176-178`, and
`changes/REQ-TANGENT-001.md:15-18`.

Required repair: when no gauge is supplied, inspect only enough of the iterator
to distinguish an empty problem from a nonempty one and return the first
tangent's `MissingGauge` diagnostic without reserving or consuming the
remainder. Add a regression using `std::iter::repeat(valid_tangent)` with no
gauge that requires `MissingGauge`, `GEORBF-E4001`, and the repeated tangent's
identifier. A companion case with an explicit gauge must continue to require
the structured `CountOverflow` result.

No other P0, P1, P2, or P3 finding was reported.

## Repair evidence (not an independent re-review)

Repair implementation head `5e99aa629118ca4b4c81927d31adf67f19822b58`
moves the already-known missing-gauge branch before iterator size inspection,
reservation, and collection. With no gauge, the constructor reads exactly the
first tangent: an empty iterator still returns `EmptyTangentProblem`, while a
nonempty iterator immediately returns source-aware `GEORBF-E4001`. With an
explicit gauge, the original complete collection path and its checked count
overflow remain unchanged.

Two independent integration regressions use `std::iter::repeat(valid_tangent)`.
The missing-gauge case requires `ErrorCode::MissingGauge`, display code
`GEORBF-E4001`, and the first tangent identifier; the explicit-gauge companion
requires `TangentProblemError::CountOverflow`. This repair changes no formula,
sign, units, enforcement, canonical relation, adapter disposition, dependency,
registry status, or later-requirement scope. A fresh project `math_reviewer`
must still independently confirm that R91-001 is closed and that no new finding
was introduced.

## Independent mathematical review

The scalar row is otherwise correct: `t^T grad f(x) = 0` lowers with a positive
unit coefficient, zero expression constant, and zero target, without a center-
argument sign. A validated unit direction makes residual scale independent of
input tangent magnitude, and hard equality plus scalar SquaredL2, AbsoluteL1,
and Huber losses are invariant under `t -> -t`.

The finite hard value row `f(x_anchor) = value_anchor` removes the global
additive constant left by derivative-only observations. No point, zero value,
relaxation, jitter, regularization, pseudoinverse, or hard-to-soft conversion is
invented. D=1/D=2/D=3 bounds, same-point multiple tangents, input order,
semantic provenance, hard/soft separation, duplicate identifiers, and the
deferred adapter dispositions are otherwise consistent with the scoped
contracts.

## Validation and disposition

- Local and remote branch heads matched reviewed implementation head `86d1d3d`;
  the worktree was clean before this evidence-only Review change.
- Draft CI run 29729498305 passed the configured Ubuntu correctness job on
  exact implementation head `86d1d3d`. The Ready-only Windows, Ubuntu, macOS,
  and benchmark-smoke matrix correctly did not run.
- The parent task passed all six focused tangent integration tests, both
  diagnostic/allocation unit regressions, the runnable example, benchmark smoke
  with checksum `3824`, all 58 requirement checks, and the complete PR diff
  whitespace check.
- After adding the evidence files, the parent task passed the complete standard
  gate: workspace format, warning-denying all-target/all-feature Clippy,
  all-feature workspace tests, workspace doctests, all 58 requirement checks,
  and `git diff --check`.
- Exact implementation head `86d1d3d` retains its recorded complete local gate.
  This final evidence wording changes only this review record and the bounded
  handoff; it changes no production code, test, manifest, schema, CI, build
  input, API, numerical behavior, registry, or dependency input.
- Repair implementation head `5e99aa6` passed all eight tangent integration
  tests, both tangent module regressions, the runnable example, benchmark smoke
  checksum `3824`, workspace format, warning-denying all-target/all-feature
  Clippy, all-feature workspace tests, workspace doctests, all 58 requirement
  checks, and `git diff --check`.

PR #91 remains Draft and REQ-TANGENT-001 remains `implemented`. Open a fresh
independent re-review task for exact PR head and R91-001. If no P0-P3 finding
remains, follow the repository's Ready-CI integration sequence. Do not begin
REQ-THICK-001 or any other requirement.

## Final independent re-review

A fresh read-only project `math_reviewer` independently reviewed exact PR head
`ab84fda560229fcb8e8c2ccf0e0361bba3751f30` against base
`60952be9cd84c098c09482b5373ec2e7665d0e7e`. It received only the bounded
requirement and dependency summaries, Issue #90 criteria and exclusions, M5
scope, relevant mathematical, architecture, problem-IR, and ADR contracts, the
complete exact PR and focused repair diffs, prior finding and repair evidence,
tests, example, benchmark, registry, handoff, CI workflow, and validation
evidence. It inherited no Implement or Repair reasoning and made no repository
or remote change.

- R91-001 is closed. The missing-gauge path calls `iterator.next()` exactly
  once before any size inspection, reservation, or collection. A nonempty
  iterator immediately returns source-aware `GEORBF-E4001`, while the empty
  case retains `EmptyTangentProblem`.
- The explicit-gauge path retains its checked iterator lower-bound plus gauge
  arithmetic, so `std::iter::repeat(valid_tangent)` still returns the
  structured `CountOverflow` result. The paired regressions require this
  distinction and the missing-gauge observation identifier.
- The scalar equality `t^T grad f(x) = 0` retains the correct positive query-
  derivative sign and zero target. Reversing `t` negates the residual, leaving
  hard equality and scalar SquaredL2, AbsoluteL1, and Huber enforcement
  invariant. A validated unit tangent separates geometry from residual scale.
- The finite hard value row `f(x_anchor) = value_anchor` removes the global
  additive-constant freedom without claiming to resolve unrelated polynomial
  or rank deficiency. CPD polynomial review remains in the existing assembly
  path.
- Hard and soft semantics, deterministic tangent-then-gauge order, duplicate-
  identifier rejection, complete semantic provenance, finite geometry and
  value validation, allocation and count errors, deferred adapter
  dispositions, benchmark wiring, and the `implemented` registry state remain
  consistent with the scoped contracts.
- No automatic gauge, jitter, regularization, pseudoinverse, constraint
  relaxation, center selection, solver vocabulary, or kernel behavior was
  introduced. Kernel formulae and center limits, complete polynomial spaces,
  rank decisions, SPD/CPD classification, anisotropy, and Hessian capabilities
  are unchanged and outside this observation-side semantic-lowering diff.

No P0, P1, P2, or P3 finding remains. The parent task passed the five standard
checks, `git diff --check`, all eight focused tangent integration tests, both
tangent module regressions, the runnable example, and benchmark smoke checksum
`3824` on the same exact head. Draft CI run 29731323902 also passed its complete
configured Ubuntu correctness gate on `ab84fda`; the Ready-only matrix correctly
did not run.

This evidence-only change updates only this review record and
`docs/progress/CURRENT.md`; it changes no production, test, manifest, schema,
CI, build, API, numerical, registry, or dependency input. PR #91 may proceed to
Ready CI. REQ-TANGENT-001 remains `implemented`, not `integrated`, until the
exact Ready evidence head passes the complete Windows, Ubuntu, and macOS
correctness and benchmark-smoke matrix, PR #91 merges exactly once, and the
isolated integration-state change completes.

## Integration evidence

The implementation integration sequence is complete. Exact Ready evidence head
`e780ad977848dddf1c90f259e0447222e4a22d9a` passed the complete Windows,
Ubuntu, and macOS correctness matrix, every configured backend path, all
benchmark-smoke workloads including `tangent_observation_compilation`, and the
requirement-registry gate in CI run 29732074353. PR #91 then squash-merged
exactly once as `968afe9c758e4fbebb2c5c04832b3d1a9b529c0d`, and Issue #90
closed as completed. Post-merge `main` run 29732840766 passed the same complete
three-platform gate on that exact merge commit.

The isolated integration-state change records only the registry, review
evidence, history index, and bounded handoff. After its own complete local and
exact Ready-head CI gates are green and that pull request is merged, stop. A
fresh task must select the next requirement; this task must not begin it.
