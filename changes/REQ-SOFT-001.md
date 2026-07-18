# REQ-SOFT-001

Added deterministic, solver-neutral per-constraint soft objectives to the
canonical problem IR. Every soft semantic equality, linear bound, interval, or
second-order cone retains its canonical relation shape, complete source
provenance, positive user residual scale, and explicit SquaredL2, AbsoluteL1,
or positive-delta Huber loss. Hard equalities, bounds, and cones remain in their
existing feasibility collections; mixed hard/soft inputs no longer reject or
alter any hard relation.

The objective contract is explicit: for relation violation `v` and user scale
`s`, the contribution is `rho(v / s)`. SquaredL2 uses `t^2`, AbsoluteL1 uses
`|t|`, and Huber uses `t^2 / 2` within its scaled-residual transition and the
matching linear tail outside it. Equality uses its signed affine residual;
bounds use distance to their closed feasible interval; cones use
`max(0, ||lhs||_2 - rhs)`. Positive scalar-unit rescaling of both relation and
scale preserves the objective value.

`CanonicalProblem` now exposes soft objectives, unit compiler-scaling entries,
loss-specific capability flags, and checked coefficient/numeric-memory
estimates that include every soft relation and its numeric metadata. Objective
order follows semantic insertion order independently of the existing hard
families. Allocation and provenance copying remain fallible and return
structured errors without partial canonical output.

Level priors now compile as canonical soft equality objectives on their
explicit level variables. `CompiledLevelProblem::priors()` remains as the
stable level-identity view, while `canonical_problem().soft_objectives()` is
the shared backend-facing representation. Priors are never promoted to hard
rows and do not claim support from the current dense equality solver.

Five independent soft-loss tests cover L2/L1/Huber truth values, independent
scales, mixed hard and soft equality/bound/cone families, exact relation-shape
retention, bound and cone violation semantics, positive scalar-unit rescaling,
invalid numeric input, deterministic provenance ordering, and `Send + Sync`.
Existing problem-IR tests now cover soft provenance and allocation failure;
level tests verify the exact prior variable, target, scale, loss, and source.
The focused benchmark compiles a deterministic 96-constraint mixed hard/soft
problem across D=1, D=2, and D=3; its smoke run completed successfully.

Rust is implemented. CLI is N/A because the stage-0 command exposes only help
and version and the complete data/schema CLI belongs to M8. C, C++, and Python
are N/A because their M9 ABI/binding requirements follow Rust API and schema
freeze; no adapter may reimplement objective compilation. This change adds no
dependency, numerical backend, schema, unsafe code, optimizer, hidden jitter,
regularization, pseudoinverse, hard-to-soft conversion, constraint deletion,
or automatic conflict repair.
