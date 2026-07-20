# PR #88 Independent Review

- Requirement: REQ-NORMAL-001
- Issue: https://github.com/qingsonger/GeoRBF/issues/87
- Pull request: https://github.com/qingsonger/GeoRBF/pull/88
- Branch: `codex/req-normal-001-observations`
- Reviewed head: `8724f288b1415b95492e2195a2f72e2032d1b9b1`
- Base head: `0ae971439a900d3649051dd22bc5ba62d4830307`
- Review date: 2026-07-20
- Result: repairs implemented for R88-001, R88-002, and R88-003; fresh
  independent re-review required

## Scope and independence

A fresh read-only project `math_reviewer` received only the bounded
REQ-NORMAL-001 summary and integrated dependency closure, Issue #87 acceptance
criteria and exclusions, the M5 plan, relevant normal, problem-IR, solver,
architecture, and ADR contracts, the complete exact PR diff, tests, example,
benchmark, registry state, handoff, and validation evidence. It inherited no
implementation reasoning and made no repository or remote change.

The reviewer checked formulae, signs, dimensions, units, the orthogonal
complement, axial sign invariance, D=1 boundaries, the convex angular domain,
ordered Lorentz layout, hard and soft semantics, provenance, near-zero
diagnostics, representability limits, determinism, allocation failure, hidden
regularization, interfaces, benchmark claims, and registry truth.

## Findings

### P1 R88-001: a positive degree angle can silently become a zero-angle hard cone

`crates/georbf/src/normal_observations.rs:533-545` accepts every finite degree
value in `[0, 90)`, but `to_radians()` can underflow a positive value to zero.
Lines 324 and 341 then use that zero as the cone slope, silently compiling

```text
||T^T grad f|| <= 0
```

instead of the requested positive angular tolerance. This strengthens a hard
constraint and violates the requirement to reject unrepresentable inputs rather
than clip or fall back.

The reviewer reproduced the defect with `f64::from_bits(1)` degrees: the
compiled right-hand-side coefficient was `0.0`, while the same smallest
positive value in radians remained `5e-324`. In exact real arithmetic the
degree angle has positive tangent. With finite `n^T grad f = f64::MAX` and
`||T^T grad f|| = f64::from_bits(1)`, the requested positive cone is feasible
but the emitted zero-angle cone is not.

Required repair: detect loss of a positive angle or positive tangent during
unit conversion and return a structured representability error if the current
`f64` IR cannot encode it. Add an independent minimum-positive-degree
regression that must not return a successful cone with a zero RHS coefficient.

### P2 R88-002: soft L1 and Huber direction losses depend on the arbitrary complement basis

`crates/georbf/src/normal_observations.rs:389-412` copies one soft enforcement
onto each D=3 complement equality. For SquaredL2, the sum of squared components
is invariant under a change of orthonormal complement basis. AbsoluteL1 and
componentwise Huber are not, so `DirectionOnly`, `AxialDirection`, and
`DirectionWithPolarity` acquire a coordinate-dependent fitting objective even
when the physical normal error is unchanged. The existing soft test at
`crates/georbf/tests/normal_observations.rs:333-366` checks only objective
counts and metadata.

For `n = e_z`, a complement residual `(1, 1)` becomes `(sqrt(2), 0)` after a
45-degree rotation around the same normal. Both have geometric norm `sqrt(2)`,
but the scalar L1 sum changes from `2` to `sqrt(2)`. With Huber delta one, the
sum changes from `1` to approximately `0.914`. This is an internal basis effect,
not a different geological observation.

Required repair: preserve a rotation-invariant grouped complement loss, or
explicitly reject unsupported L1 and Huber enforcement for multi-row complement
semantics. Add D=3 rotation regressions comparing the full canonical soft
objective for the same physical case.

### P2 R88-003: AngularCone final allocations bypass the structured failure contract

`crates/georbf/src/normal_observations.rs:355-361` creates the final roles and
constraints with two infallible `vec!` allocations. Other constructor paths use
`try_reserve_exact` and return `NormalObservationError::AllocationFailed`, and
`changes/REQ-NORMAL-001.md:27` claims allocation failures are structured. An
allocator failure at either final allocation can terminate the process instead
of returning the declared error.

Required repair: build both final vectors through the existing fallible
reservation path. Add allocation-failpoint regressions that fail the roles and
constraints allocations separately after the provenance and basis allocations
succeed; each must return `AllocationFailed { requested: 2 }` without aborting.

No other P0, P1, P2, or P3 finding was reported.

## Repair evidence (not an independent re-review)

Repair implementation head `e94d19bf8baeb94901686f44499e7fdcf9e503c4`
addresses only the three findings above:

- R88-001: positive angular inputs now return the structured
  `AngularConeAngleNotRepresentable` error if degree conversion or tangent
  evaluation loses the positive cone slope. An independent regression rejects
  `f64::from_bits(1)` degrees and proves that the same smallest positive value
  in radians retains a nonzero canonical right-hand-side coefficient.
- R88-002: D=3 complement-based `DirectionOnly`, `AxialDirection`, and
  `DirectionWithPolarity` now reject componentwise AbsoluteL1 and Huber losses
  with `UnsupportedComplementSoftLoss`. SquaredL2 remains supported, and a
  regression evaluates the complete canonical objective before and after a
  45-degree rotation about the same normal. D=2's single complement row remains
  available for L1, while scalar cone and bound losses are unchanged.
- R88-003: final AngularCone role and constraint vectors now use separately
  fallible exact reservations. Storage-targeted allocation failpoints force
  each final reservation independently after provenance, basis, and cone-row
  allocation and receive `AllocationFailed { requested: 2 }` without partial
  success.

Ten focused integration tests and two allocation-failpoint unit tests pass.
After the final code change, the exact implementation tree passed
`cargo fmt --all -- --check`, workspace/all-target/all-feature Clippy with
warnings denied, workspace/all-feature tests, workspace doctests, and
`cargo xtask requirements check` (58 requirements). The requirement remains
`implemented`, PR #88 remains Draft, and none of these repair assertions is a
substitute for the required fresh independent re-review.

## Independent mathematical review

The gradient equalities, oriented lower-bound sign, ordered Lorentz layout,
D=1 availability and rejection boundary, finite nonnegative minimum, absence
of an invented axial magnitude, and deterministic provenance order are
otherwise consistent with the scoped contracts. The reviewed complement has
the expected orthonormal structure and cross-product sign, and the tested
non-axis-aligned axial reversal produces binary-identical canonical rows.

Near-zero gradient review remains diagnostic-only and its scaled norm avoids
intermediate square overflow and underflow. The PR introduces no hidden
regularization, jitter, pseudoinverse, hard-to-soft conversion, constraint
deletion, solver geological term, or unconditional Hessian claim. Rust and the
deferred CLI/C/C++/Python dispositions remain truthful, and the requirement is
correctly `implemented` rather than `integrated`.

## Validation and disposition

- Local and remote branch heads matched reviewed head `8724f28`; the worktree
  was clean before this evidence-only Review change.
- Draft CI run 29723009629 passed the complete configured Ubuntu correctness
  job on exact reviewed head `8724f28`. The Ready-only Windows, Ubuntu, macOS,
  and benchmark-smoke matrix correctly did not run.
- The reviewer and parent task each passed all eight focused normal-observation
  tests. Those tests do not cover R88-001, R88-002, or R88-003.
- The parent task reproduced the example result of one cone and projection
  lower bound `0.05`; benchmark smoke passed with checksum `11088`. Its local
  timing is not compared across build states or machines.
- Exact implementation head `8724f28` retains the recorded complete standard
  gate and now also has green Draft CI. This Review task changes only this
  review record and the bounded handoff; no production, test, manifest, schema,
  CI, build input, API, or numerical behavior changes.

PR #88 must remain Draft and REQ-NORMAL-001 must remain `implemented`. Open a
fresh Review/re-review task for exact repair implementation head `e94d19b` and
the final evidence-only head. The independent reviewer must verify that
R88-001, R88-002, and R88-003 are closed and check for new P0-P3 findings. Do
not begin REQ-TANGENT-001 or any other requirement.
