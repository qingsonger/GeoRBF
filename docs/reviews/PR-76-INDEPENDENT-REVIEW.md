# PR #76 Independent Review

- Requirement: REQ-LINEQ-001
- Issue: https://github.com/qingsonger/GeoRBF/issues/75
- Pull request: https://github.com/qingsonger/GeoRBF/pull/76
- Branch: `codex/req-lineq-001-linear-bounds`
- Reviewed head: `0da5084c3b4f7f909299069c3c8dcf3145d1f282`
- Original implementation and complete local-gate head:
  `8931260b6d37aa87bd82fa9416bd97d119c6d134`
- Re-reviewed repair and complete local-gate head:
  `b1f15d547333e17b8c8462014046a7b93e5ece00`
- Pre-re-review evidence head: `8eb5fe1031c48f7c9c824c1ebb01b3ff2b7274af`
- Base head: `639289fad1b03f84efd0b7a590516cbca74d5429`
- Review date: 2026-07-19; fresh re-review: 2026-07-19
- Result: clean fresh re-review; R76-001 is closed and no P0-P3 finding remains

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

## Repair evidence (not an independent re-review)

The Repair task first added
`field_level_composition_rejects_duplicate_observation_ids`, which reused level
definition observation ID 100 in a separately valid field hard bound. The test
failed against the reviewed implementation because composition returned a
canonical problem instead of `ProblemIrError::DuplicateObservationId`.

Exact repaired implementation head
`b1f15d547333e17b8c8462014046a7b93e5ece00` enumerates provenance identifiers
from hard equalities, hard bounds, hard cones, and soft objectives in both
canonical inputs and rejects the first cross-input duplicate before appending
any row or objective. It does not modify, drop, reorder, soften, scale, or
regularize a relation.

On that exact clean head, all eight linear-constraint tests and all 21 level
tests passed. The complete standard gate also passed: formatting, warning-
denying workspace/all-target/all-feature Clippy, all-feature workspace tests,
workspace Rustdoc, all 58 requirement checks, and `git diff --check`. The
subsequent handoff commit changes only this review record and
`docs/progress/CURRENT.md`; no production, test, manifest, schema, CI, build,
registry, API, numerical, or dependency input changed after the gate.

This repair evidence is not an independent re-review. PR #76 remains Draft;
Ready CI, merge, and integration-state recording have not occurred.

## Fresh independent re-review

A new read-only project `math_reviewer` independently re-reviewed the complete
implementation through exact repair head
`b1f15d547333e17b8c8462014046a7b93e5ece00`. It received only the bounded
requirement summary and integrated dependency closure, Issue #75 criteria and
exclusions, the M4 plan, scoped normative documents, the complete implementation
and repair diffs, R76-001, and recorded validation evidence. It did not inherit
the implementation or Repair task reasoning and made no repository or remote
change. It also verified that `b1f15d5..8eb5fe1` changes only this review record
and the bounded handoff.

R76-001 is closed, and no P0, P1, P2, or P3 finding remains. Composition scans
stable observation identifiers across hard equalities, hard bounds, hard cones,
and soft objectives in both canonical inputs. It rejects the first cross-input
duplicate before deconstruction, reservation, or append. The regression reuses
level-definition identifier 100 in an otherwise valid field hard bound and
requires `ProblemIrError::DuplicateObservationId`.

The reviewer independently reconfirmed lower, upper, closed-interval, all four
inside/outside orientation, scalar-gap, layer-order, and increasing/decreasing
monotonicity signs; directional-unit enforcement; affine-constant shifting;
constant-row feasibility; exact equal and sign-reversed interval intersection;
hard/soft separation; deterministic provenance and ordering; identity scaling;
allocation and checked-count behavior; and the absence of a solver, hidden
regularization, jitter, pseudoinverse, relation deletion, or hard-to-soft
conversion. The benchmark's 6,400-byte estimate independently recomputes from
112 sparse terms and 576 scalar slots. SPD/CPD, center limits, polynomial-rank,
rotation-invariance, positive-definiteness, and Hessian review remain
inapplicable because the PR changes no kernel, basis, assembly, solver,
anisotropy, or fitted-model mathematics.

The reviewer passed all eight linear-constraint tests, all 21 level tests, four
canonical provenance-allocation tests, all 30 georbf doctests, the example, the
benchmark smoke with checksum 12,800, formatting, all 58 requirement checks,
and `git diff --check` over the complete PR. The complete standard gate was not
rerun because it already passed on immutable repair head `b1f15d5`, and this
re-review changes only review and handoff documentation.

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

## Integration evidence

The implementation integration sequence is complete. Exact Ready head
`1541eb761ce7acf7dec8d4445875f499a6868804` passed the complete Windows,
Ubuntu, and macOS correctness matrix, both configured backend suites, every
benchmark smoke, and the requirement-registry gate in CI run 29671462544. PR
#76 then squash-merged exactly once as
`42768a80cadd261d9d45e35a920e8ac7cc929558`, and Issue #75 closed as
completed. Post-merge `main` run 29671754311 passed the same complete
three-platform gate on that exact merge commit.

The isolated integration-state change records the registry, review evidence,
history index, and bounded handoff only. After its own complete local and exact
Ready-head CI gates are green and its pull request is merged, stop. A fresh task
must select the next requirement; this task must not begin it.
