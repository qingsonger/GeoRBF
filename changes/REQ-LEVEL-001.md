# REQ-LEVEL-001

Added an immutable Rust level semantic layer around the existing scalar-field
functional and canonical IR contracts. `LevelDefinition` retains a stable
`LevelId`, complete source provenance, and an explicit fixed, unknown, or prior
value policy. Priors validate a finite mean, positive finite scale, and an
explicit SquaredL2, AbsoluteL1, or positive-delta Huber loss. They remain typed
objective metadata in `CompiledLevelProblem`; no equality-only solver support
is implied.

`LevelMembership<D>` represents `f(x_i) - h_k = 0`, and `LevelOrder`
represents `h_upper - h_lower >= minimum_gap`. Compilation preserves caller
field blocks, appends one deterministic `levels` block, and emits sparse hard
membership equalities, fixed-value equalities, and order lower bounds with
their original provenance. The caller linearizer can address only field
variables; level coefficients and signs are owned by the level compiler.
Canonical output contains no geological identifiers or relations.

Construction rejects fewer than two levels, empty memberships, duplicate
level or observation identifiers, undefined references, invalid fixed/prior/
gap scalars, self edges, and isolated unknown levels. A deterministic Kahn pass
produces a stable topological order or returns the cyclic edge sources. Gauge
review forms connected components from level-order edges and memberships to a
shared field node, then requires a fixed or prior anchor in every component.
Nonzero contrast must occur in the field-connected component through a
positive gap or distinct fixed/prior anchors.

Hard-conflict review rejects the same field functional assigned to distinct
fixed values. It also propagates longest minimum-gap paths through the DAG and
rejects fixed endpoints that cannot satisfy a direct or transitive required
gap, retaining endpoint and path sources. A scale-aware roundoff allowance
makes the semantic precheck conservative at a floating-point boundary; emitted
individual hard rows are never changed, dropped, softened, or regularized.

Ten independent tests cover fixed/unknown/prior compilation, explicit variable
indices and signs, prior retention, deterministic topology, cycle sources,
transitive and same-functional fixed conflicts, per-component gauge, missing
field contrast, isolation, undefined references, constructor bypass attempts,
numeric validation, linearizer failures, out-of-range affine terms, and
`Send + Sync` across D=1, D=2, and D=3 public problem types. The focused
benchmark builds, validates, and canonicalizes a deterministic 64-level chain;
the smoke run completed successfully.

Rust is implemented. CLI is N/A because the stage-0 command exposes only help
and version and the complete data/schema CLI belongs to M8. C, C++, and Python
are N/A because their M9 ABI/binding requirements follow Rust API and schema
freeze; no adapter may reimplement this compiler. This change adds no runtime
dependency, solver backend, schema, unsafe code, hidden jitter, regularization,
pseudoinverse, constraint relaxation, or reference-differencing model type.
