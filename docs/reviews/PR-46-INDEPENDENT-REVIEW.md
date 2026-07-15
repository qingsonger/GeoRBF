# PR #46 Independent Review

- Date: 2026-07-15
- Requirement: REQ-CPD-001
- Issue: #45
- Pull request: #46
- Reviewed head: `85f2ae3207c8f0677463fc4bd00944e5d71cbd0a`
- Base: `origin/main` at
  `98593115d73f1347a67deaa2d6d8d77a2d1aee87`
- Result: one P1 and one P2 finding; PR #46 must remain Draft

## Review scope and independence

A fresh read-only independent `math_reviewer` inspected the Issue #45
acceptance criteria, the complete 17-file PR diff, the bounded REQ-CPD-001
requirement and dependency summaries, the M2 plan, the CPD and solver
policies, ADR-0004/0007/0009, the change and benchmark records, and exact
Draft-CI evidence. It did not inherit the implementation reasoning and made no
repository or remote changes.

The review covered polynomial actions, dimensions and units, deterministic
equilibration, RRQR and SVD thresholds, ambiguity boundaries, null-space
construction and verification, `Q^T w = 0` provenance, projected energy,
hard-failure behavior, hidden regularization, backend semantics, allocations,
diagnostics, interfaces, independent truth tests, benchmark claims, CI, and
requirement evidence.

## Findings

### P1-1: equilibration can silently underflow a full-rank action matrix

`crates/georbf/src/cpd.rs:939-951` checks that the cumulative row multiplier
is finite and nonzero, but it does not check whether dividing an originally
nonzero entry by the row infinity norm produces zero. The following column
pass at `crates/georbf/src/cpd.rs:954-970` then treats an underflowed all-zero
column as an original structural zero and skips its scale.

The representable matrix

```text
Q = [[1e308, 1e-16],
     [1e308, 2e-16]]
```

has determinant `1e292` and exact rank two. The first row pass nevertheless
rounds both small entries divided by `1e308` to floating-point zero, so the
backend receives the rank-one matrix `[[1,0],[1,0]]`. The cumulative row
multipliers remain finite and nonzero at `1e-308`, so the existing guard does
not fire. A nonzero action has therefore been erased before the opposite-axis
scale is considered.

This violates the unrepresentable-scaling rule in ADR-0009 and the
unit-invariant rank contract in `docs/math/CPD_AND_POLYNOMIALS.md`. It can
misclassify an exactly full-rank hard system as deficient.

Required repair and regression:

- add a scale regression near
  `coordinate_units_and_nonzero_functional_scaling_preserve_rank` using two
  same-unit value-functional combinations whose actions form the matrix above;
- require either preserved full-rank classification or
  `UnrepresentableEquilibrationScale`, never `RankDeficient`; and
- detect nonzero-to-zero entry underflow during each scaling operation, or use
  a representation that does not commit that underflow before the opposite-
  axis scale is known.

### P2-1: SVD non-convergence discards available rank diagnostics

`diagnose_rank` has already computed equilibration scales, zero row and column
indices, original and scaled matrices, the RRQR diagonal, threshold, and rank
at `crates/georbf/src/cpd.rs:836-855`. If `SVD::try_new` fails, however,
`crates/georbf/src/cpd.rs:856-860` returns `SvdDidNotConverge`, whose definition
at `crates/georbf/src/cpd.rs:607-611` contains only the iteration limit.

This conflicts with `docs/architecture/SOLVER_POLICY.md` and ADR-0009, which
require structured rank evidence to remain available when bounded SVD review
does not converge. The missing evidence also prevents a caller from
distinguishing the diagnosed matrix and RRQR screen from an opaque backend
failure.

Required repair and regression:

- add a test-only backend seam that forces SVD non-convergence;
- assert that the error retains original and scaled norms, row and column
  scales, zero indices, RRQR diagonal, threshold and rank, and the iteration
  limit; and
- mark SVD-derived fields and the final effective decision explicitly
  unavailable rather than fabricating them.

## Validation evidence

- Draft CI run 29386068937 passed on exact reviewed head `85f2ae3`; the ready
  Windows, Ubuntu, and macOS matrix and benchmark-smoke job was correctly
  skipped.
- The implementation task's complete local standard gate remains evidence for
  the unchanged production, tests, manifests, schemas, and build inputs.
- This Review adds only the independent review record, its registry link, and
  the bounded handoff. All 58 requirement checks and `git diff --check` pass.

## Repair evidence pending fresh re-review

Repair code/test head `d5c6a89eaa9045f5ec8f7bf6548f1b82eea21a71`
addresses only P1-1 and P2-1:

- P1-1: every row and column scaling step now rejects a nonzero entry that
  would round to zero. The independent regression assembles the exact
  full-rank action matrix `[[1e308,1e-16],[1e308,2e-16]]` from same-unit value
  functionals and forbids `RankDeficient`.
- P2-1: a test-only backend seam forces bounded SVD non-convergence. The
  structured error retains original and scaled norms, row and column scales,
  original zero indices, RRQR diagonal, threshold, rank, and the iteration
  limit; all SVD-derived fields and the final decision are explicitly `None`.

Both focused regressions and the complete stable-head standard gate passed:

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo test --doc --workspace
cargo xtask requirements check
```

The requirement checker reports all 58 v1 entries valid, and
`git diff --check` passes. `cargo-nextest`, `cargo-deny`, `cargo-audit`, and
`cargo-semver-checks` are unavailable locally; Miri is unavailable for pinned
Rust 1.96.1. Sanitizers, executable fuzzing, mutation testing, allocation
instrumentation, and API/ABI/schema snapshot checks remain assigned to later
requirements and release gates. Local `actionlint` is unavailable.

## Fresh re-review of the repaired head

- Date: 2026-07-15
- Reviewed head: `687731f0807e7c541123ae1c419d724b458546d0`
- Repair code/test head: `d5c6a89eaa9045f5ec8f7bf6548f1b82eea21a71`
- Base: `98593115d73f1347a67deaa2d6d8d77a2d1aee87`
- Result: P1-1 and P2-1 closed; one new P1 and two new P2 findings;
  PR #46 must remain Draft

A second fresh read-only `math_reviewer` received only the bounded requirement
and dependency summaries, normative documents and ADRs, complete PR and repair
diffs, and validation evidence. It verified that the final-head delta after the
repair code/test head changes only this review record and the bounded handoff.
It independently inspected the complete repaired implementation rather than
only the original findings and made no repository or remote changes.

### Closed: P1-1 equilibration underflow

Every row and column scaling operation now rejects a nonzero entry that would
round to zero at `crates/georbf/src/cpd.rs:1079-1088` and `1106-1116`, in
addition to validating cumulative multipliers. The exact extreme-scale
regression at `crates/georbf/tests/cpd.rs:305-329` confirms that the
representable full-rank action is never mislabeled `RankDeficient`.

### Closed: P2-1 SVD non-convergence evidence

The incomplete diagnostic at `crates/georbf/src/cpd.rs:185-228`, `943-955`,
and `1009-1040` retains shapes, norms, scales, original zero indices, RRQR
diagonal, threshold and rank, plus the iteration limit. Every SVD-derived field
and the final decision is explicitly unavailable. The forced regression at
`crates/georbf/src/cpd.rs:1352-1408` exercises that path.

### P1-2: null-space construction discards the safe equilibration

`crates/georbf/src/cpd.rs:353-354` passes the original action matrix to
`construct_null_space`, and `crates/georbf/src/cpd.rs:1140-1146` performs an
unpivoted QR on that original matrix. Full rank was established on
`D_row Q D_column`, but QR of the unscaled matrix need not remain numerically
representable.

For the valid D=1, order-one action

```text
Q = [[1e200],
     [1e200]],
```

equilibration safely produces `[[1],[1]]`, the exact rank is one, and the null
space is spanned by `[1,-1]`. Nalgebra's Householder norm on the original
column squares the entries and overflows. The dual case
`[[1,0],[0,1e-308],[0,1e-308]]` has a representable, clearly full-rank
equilibrated action but loses the original second-column norm through
underflow. A nonzero functional-row scale can therefore turn a valid system
into a construction or verification error, violating the scale-invariance
contract in `docs/math/CPD_AND_POLYNOMIALS.md`.

Required repair and regression:

- construct from the safely equilibrated action and map the null basis back
  with `z = D_row u` before deterministic reorthogonalization; using the
  scaled basis unchanged is mathematically incorrect;
- compare D=1, order-one value centers with coefficients `1` and `1e200`,
  requiring both assemblies to succeed and independently satisfy `Q^T Z = 0`
  and `Z^T Z = I`; and
- add a derivative-row or equivalent action case at `1e-308` to cover the
  underflow direction.

### P2-2: binding infinity residuals are maximum entries

`crates/georbf/src/cpd.rs:1206-1234` records the largest individual entry of
the column-scaled `Q^T Z`, and `crates/georbf/src/cpd.rs:1236-1247` does the
same for `Z^T Z-I`. The public contract at
`crates/georbf/src/cpd.rs:233-240` calls both values infinity residuals, whose
matrix norm is `max_i sum_j |A_ij|`, not `max_ij |A_ij|`. The checks at
`crates/georbf/src/cpd.rs:355-357` can therefore understate aggregate residual
by up to the nullity and accept a residual above the documented tolerance.

Required regression: exercise the private verifier with two residual entries
in one row, each `0.75 * tolerance`, and require a reported infinity norm of
`1.5 * tolerance`; independently recompute both matrix infinity norms.

### P2-3: original-unit weight residual can discard NaN

`crates/georbf/src/cpd.rs:1274-1286` forms unscaled products before summation
and folds them with `f64::max`. Oppositely signed overflowing products can
produce NaN, while `finite.max(NaN)` retains the finite operand and can
fabricate a zero original-unit residual. For `Q^T = [10,10]` and large,
nearly cancelling finite weights near `+/-7e307`, each product overflows even
though the exact residual can be finite and nonzero. This conflicts with the
structured non-finite-result and original-unit diagnostic contracts.

Required regression: pass those actions and weights with a representable
relative difference near `1e-15` to the private residual helper and require a
finite nonzero residual derived by a scaled computation, or an explicit
unrepresentable diagnostic, never zero obtained by discarding NaN.

## Re-review validation and disposition

- Exact final-head Draft CI run 29387532506 passed its complete Ubuntu job;
  the ready Windows, Ubuntu, macOS, and benchmark-smoke matrix was correctly
  skipped.
- The complete standard gate passed on exact repair code/test head `d5c6a89`.
  The final-head delta before this review conclusion is documentation-only, so
  that immutable code/test evidence remains applicable.
- The reviewer found no P0 or P3 issue. Polynomial actions, signs, complete
  polynomial spaces, eight-pass equilibration, thresholds and ambiguity band,
  provenance, projected energy, hard failures, determinism, allocation shape,
  interface dispositions, and benchmark routing otherwise match the scoped
  contracts.

PR #46 must remain Draft. A fresh Repair task must address only P1-2, P2-2,
and P2-3, add their regressions, run the required checks, and stop for another
fresh independent re-review. This Review task must not repair production code,
mark the PR ready, merge it, or begin another requirement.

## Repair evidence pending fresh re-review: P1-2, P2-2, and P2-3

Repair code/test head `10d3892381356ed5453e1c58b5daceefee037dda`
addresses only the three findings from the second independent review:

- P1-2: rank diagnosis now returns the safely equilibrated action matrix used
  by RRQR/SVD. Null-space QR constructs `U` from that matrix, maps each column
  with `z = D_row u`, and applies deterministic twice-reorthogonalization with
  a stable norm. Independent D=1 regressions cover value rows scaled from `1`
  to `1e200` and derivative rows at `1e-308`; both independently recompute the
  column-scaled `Q^T Z` and `Z^T Z-I` matrix infinity norms.
- P2-2: binding verification now sums absolute residual entries across each
  matrix row before taking the maximum. A private verifier regression places
  two `0.75 * tolerance` entries in one row and requires the reported side
  residual to be `1.5 * tolerance`; it also independently recomputes the
  orthonormality matrix infinity norm.
- P2-3: original-unit residuals are recovered from normalized dot products by
  trying representable multiplication orders, never by forming overflowing
  original products. A regression with `Q^T = [10,10]` and nearly cancelling
  weights near `+/-7e307` requires the finite nonzero residual; an actually
  unrepresentable restored weight residual returns the structured
  `UnrepresentableOriginalWeightResidual` error.

All focused regressions pass. The complete standard gate passed on exact
repair code/test head `10d3892`: formatting, warning-denying workspace Clippy,
all-feature workspace tests, workspace doc tests, and all 58 requirement
checks. `git diff --check` also passed. Four consecutive repaired benchmark
runs produced the bit-identical checksum `-4.97657470788226419e-12` and the
updated local baseline recorded in `docs/benchmarks/REQ-CPD-001.md`.

`cargo-nextest`, `cargo-deny`, `cargo-audit`, and `cargo-semver-checks` remain
unavailable locally; Miri remains unavailable for pinned Rust 1.96.1.
Sanitizers, executable fuzzing, mutation testing, allocation instrumentation,
and API/ABI/schema snapshot checks remain assigned to later requirements and
release gates. Local `actionlint` is unavailable. The subsequent review-record
and bounded-handoff update is documentation-only, so the stable code/test-head
gate remains applicable. PR #46 must remain Draft for a fresh independent
re-review of the complete repaired diff.

## Fresh re-review of the scaled null-space repair

- Date: 2026-07-15
- Reviewed head: `9d7177eb034ae07e8ef04a915a0fa06664b8450e`
- Repair code/test head: `10d3892381356ed5453e1c58b5daceefee037dda`
- Base: `98593115d73f1347a67deaa2d6d8d77a2d1aee87`
- Result: P1-2, P2-2, and P2-3 closed; one new P2 finding; PR #46
  must remain Draft

A third fresh read-only `math_reviewer` received only the bounded requirement
and dependency summaries, Issue #45 acceptance criteria, normative documents
and ADRs, the complete PR diff, and validation evidence. It independently
reviewed the exact repaired head without inheriting the Repair reasoning and
made no repository or remote changes.

### Closed: P1-2 scaled null-space construction

Null-space QR now uses the safely equilibrated action and maps its basis
through the recorded row scales before stable deterministic
reorthogonalization. The reported common-`1e200` value-row overflow and
`1e-308` derivative-row underflow construction failures are closed by the
implementation and independent regressions.

### Closed: P2-2 matrix infinity residuals

Both side-condition and orthonormality binding quantities now use maximum
absolute row sums. The two-entry aggregate regression independently recomputes
both matrix infinity norms.

### Closed: P2-3 overflowing original-unit weight products

Original-unit weight residual recovery no longer forms the reported
overflowing products or folds NaN into zero. The independent regression
restores a finite nonzero residual, and an unrepresentable restored value is a
structured error.

### P2-4: column normalization can fabricate zero original-unit residuals

`crates/georbf/src/cpd.rs:1230-1235` normalizes mapped row scales by their
maximum without detecting nonzero-to-zero underflow. The null-space and
expanded-weight residual helpers at `crates/georbf/src/cpd.rs:1271-1287` and
`1328-1341` independently divide an original action column by its maximum.
That normalization can also underflow nonzero actions to zero, after which
`rescale_residual` at `crates/georbf/src/cpd.rs:1364-1366` immediately reports
zero rather than recovering the original-unit residual.

The independent public-API reproducer uses D=1, CPD order one, and

```text
Q = [1e308, 1e-308, 1e-308]^T.
```

Equilibration succeeds with row scales `[1e-308, 1e308, 1e308]`. Assembly
returns a basis containing, up to sign,
`z = [0, -1/sqrt(2), -1/sqrt(2)]`. Direct original-unit evaluation gives

```text
Q^T z = -sqrt(2) * 1e-308
      ~= -1.414213562373095e-308,
```

which is finite and representable. Nevertheless, the reported original
side-condition residual and the corresponding expanded-weight original
residual are both `0.0`, because column normalization rounds the action to
`[1,0,0]`. The basis residual remains within the documented relative
tolerance, so this is a P2 diagnostic and provenance failure rather than a
rank or feasibility misclassification. It violates the contract to recover a
truthful original-unit residual or fail explicitly when restoration is
unrepresentable.

Required repair and regression:

- assemble the D=1 order-one system above and independently recompute every
  `Q^T Z` entry plus a unit-coordinate expanded-weight residual in original
  units;
- require the reported original matrix-infinity residual to match the finite
  nonzero independent result, or return an explicit structured
  unrepresentable-arithmetic error, never zero; and
- make column normalization exponent-aware or detect nonzero-to-zero
  normalization before the residual is lost.

## Third-review validation and disposition

- Exact reviewed-head Draft CI run 29390599350 passed its complete Ubuntu job;
  the ready Windows, Ubuntu, macOS, and benchmark-smoke matrix was correctly
  skipped.
- The complete standard gate passed on exact repair code/test head `10d3892`.
  The final reviewed-head delta is documentation-only, so that immutable
  code/test evidence remains applicable.
- The reviewer found no P0, P1, or P3 issue. P1-2, P2-2, and P2-3 are closed;
  P2-4 remains.

PR #46 must remain Draft. A fresh Repair task must address only P2-4, add the
required independent regression, run focused checks and the final standard
gate, update the repair evidence and bounded handoff, commit, push, and stop
for another fresh independent re-review. This Review task must not repair
production code, mark the PR ready, merge it, or begin another requirement.

## Repair evidence pending fresh re-review: P2-4

Repair code/test head `6af215f2758360513fce2b2cdf0d63914dd11bc7`
addresses only P2-4:

- mapped basis columns now decompose each row-scale product before a common
  power-of-two normalization, so neither multiplication order nor an early
  `row_scale / maximum` division can erase a counterbalancing factor;
- original-unit null-space and expanded-weight residuals are accumulated from
  product-wise binary mantissas and exponents with compensated summation,
  independently of the column-max normalization used for the relative
  tolerance check;
- a non-finite restored null-space residual is the structured
  `UnrepresentableOriginalNullSpaceResidual`, while an unrepresentable
  expanded-weight residual remains `UnrepresentableOriginalWeightResidual`;
  neither path folds NaN into zero; and
- the independent public-API regression assembles the exact D=1 order-one
  action `Q=[1e308,1e-308,1e-308]^T`, directly recomputes every `Q^T Z`
  entry in original units, and checks every unit-coordinate expanded weight.
  The reported matrix-infinity residual is finite, nonzero, and matches the
  independent result.

The focused public CPD target and four private CPD diagnostic regressions
pass. On exact repair code/test head `6af215f`, the complete standard gate
passed:

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo test --doc --workspace
cargo xtask requirements check
```

The requirement checker reports all 58 v1 entries valid, and
`git diff --check` passes. The one-iteration optimized benchmark smoke retained
checksum `-4.97657470788226419e-14` and completed in 1.1175 ms on the recorded
local environment. `cargo-nextest`, `cargo-deny`, `cargo-audit`, and
`cargo-semver-checks` remain unavailable locally; Miri is unavailable for
pinned Rust 1.96.1. Sanitizers, executable fuzzing, mutation testing,
allocation instrumentation, and API/ABI/schema snapshot checks remain assigned
to later requirements and release gates. Local `actionlint` is unavailable.

PR #46 remains Draft. This Repair task must stop after pushing for a fresh
independent re-review; it must not mark the PR ready, merge it, or begin another
requirement.

## Fresh re-review of the extreme-range residual repair

- Date: 2026-07-15
- Reviewed head: `cf4976ee7e575da1856d5871f6f6f744fccd43d4`
- Repair code/test head: `6af215f2758360513fce2b2cdf0d63914dd11bc7`
- Base: `98593115d73f1347a67deaa2d6d8d77a2d1aee87`
- Result: P2-4 closed; one new P2 finding; PR #46 must remain Draft

A fourth fresh read-only `math_reviewer` received only the bounded requirement
and dependency summaries, Issue #45 acceptance criteria, normative documents
and ADRs, the complete PR and P2-4 repair diffs, and validation evidence. It
did not inherit the Repair reasoning, independently inspected the exact repaired
head, and made no repository or remote changes.

### Closed: P2-4 extreme-range original-unit residuals

The original reproducer `Q=[1e308,1e-308,1e-308]^T` is closed. Exponent-aware
mapped-basis products and product-wise original-unit accumulation preserve the
finite nonzero residual for every `Q^T Z` column and the corresponding
unit-coordinate expanded weights. The independent reviewer reran the focused
public regression on the exact reviewed head and confirmed its reported
original-unit residual is nonzero.

### P2-5: original-unit accumulation can still fabricate zero

`original_dot_abs` in `crates/georbf/src/cpd.rs:1481-1513` has two independent
false-zero paths:

- At lines 1487-1488, a nonzero restored exponent below the minimum subnormal
  exponent returns `Some(0.0)`. The single exact product
  `2^-1022 * 2^-53 = 2^-1075` is nonzero but unrepresentable and must instead
  reach the structured `UnrepresentableOriginal*Residual` path.
- At lines 1505-1513, each mantissa product is rounded to one `f64` before the
  compensated sum. With `epsilon=2^-52`, the exact two-term dot product
  `((1+epsilon)(1-epsilon)) + (-1)(1)` is
  `-epsilon^2 = -2^-104`, which is finite and representable. The first product
  nevertheless rounds to `1.0`, cancels the second product exactly, and the
  helper reports zero. Independent exact binary-rational arithmetic gives the
  residual magnitude `4.930380657631324e-32`.

The helper supplies both null-space original residuals at
`crates/georbf/src/cpd.rs:1304` and expanded-weight original residuals at line
1373. The defect can therefore make public diagnostics and `Q^T w = 0`
provenance report a fabricated zero. Because the relative residual can remain
within tolerance and rank or feasibility is not misclassified, this is P2.

Required repair and regressions:

- test the two-term near-cancellation above and require the exact
  `epsilon^2` magnitude rather than zero;
- test `f64::MIN_POSITIVE * (0.5 * f64::EPSILON)` and require the helper to
  report unrepresentable arithmetic, with both public callers mapping it to
  their structured `UnrepresentableOriginal*Residual` errors; and
- add a public D=1 order-one near-cancellation action/basis or unit-coordinate
  expanded-weight case whose original-unit residual is checked against
  independent exact-binary or double-double truth.

## Fourth-review validation and disposition

- Exact reviewed-head Draft CI run 29392843498 passed its complete Ubuntu
  correctness job; the ready Windows, Ubuntu, macOS, and benchmark-smoke matrix
  was correctly skipped.
- The complete standard gate passed on exact repair code/test head `6af215f`.
  The exact final-head delta changes only this review record and the bounded
  handoff, so that immutable code/test evidence remains applicable.
- The reviewer found no P0, P1, or P3 issue. Formulae, signs, D=1/D=2/D=3
  polynomial spaces, derivative actions, equilibration, rank thresholds,
  ambiguity handling, scaled null-space mapping, matrix infinity norms, hard
  failures, absence of hidden regularization, projection/KKT equivalence,
  determinism, allocation shape, interface dispositions, and evidence were
  otherwise consistent with the scoped contracts.

PR #46 must remain Draft. A fresh Repair task must address only P2-5, add the
required independent regressions, run focused checks and the final standard
gate, update repair evidence and the bounded handoff, commit, push, and stop
for another fresh independent re-review. This Review task must not repair
production code, mark the PR ready, merge it, or begin another requirement.

## Repair evidence pending fresh re-review: P2-5

Repair code/test head `06ad419c06fd4c887c32be8a8dcd6ff9e1061c68`
addresses only P2-5:

- original-unit dot products now enter every exact finite `f64` significand
  product into a fixed 67-limb signed binary accumulator and round the complete
  sum once, without heap allocation or per-product floating-point rounding;
- exact zero remains zero, finite normal and subnormal results use
  round-to-nearest-even, an exact nonzero result that would round to zero
  reaches the existing structured `UnrepresentableOriginal*Residual` path,
  and overflow remains structured rather than fabricated;
- private caller regressions require the exact `2^-104` cancellation residual
  through null-space and expanded-weight diagnostics, and require the exact
  `2^-1075` product to map to both structured error variants; and
- a public D=1 order-one system compares the null-space and unit-coordinate
  expanded-weight residuals with independent fused double-double truth.

The complete public CPD target passes all 13 tests, and all six private CPD
diagnostic regressions pass. On exact repair code/test head `06ad419`, the
complete standard gate passed:

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo test --doc --workspace
cargo xtask requirements check
```

The requirement checker reports all 58 v1 entries valid, and
`git diff --check` passes. The optimized one-iteration benchmark smoke retained
checksum `-4.97657470788226419e-14` and completed in 0.7554 ms on the recorded
local environment, within the existing local 0.706--1.125 ms complete-assembly
baseline.

`cargo-nextest`, `cargo-deny`, `cargo-audit`, and `cargo-semver-checks` remain
unavailable locally; Miri is unavailable for pinned Rust 1.96.1. Sanitizers,
executable fuzzing, mutation testing, allocation instrumentation, and
API/ABI/schema snapshot checks remain assigned to later requirements and
release gates. Local `actionlint` is unavailable. This review-evidence and
bounded-handoff update changes documentation only, so the immutable
code/test-head gate remains applicable.

PR #46 remains Draft. This Repair task must stop after pushing for a fresh
independent re-review; it must not mark the PR ready, merge it, or begin
another requirement.

## Fifth fresh re-review of the exact-binary accumulation repair

- Date: 2026-07-15
- Reviewed head: `062bae329bbd2194b93d7708a428852c459eccfd`
- Repair code/test head: `06ad419c06fd4c887c32be8a8dcd6ff9e1061c68`
- Base: `98593115d73f1347a67deaa2d6d8d77a2d1aee87`
- Result: P2-5 closed; no P0-P3 finding remains

A fifth fresh read-only `math_reviewer` received only the bounded requirement
and dependency summaries, Issue #45 acceptance criteria, normative documents
and ADRs, the complete PR diff, and validation evidence. It independently
reviewed the exact repaired head without inheriting the Repair reasoning and
made no repository or remote changes.

### Closed: P2-5 exact original-unit accumulation

The fixed 67-limb signed binary accumulator retains every exact finite `f64`
significand product and rounds the completed signed sum once. It preserves the
finite representable `2^-104` near-cancellation residual, rejects the exact
nonzero `2^-1075` result as unrepresentable, and has sufficient exponent and
carry capacity for finite `f64` products and a `usize`-bounded dot product.
Both the null-space and expanded-weight diagnostic callers map the
unrepresentable result to their existing structured errors, with independent
private and public regressions.

### Complete-diff disposition

No P0, P1, P2, or P3 finding remains. The reviewer independently confirmed
the polynomial actions and derivative signs, D=1/D=2/D=3 complete polynomial
spaces, scale-aware equilibration and rank decisions, conservative ambiguity
handling, mapped null-space construction, matrix-infinity residuals, hard
failure semantics, projection without hidden regularization, provenance,
allocation shape, interface dispositions, diagnostics, benchmark routing,
and requirement evidence. Positive-definiteness classification and Hessian
capability remain explicitly outside this atomic requirement.

The reviewer ran:

```text
cargo test -p georbf --test cpd
cargo test -p georbf --lib cpd::tests
cargo fmt --all -- --check
cargo xtask requirements check
git diff --check
```

All 13 public CPD tests, all six private CPD regressions, formatting, and all
58 registry checks passed. Both exact P2-5 regressions also passed separately.
The reviewer verified that `06ad419..062bae3` changes only this review record
and the bounded handoff, so the complete standard gate on stable code/test
head `06ad419` remains applicable. Draft CI run 29394931421 passed its complete
Ubuntu correctness job on exact reviewed head `062bae3`.

PR #46 may advance to ready after this clean-review evidence is pushed. The
resulting exact ready head must pass the complete Windows, Ubuntu, and macOS
matrix with every benchmark smoke workload before merge.

## Integration evidence

- Clean-review evidence was pushed in documentation-only commit `bf69ed4`, and
  PR #46 was marked ready without changing the reviewed implementation.
- Ready-head CI run 29396342123 passed Windows, Ubuntu, and macOS on exact head
  `bf69ed42ec614612cedb32cbcb0bb3b4f771cfc4`, including every benchmark smoke
  workload.
- PR #46 squash-merged exactly once as
  `0c1937360b9467b05cb9e4e1d7d58f6cba9ff46f`; Issue #45 closed.
- Post-merge `main` CI run 29396715017 passed the complete three-platform
  correctness, benchmark-smoke, and requirement-registry gate on exact merge
  commit `0c19373`.

The isolated integration-state change records REQ-CPD-001 as `integrated` and
advances the bounded handoff without starting the next requirement. It changes
no production code, tests, manifests, schemas, build inputs, APIs, numerical
behavior, dependencies, tags, or releases.
