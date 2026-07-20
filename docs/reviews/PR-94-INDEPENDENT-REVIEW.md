# PR #94 Independent Review

- Requirement: REQ-THICK-001
- Issue: https://github.com/qingsonger/GeoRBF/issues/93
- Pull request: https://github.com/qingsonger/GeoRBF/pull/94
- Branch: `codex/req-thick-001-local-thickness`
- Reviewed head: `0821084b36d9602c2b34cc9bedd3cf20380a335d`
- Base head: `1d770395f1022c81d7ad314c4d38221a5c1a66c4`
- Review date: 2026-07-20
- Result: one P2 and two P3 findings; Repair required

## Scope and independence

A fresh read-only project `math_reviewer` received only the bounded
REQ-THICK-001 summary and integrated dependency closure, Issue #93 acceptance
criteria and exclusions, the M5 plan, relevant thickness, problem-IR, solver,
architecture, and ADR contracts, the exact PR diff, tests, example, benchmark,
registry state, handoff, and validation evidence. It inherited no implementation
reasoning and made no repository or remote change.

The reviewer independently checked the first-order thickness derivation,
ordered-cone signs, dimensions and units, D=1/D=2/D=3 bounds, explicit level
indices, hard enforcement, Cartesian order, rotation invariance, provenance,
duplicate identifiers, field-variable boundaries, representability,
allocation behavior, diagnostics, interface dispositions, benchmark and CI
wiring, and registry truth.

## Findings

### P2 THICK-REV-001: constraint collection misuses the `size_hint` upper bound and permits unchecked growth allocation

`crates/georbf/src/thickness.rs:350-359` selects
`maximum.unwrap_or(minimum)` from `Iterator::size_hint()` and reserves the
entire value before reading an item. An iterator upper bound is permitted to be
loose. A valid one-element iterator reporting `(0, Some(usize::MAX))` is
therefore rejected with `AllocationFailed` even though collecting its actual
item requires trivial storage.

For a valid unknown-length iterator reporting `(0, None)`, the same collector
reserves zero and then calls `Vec::push` without checking capacity. A growth
allocation can consequently bypass the structured `AllocationFailed` contract
documented at `crates/georbf/src/thickness.rs:188-194`. This differs from the
existing fallible collection pattern in the level and problem-IR layers and
can terminate instead of returning an error.

Required regression tests:

- collect a one-element iterator reporting `(0, Some(usize::MAX))` and require
  successful construction of one cone;
- use a module allocation failpoint with a two-element `(0, None)` iterator to
  fail the growth reservation, require
  `ThicknessCanonicalizationError::AllocationFailed`, and prove no partial
  canonical problem is returned.

### P3 THICK-REV-002: tests do not prove the sample point and complete provenance cross the compilation boundary

Production code appears to put `constraint.point` into every Cartesian
derivative atom at `crates/georbf/src/thickness.rs:363-384` and to clone the
complete semantic provenance into the cone at
`crates/georbf/src/thickness.rs:329-336`. The test linearizer at
`crates/georbf/tests/thickness.rs:131-151` ignores the point and supplied
provenance, however, and the cone assertions at lines 175-192 check only the
observation identifier. A regression that drops the explicit point, source
path, line, original units, field path, or constraint group would therefore
retain all eight current focused test successes.

Required regression: use a nontrivial D=3 point and provenance with distinct
fields, assert every axis callback receives the exact point and complete
borrowed provenance, then assert the final cone owns a complete equal
provenance value.

### P3 THICK-REV-003: constant scaling overflow and underflow lack regression evidence

The normative contract at `docs/math/THICKNESS.md:50-53` rejects unrepresentable
coefficient or constant products. The branch at
`crates/georbf/src/thickness.rs:417-423` appears to implement the constant
case, but `crates/georbf/tests/thickness.rs:400-431` exercises only nonzero
coefficient overflow and underflow. The
`ScaledGradientConstantNotRepresentable` path has no independent evidence.

Required regression: return pure affine constants `f64::MAX` scaled by two and
`f64::MIN_POSITIVE` scaled by the minimum positive subnormal, require
`ScaledGradientConstantNotRepresentable` for both, and retain acceptance of an
exact zero constant.

No other P0, P1, P2, or P3 finding was reported.

## Independent mathematical review

For the local first-order model, the maximum scalar change per unit distance
along the gradient normal is `||grad f||_2`. Crossing a level gap `delta_h`
therefore takes first-order normal distance `delta_h / ||grad f||_2`, so a
minimum distance `T_min` is equivalent to

```text
T_min ||grad f(x)||_2 <= h_upper - h_lower.
```

The implementation multiplies every Cartesian derivative row by the same
positive `T_min`, places those rows on the Lorentz left side, and uses `+1` for
the upper level and `-1` for the lower level on the right. The signs,
dimensions, units, explicit level-variable indices, and D=1/D=2/D=3 shape are
correct. Since `||Q grad f||_2 = ||grad f||_2` for orthogonal `Q`, the complete
Cartesian norm is rotation invariant.

The constraint remains hard and introduces no softening, hidden
regularization, automatic scaling, relaxation, solver vocabulary, CPD/SPD
claim, polynomial/rank behavior, or Hessian promise. Finite points, positive
finite thickness, unknown levels, duplicate identifiers, field-prefix
validation, diagnostic separation, immutability, `Send + Sync`, and the
deferred CLI/C/C++/Python dispositions are otherwise consistent with the
scoped contracts.

## Validation and disposition

- Local and remote branch heads matched reviewed head `0821084b`; the worktree
  was clean before this evidence-only Review change.
- Draft CI run 29739383159 passed its configured Ubuntu correctness job on the
  exact reviewed head. The Ready-only Windows, Ubuntu, macOS, and benchmark
  matrix was skipped and is not claimed as passed.
- The independent reviewer passed all eight focused thickness tests, the
  thickness Rustdoc compile-fail check, the runnable example, benchmark smoke
  checksum `8304`, all 58 requirement checks, and `git diff --check`.
- The parent task independently passed the same eight focused tests, runnable
  example, benchmark smoke checksum `8304`, all 58 requirement checks, and the
  complete PR diff whitespace check.
- After adding the evidence files, the parent task passed the complete standard
  gate: workspace format, warning-denying all-target/all-feature Clippy,
  all-feature workspace tests, workspace Rustdoc, all 58 requirement checks,
  and `git diff --check`.
- Exact implementation head `f91ca4a` retains its recorded complete standard
  local gate. The independent reviewer did not rerun the complete workspace
  gate and no unexecuted check is claimed as passed.
- This final evidence wording changes only this review record and the bounded
  handoff; it changes no production, test, manifest, schema, CI, build input,
  API, numerical behavior, registry, or dependency input.

PR #94 must remain Draft and REQ-THICK-001 must remain `implemented`. Open a
fresh Repair task scoped only to THICK-REV-001, THICK-REV-002, and
THICK-REV-003. Add the independent regressions, implement the smallest
collector repair, rerun focused and final standard checks after the last code
change, update this review evidence and the bounded handoff, commit, push, and
stop for a fresh independent re-review. Do not begin REQ-THICK-002 or another
requirement.
