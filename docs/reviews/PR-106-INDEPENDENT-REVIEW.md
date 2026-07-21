# PR #106 Independent Review

- Requirement: REQ-ANISO-002
- Issue: https://github.com/qingsonger/GeoRBF/issues/105
- Pull request: https://github.com/qingsonger/GeoRBF/pull/106
- Branch: `codex/req-aniso-002-orientation-tensor`
- Reviewed head: `2c33c3f0affbceed39659f07203982bbc9a0756e`
- Base head: `d34458f6c29d1b56f2832ddac9356d28a87a3f8f`
- Review date: 2026-07-21
- Result: P1 ANISO002-REV-001 and P3 ANISO002-REV-002 require Repair

## Scope and independence

A fresh read-only project `math_reviewer` received only the bounded
REQ-ANISO-002 summary and integrated dependency closure, Issue #105 acceptance
criteria and exclusions, the M6 plan, ANISOTROPY and ADR-0009/ADR-0010
contracts, the exact PR diff, tests, benchmark and CI wiring, registry,
handoff, and validation evidence. It inherited no Implement reasoning and made
no repository or remote change.

The reviewer independently checked formulae, dimensions, PSD classification,
repeated eigenspaces, finite and extreme arithmetic, bounded eigensolver
semantics, deterministic ordering, cross-validation, weight normalization,
influence bounds, rotation covariance, hidden numerical adjustments,
allocations, public interfaces, tests, documentation, CI, and requirement
truthfulness.

## Findings

### ANISO002-REV-001 — P1: leave-one-out ratio scores are not rotation invariant

Affected code and contract:

- `crates/georbf/src/orientation_tensor.rs:469-475`
- `crates/georbf/src/orientation_tensor.rs:1005-1025`
- `docs/architecture/ANISOTROPY.md:126-140`

Each fold scores squared projections against individual training-tensor
eigenvectors. When a training tensor has a repeated eigenspace, those
projections depend on the arbitrary basis chosen inside that eigenspace; only
their sum over the repeated space is invariant.

This ambiguity is not diagnosed by the full-result eigengaps. For valid D=3
samples `e1` with weight 3, `e1` with weight 2, and `e2` with weight 1, the
full tensor has distinct eigenvalues `5/6`, `1/6`, and `0`. Holding out `e2`
leaves the rank-one tensor `e1 e1^T`, whose null eigenspace is two-dimensional.
For ratios `[2, 1.5, 1]`, the expected shares are `(16,9,4)/29`. Rotating the
complete dataset by 45 degrees about `e1` changes that fold's observed shares
from `(0,1,0)` to `(0,1/2,1/2)`. The fold loss changes by exactly `19/58` and
the published weighted candidate score by `19/348`, despite identical
geometry.

The permitted two-positive-sample D=3 case can change the selected candidate.
An exact-rational probe with `[1,1,1]` and `[10,10,1]` produced unrotated scores
`2/3` and `6734/13467`, selecting `[10,10,1]`. After a global rotation the
isotropic score was `5/12` and the `[10,10,1]` score was at least
`26735/53868`, selecting `[1,1,1]`.

Impact: physically equivalent rotated inputs can publish different candidate
scores and select different principal-axis ratios. A fresh Repair must add a
three-sample rotation regression asserting invariant scores and selection,
then score repeated eigenspaces invariantly or explicitly reject and diagnose
ambiguous folds through a documented policy.

### ANISO002-REV-002 — P3: advertised influence range can be exceeded

Affected code and contract:

- `crates/georbf/src/orientation_tensor.rs:190-197`
- `crates/georbf/src/orientation_tensor.rs:1060-1086`

The public contract promises `normalized_tensor_change` in `[0,1]`, but the
raw floating computation does not preserve that theoretical bound. A
dependency-free IEEE-double probe using orthogonal directions proportional to
`[-8,-8,-8]` and `[-8,1,7]`, with finite positive weights `f64::MAX` and `1.0`,
reproduced `1.0000000000000002` when the dominant sample was removed. The
normalized direction norms round slightly above one, so the Frobenius result
exceeds the documented upper bound by one ulp.

Impact: valid input can violate a public diagnostic invariant and downstream
range validation. A fresh Repair must add this extreme-weight regression for
every per-sample and maximum influence and apply an explicit, documented
roundoff-bound policy.

No additional P0, P1, P2, or P3 finding was identified.

## Validation and disposition

- The reviewer verified the exact base/head, merge base, complete eleven-file
  PR diff, and clean scoped worktree. The tail from complete-gate head
  `2d65666` to reviewed head `2c33c3f` changes only `requirements/v1.yaml` and
  `docs/progress/CURRENT.md`.
- The reviewer and parent Review task each passed all 11 public orientation-
  tensor tests and all 58 requirement checks. The parent also passed the D=4
  compile-fail Rustdoc test and complete PR whitespace check.
- Dependency-free exact-rational and IEEE-double probes independently
  reproduced ANISO002-REV-001 and ANISO002-REV-002. An initial NumPy probe was
  unavailable because NumPy is not installed and was replaced by those
  dependency-free probes.
- Draft CI passed its configured Ubuntu correctness gate on exact reviewed
  head `2c33c3f`. The Ready-only Windows, Ubuntu, macOS, and benchmark-smoke
  matrix was skipped as designed and is not claimed as passed.
- Exact implementation head `2d65666` retains the complete standard local gate
  recorded by Implement: workspace format, warning-denying all-target/all-
  feature Clippy, all-feature workspace tests, workspace Rustdoc, all 58
  requirement checks, and complete diff whitespace validation.
- The full workspace gate and benchmark were not rerun in this Review task.
  `actionlint`, nextest, deny, audit, semver, Miri, sanitizers, fuzzing,
  mutation testing, allocation instrumentation, and API/ABI/schema snapshots
  remain unavailable or deferred. No unexecuted check is claimed as passed.

PR #106 remains Draft and REQ-ANISO-002 remains `implemented`, not
`integrated`. A fresh Repair task must address only ANISO002-REV-001 and
ANISO002-REV-002, add the required regressions, run focused checks and one
complete stable-head standard gate after the final code change, update this
record and the bounded handoff, push, and stop for a fresh independent
re-review. Do not begin another requirement.

## Repair evidence pending fresh independent re-review

Repair code/test head `1f1fdc6dbbe6b69bf7872f9d6a7eae0ca5e51c67`
addresses only ANISO002-REV-001 and ANISO002-REV-002:

- ANISO002-REV-001: leave-one-out loss now groups every maximal consecutive
  training-fold eigenspace whose adjacent normalized gaps are at most
  `64 D epsilon`. It compares the total observed and expected share inside
  each unresolved group, so an arbitrary orthonormal basis change within a
  repeated eigenspace cannot change the score. A public three-sample D=3
  global-rotation regression checks all candidate scores and the selected
  ratios.
- ANISO002-REV-002: the exact PSD trace-one influence remains bounded by one.
  A finite computed value no greater than `1 + 64 D^2 epsilon` is explicitly
  recorded as one; a larger overshoot is a structured
  `InfluenceOutsideRoundoffTolerance` error. A public D=3 regression with
  `f64::MAX` and unit weights reproduces the former one-ulp overshoot and
  checks every influence and the aggregate maximum against `[0,1]`.

Both regressions failed against the pre-repair implementation. The repeated-
eigenspace case changed a candidate score from approximately `0.39675` to
`0.45422` under a global rotation, and the extreme-weight case produced
`1.0000000000000002`. Both pass at the repair head. All 13 focused public
orientation-tensor tests, the D=4 compile-fail Rustdoc contract, warning-
denying georbf all-target/all-feature Clippy, the runnable example, and the
optimized benchmark smoke passed. The smoke reported approximately 4.58 us
per four-sample, three-candidate D=3 estimate over 2,000 estimates with the
unchanged checksum `1.00428812046557887e4`.

After the final production change, exact repair head `1f1fdc6` passed the
complete standard workspace gate: format, warning-denying all-target/all-
feature Clippy, all workspace tests with all features, workspace Rustdoc, all
58 requirement checks, and complete diff whitespace validation. The
subsequent review-record and bounded-handoff commit changes documentation only
and does not invalidate that gate. The unavailable-check list recorded above
is unchanged, and no unavailable check is claimed as passed.

This section records Repair evidence only and does not independently close the
findings. PR #106 remains Draft and REQ-ANISO-002 remains `implemented`. A
fresh independent mathematical and numerical re-review of the complete
repaired PR diff is required next. This Repair does not mark the PR ready,
merge it, or begin another requirement.

## Independent re-review of the Repair

- Re-reviewed head: `627d360c21ed6f06e437c8c7d83de4d751ac777f`
- Repair code/test head: `1f1fdc6dbbe6b69bf7872f9d6a7eae0ca5e51c67`
- Base head: `d34458f6c29d1b56f2832ddac9356d28a87a3f8f`
- Result: ANISO002-REV-001 and ANISO002-REV-002 are closed; P2
  ANISO002-REV-003 requires Repair

A fresh read-only project `math_reviewer` received only the bounded
REQ-ANISO-002 summary and dependency closure, Issue #105 criteria, the M6
plan, ANISOTROPY and ADR-0009/ADR-0010 contracts, the complete PR and Repair
diffs, tests, example, benchmark, CI wiring, scoped registry entry, handoff,
and validation evidence. It inherited no Repair reasoning and changed no
repository or remote state.

### Original-finding closure

- ANISO002-REV-001 is closed. For an unresolved group `G`, the sum of squared
  projections equals projection onto the eigenspace and is invariant under an
  orthonormal basis change inside it. The explicit `64 D epsilon` grouping
  implements that structural repair, and the global-rotation regression
  passes.
- ANISO002-REV-002 is closed. The exact normalized Frobenius influence of two
  PSD trace-one tensors is at most one. The explicit `64 D^2 epsilon`
  overshoot band records one only inside the documented band and otherwise
  returns a structured error; the extreme-weight regression passes.

### ANISO002-REV-003 - P2: grouped loss can select the wrong candidate from normalization roundoff

Affected code and contract:

- `crates/georbf/src/orientation_tensor.rs:1008-1013`
- `crates/georbf/src/orientation_tensor.rs:1042-1049`
- `crates/georbf/src/orientation_tensor.rs:1079-1082`
- `docs/architecture/ANISOTROPY.md:126-145`
- Missing interaction coverage in
  `crates/georbf/tests/orientation_tensor.rs:255-307`

`expected_shares` divides every component independently, so their represented
sum need not be exactly one. When every axis belongs to one unresolved
training-fold eigenspace, the exact grouped loss is zero for every candidate:
both observed and expected total mass are one. The implementation can instead
introduce a candidate-dependent loss solely from normalization roundoff.

An independent counterexample uses a direction proportional to `[1,2,2]`
with weight `f64::MAX`, plus `e1`, `e2`, and `e3` with unit weights, and
candidates `[3,2,1]` and `[4,2,1]`. Holding out the dominant sample leaves the
exactly isotropic tensor `I/3`; the all-axis grouped loss must therefore be
zero for both candidates. Across the three minor folds, exact loss is
`1913/2646` for `[3,2,1]` and `1654/1323` for `[4,2,1]`, a difference of
`155/294`, so `[3,2,1]` must win.

In binary64, however, the represented `[3,2,1]` shares sum to
`0.9999999999999999`, injecting loss `1.232595164407831e-32` into the
dominant fold, while the `[4,2,1]` shares sum to exactly one. The real
minor-fold evidence is weighted by `1/f64::MAX`, approximately `5.56e-309`,
so the artificial normalization loss dominates and reverses selection to
`[4,2,1]`.

Impact: valid explicitly supported extreme finite weights can publish the
wrong cross-validated ratio because of candidate-specific probability-mass
roundoff. A fresh Repair must add this four-sample D=3 regression, assert the
independent score ordering and selection of `[3,2,1]`, and ensure grouped
expected mass honors the normalized-share invariant, especially for a group
spanning every axis.

No P0, P1, or P3 finding and no additional P2 finding was identified.

### Re-review validation and disposition

- The reviewer passed all 13 focused orientation-tensor tests, formatting,
  warning-denying georbf all-target/all-feature Clippy, the orientation-tensor
  Rustdoc compile-fail contract, the runnable example, all 58 requirement
  checks, and complete PR whitespace validation.
- The optimized benchmark smoke retained checksum
  `1.00428812046557887e4`. Independent exact-rational and IEEE-754 probes
  verified both original repairs and reproduced ANISO002-REV-003.
- The complete workspace test and Rustdoc gates were not rerun in re-review.
  Exact Repair head `1f1fdc6` retains its complete standard local gate, and
  the later commits through the re-reviewed head change only the review and
  handoff Markdown.
- Ready-only Windows, Ubuntu, macOS, and benchmark-smoke CI has not run and is
  not claimed as passed. The unavailable/deferred check list above is
  unchanged.

PR #106 remains Draft and REQ-ANISO-002 remains `implemented`, not
`integrated`. Open a fresh bounded Repair task for ANISO002-REV-003 only; do
not repair production code, mark the PR ready, merge it, or begin another
requirement in this Review task.
