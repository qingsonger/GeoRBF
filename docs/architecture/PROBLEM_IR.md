# Problem Intermediate Representations

## SemanticProblemIR

The semantic form retains stable observation identifiers, source file and
one-based line, original units, field path, optional constraint group, compiled
functional expressions, relation, enforcement, loss, and execution options.
The Rust type is idiomatically named `SemanticProblemIr<D>` and is available
only for D=1, D=2, and D=3. It rejects empty problems, duplicate observation
identifiers, empty metadata, malformed intervals and cones, invalid soft-loss
parameters, count overflow, and allocation failure without partial success.

The implemented functional layer precedes this IR and preserves only an opaque
stable provenance identifier per expression term. It does not claim that the
identifier is a complete semantic source location. `SemanticProblemIr` maps
each complete relation to richer observation and input provenance without
changing the mathematical atom or inserting a kernel-derivative sign.

Semantic expressions contain an `ObservationFunctional<D>` and a finite
constant. Relations are equality, one- or two-sided linear bound, or
second-order cone. Enforcement is always explicit: hard constraints compile in
this requirement, while soft enforcement compiles to an explicit objective
contribution with a positive scale and SquaredL2, AbsoluteL1, or positive-delta
Huber metadata. Exact optimizer-specific quadratic and epigraph lowering still
waits for its approved backend.

`REQ-LINEQ-001` adds an immutable semantic constructor layer immediately above
this IR. It owns explicit lower/upper/interval signs, caller-selected scalar
orientation for closed inside/outside bounds, `upper - lower` scalar-gap signs,
and increasing/decreasing directional-monotonicity signs. It lowers to ordinary
`SemanticConstraint` values and adds no canonical relation family. Existing
`LevelOrder` compilation remains the sole explicit-level path.

`CompiledLevelProblem` can consume a separately canonicalized field-only bound
problem whose named variable blocks exactly equal its field prefix. It appends
hard field bounds after level-order rows and soft field-bound objectives after
level priors, then reconstructs one immutable `CanonicalProblem`. Memberships,
fixed rows, order rows, priors, and field relations retain their original
coefficients, bounds, provenance, and relative order. Non-bound relation
families, mismatched spaces, and stable observation identifiers duplicated
across the two canonical inputs fail structurally. The cross-input identity
check covers both hard relations and soft objectives before any record is
appended.

## CanonicalProblem

The complete planned numerical form is

```text
minimize 0.5 z^T H z + g^T z
subject to
    A_eq z = b_eq
    lower <= A_lin z <= upper
    ||F_j z + f_j||_2 <= c_j^T z + d_j.
```

`REQ-IR-001` implements the constraint portion of this form. The compiler
accepts an explicit caller linearizer that maps each compiled functional to a
finite sparse affine expression over declared variable blocks. This keeps
kernel, basis, polynomial, and center assembly in later layers. The IR owns all
relation mapping: for `a^T z + q = t` it stores `a^T z = t-q`; linear bounds are
shifted by the same constant; cone expressions retain their affine constants.
The linearizer is never asked to insert relation signs or constants.

`CanonicalProblem` records deterministic variable-block offsets, equality
rows, two-sided linear rows, cones, complete row provenance, explicit identity
scaling, required solver capabilities, and a checked numeric-storage estimate.
It contains no level, horizon, normal, tangent, stratigraphy, lithology, kernel,
or third-party linear-algebra type. Canonicalization validates sparse ordering,
finite nonzero coefficients, variable indices, shifted-scalar overflow,
allocation, and estimate arithmetic before returning an immutable result.
Every owned provenance string is deep-copied through a fallible reservation;
failure returns `AllocationFailed` before any partial canonical problem can be
observed. Canonicalization does not scale, regularize, add jitter or hidden
variables, relax constraints, or select a solver.

Before a `CanonicalProblem` is returned, hard constant equality and bound rows
are checked, and every pair of exactly proportional hard affine equality/bound
rows is placed in one orientation for a closed-interval intersection. A
conflict retains the complete provenance of the one or two originating rows
through fallible owned storage. Positive row scaling and sign reversal do not
change the decision. Soft objectives are excluded from this feasibility
review. The check is intentionally not a general linear-program feasibility
algorithm and performs no tolerance-based rejection, relaxation, or repair.

The public constraint-review layer separately reports exact duplicate and
scale-aware near-duplicate hard affine functionals. It independently
infinity-normalizes each sparse row, compares both sign orientations, and uses
the explicit dimensionless `128 * epsilon` threshold only for warnings. Pair
order is deterministic, complete source provenance is cloned fallibly, and the
canonical coefficients, targets, bounds, ordering, and enforcement remain
unchanged. Hard cones are counted but not reinterpreted as affine rows; general
infeasibility uses the convex adapter's independently reviewed certificate.

Centers and observations remain separate through both forms. Later semantic
compilers and assembly requirements add their own finite-value, unit,
normalization, duplicate/conflict, polynomial-rank, derivative-capability,
anisotropy, gauge, contrast, and operational memory-limit checks before
allocation or solution; this IR does not claim those later validations.

At the pre-IR functional boundary, `ObservationFunctional<D>` and
`CenterRepresenter<D>` remain different Rust types. Semantic IR accepts only
the observation-side wrapper. A caller-provided affine linearizer is the sole
bridge to canonical numeric rows; neither IR chooses centers or assembles a
kernel matrix.
