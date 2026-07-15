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

## Disposition

PR #46 remains Draft. The Repair evidence above does not independently close
P1-1 or P2-1. A fresh read-only independent re-review must examine the repaired
head without inheriting the Repair reasoning, confirm whether both findings
are closed, and check for new P0-P3 findings. Do not mark the PR ready, merge,
or begin another requirement before that re-review is clean.
