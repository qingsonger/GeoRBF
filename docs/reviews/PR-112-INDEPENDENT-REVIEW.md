# PR #112 Independent Review

- Requirement: REQ-ANISO-003
- Issue: https://github.com/qingsonger/GeoRBF/issues/111
- Pull request: https://github.com/qingsonger/GeoRBF/pull/112
- Branch: `codex/req-aniso-003-export-diagnostics`
- Reviewed head: `a698362a420dfe3743471cefe2fa14a52b76e991`
- Repair head: `4426a303f5f9e22985514330a8f8194dd7ea887d`
- Stable full-gate head: `4426a303f5f9e22985514330a8f8194dd7ea887d`
- Base head: `37cb91d`
- Review date: 2026-07-23
- Result: Repair evidence recorded for ANISO003-REV-001; fresh independent
  re-review is required before the finding can be closed

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

## Repair evidence

Repair head `4426a30` changes only the independent
`diagnostic_schema_preserves_controls_background_weights_and_coverage` test.
It adds exact assertions for:

- the spheroid axis `(1, 0)`, explicit provenance, and paired lengths
  `(3, 1.5)`;
- the caller-ordered ellipsoid axes `(-1, 0)` and `(0, 1)`, explicit
  provenance for both, paired lengths `[4, 1]`, and the caller's exact
  `8 epsilon` orthogonality tolerance;
- per-control condition numbers two and four; and
- the mixture summary maximum condition number four.

The regression passes against the existing implementation, so this Repair
changes no production code, API, numerical behavior, manifest, registry,
schema, or dependency. All four focused diagnostic integration tests and
warning-denying focused Clippy pass. Exact repair head `4426a30` also passes
workspace format, warning-denying workspace all-target/all-feature Clippy,
all-feature workspace tests, all workspace Rustdoc tests, all 58 requirement
checks, and complete diff whitespace validation.

This evidence shows the requested regression was added but does not
self-resolve the independent finding. A fresh read-only re-review must confirm
ANISO003-REV-001 is closed and check for new P0-P3 findings.

## Validation and disposition

- The parent Review task passed all four public `anisotropy_diagnostics`
  integration tests, the D=4 compile-fail Rustdoc test, the runnable example,
  and complete diff whitespace validation on reviewed head `a698362`.
- Draft CI passed its configured Ubuntu correctness gate on the same reviewed
  head. The Ready-only Windows, Ubuntu, macOS, and benchmark-smoke matrix was
  skipped as designed and is not claimed as passed.
- Exact repair head `4426a30` now supplies the complete stable full gate.
  Nextest, deny, audit, semver, Miri, sanitizers, fuzzing, mutation testing,
  and API/ABI/schema snapshots remain unavailable or deferred. No unexecuted
  check is claimed as passed.
- Signed weights and squared coverage have the documented dimensions and
  independent hand calculation. The export preserves the strict-background
  diagonal-congruence SPD proof, adds no CPD path, changes no Hessian
  capability, and performs no jitter, clipping, pseudoinverse, regularization,
  refit, or field mutation.

PR #112 remains Draft and REQ-ANISO-003 remains `in_progress`, not integrated.
A fresh Review task must independently re-review repair head `4426a30`, confirm
whether ANISO003-REV-001 is closed, and check for new findings. Do not begin
another requirement.
