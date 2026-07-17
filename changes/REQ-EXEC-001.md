# REQ-EXEC-001

Added caller-owned deterministic execution controls for the long synchronous
Rust core paths. `CancellationToken` is cloneable and `Send + Sync`, shares one
sticky atomic state, and is borrowed through `ExecutionControl` together with an
optional synchronous `ProgressSink`. Typed `ProgressEvent` values identify the
operation and work boundary, retain the requested deterministic policy, report
the truthful effective worker count, and provide monotonic completed/total work
without core stdout or stderr output.

`FieldProblem::try_assemble_with_control`,
`DenseEqualitySystem::try_solve_with_control`,
`try_solve_field_with_control`, and `FittedField::try_fit_with_control` preserve
the existing convenience APIs while propagating controls through assembly and
solving. Cancellation is checked before work and at deterministic boundaries
around kernel and polynomial rows, CPD work, canonicalization, symmetry review,
memory review, rank review, factorization, refinement, and residual review.
Backend SVD and factorization calls remain indivisible. A cancellation returns
`ExecutionError::Cancelled` through the owning structured error and exposes no
partial system, solution, or fitted model.

Post-review repair gives terminal completion one linearization point and makes
post-call error priority explicit. Cancellation observed immediately after
fallible memory, rank, factorization/solve, residual, or refinement work takes
priority over that work's concurrent numerical failure without publishing a
successful stage. A sink cancellation from `Completed` is post-completion.
Progress totals are maximum work budgets, while completed counts include only
performed work and therefore remain below the budget when refinement stops
early.

The implemented dense path is serial. An absent explicit thread count or one is
accepted; any larger value is rejected before numerical work with
`ExecutionError::UnsupportedThreadCount`. This avoids claiming or silently
clamping an unavailable parallel capability. Deterministic mode retains the
existing fixed ordering; disabling the request permits future nondeterministic
implementations but does not alter the current serial result.

Independent behavior tests cover cloned cross-thread cancellation, pre-start
and progress-triggered cancellation, callback-time terminal cancellation,
post-call cancellation precedence over failing rank and factorization work,
exact typed event sequences and counts for optional refinement and explicit
regularization, monotonic progress, repeated bit-identical solutions and
diagnostics, repeated identical progress, explicit one-thread propagation,
rejection of two threads before events, retained field execution options, and
controlled field assembly. The existing field, solver, model, diagnostics, and
workspace suites cover unchanged convenience paths and the absence of core
output macros.

The post-call failure-priority regressions traverse the public controlled dense
solve. Test-only one-shot hooks fail the real rank-review and factorization call
sites while a separate cancellation thread is coordinated through a two-phase
barrier. The tests therefore prove cancellation priority at the production
checkpoint and prove that the failed stage emits no successful event; non-test
builds contain neither hook state nor an injected failure branch.

Rust is implemented. The CLI is N/A because its Stage 0 surface exposes only
help and version, not a long-running core operation. C, C++, and Python are N/A
because the Stage 0 FFI/Python crates export no ABI or module and the C++ wrapper
does not yet exist; later M9 parity requirements must map these Rust controls
without reimplementing them. Benchmarking is N/A per the registry because this
requirement is behavior-focused and later performance requirements own fixed
thread-count baselines.

This requirement adds no runtime dependency, global thread pool, parallel
algorithm, solver-policy change, schema, adapter scaffold, hidden fallback,
regularization, pseudoinverse, or constraint relaxation.
