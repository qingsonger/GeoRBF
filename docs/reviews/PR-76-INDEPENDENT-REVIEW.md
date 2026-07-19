# PR #76 Independent Review

- Requirement: REQ-LINEQ-001
- Issue: https://github.com/qingsonger/GeoRBF/issues/75
- Pull request: https://github.com/qingsonger/GeoRBF/pull/76
- Branch: `codex/req-lineq-001-linear-bounds`
- Reviewed head: `0da5084c3b4f7f909299069c3c8dcf3145d1f282`
- Exact implementation and complete local-gate head:
  `8931260b6d37aa87bd82fa9416bd97d119c6d134`
- Base head: `639289fad1b03f84efd0b7a590516cbca74d5429`
- Review date: 2026-07-19
- Result: repair required for R76-001; no other P0-P3 finding remains

## Scope and independence

A fresh read-only project `math_reviewer` received only the bounded
REQ-LINEQ-001 summary and integrated dependency closure, Issue #75 acceptance
criteria and exclusions, the M4 plan, relevant mathematical, architecture, and
level-variable contracts, the complete PR diff, tests, example, benchmark,
registry evidence, and exact-head validation state. It did not inherit the
implementation task's reasoning and made no repository or remote changes.

The reviewer independently checked signs and units, the closed inside/outside
boundary convention, scalar-gap and layer-order orientation, monotonicity
direction and functional shape, exact equal/sign-reversed row comparison,
constant and pairwise hard infeasibility evidence, hard/soft separation,
D=1/D=2/D=3 restrictions, allocation and count paths, identity scaling,
capability and memory metadata, immutable/thread-safe values, interface
exclusions, benchmark scope, hidden solver policy, and registry/documentation
truth. SPD/CPD classification, center limits, polynomial rank, positive
definiteness, Hessians, and kernel rotation behavior are not applicable because
this PR changes no kernel, basis, solver, or fitted-model mathematics.

## Findings

There are no P0, P1, or P3 findings. The following P2 finding is actionable.

### P2 R76-001: field/level composition permits duplicate stable observation identifiers

`SemanticProblemIr::try_new` rejects duplicate observation identifiers within
one field problem at `crates/georbf/src/problem_ir.rs:459-476`, and
`LevelProblem` checks definitions, memberships, and order records together at
`crates/georbf/src/levels.rs:1080-1113`. However,
`CanonicalProblem::try_append_field_linear_problem` checks only compatible
variable blocks and allowed relation families before appending the field rows
and objectives at `crates/georbf/src/problem_ir.rs:1230-1291`.

Consequently, two independently valid inputs can reuse one `ObservationId`
across a level definition, membership, order, or prior and a field bound. The
composition then returns one canonical problem with ambiguous stable source
identities. The current composition test deliberately uses disjoint field IDs
105 and 106 after level IDs 100 through 104 at
`crates/georbf/tests/linear_constraints.rs:511-544`, so it does not exercise
the cross-problem invariant. This violates Issue #75's requirement to reject
duplicate stable identifiers.

Repair must add a regression that gives a field bound the same observation ID
as a level record, composes the otherwise compatible problems, and requires a
structured duplicate-identifier error rather than a canonical problem
containing both records. The smallest production repair must recheck complete
provenance identity across the combined hard and soft records without changing,
dropping, or rewriting any relation.

## Independent truth summary

- Lower, upper, and closed interval forms preserve their stated signs.
- Inside/outside uses the caller-selected orientation and weak inequalities, so
  the boundary belongs to both closed sides without a hidden epsilon.
- Scalar and level gaps consistently use `upper - lower >= minimum_gap`.
- Increasing monotonicity is `u^T grad f >= minimum_rate`; decreasing is
  `u^T grad f <= -minimum_rate`, and only one coefficient-one directional
  derivative is accepted.
- Exact sign reversal maps `[lower, upper]` to `[-upper, -lower]`; constant
  rows reject precisely the intervals that exclude zero.
- Soft bounds remain objectives and do not participate in hard feasibility
  review. No backend, jitter, regularization, pseudoinverse, relaxation,
  constraint deletion, or hard-to-soft conversion was introduced.
- The benchmark's 6,400-byte estimate independently recomputes from 112 sparse
  terms at 16 bytes plus 576 scalar slots at 8 bytes.

Apart from R76-001, the reviewed implementation, tests, benchmark, interfaces,
registry, and documentation were consistent with the bounded requirement.

## Validation evidence reviewed

- The exact implementation head passed the recorded complete standard local
  gate: formatting, warning-denying all-target/all-feature Clippy, all-feature
  workspace tests, workspace Rustdoc, all 58 requirement checks, and
  `git diff --check`.
- Exact PR head `0da5084` passed Draft Ubuntu correctness CI in run 29667744094.
- The reviewer independently repeated all seven focused tests, the complete
  Rustdoc suite, the example, benchmark smoke, all 58 requirement checks, and
  `git diff --check`; all passed.
- The parent Review task independently passed all seven linear-constraint tests,
  all 21 level tests, the example, benchmark smoke, all georbf Rustdoc, all 58
  requirement checks, and `git diff --check`.
- `8931260..0da5084` changes only `docs/progress/CURRENT.md`; no production,
  test, manifest, schema, CI, build, registry, API, numerical, or dependency
  input changed after the complete local gate.
- Ready-only Windows, Ubuntu, macOS, and benchmark-smoke CI has not run.

## Required next action

Open a fresh Repair task limited to R76-001. Add the independent cross-problem
duplicate-ID regression, implement the smallest complete structured rejection,
run focused checks and then the complete standard workspace gate once on the
stable repaired head, update this review evidence and the bounded handoff,
commit, push, and stop for a fresh independent re-review. Do not mark PR #76
Ready, merge it, or begin another requirement in that Repair task.
