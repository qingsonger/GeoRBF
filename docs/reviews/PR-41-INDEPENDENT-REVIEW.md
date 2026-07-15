# PR #41 Independent Review

- Date: 2026-07-15
- Requirement: REQ-SPIKE-002
- Pull request: #41
- Initial reviewed implementation head: `9cd0c306cc328822df211fac854c34d15606960f`
- Latest re-reviewed head: `66ed708a097bd55235f9a4be012c44870a2ffe33`
- Base: `origin/main`
- Result: clean latest re-review; no P0, P1, P2, or P3 finding remains

## Review scope and independence

The review used a fresh task and a read-only independent `math_reviewer` that
did not inherit the implementation reasoning. The bounded inputs were the
REQ-SPIKE-002 `show` and `deps` summaries, Issue #40 acceptance criteria,
`docs/architecture/SOLVER_POLICY.md`, ADR-0009, the PR diff, the change and
benchmark records, the excluded harness documentation, and exact validation
evidence for the reviewed head. The dependency closure contains only
REQ-BOOTSTRAP-001, which is integrated.

The review covered rank formulae and thresholds, deterministic equilibration,
finite and shape validation, RRQR/SVD responsibilities, hidden fallback and
regularization risk, independent truth cases, feature configurations,
dependency isolation, interface dispositions, benchmark claims, CI coverage,
and requirement-registry truth.

## Findings

### P2-1: the unresolved near-threshold case is exactly rank deficient

`spikes/rank-backends/src/main.rs:305` passes `f64::EPSILON / 4.0` to
`near_dependent`, which stores the perturbation as `2.0 + delta` at line 272.
The spacing above `2.0` is `2 * f64::EPSILON`, so this addition rounds back to
exactly `2.0`. The supposedly unresolved matrix is therefore bit-for-bit the
same matrix as the exact-deficiency case at lines 292--299. The other sample,
`1.0e-12`, is clearly resolved rather than a threshold-adjacent truth case.

An independent floating-point check reproduced:

```text
delta=5.55111512312578270e-17 stored_delta=0.00000000000000000e+00 equal_to_two=True
delta=9.99999999999999980e-13 stored_delta=1.00008890058234101e-12 equal_to_two=False
```

Consequently the test named `near_threshold_cases_receive_svd_review` does
not exercise the adopted `max(m,n) * eps * sigma_max` boundary and cannot
support the near-rank-threshold acceptance criterion or the corresponding
claims in ADR-0009 and `changes/REQ-SPIKE-002.md`.

Required repair and regression:

- construct representable perturbations on both sides of the post-
  equilibration SVD threshold and assert that each perturbed matrix differs
  from the exact-deficiency matrix;
- derive the expected classifications from an analytic or independently
  high-precision singular-value truth calculation rather than the candidate
  backend under test; and
- assert the expected SVD review rank and retain RRQR disagreement or
  threshold adjacency as explicit diagnostic evidence.

### P2-2: disabling all backend features produces a vacuous green result

Both dependencies are optional in `spikes/rank-backends/Cargo.toml:10-17`.
With no feature selected, `Backend::ALL` is empty at
`spikes/rank-backends/src/main.rs:20-25`. Every backend assertion in lines
276--355 is inside a loop over that empty slice, and the benchmark loop at
lines 239--246 also performs no backend work.

The review reproduced both false-success paths:

```text
cargo test --manifest-path spikes/rank-backends/Cargo.toml --no-default-features
# 6 passed; 0 failed

cargo run --manifest-path spikes/rank-backends/Cargo.toml \
  --no-default-features --release -- --smoke
# prints only the CSV header and exits successfully
```

This is a placeholder success path: a configuration that evaluates no RRQR or
SVD backend reports successful tests and benchmark execution.

Required repair and regression:

- reject the zero-backend configuration at compile time or before reporting
  success; and
- add a negative configuration check proving that `--no-default-features`
  cannot pass without selecting at least one backend, while preserving the
  existing faer-only, nalgebra-only, and all-feature positive checks.

## Validation evidence

The following focused checks passed locally on the reviewed implementation
head:

```text
cargo test --manifest-path spikes/rank-backends/Cargo.toml --all-features
cargo test --manifest-path spikes/rank-backends/Cargo.toml \
  --no-default-features --features faer-backend
cargo test --manifest-path spikes/rank-backends/Cargo.toml \
  --no-default-features --features nalgebra-backend
cargo clippy --manifest-path spikes/rank-backends/Cargo.toml \
  --all-targets --all-features -- -D warnings
```

The production workspace dependency tree contains neither nalgebra nor faer.
Draft Ubuntu CI run 29343523143 passed on the exact reviewed head. The complete
Windows, Ubuntu, and macOS ready-head matrix and benchmark-smoke gate has not
run and must remain deferred until a fresh repair and clean re-review.

The version, license, MSRV, dependency-count, and crate-archive evidence is
consistent with the pinned lockfile and locally inspectable metadata. The ADR
does not record the exact unsafe-source scan expression, file scope, or
feature/target filters, so its exact unsafe-match counts are not strictly
reproducible. This is retained as a documentation risk for the production
dependency re-audit required by REQ-CPD-001, rather than a third finding in
this bounded spike review.

Wide matrices, explicit zero rows and columns, RRQR scaling invariance, and an
independent high-precision guard-band suite remain untested. ADR-0009 already
assigns the production guard band and high-precision review evidence to
REQ-CPD-001; the repaired spike still must supply the threshold-boundary truth
case required by P2-1.

The implementation task's stable-head standard workspace gate is retained as
evidence because the reviewed implementation head is unchanged. This Review
adds only review and handoff evidence plus the review-document registry link;
it does not change production code, tests, manifests, schemas, or build inputs.

## Repair evidence pending fresh re-review

Repair code/test head `30bd49520131ff085fd538c93ad767455cdade43`
addresses only P2-1 and P2-2. The threshold regression now constructs
already-equilibrated matrices whose repeated third-row coefficient is exactly
12 and 15 representable ULPs above `0.5`. Both differ bit-for-bit from the
exact-deficiency matrix and remain unchanged by all eight equilibration
passes.

An independent analytic calculation splits each matrix into the exact unit
singular direction `(1,-1,0)/sqrt(2)` and a two-dimensional block. For
coefficient `a`, that block has Frobenius norm squared `4 + 2*a^2` and
determinant `1 - 2*a`; the regression computes its largest singular value from
the stable quadratic root and its smallest as the absolute determinant divided
by the largest. The 12-ULP case is below the recorded SVD threshold with
analytic rank two, while the 15-ULP case is above it with analytic rank three.
Both are within 12% of the threshold, and faer 0.24.4 and nalgebra 0.35.0
produce the expected SVD review ranks with explicit threshold-adjacency
assertions.

The harness now emits a compile-time error unless `faer-backend` or
`nalgebra-backend` is enabled. CI verifies that the zero-backend check fails
with that exact diagnostic, and tests the faer-only, nalgebra-only, and
all-feature positive configurations separately.

On the stable repair head, spike formatting, warning-denying all-target
Clippy, all three 6/6 feature-test configurations, the expected zero-backend
failure, and the release smoke workload passed. The complete workspace
formatting, warning-denying Clippy, all 139 tests, all 25 doctests and
compile-fail tests, all 58 requirement checks, and `git diff --check` also
passed. The subsequent review-record and bounded-handoff commit changes only
documentation. A fresh independent re-review is still required before either
finding can be considered closed or PR #41 can be marked ready.

## Fresh re-review result

A fresh read-only `math_reviewer` independently inspected the complete PR diff
at exact head `431da7ffeb39e49d1dd5d0df94318fe5cb75dc34`. It did not inherit the
Repair reasoning. It found no P0, P1, or P2 issue and confirmed that both
original P2 findings are closed, but found the following new P3 evidence
defect.

### P3-1: repair evidence names a nonexistent full commit object

The repair-evidence paragraph at line 131 records
`30bd4952105acc6a04a7dcaff72493692f29d051` as the repair code/test head. That
object does not exist. The actual Git commit resolved by the documented short
prefix is `30bd49520131ff085fd538c93ad767455cdade43`, whose parent is review
evidence commit `e1db3492866da63115784432977f3c1e7d039b56`. The PR body's
Repair update repeats the same nonexistent full object name.

The incorrect full hash breaks exact reproduction of durable repair evidence;
the valid short prefix elsewhere does not make an explicitly recorded invalid
object truthful. A Repair task must correct both evidence locations to the
actual full hash and verify it with `git cat-file -e <hash>^{commit}`. Because
that repair creates a new PR head, another fresh independent re-review is
required before the PR can be marked ready.

The reviewer independently confirmed P2-1 closed. For the already-equilibrated
matrix with its repeated coefficient 12 ULPs above `0.5`, the analytic ratio
`sigma_min / tau_svd` is approximately `0.8888888888888884`, so its rank is
two. At 15 ULPs the ratio is approximately `1.1111111111111103`, so its rank is
three. These results follow from the exact singular direction and the
two-dimensional block invariants, not from either candidate backend. Both
inputs are representable, bitwise distinct, unchanged by eight equilibration
passes, and within approximately 11.1% of the strict threshold.

The reviewer also confirmed P2-2 closed. The zero-backend configuration fails
at compile time with the required diagnostic, while CI retains all-feature,
faer-only, and nalgebra-only positive tests plus the exact negative check in
both Draft and ready paths. Focused reproduction passed. No pseudoinverse,
hidden regularization, production numerical dependency, public interface, or
out-of-scope CPD implementation was introduced.

Draft Ubuntu CI run 29373908569 passed on exact PR head `431da7f`. The complete
Windows, Ubuntu, and macOS ready-head matrix and benchmark-smoke gate remains
intentionally unrun because P3-1 blocks the ready transition.

## Second fresh re-review result

A second fresh read-only `math_reviewer` independently inspected the complete
PR diff at exact head `66ed708a097bd55235f9a4be012c44870a2ffe33`.
It received only the bounded requirement and dependency summaries, Issue #40,
the normative solver policy and ADR, the complete diff, durable spike and
benchmark evidence, and exact validation state. It inherited no Repair
reasoning and found no P0, P1, P2, or P3 issue.

P2-1 remains closed: the 12- and 15-ULP already-equilibrated cases are
bitwise distinct, bracket the strict SVD threshold under independent analytic
singular-value truth, and produce the required backend review ranks. P2-2
remains closed: the empty feature selection fails at compile time, while both
single-backend and the combined configurations execute all six cases. P3-1 is
closed: the valid repair object
`30bd49520131ff085fd538c93ad767455cdade43` and its parent resolve, and the
invalid full hash remains only in the historical finding narrative.

Focused spike formatting, warning-denying Clippy, all three positive feature
test configurations, the expected zero-backend failure, release smoke,
requirement validation, and `git diff --check` passed. Draft Ubuntu CI run
29375239847 passed on the exact reviewed head. The following clean-review
evidence commit is documentation-only; the stable repair code/test head and
its complete local standard gate remain unchanged. The ready-head Windows,
Ubuntu, macOS, and benchmark-smoke matrix remains pending the ready event.

## Disposition

PR #41 may advance through the mandatory ready-head integration sequence.
REQ-SPIKE-002 remains `documented` until the implementation PR is merged and
an isolated integration-state change records the completed evidence. This
Review task must not start REQ-CPD-001.

## Integration evidence

- Final clean-review evidence head
  `3e6f4e1ce1379b03b5e6875cdf008ed96b0d1753` passed the complete Windows,
  Ubuntu, and macOS ready matrix, including every benchmark smoke workload, in
  run 29376057562.
- PR #41 squash-merged exactly once as
  `4c1ddeb5448d13f5657d00f9a8a3be3081a6892b`; Issue #40 closed as completed.
- Post-merge `main` run 29376336046 passed the same complete three-platform
  correctness, benchmark-smoke, and requirement-registry gate on that exact
  merge commit.
- Integration-state PR #42 contains only the isolated registry and progress
  evidence change. Its complete local standard gate passed before the ready
  transition; it changes no production code, tests, manifest, schema, build
  input, API, numerical behavior, dependency, tag, or release.
