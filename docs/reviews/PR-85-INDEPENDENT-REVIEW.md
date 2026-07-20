# PR #85 Independent Review

- Requirement: REQ-INFEAS-001
- Issue: https://github.com/qingsonger/GeoRBF/issues/84
- Pull request: https://github.com/qingsonger/GeoRBF/pull/85
- Branch: `codex/req-infeas-001-diagnostics`
- Reviewed head: `1833b7ea8e8a414fdcb012c399dd1e35e54e6f2a`
- Repaired and re-reviewed head: `a6a5fd825b73a794824861bb32e1754727df386c`
- CI repair head pending fresh re-review: `1982d89af58344e3150cd7e547c8ac0b30ddab02`
- Base head: `98a4df477f4ca72a2c64024c1c282c9bd1a25a44`
- Review date: 2026-07-20
- Result: R85-001 and R85-002 closed; P2 R85-003 repaired pending fresh re-review

## Scope and independence

A fresh read-only project `math_reviewer` received only the bounded
REQ-INFEAS-001 summary and integrated dependency closure, Issue #84 acceptance
criteria and exclusions, the M4 plan, relevant constraint, problem-IR, solver,
architecture, and ADR contracts, the complete exact PR diff, tests, example,
benchmark, registry state, and validation evidence. It inherited no
implementation reasoning and made no repository or remote change.

The reviewer checked exact and near-proportional classification, signs,
orientations, dimensions, units, scale invariance, closed-interval conflict
review, constant equalities, hard-constraint preservation, soft exclusion,
general convex certificates, source and ordering retention, allocation failure,
thread-safety, interface dispositions, benchmark claims, and registry truth.

## Findings

### P0 R85-001: rounded divide-and-multiply is not an exact proportionality test

`crates/georbf/src/problem_ir.rs:1919`, `1997-2005`, and
`crates/georbf/src/infeasibility.rs:124-128` derive a scale with one floating-
point division and declare rows exactly proportional when multiplying every
second-row coefficient by that rounded scale reproduces the first-row bits.
Rounded multiplication makes this condition neither sufficient nor necessary.

For a false positive, let

```text
a = [1, 1]
b = [3, next_up(3)] = [3, 3 + 2^-51]
a z = 0
b z = 3
```

Both rounded products `(1/3) * 3` and `(1/3) * next_up(3)` equal `1`, so the
implementation treats the rows as exact and rejects the singleton intervals
`[0, 0]` and `[1, 1]`. The determinant is `2^-51`, however, so the rows are not
parallel; `x = -y`, `y = 3 * 2^51` is a finite exact-real solution. An
approximately parallel row has therefore been promoted from warning-only
evidence to a false infeasibility decision, directly weakening a hard
constraint.

The converse also fails. The exactly proportional binary-float rows

```text
a = [1, 7, 13]
b = [49, 343, 637] = 49 a
```

are not recognized because rounded `(1/49) * b` produces values below the
corresponding entries of `a`. Conflicting equalities `a z = 0` and `b z = 49`
can pass canonical construction, while duplicate diagnostics label the rows
`NearDuplicate` instead of `Duplicate`.

Required repair: replace the rounded multiply-back predicate with a test of
exact proportionality between the exact binary floating-point values. Add one
regression covering both directions: the one-ULP nonparallel system must
canonicalize and may only warn as near-duplicate; the integer 49x pair must be
`Duplicate`, and inconsistent right-hand sides must return a source-complete
exact-conflict error.

### P1 R85-002: interval reorientation silently loses overflow and underflow conflicts

`crates/georbf/src/problem_ir.rs:1922-1925` and `2014-2019` transform interval
endpoints with ordinary floating-point multiplication. If a finite endpoint
overflows, `scale_interval` returns `None` and its caller silently skips the
pair. If it underflows, the rounded zero is treated as an exact endpoint.

Two finite-input counterexamples are:

```text
1e308 x = 0       and x >= 1e308
1e-308 x = 0      and x = 1e-308
```

The first transformed lower bound overflows to positive infinity and the
conflict is skipped. In the second, the transformed nonzero right-hand side
underflows to zero and distinct singleton intervals collapse. Both infeasible
systems can therefore pass canonical construction, contradicting exact closed-
interval review and positive row-scale invariance.

Required repair: compare transformed finite endpoints without lossy overflow
or underflow. Add source-aware single-variable regressions for both examples;
each must return an exact-conflict error rather than canonicalize, skip the
pair, or collapse a nonzero endpoint.

No other P0, P1, P2, or P3 finding was reported.

## Independent mathematical review

Two coefficient vectors are exactly proportional only when all corresponding
two-by-two determinants vanish over the exact binary floating-point values.
Floating-point division followed by rounded multiplication is not equivalent
to that condition. Once an exact nonzero proportionality is proved, the second
closed interval must be mapped by the scale, with endpoint reversal for a
negative scale, without changing finite-input real-number semantics. Approximate
proportionality may only produce diagnostics and can never prove infeasibility.

The existing equality/bound signs, same/reversed diagnostic orientation,
warning-only near-duplicate threshold, hard/soft separation, deterministic
family ordering, unchanged canonical rows, general multi-row certificate path,
source retention, D=1/D=2/D=3 availability, and `Send + Sync` evidence were
otherwise consistent with the scoped contracts.

## Validation and disposition

- Local and remote branch heads matched reviewed head `1833b7e`; the worktree
  was clean before this evidence-only Review change.
- Draft CI run 29714495028 passed the configured Ubuntu correctness job on the
  exact reviewed head. The Ready-only Windows, Ubuntu, macOS, and benchmark-
  smoke matrix correctly did not run.
- The reviewer and parent task each passed all five focused infeasibility tests.
  Those tests use only single-coefficient proportional rows and do not cover
  R85-001 or R85-002.
- The reviewer reproduced the example result of one exact and two near-
  duplicate pairs and ran the benchmark smoke at 161.68 microseconds per review
  with checksum 768. The recorded implementation timing remains a qualified
  baseline, not a performance promise.
- The parent task passed all 58 requirement checks and `git diff --check`.
  Exact implementation commit `63f34ed` retains its recorded complete standard
  gate; `63f34ed..1833b7e` changed only registry and handoff evidence.
- Rust, CLI, C, C++, Python, benchmark, and `implemented` registry dispositions
  remain truthful. No production code changed in this Review task.

PR #85 must remain Draft and REQ-INFEAS-001 must remain `implemented`. Open a
fresh Repair task limited to R85-001 and R85-002, add the required independent
regressions, run focused checks and the final stable-head standard gate, update
this review evidence and the bounded handoff, push, and stop for a fresh
independent re-review. Do not begin another requirement.

## Repair evidence pending fresh re-review

Repair commit `680d497d424fe3a611376b6bc415173ff9d2f6e2` addresses only
R85-001 and R85-002. Exact proportionality now compares the exact binary
coefficient products for every corresponding determinant. Exact interval
ordering uses the same product representation and does not materialize a row
quotient or transformed endpoint; any finite `f64` projection is created only
after the exact conflict decision for structured endpoint evidence.

Three independent public-canonicalization regressions reproduce all four
review counterexamples. `[1, 1]` and `[3, next_up(3)]` canonicalize and remain
warning-only near duplicates. `[1, 7, 13]` and `[49, 343, 637]` are exact
duplicates, and inconsistent right-hand sides return both sources in an exact
conflict. The `1e308` overflow and `1e-308` underflow interval cases both
return source-complete exact-conflict errors with strictly ordered evidence.

Focused validation passed all eight infeasibility tests, all 11 problem-IR
tests, all eight linear-constraint tests, and the `georbf` all-targets,
all-features Clippy gate. After the last code and normative-document change,
the stable repair head passed all five standard checks: format, workspace
all-targets/all-features Clippy with warnings denied, workspace
all-features tests, workspace doctests, and all 58 requirement checks.
`git diff --check` also passed. The unavailable later-tool dispositions remain
unchanged from the original review evidence.

This section records Repair evidence only. It does not independently re-review
or close R85-001 or R85-002. PR #85 remains Draft and REQ-INFEAS-001 remains
`implemented`; the next task must be a fresh independent re-review of the exact
repaired head and must not begin another requirement.

## Fresh independent re-review

A new read-only project `math_reviewer` independently re-reviewed exact PR head
`a6a5fd825b73a794824861bb32e1754727df386c` against base
`98a4df477f4ca72a2c64024c1c282c9bd1a25a44`. It received only the bounded
requirement and dependency summaries, Issue #84 criteria and exclusions, the
M4 plan, scoped mathematical, architecture, solver, and ADR contracts, the
complete diff, original findings, repair regressions, benchmark and validation
evidence, and the Ready CI workflow. It inherited no Implement or Repair
reasoning and made no repository or remote change.

R85-001 is closed. `exact_row_scale` tests every proportionality determinant as
an exact equality between two products of the binary coefficient values. Each
finite `f64` contributes at most a 53-bit significand, so the product fits the
106-bit range of the `u128` representation without rounded division or
multiply-back. The one-ULP nonparallel pair remains warning-only, while the
integer 49x pair is an exact duplicate and its inconsistent targets retain both
ordered sources in an exact-conflict error.

R85-002 is closed. Proportional interval endpoints are ordered by exact binary
cross-products. Negative proportionality swaps the later interval endpoints
and reverses the comparison for a negative denominator. Quotients are used
only after the exact decision to project finite diagnostic evidence; the
adjacent-float fallbacks preserve strict `lower > upper`. The finite overflow
and underflow regressions both retain their two ordered sources and cannot skip
or collapse the conflicts.

### P2 R85-003: the new benchmark is absent from Ready CI

The Ready-only three-platform workspace job in `.github/workflows/ci.yml` runs
the existing benchmark-smoke sequence and then proceeds directly from the
`convex_solver` benchmark to requirement-registry validation. The new
`constraint_diagnostics` benchmark is declared in `crates/georbf/Cargo.toml`,
implements `--smoke`, and is truthfully listed as implemented in the registry,
but the workflow never invokes it. Exact Ready-head CI can therefore pass
without exercising REQ-INFEAS-001's representative benchmark.

Required repair: add a Ready-workspace step running
`cargo bench -p georbf --bench constraint_diagnostics -- --smoke`. The repaired
exact Ready head must pass that step on Windows, Ubuntu, and macOS with the
fixed smoke checksum `768`.

No other P0, P1, P2, or P3 finding was identified. Hard rows remain unchanged;
approximate rows remain warning-only; soft objectives and cones remain
separate; general infeasibility remains on the independently reviewed
certificate path. No hidden regularization, presolve, pseudoinverse,
constraint relaxation, unsafe code, or relevant SPD/CPD, center-limit,
polynomial, or Hessian change was found.

The reviewer inspected the exact base, head, merge base, complete PR diff,
repair diff, changed implementation, tests, example, benchmark, manifest,
registry, bounded documents, and CI workflow. `680d497..a6a5fd8` changes only
this review record and `docs/progress/CURRENT.md`; exact whitespace checks for
the complete and evidence-only diffs passed. The parent task independently
reran all eight infeasibility tests and the benchmark smoke on `a6a5fd8` with
checksum `768`, and `git diff --check` passed. Draft Ubuntu CI run 29716310057
passed on that exact head. The complete standard gate recorded for stable
repair head `680d497` remains applicable because the later commits changed no
production, test, manifest, schema, CI, build, API, numerical, or dependency
input.

PR #85 must remain Draft and REQ-INFEAS-001 must remain `implemented`. Open a
fresh Repair task limited to R85-003, add only the missing Ready-workspace
benchmark-smoke step, run the focused workflow check and the final standard
gate on the stable repaired head, update this evidence and the bounded handoff,
push, and stop for another fresh independent re-review. Do not begin another
requirement.

## R85-003 repair evidence pending fresh re-review

Repair commit `1982d89af58344e3150cd7e547c8ac0b30ddab02` addresses only
R85-003. The Ready-only `workspace` job now runs
`cargo bench -p georbf --bench constraint_diagnostics -- --smoke` immediately
after the existing convex-solver smoke. Because that job retains its
Windows/Ubuntu/macOS matrix, every Ready head and `main` run will exercise the
representative REQ-INFEAS-001 workload on all three platforms. Draft behavior
is unchanged.

The focused benchmark smoke passed locally with checksum `768`, and
`git diff --check` passed. Local `actionlint` is unavailable and is not claimed
as passed. After the workflow change, the stable repair head passed all five
standard checks: format, workspace all-targets/all-features Clippy with
warnings denied, workspace all-features tests, workspace doctests, and all 58
requirement checks.

This section records Repair evidence only; it does not independently close
R85-003. PR #85 remains Draft and REQ-INFEAS-001 remains `implemented`. The
next task must freshly re-review the exact PR head and must not begin another
requirement.
