# PR #67 Independent Review

- Requirement: REQ-EXEC-001
- Issue: https://github.com/qingsonger/GeoRBF/issues/66
- Pull request: https://github.com/qingsonger/GeoRBF/pull/67
- Branch: `codex/req-exec-001-deterministic-execution-controls`
- Reviewed head: `1b2325b3a1904e99c52ab6fbf665f22bcd5d5275`
- Stable implementation head: `ef16599`
- Repair head: `947888a`
- Base head: `eaa7430fabafd1c8890306f9240afd4feb596e96`
- Review date: 2026-07-17
- Result: three P2 findings; repair recorded, fresh re-review required

## Scope and independence

A fresh read-only independent reviewer received only the bounded REQ-EXEC-001
requirement and integrated dependency summaries, Issue #66 acceptance criteria
and exclusions, the M3 plan, scoped architecture, solver-policy and ADR-0010
contracts, the complete PR diff, tests, registry evidence, and exact-head
validation state. It did not inherit the implementation task's reasoning and
made no repository or remote changes.

The reviewer checked API ownership and lifetimes, atomic ordering, cancellation
races and latency, progress ordering and counts, callback panic and reentrancy,
partial-result behavior, thread-count validation, field-option propagation,
rank, factorization, refinement and residual paths, structured error sources,
allocation boundaries, interface dispositions, hidden recovery, requirement
evidence, and the applicable mathematical and numerical contracts.

## Findings

### P2 R67-001: `Completed` may be followed by a cancellation error

`crates/georbf/src/execution.rs:103` defines `Completed` as successful operation
completion. `ProgressTracker::complete` at `crates/georbf/src/execution.rs:308-312`
checks cancellation, reports `Completed`, and then checks cancellation again.
A synchronous progress callback that cancels the token upon receiving
`Completed` therefore observes a successful terminal event before the Rust call
returns `ExecutionError::Cancelled { stage: Completed }`.

This gives one operation contradictory terminal states and may let an adapter
publish success before receiving an error. Repair must define one terminal
linearization point. The smallest expected policy is to check cancellation,
report `Completed`, and return success, treating callback-time or later
cancellation as post-completion. If another policy is selected, no error path
may emit an event publicly defined as successful completion.

Required regression: use a sink that cancels at `ExecutionStage::Completed`
and prove exactly one coherent outcome: success with a final `Completed` event,
or cancellation with no `Completed` event.

### P2 R67-002: failed numerical calls bypass the post-call cancellation check

Rank review and factorization at `crates/georbf/src/solver.rs:1113`,
`crates/georbf/src/solver.rs:1135`, and `crates/georbf/src/solver.rs:1169` use
`?` before the following progress checkpoint. If another thread requests
cancellation during an SVD or factorization and that operation itself returns
an error, the numerical error returns immediately and the promised post-call
cancellation observation never occurs. Similar fallible residual/refinement
computations have the same control shape.

This contradicts `docs/architecture/ARCHITECTURE.md:161-162`, which says a
request made during an indivisible backend call is observed immediately after
the call. Error precedence therefore depends on whether the backend succeeds.

Repair must retain each fallible numerical result, perform the post-call
cancellation checkpoint without publishing a successful stage, and then apply
one documented priority between concurrent cancellation and numerical failure.

Required regressions: deterministic test-only barriers or hooks for rank review
and factorization must request cancellation during a failing operation, verify
the documented error priority, and verify that no successful stage is emitted.

### P2 R67-003: early refinement completion counts work that did not run

The solver total at `crates/georbf/src/solver.rs:412-414` includes
`maximum_refinement_steps`. Refinement can stop early at
`crates/georbf/src/solver.rs:1188-1190` when the initial residual is already zero
or a candidate does not improve it. `ProgressTracker::complete` nevertheless
sets `completed = total` at `crates/georbf/src/execution.rs:310`, counting every
skipped refinement slot as completed work even though `ProgressEvent::completed`
is documented at `crates/georbf/src/execution.rs:130` as completed work units.

The current tests at `crates/georbf/tests/execution.rs:155` and
`crates/georbf/tests/execution.rs:231` assert only monotonicity, upper bounds,
and repeat equality, so they do not detect the false jump.

Repair must define whether total denotes actual work or a maximum budget. With
the current completed-work definition, skipped refinement reviews cannot be
reported as executed; any alternative skipped-slot semantics must be explicitly
typed and documented.

Required regressions must assert the full stage sequence and every
`(completed, total)` pair for an exact-zero residual, first-candidate early
stop, explicit regularization with a second rank review, and a solve that uses
the full refinement allowance.

## Independent truth and unaffected contracts

- SPD assembly has `N(N+1)/2` kernel events plus canonicalization and symmetry;
  the current total formula is correct.
- CPD assembly adds one CPD construction, `N` polynomial rows, and one projected
  energy event; the total `N(N+1)/2 + N + 4` is correct.
- A non-regularized dense solve has five fixed successful boundaries: memory,
  original rank, factorization, initial residual, and final residual. Explicit
  regularization correctly adds one effective-rank review. Only optional
  refinement slots create a count mismatch.
- Release/acquire ordering is sufficient for the sticky shared cancellation
  state, and all clones share one `Arc<AtomicBool>`.
- Controls remain borrowed and are not retained in systems, solutions, or
  fitted fields. Callbacks run synchronously without a core lock; callback
  panic unwinds rather than becoming false success.
- Thread counts greater than one are rejected before numerical work or progress
  in direct solve, field assembly, retained-field solve, and fitted-field fit.
- This PR does not change matrix formulae, signs, dimensions, units, CPD
  null-space construction, rank thresholds, factorization mathematics,
  regularization policy, residual mathematics, or hard constraints. It adds no
  jitter, fallback, pseudoinverse, or constraint relaxation.
- The registry remains `implemented`, and the Stage 0 CLI, C, C++, Python, and
  benchmark dispositions remain truthful.

## Independently verified evidence

- Local HEAD and the remote PR branch matched reviewed head `1b2325b`; the
  worktree was clean before the review record was added.
- Draft Ubuntu CI run 29550596570 passed the correctness gate on that head; the
  Ready-only three-platform and benchmark-smoke matrix correctly did not run.
- The reviewer ran `cargo test -p georbf --test execution` (six passed),
  `cargo test -p georbf --all-features`, `cargo test --doc -p georbf`, and
  `git diff --check origin/main...HEAD`; all passed.
- A scan of the added diff found no stdout/stderr output macro.
- The stable implementation head `ef16599` retains the previously recorded
  complete local standard gate. The later reviewed head changes only registry
  and bounded-handoff PR linkage evidence.

## Repair disposition

PR #67 must remain Draft and REQ-EXEC-001 remains `implemented`. A fresh Repair
task must address only R67-001, R67-002, and R67-003, add the specified
independent regressions, rerun focused checks during development, and run the
complete standard gate after the final code change. This Review task does not
repair production code or begin another requirement.

## Repair evidence

Fresh Repair commit `947888a` addresses only the three recorded findings:

- R67-001 gives `Completed` one success linearization point. Cancellation is
  checked before the event, while cancellation requested by its synchronous
  callback is post-completion and affects only later token reuse.
- R67-002 retains every fallible staged result, checks cancellation immediately
  after each call without publishing a successful stage, and gives cancellation
  observable at that checkpoint priority over the concurrent numerical error.
- R67-003 defines `total` as the checked maximum work budget and preserves the
  actual completed-work count when optional refinement stops early.

The repair adds independent regressions for terminal callback cancellation,
concurrent cancellation with failing rank and factorization calls, and every
stage/count pair for exact-zero residual, first-candidate termination, explicit
regularization with its second rank review, and full refinement-budget use.
Focused execution-control, failure-priority, all-feature core, and core Rustdoc
checks passed. The stable repair tree also passed format, warning-denying
workspace Clippy, all-feature workspace tests, workspace Rustdoc, all 58
requirement checks, and `git diff --check`.

This evidence records the Repair only. It does not independently re-review the
repair, mark the PR ready, run ready-only CI, merge, integrate REQ-EXEC-001, or
begin another requirement. PR #67 remains Draft and the registry remains
`implemented` pending a fresh independent re-review.
