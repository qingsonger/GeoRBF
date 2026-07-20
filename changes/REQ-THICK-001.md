# REQ-THICK-001

Kept scalar level gaps and sampled local normal-thickness constraints as two
different mathematical and diagnostic capabilities. Existing `LevelOrder`
gaps remain hard canonical linear bounds `h_upper - h_lower >= delta_h` and
now expose the `ScalarLevelGap / ScalarOnly` diagnostic classification. They
are not described as geometric distance constraints.

Added immutable `LocalNormalThickness<D>` Rust semantics for exactly D=1,
D=2, and D=3. Each value owns distinct lower and upper level identifiers, a
finite sample point, a positive finite minimum thickness, and caller-owned
provenance. Compilation appends the hard ordered Lorentz cone
`T_min ||grad f(x)|| <= h_upper - h_lower` to an existing explicit
`CompiledLevelProblem`. Cartesian derivative functionals are requested in
deterministic axis order; the compiler owns the thickness multiplication and
both level signs. The canonical solver sees only affine rows and a cone.

The local diagnostic classification is
`SampledLocalNormalCone / SampledLocalFirstOrder`. Rustdoc and the normative
thickness document explicitly state that this is a sampled first-order
sufficient condition, not a global Euclidean separation and not evidence that
the future sampled geometric validator ran. Point coordinates, thickness, and
the caller's gradient linearization must use one explicit consistent
coordinate system.

Construction and compilation reject equal or unknown endpoint levels,
non-positive or non-finite thickness, empty local input, field linearizations
that reach into level variables, caller linearization failure, duplicate
stable identifiers, allocation failure, and thickness products that overflow
or underflow to zero. There is no softening, automatic scale, implicit gauge,
hidden regularization, constraint relaxation, geometric search, automatic
constraint generation, or automatic refit.

Ten independent integration tests plus one module allocation-failure
regression cover scalar/cone separation, diagnostic kind and guarantee, exact
D=1/D=2/D=3 Cartesian row ordering, endpoint signs, an analytic parallel-level
boundary with gradient `(3,4)`, complete point and provenance transfer,
immutability and `Send + Sync`, invalid construction, empty input, unknown
levels, duplicate existing and local identifiers, indexed linearizer errors,
field-variable boundaries, fallible collection for loose and unknown iterator
lengths, and coefficient and affine-constant multiplication overflow and
underflow. A runnable example and a deterministic 32-cone D=3 compilation
benchmark are included, with Ready/main benchmark-smoke wiring.

The independent-review repair makes constraint collection reserve only a
trusted iterator lower bound and uses fallible growth before every capacity-
exceeding push. It therefore neither rejects a trivial valid iterator because
of a loose upper bound nor permits unknown-length growth to bypass the
structured allocation error contract. The repair changes no formula, sign,
diagnostic, interface, or registry state.

Rust is implemented. CLI is N/A because the stage-0 command exposes only help
and version and complete project/schema commands belong to M8. C, C++, and
Python are N/A because their M9 requirements follow Rust API and schema freeze;
no adapter may reimplement the cone signs or geometric guarantee. The focused
benchmark is implemented. Sampled geometric validation remains exclusively in
REQ-THICK-002; no multi-field, local-anisotropy, persistence, adapter, release,
or subsequent requirement work was added.
