# PR #73 Independent Review

- Requirement: REQ-SOFT-001
- Issue: https://github.com/qingsonger/GeoRBF/issues/72
- Pull request: https://github.com/qingsonger/GeoRBF/pull/73
- Branch: `codex/req-soft-001-per-constraint-soft-losses`
- Reviewed head: `978e400b2f9b25b9f84ac3102ff40388c44b42d8`
- Base head: `d7cec28fcd70d6f9f3d6a596d339695d73af6706`
- Review date: 2026-07-18
- Result: one P2 finding; Repair required

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

Open a fresh Repair task limited to R73-001. Add the independent soft-only
relation-capability regressions, apply the smallest complete capability-metadata
repair, rerun focused checks and the final standard gate on the stable repaired
head, update this review record and the bounded handoff, commit, push, and stop
for a fresh independent re-review. Do not begin another requirement.
