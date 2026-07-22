# PR #109 Independent Review

- Requirement: REQ-TREND-002
- Issue: https://github.com/qingsonger/GeoRBF/issues/108
- Pull request: https://github.com/qingsonger/GeoRBF/pull/109
- Branch: `codex/req-trend-002-region-controls`
- Reviewed head: `9781e8fd6ac140b444a3858c3e1ce38565f42f85`
- Base head: `8535880c2d9cf2d580ac97bddf0610f9f6a68f61`
- Review date: 2026-07-22
- Result: P1 TREND002-REV-001 and P2 TREND002-REV-002/003 require Repair

## Scope and independence

A fresh read-only project `math_reviewer` received only the bounded
REQ-TREND-002 summary and integrated dependency closure, Issue #108 acceptance
criteria and exclusions, the M6 plan, ANISOTROPY and ADR-0005/ADR-0008
contracts, the exact PR diff, tests, benchmark and CI wiring, registry,
handoff, and validation evidence. It inherited no Implement reasoning and made
no repository or remote change.

The reviewer independently checked formulae, signs, dimensions, units,
SPD/CPD classification, center and boundary limits, rotation invariance,
positive definiteness, Hessian and C2 capabilities, finite and extreme
arithmetic, independent truth, allocations, hidden numerical adjustments,
public interfaces, diagnostics, tests, documentation, CI, and requirement
truthfulness. Polynomial rank, hard-constraint, infeasibility, and solver paths
do not apply to this compiler requirement.

## Findings

### TREND002-REV-001 - P1: regional derivatives lose representable values

Affected code:

- `crates/georbf/src/local_trend.rs:1415-1423`
- `crates/georbf/src/local_trend.rs:1445-1450`

`smootherstep_jet` first forms the dimensionless `t^2` and the cancellation-
prone expanded second derivative `60 t (2 t^2 - 3 t + 1)`. Only afterward does
`axis_region_gate` multiply by `1 / width` or `1 / width^2`. Accepted narrow
transition widths can therefore erase a physical derivative that remains
finite and representable.

For valid `width = 1e-153`, `x = 1e-323`, and
`t = x / width ~= 9.88131291682493e-171`, represented `t^2` underflows and the
implementation returns a zero first derivative. Independent evaluation of

```text
S'(t) / width = 30 t^2 (1 - t)^2 / width
```

gives approximately `2.929210348806549e-186`, which is finite and
representable. At `x = next_down(width)`, the represented parameter is
`next_down(1)`. The expanded second-derivative factor rounds to zero, while the
equivalent factored expression

```text
60 t (t - 1) (2 t - 1) / width^2
```

is approximately `-6.661338147750937e291`, also finite and representable.

Impact: accepted input silently loses required regional gradient and Hessian
terms, violating the C2/product-rule derivative and explicit representability
contracts. A fresh Repair must add private one-dimensional `axis_region_gate`
regressions at both points, compare against independently scaled and factored
analytic truth, and require finite nonzero derivatives before implementing the
smallest scale-safe evaluation.

### TREND002-REV-002 - P2: exact compact support can error outside the region

Affected code:

- `crates/georbf/src/local_trend.rs:1509-1518`
- `crates/georbf/src/local_trend.rs:1666-1672`

The region jet is known before the Gaussian displacement, but the evaluator
always forms `point - center` before using an exactly zero gate. For region
`[-1, 1]`, center `-f64::MAX`, and finite query `f64::MAX`, the gate and all
demanded derivatives are exactly zero, so the regional basis jet is
identically zero. The subtraction nevertheless overflows and returns
`NonFiniteWeightDisplacement`.

Impact: a compactly supported weight does not provide its documented exact
zero behavior outside the region. A fresh Repair must add a coverage or
private regional-weight regression through Hessian order for that configuration
and require a zero regional contribution without error before short-circuiting
the provably zero gate.

### TREND002-REV-003 - P2: required independent truth evidence is incomplete

Affected evidence:

- `crates/georbf/tests/trend_controls.rs:99-150`
- `crates/georbf/tests/trend_controls.rs:210-250`
- `crates/georbf/tests/trend_controls.rs:378-441`
- `crates/georbf/tests/trend_controls.rs:445-515`
- `changes/REQ-TREND-002.md:41-45`

The spheroidal/ellipsoidal test checks ordering, diagnostics, retained lengths,
and coverage, but not the compiled transform, metric, or manual kernel truth.
The finite-difference test checks only diagonal Hessian entries. No compiler-
level excessive-condition rejection is tested. Reference-gradient failures
cover only missing-project and below-minimum paths, not unknown fields,
unavailable evaluation, zero gradients, or unrepresentable norms. The change
fragment therefore overstates satisfaction of Issue #108's explicit
independent-test acceptance criterion.

Impact: required mathematical and structured-error behavior can regress while
the stated evidence remains green. A fresh Repair must add:

1. rotated spheroidal and ellipsoidal comparisons against hand-formed `B`
   matrices or manual kernel evaluations;
2. compiler rejection of a known excessive length ratio under an explicit
   maximum-condition policy;
3. an independent finite-difference check of a mixed regional Hessian entry;
4. a compact reference-gradient error table for unknown, unavailable, zero,
   and unrepresentable cases.

No additional P0, P1, P2, or P3 finding was identified.

## Validation and disposition

- The reviewer verified the exact base/head, merge base, twelve-file PR diff,
  and clean scoped worktree. The tail from stable evidence head `22760ef` to
  reviewed head `9781e8f` changes only `docs/progress/CURRENT.md`.
- The reviewer and parent Review task each passed all six public
  `trend_controls` tests. The parent also ran the public example successfully.
- Draft CI passed its configured Ubuntu correctness gate on exact reviewed
  head `9781e8f`. The Ready-only Windows, Ubuntu, macOS, and benchmark-smoke
  matrix was skipped as designed and is not claimed as passed.
- Exact stable evidence head `22760ef` retains the complete standard local gate
  recorded by Implement: workspace format, warning-denying all-target/all-
  feature Clippy, all-feature workspace tests, workspace Rustdoc, all 58
  requirement checks, and complete diff whitespace validation.
- The full workspace gate and benchmark were not rerun in this Review task.
  `actionlint`, nextest, deny, audit, semver, Miri, sanitizers, fuzzing,
  mutation testing, allocation instrumentation, and API/ABI/schema snapshots
  remain unavailable or deferred. No unexecuted check is claimed as passed.
- The fixed-kernel diagonal-congruence construction and constant strict-PD
  background otherwise preserve SPD; CPD kernels are explicitly rejected;
  direction-jump signs, dimensions, and reference normalization are otherwise
  correct; Hessian capability remains kernel-gated; and no hidden
  regularization or new dependency was found.

PR #109 remains Draft and REQ-TREND-002 remains `implemented`, not
`integrated`. A fresh Repair task must address only TREND002-REV-001,
TREND002-REV-002, and TREND002-REV-003, add the required regressions, run
focused checks and one complete stable-head standard gate after the final code
change, update this record and the bounded handoff, push, and stop for a fresh
independent re-review. Do not begin another requirement.

## Repair evidence pending fresh independent re-review

Repair code/test/evidence head `5f357891de88f9d3f64ed671e2769d8f6ce84c66`
addresses only the three findings above and remains pending a fresh independent
re-review:

- TREND002-REV-001: physical smootherstep derivatives are now formed with the
  width scale already present and the second derivative uses the factored
  `60 t (t - 1) (2 t - 1) / width^2` expression. Private D=1 regressions retain
  finite nonzero first and second derivatives at both reviewed extreme points.
- TREND002-REV-002: an exactly zero regional jet returns before Gaussian
  displacement formation. A private Hessian-order regression uses region
  `[-1, 1]`, center `-f64::MAX`, and query `f64::MAX` and requires an exact
  zero jet without error.
- TREND002-REV-003: public independent evidence now compares rotated
  spheroidal and ellipsoidal metrics with hand-formed `B`, rejects a 100:1
  length ratio under maximum condition 10, checks a mixed regional Hessian by
  four-point finite differences, and tables structured unknown-field,
  unavailable-at-center, zero-gradient, and unrepresentable-norm failures.

Focused validation passed all nine public `trend_controls` tests, all three
private repair regressions, the runnable example, and the release-mode focused
benchmark smoke (approximately 11.4 us for four controls and 43.0 us for
sixteen controls on this development machine). Exact repair head `5f35789`
passed workspace format, warning-denying workspace all-target/all-feature
Clippy, all-feature workspace tests, workspace Rustdoc, all 58 requirement
checks, and complete diff whitespace validation. The first full-gate attempt
stopped at Clippy on test-only lint violations; those tests were corrected and
the complete gate was rerun from the beginning to green.

This evidence does not close the independent findings by itself. PR #109
remains Draft and REQ-TREND-002 remains `implemented`, not `integrated`, until
a fresh read-only re-review confirms the repairs and checks for new findings.
