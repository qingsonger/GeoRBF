# PR #73 Independent Review

- Requirement: REQ-SOFT-001
- Issue: https://github.com/qingsonger/GeoRBF/issues/72
- Pull request: https://github.com/qingsonger/GeoRBF/pull/73
- Branch: `codex/req-soft-001-per-constraint-soft-losses`
- Reviewed head: `978e400b2f9b25b9f84ac3102ff40388c44b42d8`
- Repair head: `530f6fd817dabcae70a304e3db2430211692615f`
- Re-reviewed repair head: `530f6fd817dabcae70a304e3db2430211692615f`
- Pre-re-review evidence head: `6687402e7ab42508637460ddbe3d7a156a45cac6`
- Base head: `d7cec28fcd70d6f9f3d6a596d339695d73af6706`
- Review date: 2026-07-18; fresh re-review: 2026-07-19
- Result: clean fresh re-review; R73-001 is closed and no P0-P3 finding remains

## Scope and independence

A fresh read-only project `math_reviewer` received only the bounded
REQ-SOFT-001 summary and integrated dependency closure, Issue #72 acceptance
criteria, the M4 plan, the relevant constraint-semantics, ADR, architecture,
and change contracts, the complete PR diff, tests, benchmark, registry
evidence, and exact-head validation state. It did not inherit the implementation
task's reasoning and made no repository or remote changes.

The reviewer independently checked loss formulae, signs, dimensions, scale
units and positive scalar-unit invariance; equality, interval, and cone
violation definitions; hard-family preservation; level-prior composition;
provenance and deterministic ordering; memory estimates and allocation paths;
capability metadata; public API and error behavior; interface dispositions;
benchmark scope; hidden optimization or regularization; and registry truth.
SPD/CPD, polynomial-rank, center-limit, and Hessian checks are not applicable
to this solver-neutral canonical-objective change.

## Findings

There are no P0, P1, or P3 findings. The following P2 finding is actionable.

### P2 R73-001: capability metadata omits soft relation geometry

`CanonicalCapabilities` is documented as the canonical constraint families
present at `crates/georbf/src/problem_ir.rs:1023-1034`, and
`CanonicalProblem::capabilities` calls it the required solver capabilities at
`crates/georbf/src/problem_ir.rs:1372-1374`. Construction at
`crates/georbf/src/problem_ir.rs:1244-1249`, however, derives the equality,
linear-bound, and second-order-cone flags only from the hard-family vectors.
Soft objectives contribute only their L2, L1, or Huber loss family.

Consequently, a problem containing only a soft second-order-cone relation
reports `has_second_order_cones == false`, even though exact lowering of its
cone-violation loss still requires second-order-cone geometry. The metadata
therefore under-reports the solver capability required by a valid canonical
problem and does not meet Issue #72's inspectable capability criterion.

Repair must make the public capability contract explicitly and consistently
include soft relation geometry, either through the existing family flags or a
separate soft-relation capability set. A regression must compile a soft-only
cone and require second-order-cone support. Soft-only equality and linear-bound
regressions must also define and lock the chosen public semantics for those
family flags.

## Independent truth summary

- For `t = v / s`, the SquaredL2, AbsoluteL1, and Huber definitions are
  dimensionless and the Huber branches meet continuously at `|t| = delta`.
- Equality residual signs are immaterial for the even losses. Bound violation
  is zero on the closed feasible interval and cone violation is
  `max(0, ||lhs||_2 - rhs)`.
- Scaling a relation and its positive residual scale by the same scalar leaves
  every objective value unchanged.
- Hard equality, bound, and cone collections remain separate and unchanged
  while soft objectives preserve their canonical relation and provenance.
- Level priors lower to the shared soft-equality form without becoming hard
  rows or gaining a false dense-solver support claim.
- Checked counts cover soft coefficients and numeric metadata, and fallible
  allocation paths return structured errors without a placeholder success.
- No backend, jitter, pseudoinverse, regularization, hard-to-soft conversion,
  or geological optimizer vocabulary was introduced.

Apart from R73-001, the reviewed implementation and independent tests were
consistent with the bounded requirement and normative contracts.

## Repair response

Exact repair head `530f6fd817dabcae70a304e3db2430211692615f`
addresses only R73-001. A new regression first reproduced the defect: an
isolated soft equality left `has_equalities` false before the repair. The same
test independently compiles soft-only equality, linear-bound, and
second-order-cone problems, verifies their hard-family collections remain
empty, and requires exactly the corresponding public relation-geometry flag.

Capability construction now starts from the hard-family collections and makes
one pass over soft objectives to add the geometry retained by each soft
relation. The public Rustdoc and REQ-SOFT-001 change fragment explicitly define
the equality, linear-bound, and cone flags as required geometry across both
hard constraints and soft objectives. Loss-family flags remain unchanged. The
repair introduces no backend, optimizer, dependency, hidden regularization,
hard-to-soft conversion, interface expansion, or unrelated requirement work.

Focused repair validation passed all 6 soft-loss tests, 11 problem-IR tests,
21 level tests, all 29 georbf Rustdoc tests, and the D=1/D=2/D=3 96-constraint
soft-objective compilation benchmark smoke. The complete standard workspace
gate then passed on the stable repair head: formatting, warning-denying
all-target/all-feature Clippy, all-feature workspace tests, workspace Rustdoc,
all 58 requirement checks, and `git diff --check`.

## Fresh independent re-review

A new read-only project `math_reviewer` independently re-reviewed the complete
implementation through exact repair head
`530f6fd817dabcae70a304e3db2430211692615f`. It received only the bounded
requirement summary and integrated dependency closure, Issue #72 criteria and
exclusions, the M4 plan, scoped normative documents, complete implementation
and repair diffs, R73-001, and recorded validation evidence. It did not inherit
the implementation or Repair task reasoning and made no repository or remote
change. It also verified that the tail from the repair head to pre-re-review
evidence head `6687402e7ab42508637460ddbe3d7a156a45cac6` changes only this review
record and the bounded handoff.

R73-001 is closed, and no P0, P1, P2, or P3 finding remains. Capability
construction now reports the set union of relation geometry used by hard
constraints and soft objectives, while the soft-loss capability set remains
independently derived from objective losses. The public Rustdoc and change
fragment state the same contract. The isolated regression compiles soft-only
equality, linear-bound, and second-order-cone inputs, proves the corresponding
hard collections remain empty, and requires exactly the matching geometry
flag. Existing mixed-family tests continue to prevent soft relations from
entering hard collections.

The reviewer independently reconfirmed the dimensionless `rho(v / s)` contract,
Huber continuity, equality-sign invariance, closed-interval and cone violation
formulae, positive joint scalar-unit invariance, hard-family preservation,
level-prior lowering, provenance and ordering, scaling and checked-memory
metadata, allocation errors, and the absence of an optimizer, hidden
regularization, jitter, pseudoinverse, or hard-to-soft conversion. SPD/CPD,
polynomial-rank, center-limit, positive-definiteness, and Hessian checks remain
inapplicable to this solver-neutral repair.

The reviewer ran the focused repaired regression, compact requirement show and
dependency checks, and `git diff --check` over the implementation, repair, and
evidence-only diffs; all passed. The parent Review task independently passed
all 6 soft-loss tests, 11 problem-IR tests, 21 level tests, all 29 georbf
Rustdoc tests, and the D=1/D=2/D=3 96-constraint benchmark smoke. The complete
standard gate was not rerun because it already passed on immutable repair head
`530f6fd`, and this re-review changes only review and handoff documentation.

## Validation evidence reviewed

- Focused soft-loss, problem-IR, and level tests passed.
- The 96-constraint mixed-objective benchmark smoke passed for D=1, D=2, and
  D=3.
- The complete local standard gate passed on the immutable reviewed head:
  format, warning-denying all-target/all-feature Clippy, all-feature workspace
  tests, workspace Rustdoc, all 58 requirement checks, and `git diff --check`.
- Draft Ubuntu correctness CI passed on the exact reviewed head. The Ready-only
  Windows, Ubuntu, macOS, and benchmark-smoke matrix remains intentionally
  unexecuted.

## Required next action

Open a fresh integration Review task limited to PR #73. Confirm that the new PR
head differs from cleanly re-reviewed repair head `530f6fd` only by review and
handoff evidence, synchronize the PR evidence, and mark it Ready. Wait for the
complete Windows, Ubuntu, and macOS correctness matrix plus every benchmark
smoke workload on that exact Ready head. Merge exactly once only if all required
CI is green, then record truthful REQ-SOFT-001 integration state in a separate
integration-state change and bounded handoff. Do not begin another requirement
in that integration task.
