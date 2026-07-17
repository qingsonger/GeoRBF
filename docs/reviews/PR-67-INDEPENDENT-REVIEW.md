# PR #67 Independent Review

- Requirement: REQ-EXEC-001
- Issue: https://github.com/qingsonger/GeoRBF/issues/66
- Pull request: https://github.com/qingsonger/GeoRBF/pull/67
- Branch: `codex/req-exec-001-deterministic-execution-controls`
- Reviewed head: `1b2325b3a1904e99c52ab6fbf665f22bcd5d5275`
- Stable implementation head: `ef16599`
- Repair head: `947888a`
- Re-reviewed head: `d9cba54`
- R67-004 repair head: `33cf9def4a418970281b3ad130dcf58ec1b29074`
- Base head: `eaa7430fabafd1c8890306f9240afd4feb596e96`
- Review date: 2026-07-17
- Result: R67-004 repaired with production-path regressions; fresh re-review required

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

## Independent repair re-review

A fresh read-only `math_reviewer` independently reviewed base `eaa7430`, prior
review evidence `f2a6171`, stable repair `947888a`, and exact PR head
`d9cba54`. It received only the bounded requirement and dependency summaries,
Issue #66, the M3 plan, scoped architecture, solver policy and ADR-0010, the
complete and repair diffs, and recorded validation evidence. It made no
repository or remote changes.

R67-001 is closed. `ProgressTracker::complete` checks cancellation before the
single successful terminal event and returns success after the callback, so a
callback-time request is post-completion. The public controlled-solve
regression proves one final `Completed` event and a successful solution.

R67-003 is closed. The total remains the checked maximum budget, `complete`
does not credit skipped work, and the public controlled-solve regressions lock
every stage and `(completed, total)` pair for zero residual, first-candidate
termination, explicit regularization, and full allowance use.

### P2 R67-004: R67-002 lacks its required production-path regression

The R67-002 implementation is structurally repaired: rank results are retained
before `finish_work`, while factorization, solve, scaling, validation,
refinement, and residual results are retained before `observe_result` or
`finish_work`. Cancellation observable at those checkpoints therefore takes
priority and a failed stage publishes no successful event.

However, the regressions at `crates/georbf/src/solver.rs:2082-2104` cancel the
token synchronously, construct an arbitrary error, and call
`ProgressTracker::finish_work` directly. The named tests at
`crates/georbf/src/solver.rs:2146-2161` never call the public controlled solve,
`diagnose_rank`, or `Factorization::try_new`; they use no second thread or
barrier. They would still pass if the guarded production call sites at
`crates/georbf/src/solver.rs:1124-1126` or
`crates/georbf/src/solver.rs:1184-1195` regressed to the original early-`?`
shape. This leaves the specifically required rank/factorization production-path
regression, and therefore R67-002 closure, unproven. No current production
semantic defect was identified.

The smallest Repair is to add test-only injected failing rank and factorization
hooks at the actual controlled-solve call sites, coordinate a separate
cancellation thread with a barrier while each injected operation is active,
invoke the public controlled solve, and assert the correct staged
`ExecutionError::Cancelled` plus absence of a successful event for that stage.
The Repair must not broaden execution semantics or alter numerical policy.

## Re-review validation

- `cargo test -p georbf --test execution`: 8 passed.
- `cargo test -p georbf concurrent_cancellation_precedes`: 2 passed, but these
  are the insufficient direct-tracker regressions described by R67-004.
- `cargo test -p georbf --all-features`: 198 unit/integration tests and 29
  doctests passed.
- `cargo test --doc -p georbf`: 29 passed.
- `git diff --check` passed for the complete, focused-repair, and evidence-only
  review ranges; no added core output macro was found.
- The immutable stable repair tree retains its recorded complete local standard
  gate. Ready-only Windows, Ubuntu, macOS, and benchmark-smoke CI did not run
  because PR #67 remains Draft.

## R67-004 repair evidence

Fresh Repair commit `33cf9de` replaces the two insufficient direct-tracker
regressions with public-path tests. One-shot test-only hooks now fail the actual
original-rank-review and factorization calls used by
`DenseEqualitySystem::try_solve_with_control`. Each hook enters a two-phase
barrier while the injected backend operation is active; a separate thread
crosses the first phase, cancels the shared token, and crosses the second phase
before the injected numerical error returns.

Both regressions invoke the public controlled solve and receive the expected
`ExecutionError::Cancelled` at `RankReview` or `Factorization`. Their exact
recorded stage sequences stop before the failed stage, proving no successful
event was published. The hook state, synchronization, and injected-error
branches are all compiled only under `cfg(test)`. The common retained-result
and post-call progress checkpoints remain unchanged, and this Repair makes no
production execution-semantic, numerical-policy, formula, factorization,
dependency, manifest, registry, or schema change.

Focused validation passed for both failure-priority regressions, all eight
execution-control integration tests, all-feature `georbf`, warning-denying
all-target core Clippy, and `git diff --check`. The stable R67-004 repair commit
then passed the complete local standard gate: format, warning-denying workspace
Clippy, all-feature workspace tests, workspace Rustdoc, all 58 requirement
checks, and `git diff --check`.

This section records Repair evidence only; it is not an independent re-review.
PR #67 remains Draft and REQ-EXEC-001 remains `implemented`. A fresh Review task
must independently re-review R67-004 and either record findings and stop or,
only if clean, follow the mandatory ready-CI-integration sequence. This Repair
does not mark the PR ready, merge, integrate the requirement, or begin another
requirement.
