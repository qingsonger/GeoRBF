# PR #112 Independent Review

- Requirement: REQ-ANISO-003
- Issue: https://github.com/qingsonger/GeoRBF/issues/111
- Pull request: https://github.com/qingsonger/GeoRBF/pull/112
- Branch: `codex/req-aniso-003-export-diagnostics`
- Reviewed head: `a698362a420dfe3743471cefe2fa14a52b76e991`
- Repair head: `4426a303f5f9e22985514330a8f8194dd7ea887d`
- Re-reviewed head: `c6b2c2632e72af0c18820d08b4e27b2f91029301`
- Stable full-gate head: `4426a303f5f9e22985514330a8f8194dd7ea887d`
- Base head: `37cb91d`
- Review date: 2026-07-23
- Result: ANISO003-REV-001 independently closed; no P0-P3 finding remains

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

This evidence did not self-resolve the independent finding. A fresh read-only
re-review subsequently confirmed that the repair closes ANISO003-REV-001 and
introduced no new P0-P3 finding.

## Independent re-review

A fresh isolated read-only project `math_reviewer` re-reviewed exact head
`c6b2c26`. It received only the bounded requirement and dependency summaries,
Issue #111, the M6 plan, ANISOTROPY and ADR-0005/ADR-0008 contracts, the
complete repaired PR diff, the original finding, and validation evidence. It
inherited no Implement or Repair reasoning and changed no repository, Git, or
GitHub state.

ANISO003-REV-001 is closed. The repaired schema regression now independently
asserts the exact spheroid and ellipsoid axes, explicit provenance, caller
axis-length pairing, ellipsoid tolerance, both control condition numbers, and
the mixture summary maximum. For the orthogonal fixtures, rows
`u_i^T / ell_i` have singular values `1 / ell_i`; therefore the `(3, 1.5)`
spheroid has condition two, the `[4, 1]` ellipsoid has condition four, the
isotropic background has condition one, and the mixture maximum is four.

The reviewer also independently confirmed the signed sample weight
`-1.5 exp(-1/2) = -0.90979598956895014`, squared-weight coverage
`5.0777287426357454`, antipodal zero jump, and orthogonal `pi/2` jump. Control,
component, sample, and low-confidence records preserve their required
deterministic order. Export uses the existing value-only weight path with
structured sample/component errors and fallible reservations. It does not
change the fixed-SPD mixture proof, add a CPD path, alter Hessian capability,
or introduce clipping, jitter, regularization, a pseudoinverse, refitting, or
field mutation.

Center kernel limits, polynomial spaces, RRQR/SVD rank decisions, hard
constraints, infeasibility, and solver behavior are not applicable because
this export neither evaluates a fixed kernel nor assembles or solves a field
system. CLI and versioned schemas remain M8 work; C, C++, and Python adapters
remain M9 work.

No P0-P3 finding remains.

## Validation and disposition

- The parent Review task passed all four public `anisotropy_diagnostics`
  integration tests, the D=4 compile-fail Rustdoc test, the runnable example,
  and complete diff whitespace validation on reviewed head `a698362`.
- Draft CI run 29975187579 passed its configured Ubuntu correctness gate on
  exact re-reviewed evidence head `c6b2c26`. The Ready-only Windows, Ubuntu,
  macOS, and benchmark-smoke matrix was skipped as designed and is not claimed
  as passed.
- Exact repair head `4426a30` now supplies the complete stable full gate.
  Nextest, deny, audit, semver, Miri, sanitizers, fuzzing, mutation testing,
  and API/ABI/schema snapshots remain unavailable or deferred. No unexecuted
  check is claimed as passed.
- Signed weights and squared coverage have the documented dimensions and
  independent hand calculation. The export preserves the strict-background
  diagonal-congruence SPD proof, adds no CPD path, changes no Hessian
  capability, and performs no jitter, clipping, pseudoinverse, regularization,
  refit, or field mutation.

The isolated re-review passed all four public `anisotropy_diagnostics`
integration tests, the D=4 compile-fail Rustdoc test, warning-denying focused
Clippy, the runnable example, all 58 requirement checks, and complete diff
whitespace validation. It verified that the tail after stable full-gate head
`4426a30` changes only Markdown evidence.

## Integration evidence

- Exact Ready evidence head:
  `556b2540b2e26265ccca3d83040ca728aa623a8e`
- Ready CI run: 29975641751
- Squash merge: `07dd290840b7531c57acb583a678e07a8ae64f00`
- Post-merge `main` CI run: 29976326673
- Integration-state branch: `codex/req-aniso-003-integration-state`

Ready CI run 29975641751 passed the complete Windows, Ubuntu, and macOS
workspace gate on exact Ready head `556b254`, including every configured
backend combination, every benchmark-smoke workload, and requirement
validation. PR #112 then squash-merged exactly once as `07dd290`; Issue #111
closed as completed. Post-merge `main` CI run 29976326673 passed the same
complete three-platform gate on exact merge commit `07dd290`.

The isolated integration-state change updates only the requirement registry,
this review evidence, the completed-history index, and the bounded handoff. It
changes no production code, test, manifest, schema, CI, build input, public
API, numerical behavior, dependency, tag, or release. The requirement may be
recorded as `integrated` in this change because implementation, tests,
documentation, interfaces, diagnostics, independent review, exact Ready-head
CI, the implementation merge, and post-merge `main` CI are now complete. The
integration-state pull request itself must still pass its local standard gate
and exact Ready-head CI before it merges.
