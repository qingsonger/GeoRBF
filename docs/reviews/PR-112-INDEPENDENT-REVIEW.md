# PR #112 Independent Review

- Requirement: REQ-ANISO-003
- Issue: https://github.com/qingsonger/GeoRBF/issues/111
- Pull request: https://github.com/qingsonger/GeoRBF/pull/112
- Branch: `codex/req-aniso-003-export-diagnostics`
- Reviewed head: `a698362a420dfe3743471cefe2fa14a52b76e991`
- Stable full-gate head: `98c72dcda7a348fdcded0d0daa5703c87d192c9a`
- Base head: `37cb91d`
- Review date: 2026-07-23
- Result: P2 finding ANISO003-REV-001 requires Repair; no other P0-P3
  finding was identified

## Scope and independence

A fresh read-only project `math_reviewer` received only the bounded
REQ-ANISO-003 summary and integrated dependency closure, Issue #111 acceptance
criteria and exclusions, the M6 plan, ANISOTROPY and ADR-0005/ADR-0008
contracts, the complete PR diff and its tests, and the recorded validation and
CI evidence. It inherited no Implement reasoning and made no repository, Git,
or GitHub change.

The reviewer independently checked formulae, signs, dimensions, units,
SPD/CPD classification, sign and rotation invariance, positive definiteness,
Hessian capability, independent truth, finite evaluation, fallible allocation,
deterministic order, hidden numerical adjustments, public interfaces,
diagnostics, documentation, tests, and requirement truthfulness. Polynomial
spaces, rank decisions, center kernel limits, hard constraints, infeasibility,
and solver behavior do not apply because this export neither evaluates a fixed
kernel nor assembles or solves a field system.

## Findings

### ANISO003-REV-001 - P2: required independent schema and condition truth is incomplete

Affected evidence:

- `crates/georbf/tests/anisotropy_diagnostics.rs:133`
- `crates/georbf/tests/anisotropy_diagnostics.rs:134`
- `crates/georbf/tests/anisotropy_diagnostics.rs:143`

The `diagnostic_schema_preserves_controls_background_weights_and_coverage`
test checks only that the first control condition number is at least one. Its
spheroidal assertion retains only the two lengths, and its ellipsoidal assertion
retains only the length array. It does not inspect the exported direction
components, per-axis provenance, ellipsoid orthogonality tolerance, the second
control condition number, or the summary maximum condition number.

Consequently a regression that replaces or swaps resolved axes while leaving
the lengths unchanged, reports either control condition number as one, or
misreports the summary maximum would leave the required independent test green.
That falls short of Issue #111's explicit spheroidal/ellipsoidal representation,
resolved axis-and-length pairing, provenance, and condition-evidence acceptance
criteria.

For independent truth, an orthogonal anisotropy transform has row
`u_i^T / ell_i`, so its singular values are `1 / ell_i` and
`kappa_2(A) = max(ell_i) / min(ell_i)`. The existing test data therefore require
condition numbers two for the `(3, 1.5)` spheroid, four for the `[4, 1]`
ellipsoid, one for the isotropic background, and four for the mixture summary.
The exported spheroid axis must be `(1, 0)`. The ellipsoid axes must remain in
caller order `(-1, 0)`, `(0, 1)`, paired respectively with lengths four and
one, with explicit provenance and the caller's orthogonality tolerance.

A fresh Repair must extend that existing test with exact assertions for those
axis components, provenance values, axis-length pairing, tolerance, both
control condition numbers, and the summary maximum. No production-code change
is indicated by this finding unless the new regression exposes one.

No additional P0, P1, P2, or P3 finding was identified.

## Validation and disposition

- The parent Review task passed all four public `anisotropy_diagnostics`
  integration tests, the D=4 compile-fail Rustdoc test, the runnable example,
  and complete diff whitespace validation on reviewed head `a698362`.
- Draft CI passed its configured Ubuntu correctness gate on the same reviewed
  head. The Ready-only Windows, Ubuntu, macOS, and benchmark-smoke matrix was
  skipped as designed and is not claimed as passed.
- Exact stable head `98c72dc` retains the complete Implement gate: workspace
  format, warning-denying all-target/all-feature Clippy, all-feature workspace
  tests, workspace Rustdoc, all 58 requirement checks, and complete diff
  whitespace validation. The tail through reviewed head `a698362` changes only
  Markdown validation evidence.
- The full workspace gate was not rerun in this Review task. Nextest, deny,
  audit, semver, Miri, sanitizers, fuzzing, mutation testing, and API/ABI/schema
  snapshots remain unavailable or deferred. No unexecuted check is claimed as
  passed.
- Signed weights and squared coverage have the documented dimensions and
  independent hand calculation. The export preserves the strict-background
  diagonal-congruence SPD proof, adds no CPD path, changes no Hessian
  capability, and performs no jitter, clipping, pseudoinverse, regularization,
  refit, or field mutation.

PR #112 remains Draft and REQ-ANISO-003 remains `in_progress`, not integrated.
A fresh Repair task must address only ANISO003-REV-001, run the focused tests
and one complete stable-head standard gate after its final test change, update
this record and the bounded handoff, push, and stop for fresh independent
re-review. Do not begin another requirement.
