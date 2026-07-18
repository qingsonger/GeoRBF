# REQ-LEVEL-001

Added an immutable Rust level semantic layer around the existing scalar-field
functional and canonical IR contracts. `LevelDefinition` retains a stable
`LevelId`, complete source provenance, and an explicit fixed, unknown, or prior
value policy. Priors validate a finite mean, positive finite scale, and an
explicit SquaredL2, AbsoluteL1, or positive-delta Huber loss. They remain typed
objective metadata in `CompiledLevelProblem`; no equality-only solver support
is implied.

`LevelMembership<D>` represents `f(x_i) - h_k = 0` and accepts exactly one
coefficient-1 Value atom; directional derivatives, scaled values, and
multi-atom expressions return a structured semantic error. `LevelOrder`
represents `h_upper - h_lower >= minimum_gap`. Compilation preserves caller
field blocks, appends one deterministic `levels` block, and emits sparse hard
membership equalities, fixed-value equalities, and order lower bounds with
their original provenance. The caller linearizer can address only field
variables; level coefficients and signs are owned by the level compiler.
Canonical output contains no geological identifiers or relations.

Construction rejects fewer than two levels, empty memberships, duplicate
level or observation identifiers, undefined references, invalid fixed/prior/
gap scalars, self edges, and isolated unknown levels. A deterministic Kahn pass
uses definition insertion order for simultaneous ties and returns exactly the
cycle-participating edge sources rather than downstream DAG edges. Gauge
review forms connected components from level-order edges and memberships to a
shared field node, then requires a fixed or prior anchor in every component.
Nonzero contrast must be forced between two membership-coupled levels through a
positive path gap or distinct fixed/prior anchors on those levels. A
membershipless level cannot supply field contrast. Distinct anchor values count
only when their levels belong to different transitive membership-equality
components, so a soft prior mean cannot manufacture contrast against a direct
or chained hard field equality. Missing-contrast diagnostics identify only the
failing field component and represent its one-level case without citing an
unrelated isolated anchor.

Hard-conflict review closes equality transitively across shared mathematical
Value evaluations regardless of functional provenance. It rejects distinct
fixed values in one equality component, retaining both fixed definitions and a
deterministic membership chain proving the conflict. It also propagates longest
minimum-gap paths through the DAG with an overflow-safe scaled representation
and rejects fixed endpoints that cannot satisfy a direct or transitive required
gap, retaining endpoint and path sources. A scale-aware roundoff allowance
makes the semantic precheck conservative at a floating-point boundary; emitted
individual hard rows are never changed, dropped, softened, or regularized.

Positive direct or transitive order paths between two levels in one transitive
membership-equality component are also rejected because the membership rows
force the corresponding level values equal. The structured
`MembershipOrderConflict` retains the deterministic equality-chain memberships
and every order edge on the selected positive path without emitting or altering
any hard row.

Twenty independent tests cover fixed/unknown/prior compilation, explicit
variable indices and signs, prior retention, membership units, deterministic
topological ties, exact cycle sources, ordinary and extreme transitive fixed
conflicts, provenance-independent same-functional conflicts with complete
sources, identical-membership positive-order infeasibility with complete path
sources, per-component gauge, membership-coupled contrast, identical-membership
fixed/prior anchor rejection and diagnostic evidence, transitive membership
equality conflicts with complete chain sources, one-level component evidence,
isolation, undefined references, constructor bypass attempts, numeric
validation, linearizer failures, out-of-range affine terms, and `Send + Sync`
across D=1, D=2, and D=3 public problem types. The focused benchmark builds,
validates, and canonicalizes a deterministic 64-level chain; the smoke run
completed successfully.

The final implementation tree passed the complete local standard gate:
formatting, warning-denying all-target/all-feature workspace Clippy,
all-feature workspace tests, workspace Rustdoc, all 58 requirement checks, and
`git diff --check`.

Rust is implemented. CLI is N/A because the stage-0 command exposes only help
and version and the complete data/schema CLI belongs to M8. C, C++, and Python
are N/A because their M9 ABI/binding requirements follow Rust API and schema
freeze; no adapter may reimplement this compiler. This change adds no runtime
dependency, solver backend, schema, unsafe code, hidden jitter, regularization,
pseudoinverse, constraint relaxation, or reference-differencing model type.
