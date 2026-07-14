# PR #41 Independent Review

- Date: 2026-07-15
- Requirement: REQ-SPIKE-002
- Pull request: #41
- Reviewed implementation head: `9cd0c306cc328822df211fac854c34d15606960f`
- Base: `origin/main`
- Result: changes requested; two P2 findings and no P0, P1, or P3 finding

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

Repair code/test head `30bd4952105acc6a04a7dcaff72493692f29d051`
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

## Disposition

PR #41 remains Draft and REQ-SPIKE-002 remains `documented`. A fresh Repair
task repaired P2-1 and P2-2 at `30bd495`. The next task must be a fresh,
independent re-review of that repair and the complete PR diff. It must not
repair code or start REQ-CPD-001.
