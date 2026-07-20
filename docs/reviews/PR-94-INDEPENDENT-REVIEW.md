# PR #94 Independent Review

- Requirement: REQ-THICK-001
- Issue: https://github.com/qingsonger/GeoRBF/issues/93
- Pull request: https://github.com/qingsonger/GeoRBF/pull/94
- Branch: `codex/req-thick-001-local-thickness`
- Reviewed head: `0821084b36d9602c2b34cc9bedd3cf20380a335d`
- Repair implementation head: `551d93f05a2f2023fc5bca5454176e111a88ed69`
- Base head: `1d770395f1022c81d7ad314c4d38221a5c1a66c4`
- Review date: 2026-07-20
- Result: clean independent re-review; Ready CI pending

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

## Repair evidence (not an independent re-review)

Repair implementation head `551d93f05a2f2023fc5bca5454176e111a88ed69`
addresses only THICK-REV-001, THICK-REV-002, and THICK-REV-003.

- THICK-REV-001: constraint collection now reserves only the iterator's
  guaranteed lower bound. If actual input outgrows capacity, each growth is
  preceded by `try_reserve`, so allocation failure returns the documented
  structured error. A one-element iterator with upper bound `usize::MAX` now
  succeeds. A module failpoint forces growth failure for a two-element
  unknown-length iterator, requires `AllocationFailed { requested: 1 }`, and
  proves the linearizer is never reached and no partial problem is returned.
- THICK-REV-002: a D=3 regression supplies point `(1.25, -2.5, 3.75)` and
  distinct values for every `SemanticProvenance` field. Every Cartesian-axis
  callback must receive the exact point and complete borrowed provenance in
  order, and the final cone must own a complete equal provenance value.
- THICK-REV-003: the scaling regression now separately requires structured
  constant overflow and underflow errors for nonzero affine constants and
  retains acceptance of an exact zero constant.

The repair changes no thickness formula, ordered-cone sign, dimension or unit
contract, hard enforcement, public API, interface disposition, dependency,
registry state, or later-requirement scope. A fresh project `math_reviewer`
must still independently confirm that all three findings are closed and that
no new P0--P3 finding was introduced.

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
- Exact repair implementation head `551d93f` passed all ten thickness
  integration tests, the module allocation-failure regression, the thickness
  Rustdoc compile-fail check, the runnable example, benchmark smoke checksum
  `8304`, all 58 requirement checks, `git diff --check`, and the complete
  standard local gate: workspace format, warning-denying all-target/all-feature
  Clippy, all-feature workspace tests, and workspace Rustdoc.

PR #94 must remain Draft and REQ-THICK-001 must remain `implemented`. Open a
fresh independent re-review task for the exact final PR head and
THICK-REV-001 through THICK-REV-003. If no P0--P3 finding remains, follow the
repository's Ready-CI integration sequence. Do not begin REQ-THICK-002 or any
other requirement.

## Final independent re-review

A fresh read-only project `math_reviewer` independently reviewed exact PR head
`522a2098d9b4390d568a784300c863cd3e963c43` against base
`1d770395f1022c81d7ad314c4d38221a5c1a66c4`. It received only the bounded
requirement and dependency summaries, Issue #93 criteria and exclusions, M5
scope, relevant mathematical, architecture, problem-IR, solver, and ADR
contracts, the complete exact PR and focused repair diffs, prior findings and
repair evidence, tests, example, benchmark, registry, handoff, CI workflow,
and validation evidence. It inherited no Implement or Repair reasoning and
made no repository or remote change.

- THICK-REV-001 is closed. Constraint collection trusts only the iterator's
  guaranteed lower bound and performs fallible growth before each capacity-
  exceeding push. The loose `usize::MAX` upper-bound regression succeeds, and
  the unknown-length allocation failpoint returns structured
  `AllocationFailed` before linearization or a partial result.
- THICK-REV-002 is closed. Every Cartesian derivative atom retains the exact
  sample point; complete borrowed provenance reaches each ordered callback and
  complete owned provenance reaches the final canonical cone. The D=3
  regression checks all three axes, the nontrivial point, and every provenance
  field.
- THICK-REV-003 is closed. Affine scaling separately rejects coefficient and
  nonzero constant overflow or underflow while retaining exact zero constants.
  The regression exercises every one of those paths.
- The local first-order derivation remains
  `T_min ||grad f(x)||_2 <= h_upper - h_lower`. Every Cartesian derivative row
  receives the same positive scale; the ordered cone uses `+1` on the upper
  level and `-1` on the lower level. Both sides have field units, and the full
  Cartesian norm is invariant under orthogonal coordinate changes.
- Public semantics remain restricted to D=1, D=2, and D=3. Derivative rows are
  validated against the field-variable prefix before level indices are used,
  and cones enter only the hard canonical collection. Unknown endpoints,
  duplicate identifiers, allocation failure, and unrepresentable products are
  structured failures without observable partial success.
- Scalar-gap and sampled-local diagnostics, the deferred M8/M9 adapter
  dispositions, benchmark and Ready-CI wiring, and the `implemented` registry
  state remain truthful. No SPD/CPD or rank decision, center selection, hidden
  regularization, relaxation, Hessian capability, geometric validation, or
  later-requirement behavior was introduced.

No P0, P1, P2, or P3 finding remains. The parent task passed the five standard
checks, `git diff --check`, all ten focused thickness integration tests, the
module allocation-failure regression, the runnable example, and benchmark
smoke checksum `8304` on the same exact head. Draft CI run 29745163151 also
passed its configured Ubuntu correctness gate on `522a209`; the Ready-only
matrix correctly did not run.

This evidence-only change updates only this review record and
`docs/progress/CURRENT.md`; it changes no production, test, manifest, schema,
CI, build, API, numerical, registry, or dependency input. PR #94 may proceed to
Ready CI. REQ-THICK-001 remains `implemented`, not `integrated`, until the
exact Ready evidence head passes the complete Windows, Ubuntu, and macOS
correctness and benchmark-smoke matrix, PR #94 merges exactly once, and the
isolated integration-state change completes.

## Integration evidence

The implementation integration sequence is complete. Exact Ready evidence head
`e1ac47aadb1b8ad91a4dbb3fafbeec486cc65eee` passed the complete Windows,
Ubuntu, and macOS correctness matrix, every configured backend path, all
benchmark-smoke workloads including `local_thickness_compilation`, and the
requirement-registry gate in CI run 29750190504. PR #94 then squash-merged
exactly once as `59a42d93361f55ebe4a2bbf9a72406d4600f513a`, and Issue #93
closed as completed. Post-merge `main` run 29752225970 passed the same complete
three-platform gate on that exact merge commit.

The isolated integration-state change records only the registry, review
evidence, history index, and bounded handoff in pull request #95. After its own
complete local and exact Ready-head CI gates are green and that pull request is
merged, stop. A fresh task must select the next requirement; this task must not
begin it.
