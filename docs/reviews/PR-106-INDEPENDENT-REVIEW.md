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

## ANISO002-REV-003 Repair evidence pending fresh independent re-review

- Repair code, test, and normative-document head:
  `7d38a45424cd3d9919f3e7532701d78a6554280f`
- Pre-repair head: `d5cd66c6345a1d82b0c5807f23a9b596faf82b0d`

The public four-sample D=3 regression from ANISO002-REV-003 was added first.
Against the pre-repair implementation it selected `[4,2,1]`: the `[3,2,1]`
candidate received artificial loss `1.232595164407831e-32`, while the
`[4,2,1]` candidate retained only its real approximately `6.9544e-309` score.

The Repair changes only grouped cross-validation loss. Every non-final
eigenspace group retains its directly summed observed and expected share; the
final group receives one minus the mass assigned to preceding groups for both
distributions. The represented groups therefore sum exactly to one. In the
all-axis unresolved fold, both masses are exactly one and its candidate loss
is exactly zero, so normalization roundoff cannot overwhelm the minor folds.
No tensor, eigendecomposition, grouping threshold, candidate, regularization,
or rank policy changes.

The regression now selects `[3,2,1]` and independently checks the scaled
candidate scores against `1913/2646` and `1654/1323`. All 14 focused public
orientation-tensor tests, warning-denying georbf all-target/all-feature
Clippy, the D=4 compile-fail Rustdoc contract, the runnable example, and the
optimized benchmark smoke passed. The smoke retained checksum
`1.00428812046557887e4` at approximately 5.41 us per estimate locally.

After the final production change, exact head `7d38a45` passed the complete
standard workspace gate: format, warning-denying all-target/all-feature
Clippy, all workspace tests with all features, workspace Rustdoc, all 58
requirement checks, and complete diff whitespace validation. The subsequent
review-record and bounded-handoff commit changes Markdown evidence only and
does not invalidate that stable-head gate.

This is Repair evidence, not independent closure. PR #106 remains Draft and
REQ-ANISO-002 remains `implemented`. A fresh independent mathematical and
numerical re-review must verify ANISO002-REV-003 and the complete PR diff
before any Ready or integration action.

## Final independent re-review after ANISO002-REV-003 repair

- Re-reviewed base: `d34458f6c29d1b56f2832ddac9356d28a87a3f8f`
- Re-reviewed repair code/test/normative-document head:
  `7d38a45424cd3d9919f3e7532701d78a6554280f`
- Re-reviewed final Repair handoff head:
  `8e467c86f01bdfcb3eabe7bcc4b9a89147cfa4c1`
- Re-review date: 2026-07-22
- Result: ANISO002-REV-001 through ANISO002-REV-003 are closed; P2
  ANISO002-REV-004 requires Repair

A fresh isolated read-only project `math_reviewer` received only the bounded
REQ-ANISO-002 summary and integrated dependency closure, Issue #105 criteria
and exclusions, the M6 plan, ANISOTROPY and ADR-0009/ADR-0010 contracts, the
complete exact PR and ANISO002-REV-003 Repair diffs, directly relevant source,
Rustdoc, tests, example, benchmark, CI wiring, registry entry, handoff, and
validation evidence. It inherited no Implement or Repair reasoning and made
no repository, Git, or GitHub change.

The reviewer verified that local and remote PR head are exact `8e467c8`, the
merge base is exact `d34458f`, the scoped worktree is clean, and
`7d38a45..8e467c8` changes only this review record and the bounded handoff.

### Prior-finding closure

- ANISO002-REV-001 is closed. Summed squared projections inside a grouped
  orthonormal eigenspace equal projection through its basis-independent
  projector. The explicit `64 D epsilon` grouping and public rotation
  regression preserve that invariant.
- ANISO002-REV-002 is closed. The exact normalized Frobenius distance between
  PSD trace-one tensors is at most one. The explicit `64 D^2 epsilon` upper
  roundoff band and structured larger-overshoot error implement the documented
  represented-arithmetic policy, and the extreme-weight regression passes.
- ANISO002-REV-003 is closed. Every non-final eigenspace group retains its
  directly summed observed and expected mass, while the final group receives
  one minus the preceding mass on both sides. A fully unresolved fold therefore
  has exactly represented unit mass and zero loss for every candidate. The
  public regression verifies `1913/2646 < 1654/1323` and selects `[3,2,1]`.

### ANISO002-REV-004 - P2: represented outer product can reject valid PSD input

Affected code and contract:

- `docs/architecture/ANISOTROPY.md:84-97`
- `crates/georbf/src/orientation_tensor.rs:312-315`
- `crates/georbf/src/orientation_tensor.rs:872-904`
- `crates/georbf/src/orientation_tensor.rs:916-959`, especially `:944-945`

A valid D=2 sample with direction proportional to `[1,30]` and unit weight
passes every public input condition. The repository normalization produces

```text
u = [0.03331483023263848, 0.9994449069791544]

C = [[0.001109877913429523, 0.03329633740288569],
     [0.03329633740288569,  0.9988901220865707]]
trace(C) = 1.0000000000000002
det(C)   = -2.168404344971009e-19
```

The exact outer product is PSD and rank one, but independently rounded matrix
entries make the represented tensor slightly indefinite. Nalgebra's symmetric
eigendecomposition returns a minimum eigenvalue of approximately
`-1.1089908126111444e-16`, and the estimator maps it to `NegativeEigenvalue`.
The parent Review task independently reproduced the same public-API failure
with fixed ratios `[1,1]` in a temporary regression and restored the clean
worktree afterward.

Impact: ordinary valid fixed estimation can fail solely because componentwise
tensor formation did not preserve the mathematical PSD invariant. A
cross-validation training fold containing one generic direction can fail for
the same reason. The represented trace also contradicts the exact trace-one
Rustdoc statement.

A fresh Repair must first add a public D=2 one-sample regression using
direction `[1,30]`, unit weight, and fixed ratios `[1,1]`; it must prove the
current failure and then succeed with a tensor/eigendecomposition satisfying
the documented PSD and normalization policy. The Repair must not use
eigenvalue clipping, jitter, hidden regularization, or an input-invalidity
fallback. A two-positive-sample leave-one-out regression is useful follow-on
coverage but is not the smallest required proof.

No P0, P1, P3, or additional P2 finding was identified.

### Final re-review validation and disposition

- The reviewer passed all 14 public orientation-tensor tests, the selected D=4
  compile-fail Rustdoc contract, formatting, all 58 requirement checks, and
  complete exact-PR whitespace validation. Dependency-free IEEE-754 probes
  verified ANISO002-REV-003 and reproduced ANISO002-REV-004.
- The parent Review task passed the same 14 public tests, warning-denying
  georbf all-target/all-feature Clippy, complete georbf Rustdoc, the runnable
  example, benchmark smoke, all 58 requirement checks, and exact-PR whitespace
  validation. The benchmark retained checksum `1.00428812046557887e4`.
- Exact Repair head `7d38a45` retains its complete standard workspace gate.
  Later commits through the reviewed head and this evidence change modify only
  Markdown review/handoff evidence and do not invalidate that gate.
- Ready-only Windows, Ubuntu, macOS, and benchmark-smoke CI has not run and is
  not claimed as passed. The recorded unavailable/deferred check list remains
  unchanged.

PR #106 remains Draft and REQ-ANISO-002 remains `implemented`, not
`integrated`. Open a fresh bounded Repair task for ANISO002-REV-004 only. Do
not repair production code, mark the PR Ready, merge it, or begin another
requirement in this Review task.

## ANISO002-REV-004 Repair evidence pending fresh independent re-review

- Repair code, test, normative-document, and complete-gate head:
  `b591a419095cd4e69043f01773c43b14fd9fc914`
- Pre-repair branch head: `e7a1ec171ffd0b7d256df19e4db1a64d9410d8dc`
- Repair date: 2026-07-22

The required public D=2 regression first reproduced the exact reviewed failure:
the valid unit direction proportional to `[1,30]`, unit weight, and fixed
ratios `[1,1]` returned `NegativeEigenvalue` with value
`-1.1089908126111444e-16` before the repair.

The repair keeps the normalized outer-product construction and closes only its
represented-arithmetic PSD boundary. After compensated accumulation, the
tensor is trace-normalized. Exact floating expansions certify every D=2/D=3
principal minor. If independently rounded off-diagonal entries cross the PSD
boundary, 64 bounded deterministic bisection steps retain the greatest
certified uniform off-diagonal factor while leaving all diagonal entries
unchanged. The result diagnostics record that factor, with one meaning no
correlation adjustment. This path adds no diagonal jitter, eigenvalue clipping,
pseudoinverse, input-invalidity fallback, or hidden regularization.

The primary bounded symmetric eigendecomposition remains the normal spectral
path. If it returns a negative roundoff value for an exact-sign-certified PSD
matrix, a bounded SVD of that same matrix supplies right singular vectors and
nonnegative singular values, which equal the PSD eigenvalues. Diagnostics
record which path ran. The regression now succeeds, verifies represented trace
one and a nonnegative represented determinant, confirms all spectral values
are nonnegative, observes a sub-unit correlation scale, and confirms the
explicit PSD-SVD path. All 15 public orientation-tensor tests pass, including
the closed ANISO002-REV-001/002/003 regressions.

Focused validation on the exact repair head passed warning-denying georbf
all-target/all-feature Clippy, georbf Rustdoc including the D=4 compile-fail
contract, the runnable example, optimized benchmark smoke, all 58 requirement
checks, and diff whitespace. The benchmark checksum remains
`1.00428812046557887e4` at approximately 9.20 us per estimate locally.

The same exact repair head passed the complete standard workspace gate:
format; warning-denying workspace all-target/all-feature Clippy; all-feature
workspace tests; workspace Rustdoc; all 58 requirement checks; and complete
diff whitespace validation. Ready-only Windows, Ubuntu, macOS, and benchmark-
smoke CI has not run and is not claimed as passed. The unavailable/deferred
check list remains unchanged.

This section records Repair evidence only and does not independently close
ANISO002-REV-004. PR #106 remains Draft and REQ-ANISO-002 remains
`implemented`. A fresh isolated mathematical and numerical re-review of the
complete PR diff and this exact repair head is required next. This Repair does
not mark the PR ready, merge it, or begin another requirement.

## Final independent re-review after ANISO002-REV-004 repair

- Re-reviewed base: `d34458f6c29d1b56f2832ddac9356d28a87a3f8f`
- Re-reviewed repair code/test/normative-document head:
  `b591a419095cd4e69043f01773c43b14fd9fc914`
- Re-reviewed final Repair handoff head:
  `cebaefff3bd940fc89c98aca0514c1340bf55c3b`
- Re-review date: 2026-07-22
- Result: ANISO002-REV-001 through ANISO002-REV-004 are closed; P2
  ANISO002-REV-005 and P3 ANISO002-REV-006/007 require Repair

A fresh isolated read-only project `math_reviewer` received only the bounded
REQ-ANISO-002 summary and integrated dependency closure, Issue #105 criteria
and exclusions, the M6 plan, ANISOTROPY and ADR-0009/ADR-0010 contracts, the
complete exact PR and ANISO002-REV-004 Repair diffs, directly relevant source,
Rustdoc, tests, example, benchmark, CI wiring, registry entry, handoff, and
validation evidence. It inherited no Implement or Repair reasoning and made
no repository, Git, or GitHub change.

The reviewer verified the clean worktree, exact local and remote PR head,
merge base, and Repair head. The tail `b591a41..cebaeff` changes only the
requirement summary, review record, and bounded handoff Markdown.

### Prior-finding closure

- ANISO002-REV-001 remains closed. Grouped squared projections are invariant
  under an orthonormal basis change inside an unresolved eigenspace, and the
  public global-rotation regression passes.
- ANISO002-REV-002 remains closed. The explicit `64 D^2 epsilon` overshoot
  band implements the documented represented influence policy, and the
  extreme-weight regression passes.
- ANISO002-REV-003 remains closed. Assigning residual observed and expected
  mass to the final group makes a fully unresolved fold exactly zero-loss; the
  independent score-ordering regression passes.
- ANISO002-REV-004 is closed for its recorded `[1,30]` regression. An
  independent exact-dyadic probe found that the retained correlation scale is
  `next_down(1)`, its determinant is positive, and the next represented scale
  one is indefinite. The public regression confirms trace one, nonnegative
  spectral output, and the explicit PSD-SVD path.

### ANISO002-REV-005 - P2: exact PSD certification loses underflowed products

Affected code and contract:

- `crates/georbf/src/orientation_tensor.rs:940-942`
- `crates/georbf/src/orientation_tensor.rs:1037-1073`
- `crates/georbf/src/orientation_tensor.rs:1076-1115`
- `crates/georbf/src/orientation_tensor.rs:1151-1165`
- `docs/architecture/ANISOTROPY.md:98-107`
- The existing determinant assertion at
  `crates/georbf/tests/orientation_tensor.rs:99-123`

`ExactExpansion::add_product` and `add_triple_product` form each binary64
product and its FMA residual. Both round to zero when the exact product lies
below the minimum binary64 subnormal, so these routines cannot certify exact
minor signs over the complete accepted finite-input domain.

Let `m = 2^-1074`, the smallest positive `f64`, and estimate one D=2 sample
with direction `[1,m]`, unit weight, and fixed ratios `[1,1]`. Unit
normalization returns represented components `[1,m]`; tensor formation
returns

```text
C = [[1, m],
     [m, 0]].
```

Its represented trace is one, but as a real matrix over the represented
entries `det(C) = -m^2 = -2^-2148 < 0`. Both binary64 products in the 2-by-2
minor round to zero, so the current certification accepts the indefinite
matrix and records correlation scale one. The existing regression computes a
binary64 determinant whose `m*m` term has already rounded away and therefore
cannot detect this boundary.

The parent Review task added a temporary public regression requiring
`C[1][1] != 0 || C[0][1] == 0`; it failed with exactly
`[[1.0, 5e-324], [5e-324, 0.0]]` and was then removed, restoring the clean
worktree. Repair must add a permanent public regression using
`f64::from_bits(1)` and preserve the exact dyadic PSD invariant without
clipping, jitter, a pseudoinverse, hidden regularization, or an input-invalidity
fallback.

### ANISO002-REV-006 - P3: normalized positive ratio share can underflow

Affected code and contract:

- `crates/georbf/src/orientation_tensor.rs:96-134`
- `crates/georbf/src/orientation_tensor.rs:1341-1352`
- `changes/REQ-ANISO-002.md:29-35`
- `docs/architecture/ANISOTROPY.md:132-136`

The ratio constructor rejects a maximum-scaled square that is already zero,
but does not check whether subsequent division by the represented square sum
erases a positive share. For D=3 ratios `[2^537, 2^537, 1]`, the scaled
squares are `[1,1,2^-1074]`, so construction succeeds. `expected_shares`
then divides by represented sum two and returns `[0.5,0.5,0]`, although the
exact third normalized share is positive. This contradicts the documented
rejection of candidates unable to form represented normalized squared shares
and silently removes one axis contribution from cross-validation.

The parent Review task added a temporary public regression requiring
`PrincipalAxisRatios::<3>::try_new([2^537,2^537,1])` to return
`NonRepresentableRatioSquare`; it failed and was removed. Repair must add that
regression unless it instead preserves every positive normalized share by a
documented represented-arithmetic construction.

### ANISO002-REV-007 - P3: leave-one-out loops allocate per sample

Affected code and repository contract:

- `crates/georbf/src/orientation_tensor.rs:870-914`
- `crates/georbf/src/orientation_tensor.rs:924-931`
- `crates/georbf/src/orientation_tensor.rs:1211-1241`
- `crates/georbf/src/orientation_tensor.rs:1354-1399`
- `crates/georbf/src/orientation_tensor.rs:1471-1487`
- `AGENTS.md:130-131`

Every `normalized_tensor` call allocates an N-element normalized-weight
`Vec`. Influence evaluation invokes it once per positive sample; candidate
cross-validation also invokes it once per held-out positive sample and creates
a heap-backed `DMatrix` for every fold. Allocation count therefore grows with
sample count inside the leave-one-out hot loops, beyond the owned result
vectors, contrary to the repository rule that batch hot paths avoid
per-element allocation.

Repair must add a serial allocation-count regression comparing fixed-ratio
estimates at small and larger sample counts, with a count bounded independently
of sample count apart from a fixed number of owned result allocations. The
implementation must reuse or avoid scratch allocation without changing the
public mathematical result.

No P0, P1, or additional P2/P3 finding was identified.

### Final re-review validation and disposition

- The isolated reviewer passed all 15 public orientation-tensor tests,
  formatting, warning-denying georbf all-target/all-feature Clippy, the D=4
  compile-fail Rustdoc contract, the runnable example, optimized benchmark
  smoke, all 58 requirement checks, and complete PR whitespace validation.
  The benchmark checksum remained `1.00428812046557887e4`.
- Dependency-free exact-dyadic and IEEE-754 probes verified
  ANISO002-REV-004 closure and established ANISO002-REV-005/006. Direct source
  inspection established the per-sample allocation paths in
  ANISO002-REV-007.
- The parent Review task passed the complete standard workspace gate on exact
  pre-evidence head `cebaeff`: format, warning-denying all-target/all-feature
  Clippy, all-feature workspace tests, workspace Rustdoc, all 58 requirement
  checks, and complete diff whitespace validation. Its two temporary public
  numerical regressions both failed as described and were removed before this
  evidence change.
- Ready-only Windows, Ubuntu, macOS, and benchmark-smoke CI has not run and is
  not claimed as passed. The recorded unavailable/deferred check list remains
  unchanged.

PR #106 remains Draft and REQ-ANISO-002 remains `implemented`, not
`integrated`. Open a fresh bounded Repair task for ANISO002-REV-005,
ANISO002-REV-006, and ANISO002-REV-007 only. Do not repair production code,
mark the PR Ready, merge it, or begin another requirement in this Review task.

## ANISO002-REV-005/006/007 Repair evidence pending fresh independent re-review

- Repair code, test, normative-document, and complete-gate head:
  `b634751d6545957d0d65039fb344108ad67169df`
- Pre-repair branch head: `45645cb1e47e9f09fc7cae3215ea136ae643a4e1`
- Repair date: 2026-07-22

The three required regressions were added before production repair. Against
the pre-repair implementation, the public D=2 minimum-subnormal case returned
`[[1, f64::from_bits(1)], [f64::from_bits(1), 0]]`; the ratio constructor
accepted `[2^537,2^537,1]`; and the serial fixed-ratio allocation audit counted
9 explicit allocations for four samples but 21 for sixteen samples.

ANISO002-REV-005 is repaired with a fixed-capacity exact dyadic accumulator.
It decomposes each finite binary64 term into its sign, integer significand, and
power-of-two exponent, multiplies at most three significands in three limbs,
and adds signed shifted products over the complete binary64 product/triple-
product exponent range. Products below `2^-1074` therefore remain present in
the exact principal-minor sign. The public `[1,f64::from_bits(1)]` case now
retains the greatest bisection-certified correlation scale whose represented
tensor is PSD, and a direct unit regression verifies negative, cancelling,
and positive exact signs for products and triples below the binary64 range.

ANISO002-REV-006 is repaired at construction. After maximum scaling, every
positive square and every normalized share obtained by division by the
represented square sum must remain positive. `[2^537,2^537,1]` now returns the
existing structured `NonRepresentableRatioSquare` error for axis 2; no ratio
is clipped, rescaled, or silently removed.

ANISO002-REV-007 is repaired without changing estimator results. Weight
normalization now uses two scalar-state passes and computes fractions on
demand. D=1, D=2, and D=3 spectral paths use stack-owned fixed-size nalgebra
matrices, and fixed arrays replace spectral ordering and axis-collection
vectors. No sample-sized weight vector or heap-backed matrix remains inside a
held-out fold. The serial audit records exactly two fixed owned-result
allocation attempts for both four and sixteen fixed-ratio samples, and exactly
five for both four and sixteen cross-validated samples.

All 16 public orientation-tensor tests and both private exact-dyadic/allocation
tests pass. Warning-denying georbf all-target/all-feature Clippy, the D=4
compile-fail Rustdoc contract, the runnable example, the optimized benchmark
smoke, all 58 requirement checks, and diff whitespace passed. The optimized
100,000-estimate smoke reported checksum `5.02144060231886397e5`, preserving
the prior per-estimate contribution, at approximately 5.15 us per estimate
locally.

After the final production, test, and normative-document change, exact head
`b634751` passed the complete standard workspace gate: format; warning-denying
workspace all-target/all-feature Clippy; all-feature workspace tests;
workspace Rustdoc; all 58 requirement checks; and complete diff whitespace
validation. No dependency, feature, lockfile, public interface, schema, CI,
registry status, or requirement scope changed.

This section records Repair evidence only and does not independently close
ANISO002-REV-005/006/007. PR #106 remains Draft and REQ-ANISO-002 remains
`implemented`. A fresh isolated mathematical and numerical re-review must
verify these repairs and the complete PR diff before any Ready or integration
action. Ready-only Windows, Ubuntu, macOS, and benchmark-smoke CI has not run
and is not claimed as passed; the unavailable/deferred check list remains
unchanged.

## Independent re-review after ANISO002-REV-005/006/007 repair

- Re-reviewed base: `d34458f6c29d1b56f2832ddac9356d28a87a3f8f`
- Re-reviewed repair code/test/normative-document head:
  `b634751d6545957d0d65039fb344108ad67169df`
- Re-reviewed final Repair handoff head:
  `ae7983c8d13b3ab3c5a44cc0aa9b3c60ee7a0008`
- Re-review date: 2026-07-22
- Result: ANISO002-REV-005 and ANISO002-REV-006 are closed;
  ANISO002-REV-007 production behavior is repaired but its regression evidence
  is insufficient; P2 ANISO002-REV-008 and P3 ANISO002-REV-009 require Repair

A fresh isolated read-only project `math_reviewer` received only the bounded
REQ-ANISO-002 summary and integrated dependency closure, Issue #105 criteria
and exclusions, the M6 plan, ANISOTROPY and ADR-0009/ADR-0010 contracts, the
complete exact PR and ANISO002-REV-005/006/007 Repair diffs, directly relevant
source, Rustdoc, tests, example, benchmark, CI wiring, registry entry, handoff,
and validation evidence. It inherited no Implement or Repair reasoning and
made no repository, Git, or GitHub change.

The reviewer verified a clean worktree, exact local/upstream/GitHub PR head,
merge base, and Repair head. The tail `b634751..ae7983c` changes only the
requirement summary, review record, and bounded handoff Markdown.

### Prior-finding closure

- ANISO002-REV-005 is closed. Every finite binary64 factor decomposes into an
  integer significand and exponent in `[-1074,971]`. The triple-product base
  exponent is `-3222`, and the greatest possible six-term accumulated bit
  offset is at most 6296, below the fixed 6400-bit accumulator capacity.
  Independent boundary probes and the public/private regressions confirm exact
  signed product and triple-product decisions below the binary64 range.
- ANISO002-REV-006 is closed. Construction checks both every positive
  maximum-scaled square and that square divided by the same represented sum
  later used for expected shares. The third share for
  `[2^537,2^537,1]` becomes zero after division by represented sum two and is
  rejected structurally.
- ANISO002-REV-007 is closed for production behavior by source inspection:
  weight normalization and every fold use fixed scalar, array, `Matrix2`, or
  `Matrix3` state, while the owned result and candidate vectors have counts
  independent of sample count. Its required independent regression evidence
  remains incomplete as ANISO002-REV-009 below.

### ANISO002-REV-008 - P2: fixed PSD search rejects valid extreme directions

Affected code and contract:

- `crates/georbf/src/orientation_tensor.rs:1083-1107`
- `docs/architecture/ANISOTROPY.md:103-110`

For a valid D=2 unit direction represented by `[1,2^-538]`, the squared second
component underflows while normalization retains represented components
`[1,2^-538]`. A unit-weight sample therefore produces

```text
C = [[1, z],
     [z, 0]], z = 2^-538.
```

Exact dyadic certification correctly rejects its determinant `-z^2`. A PSD
represented closure requires the off-diagonal entry to round to zero. The
fixed loop tests only scales `2^-1` through `2^-64`; its final off-diagonal is
still the nonzero value `2^-602`, so every candidate remains exactly
indefinite. Because the implementation writes `*tensor` only after an accepted
candidate, it retains the original matrix and returns
`NonFiniteNumericalResult("positive-semidefinite tensor representation")` for
this valid input. An independent binary64 probe found the first accepted scale
at `2^-537`, outside the fixed search.

Repair must add a public fixed-ratio D=2 regression using one sample
`[1.0, 2.0_f64.powi(-538)]`, requiring success, represented trace one, and
exact represented PSD. It must replace the absolute 64-step assumption with a
bounded construction that covers the complete accepted binary64 domain while
retaining the greatest certified represented correlation factor. It must not
introduce clipping, jitter, a pseudoinverse, hidden regularization, or an
input-invalidity fallback.

### ANISO002-REV-009 - P3: allocation regression counts annotations

Affected code and test:

- `crates/georbf/src/orientation_tensor.rs:38-46`
- manual markers at `crates/georbf/src/orientation_tensor.rs:572-573`,
  `:1520-1524`, `:1556-1557`, and `:1643-1644`
- `crates/georbf/src/orientation_tensor.rs:1696-1730`

`EXPLICIT_ALLOCATION_ATTEMPTS` changes only when production code manually calls
`record_allocation_attempt`; it does not observe the allocator. An unannotated
`Vec`, `Box`, or heap-backed backend reintroduced inside either held-out loop
would leave the asserted counts unchanged. The current test therefore does not
independently prove or protect the sample-count-independent allocation property
required by ANISO002-REV-007, even though the reviewed production path currently
has that property.

Repair must replace or supplement the annotation counter with a serial test
that observes actual allocator calls around only `try_estimate`, with estimator
and input construction plus warm-up outside the measured region. It must
compare four and sixteen samples for both fixed and cross-validated policies.

No P0, P1, or additional P2/P3 finding was identified.

### Re-review validation and disposition

- The isolated reviewer executed the existing focused binaries: all 16 public
  orientation-tensor tests and both private exact-dyadic/allocation tests
  passed. The runnable example and optimized benchmark smoke passed; the
  reviewer's benchmark checksum was `1.00428812046557887e4`.
- Independent IEEE-754 probes verified ANISO002-REV-005/006 boundaries and
  reproduced ANISO002-REV-008. Direct source and nalgebra fixed-matrix review
  established the current production allocation behavior and
  ANISO002-REV-009.
- Complete and Repair diff whitespace validation passed. Exact final Repair
  handoff head `ae7983c` passed the Draft Ubuntu correctness CI. Exact code
  head `b634751` retains its recorded complete local standard gate; the later
  tail is Markdown only.
- The reviewer did not perform a fresh Cargo build, Clippy, workspace test,
  Rustdoc, or full requirement gate and did not claim those checks. Ready-only
  Windows, Ubuntu, macOS, and benchmark-smoke CI did not run. The recorded
  unavailable/deferred check list remains unchanged.

PR #106 remains Draft and REQ-ANISO-002 remains `implemented`, not
`integrated`. Open a fresh bounded Repair task for ANISO002-REV-008 and
ANISO002-REV-009 only. The Repair must also provide the actual-allocation
regression needed to complete ANISO002-REV-007 closure. Do not repair
production code, mark the PR Ready, merge it, or begin another requirement in
this Review task.

## ANISO002-REV-008/009 Repair evidence pending fresh independent re-review

- Repair code, tests, manifest, lockfile, and normative-document head:
  `1a95323ffd9b8ba43eb5f0390aa02d812edfdba2`
- Repair date: 2026-07-22
- Scope: ANISO002-REV-008 and ANISO002-REV-009 only, including the remaining
  actual-allocation evidence obligation for ANISO002-REV-007

The required public D=2 regression was added before production repair and
failed on the reviewed head with
`NonFiniteNumericalResult("positive-semidefinite tensor representation")` for
one valid `[1,2^-538]` sample. The repaired search bisects the ordered positive
binary64 scale bit patterns from zero through one instead of performing 64
real-number halvings. It therefore covers the complete represented interval
and selects the greatest exactly certified scale. The regression succeeds,
retains represented trace one, produces zero off-diagonal entries and a
nonnegative spectrum, and records the exact boundary scale `2^-537`. No input
rejection, clipping, jitter, pseudoinverse, or hidden regularization was added.

All `EXPLICIT_ALLOCATION_ATTEMPTS` state and manual production markers were
removed. A dedicated one-test integration binary constructs both estimators and
their four- or sixteen-sample inputs, warms the selected path, and then measures
actual allocator calls around only `try_estimate`. It observes exactly two
allocations for both fixed-ratio sample counts and exactly five allocations for
both cross-validated sample counts. This protects the sample-count-independent
property from unannotated `Vec`, `Box`, or heap-backed fold scratch and supplies
the missing ANISO002-REV-007 regression evidence.

The measurement uses exactly pinned test-only `allocation-counter` 0.8.1. The
complete maintenance, license, MSRV, platform, size, unsafe, and alternatives
review is recorded in `changes/REQ-ANISO-002.md`. The crate has no transitive
dependencies and does not enter production artifacts; GeoRBF source remains
free of unsafe code.

Focused validation passed all 17 public orientation-tensor tests, the dedicated
actual-allocation test, the private exact-dyadic test, warning-denying georbf
all-target/all-feature Clippy, the runnable example, optimized benchmark smoke,
and diff whitespace. The smoke retained checksum `1.00428812046557887e4` at
approximately 6.53 us per estimate locally.

After the final code, test, manifest, lockfile, and normative architecture
change, the stable state passed the complete standard workspace gate: format,
warning-denying all-target/all-feature Clippy, all-feature workspace tests,
workspace Rustdoc, all 58 requirement checks, and complete diff whitespace.
The only later pre-commit edit was change-fragment Markdown and did not alter a
production, test, manifest, schema, or build input.

This section records Repair evidence only and does not independently close
ANISO002-REV-008 or ANISO002-REV-009. PR #106 remains Draft and
REQ-ANISO-002 remains `implemented`. A fresh isolated mathematical/numerical
re-review of the complete PR and exact Repair head is required next. This
Repair does not mark the PR Ready, merge it, or begin another requirement.

## Fresh independent re-review after ANISO002-REV-008/009 Repair

- Exact reviewed head: `f99be614e9f0f8fbaec57279180d2872b458c406`
- Latest production/test Repair head:
  `1a95323ffd9b8ba43eb5f0390aa02d812edfdba2`
- Base: `d34458f6c29d1b56f2832ddac9356d28a87a3f8f`
- Re-review date: 2026-07-22
- Result: ANISO002-REV-007 and ANISO002-REV-009 are closed;
  ANISO002-REV-008 remains open as the P2 finding below

A fresh isolated read-only project `math_reviewer` received only the bounded
REQ-ANISO-002 summary and integrated dependency closure, Issue #105 criteria
and exclusions, the M6 plan, ANISOTROPY and ADR-0009/ADR-0010 contracts, the
complete PR and latest Repair diffs, directly relevant source/tests, and the
recorded validation evidence. It inherited no Repair reasoning transcript and
made no repository change.

### ANISO002-REV-008 - P2: rounded D=3 PSD acceptance is nonmonotone

Affected code and contracts:

- `crates/georbf/src/orientation_tensor.rs:1068-1078`
- `docs/architecture/ANISOTROPY.md:103-107`
- the greatest-certified-scale claim in the Repair evidence above

The ordered-bit bisection assumes that exact PSD certification remains
monotone after every off-diagonal product is independently rounded to
binary64. Although the unrounded affine family `D + scale O` intersects the
convex PSD cone in an interval, the represented family
`round(original_ij * scale)` need not. In D=3, independently changing the
three rounded correlations can alternate the exact determinant sign.

A valid unit-weight single-sample public counterexample starts from direction

```text
[0.2929103819395529, 0.39358823180141855, -0.3403261034581484]
```

and forms the represented normalized tensor

```text
[[ 0.24064309056141503,  0.3233558601853925,  -0.2795978919989036 ],
 [ 0.3233558601853925,   0.4344982940183372,  -0.37570003220289544],
 [-0.2795978919989036,  -0.37570003220289544,  0.3248586154202477 ]].
```

Independent exact rational evaluation of every represented principal minor
gives this acceptance sequence near one:

| Scale bits | Scale | Exact determinant sign | PSD |
| --- | ---: | ---: | --- |
| `1.0.to_bits() - 3` | `0.9999999999999997` | positive | yes |
| `1.0.to_bits() - 2` | `0.9999999999999998` | negative | no |
| `1.0.to_bits() - 1` | `0.9999999999999999` | positive | yes |
| `1.0.to_bits()` | `1.0` | negative | no |

All three two-by-two principal minors are also positive at
`1.0.to_bits() - 1`. It is therefore the greatest certified represented
factor because the only larger factor, one, is rejected. The public estimator
instead returns `1.0.to_bits() - 3`; the parent Review task independently
reproduced that result and removed its temporary probe before recording this
evidence.

The returned tensor remains PSD, but the implementation discards more
correlation than the normative maximum-retention policy permits and the
diagnostic does not report the greatest certified factor. Repair must add a
public D=3 fixed-ratio regression for the sample above with ratios `[1,1,1]`,
requiring the returned scale bits to equal `1.0.to_bits() - 1`, exact
represented PSD, rejection at one, and explicit coverage of the intervening
accepted--rejected--accepted sequence. The replacement bounded construction
must prove maximality without assuming represented PSD monotonicity and must
not add clipping, jitter, a pseudoinverse, hidden regularization, or an input
rejection for this valid sample.

No P0, P1, P3, or additional P2 finding was identified. ANISO002-REV-007 is
closed because production fold state is fixed-size and the actual warmed
allocator regression observes constant calls for four and sixteen samples
under both policies. ANISO002-REV-009 is closed because manual annotations are
gone and the pinned test-only allocator observes actual allocation calls.

### Re-review validation and disposition

- The isolated reviewer passed all 17 public orientation-tensor tests, the
  dedicated allocation test, the private exact-dyadic test, the runnable
  example, format, strict georbf all-target/all-feature Clippy, all 58
  requirement checks, benchmark smoke with checksum
  `1.00428812046557887e4`, and complete diff whitespace validation.
- The parent Review task passed the complete standard workspace gate on exact
  reviewed head `f99be61`: format, warning-denying workspace
  all-target/all-feature Clippy, all-feature workspace tests, workspace
  Rustdoc, all 58 requirement checks, and complete diff whitespace. It also
  passed the example and optimized benchmark smoke with the same checksum.
- Draft Ubuntu CI run 29885690427 passed on exact reviewed head `f99be61`.
  Ready-only Windows, Ubuntu, macOS, and benchmark-smoke CI did not run and are
  not claimed as passed. The recorded unavailable/deferred checks remain
  unexecuted and are not claimed as passed.

PR #106 remains Draft and REQ-ANISO-002 remains `implemented`, not
`integrated`. Open a fresh bounded Repair task for ANISO002-REV-008 only. Do
not repair production code, mark the PR Ready, merge it, or begin another
requirement in this Review task.
