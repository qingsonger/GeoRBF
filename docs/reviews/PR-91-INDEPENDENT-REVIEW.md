# PR #91 Independent Review

- Requirement: REQ-TANGENT-001
- Issue: https://github.com/qingsonger/GeoRBF/issues/90
- Pull request: https://github.com/qingsonger/GeoRBF/pull/91
- Branch: `codex/req-tangent-001-tangent-constraints`
- Reviewed head: `86d1d3dcc948d70f6825822d1efe94b92b8b4f5b`
- Base head: `60952be9cd84c098c09482b5373ec2e7665d0e7e`
- Review date: 2026-07-20
- Result: P2 R91-001; repair required

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

PR #91 must remain Draft and REQ-TANGENT-001 must remain `implemented`. Open a
fresh Repair task limited to R91-001, add the required independent regressions,
run focused checks and the final stable-head standard gate, update this review
evidence and the bounded handoff, push, and stop for a fresh independent
re-review. Do not begin REQ-THICK-001 or any other requirement.
