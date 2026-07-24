# PR #127 Independent Review

- Requirement: REQ-TUNE-001
- Issue: https://github.com/qingsonger/GeoRBF/issues/126
- Pull request: https://github.com/qingsonger/GeoRBF/pull/127
- Branch: `codex/req-tune-001-deterministic-tuning`
- Base head: `4093c26590b6e25324c87103ef3d0f8223f2469c`
- Reviewed head: `555157ce9f1ac356ac1c0fc13630ffe675e06a26`
- Draft CI run: 30058923924
- Review date: 2026-07-24
- Result: changes required; two P1, two P2, and one P3 finding

## Scope and independence

A fresh read-only project `math_reviewer` received only the bounded
REQ-TUNE-001 summary and integrated dependency closure, Issue #126 acceptance
criteria and exclusions, the M7 plan, the relevant solver policy and
ADR-0007, the complete base-to-head diff, tests, benchmark evidence, and exact
validation results. It inherited no Implement reasoning and made no repository
or remote change.

The reviewer independently checked scoring formulae, dimensions and units,
inclusive bounds, fold construction, deterministic seeds and exact ties,
evidence validation, failure behavior, hidden regularization and fallback,
interface dispositions, immutable D=1/D=2/D=3 API coverage, allocation
behavior, tests, benchmark and CI wiring, and registry state.

## Findings

### P1 TUNE001-REV-001: the GCV score omits the canonical normalization by `n`

`crates/georbf/src/tuning.rs:1484-1485` implements
`RSS / (n - effective_dof)^2`, and the assertion at
`crates/georbf/tests/tuning.rs:263-281` expects that value. The same formula is
claimed at `docs/architecture/SOLVER_POLICY.md:308-309` and
`changes/REQ-TUNE-001.md:21-23`.

For a linear smoother `A`, canonical generalized cross-validation is

```text
V = (RSS / n) / ((n - trace(A)) / n)^2
  = n * RSS / (n - trace(A))^2.
```

The implemented expression is `V / n`. It has the same minimizer only when all
candidates use one common observation count, but the evaluator API neither
enforces nor documents that invariant. Allowed evidence can therefore reverse
selection:

```text
A: RSS=1,    n=100, df=90 -> current 0.01, canonical 1
B: RSS=0.02, n=2,   df=1  -> current 0.02, canonical 0.04
```

The current code selects A while canonical GCV selects B. The independent
truth source is Golub, Heath, and Wahba, equation 1.4, in
[Generalized Cross-Validation as a Method for Choosing a Good Ridge
Parameter](https://www.stat.cmu.edu/technometrics/70-79/VOL-21-02/v2102215.pdf).

Required Repair: change the existing `n=4`, `df=2`, `RSS=1` truth assertion
from `0.25` to `1.0`. Add candidate-specific observation counts and either
require an explicit common-count mismatch error or otherwise prove the
canonical score is used. A common observation population should normally be
enforced.

### P1 TUNE001-REV-002: the distance score overflows for valid bounded values

`crates/georbf/src/tuning.rs:1365-1367` evaluates
`ln(value / target)`. Both operands can be finite, strictly positive, and
inside valid bounds while their quotient overflows or underflows.

With `value = f64::MAX` and `target = f64::MIN_POSITIVE`, the direct quotient
is infinity, but the equivalent stable expression
`ln(value) - ln(target)` is finite, approximately `1418.17913142565`, with a
finite squared residual. The current strategy returns `NonFiniteScore` for an
input accepted by its own domain and bounds validation.

Required Repair: add a D=1 regression with two points separated by
`f64::MIN_POSITIVE`, one `f64::MAX` length candidate, and matching inclusive
bounds. Distance tuning must produce a finite score. Cover the reciprocal
extreme as well if practical, then compute the log ratio without forming the
quotient.

### P2 TUNE001-REV-003: one-fold cross-validation has an empty training set

`crates/georbf/src/tuning.rs:1391-1428` accepts `fold_count == 1`. The sole
validation fold contains every observation, leaving the training complement
promised by `crates/georbf/src/tuning.rs:404-414` empty. This is not
cross-validation and can let an evaluator report apparently valid evidence
from an empty or trend-only fit.

The existing precondition test at
`crates/georbf/tests/tuning.rs:390-421` covers only a fold count greater than
the observation count.

Required Repair: with at least two observations, request one fold and require
a structured error before evaluation plus zero evaluator calls. Valid
cross-validation must require `2 <= folds <= observations`.

### P2 TUNE001-REV-004: CV diagnostics discard raw weighted fold evidence

The evaluator returns `CrossValidationEvidence { weighted_squared_error,
weight }`, but `crates/georbf/src/tuning.rs:485-489` and
`crates/georbf/src/tuning.rs:790-812` retain only each quotient in
`fold_losses`. The total weighted score cannot generally be reconstructed or
audited from those diagnostics, especially for three or more unequal fold
weights. This conflicts with the complete evidence claims at
`docs/architecture/SOLVER_POLICY.md:307-319` and
`changes/REQ-TUNE-001.md:31-33`.

Required Repair: have a three-fold evaluator return unequal weights and
weighted errors. Diagnostics must retain every exact
`CrossValidationEvidence` pair, and recomputing
`sum(weighted_squared_error) / sum(weight)` from them must reproduce the
candidate score.

### P3 TUNE001-REV-005: stable sorting bypasses structured allocation failure

The work vectors are fallibly reserved, and `try_tune` advertises allocation
errors, but `crates/georbf/src/tuning.rs:1337` and
`crates/georbf/src/tuning.rs:1403-1406` use stable slice sorting. Rust stable
sort allocates auxiliary storage outside the explicit
`try_reserve_exact`/`TuningError::AllocationFailed` path at
`crates/georbf/src/tuning.rs:1551-1558`.

Both orderings already have total deterministic keys, so the corresponding
in-place unstable sorts preserve the required deterministic result without an
untracked allocation.

Required Repair: use nonallocating in-place ordering. Add an isolated
allocation-counting regression over sufficiently large nearest-distance and
fold-order slices, with work vectors already reserved, requiring zero
allocations during ordering.

No other P0, P1, P2, or P3 finding was identified.

## Independent truth and validation

The reviewer independently confirmed:

- all five parameter domains, inclusive bounds, active candidate/bound parity,
  and exact duplicate rejection;
- the real-arithmetic distance objective is dimensionless and invariant under
  common length-unit scaling and orthogonal coordinate transformations;
- locations `[0, 1, 4]` have nearest distances `[1, 1, 3]`, median one, and
  the expected ordinary-scale scores;
- CV currently aggregates `sum(weighted SSE) / sum(weight)` and builds balanced
  nonempty validation folds when `1 <= k <= n`;
- SplitMix64 fold and tie keys depend only on seed and candidate or observation
  index, with total deterministic ordering;
- power selection minimizes a validated nonnegative maximum squared power;
- traversal and failure handling add no jitter, automatic regularization,
  pseudoinverse, candidate skipping, criterion fallback, or semantic mutation;
- `Dim<D>: SupportedDimension` constrains the public core to D=1, D=2, and
  D=3; and
- Rust exports, later-milestone interface N/A dispositions, benchmark
  registration, Ready/main smoke wiring, and the `in_progress` registry state
  are consistent.

The isolated reviewer and parent Review task independently passed:

- all 11 tuning integration tests;
- the tuning rustdoc example;
- the five-strategy release benchmark smoke;
- the 58-requirement registry check; and
- the complete base-to-head whitespace check.

Draft CI run 30058923924 passed its configured Ubuntu correctness job on exact
reviewed head `555157c`; the Ready-only Windows, Ubuntu, and macOS benchmark
matrix was skipped as designed and is not claimed. The stable implementation
head had already passed the complete standard local gate after the last
production, test, manifest, CI, and registry change. This Review changes only
Markdown evidence, so that immutable complete gate remains applicable.

PR #127 must remain Draft and REQ-TUNE-001 remains `in_progress`. A fresh
Repair task must address only TUNE001-REV-001 through TUNE001-REV-005, add the
specified regressions, run focused checks and one complete stable-head standard
gate after the last production or test change, update this record and the
bounded handoff, push, and stop for another fresh independent re-review. This
Review does not repair production code, mark the PR ready, merge it, or begin
REQ-PERF-001.

## Repair evidence

Repair implementation head `ae570a5f936c8e133f80f4e132b0a9a6b91fd330`
addresses only TUNE001-REV-001 through TUNE001-REV-005:

- TUNE001-REV-001: GCV now evaluates canonical
  `n * RSS / (n - effective_dof)^2`; a structured mismatch rejects candidates
  that report different observation counts. The truth score for `n=4`,
  `df=2`, `RSS=1` is `1.0`, and a `100`-versus-`2` count regression rejects the
  second candidate.
- TUNE001-REV-002: distance scoring computes `ln(value) - ln(target)` without
  forming an overflow- or underflow-prone quotient. D=1 regressions cover both
  `MAX / MIN_POSITIVE` and its reciprocal.
- TUNE001-REV-003: fold construction now rejects one fold before evaluator
  dispatch, with a regression proving zero evaluator calls.
- TUNE001-REV-004: candidate diagnostics retain each exact
  `CrossValidationEvidence` pair. A three-fold unequal-weight regression
  recomputes the reported score from raw weighted squared errors and weights.
- TUNE001-REV-005: nearest-distance and seeded fold order use in-place unstable
  sorting with deterministic total comparisons. An allocation-counter
  regression over pre-reserved 4096-entry work vectors observes zero allocator
  calls during either ordering.

Focused validation passed all 14 tuning integration tests, the isolated
ordering-allocation unit regression, warning-denying georbf all-target/all-
feature Clippy, and both smoke and 128-candidate five-strategy release
benchmarks. The exact implementation head then passed the complete standard
gate: format, warning-denying workspace/all-target/all-feature Clippy,
all-feature workspace tests, workspace doctests, the 58-requirement registry
check, and the complete PR whitespace check.

Only this Markdown repair evidence and the bounded handoff change after the
validated implementation head. These are Repair claims, not an independent
re-review. PR #127 remains Draft and REQ-TUNE-001 remains `in_progress`; a
fresh isolated `math_reviewer` must verify the repairs and check for new P0--P3
findings before any Ready transition.
