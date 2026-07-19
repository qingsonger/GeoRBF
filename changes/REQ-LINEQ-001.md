# REQ-LINEQ-001

Added immutable D=1/D=2/D=3 Rust semantic constructors for lower bounds, upper
bounds, closed intervals, explicitly oriented closed inside/outside bounds,
scalar gaps, and directional monotonicity. Inside/outside callers must choose
whether inside values lie at or below or at or above the scalar boundary.
Scalar gaps compile as `upper - lower >= minimum_gap`. Monotonicity accepts
exactly one coefficient-one DirectionalDerivative atom: increasing compiles to
a nonnegative lower rate and decreasing to the corresponding negative upper
rate. Non-finite thresholds, reversed intervals, invalid gaps/rates, malformed
monotonicity functionals, and invalid soft-loss metadata return structured
errors before a semantic constraint is exposed.

Every constructor lowers to the existing provenance-bearing
`SemanticConstraint` and then to `CanonicalLinearBound`; no optimizer-facing
geological type or new canonical relation was added. Scalar-gap functional
terms are combined through fallible reserved storage. Existing explicit
`LevelOrder` remains the only layer-order representation and continues to emit
`h_upper - h_lower >= minimum_gap`, so this change does not introduce a second
level model.

Canonical construction now rejects a hard constant row whose closed interval
excludes zero. It also compares hard bound rows for exact coefficient equality
or exact sign reversal, transforms them to one orientation, and rejects a
disjoint interval pair with complete earlier/later source provenance. Conflict
evidence uses explicitly reserved fallible storage. Soft objectives are not
misclassified as feasibility conditions. The review is exact and deliberately
does not claim general LP infeasibility, near-duplicate detection, scaling,
regularization, jitter, pseudoinverse repair, constraint deletion, or
hard-to-soft conversion.

Six independent tests cover all relation mappings and signs, both region
orientations, scalar-gap coefficient signs, increasing/decreasing monotonicity,
invalid numeric and functional shapes, exact same/sign-reversed hard conflicts,
constant-row infeasibility, feasible touching intervals, provenance and
insertion order, D=1/D=2/D=3, and `Send + Sync`. Existing level tests retain
independent layer-order gap, DAG, gauge, contrast, and infeasibility truth. A
runnable example and a deterministic 96-constraint benchmark accompany the
Rustdoc and normative documentation.

Rust is implemented. CLI is N/A because the stage-0 command exposes only help
and version and complete project/schema commands belong to M8. C, C++, and
Python are N/A because their M9 requirements follow Rust API and schema freeze;
no adapter may reimplement these signs. The focused benchmark is implemented.
No dependency, backend, schema, unsafe code, fitting claim, normal/tangent/cone
semantics, local thickness, local anisotropy, or multi-field work was added.
