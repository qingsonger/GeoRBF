# REQ-TREND-001

Issue #102 defines the acceptance criteria and exclusions for the
positive-definite local trend mixture primitive. The Rust core now exposes an
immutable `LocalTrendMixture<D>` for exactly D=1, D=2, and D=3. Each ordered
component retains one existing configured kernel, one fixed validated global
anisotropy, and one analytic spatial weight. Construction inspects kernel
metadata and rejects CPD members explicitly with the component index and CPD
order; it never adds a polynomial side space or hides conditional
definiteness.

One caller-selected component is the strict background. Its v1 weight must be
a finite nonzero constant, which is invertible as a diagonal congruence for
every finite distinct point set. A finite closed operational domain and a
positive finite minimum absolute background weight make the conditioning
policy explicit. Other components may use signed constant or analytic
Gaussian weights and contribute positive-semidefinite congruence terms. No
arbitrary location-dependent point-pair metric is introduced.

Value, query gradient, and query Hessian evaluation includes every product-rule
term from both the spatial weight and fixed anisotropic kernel. Aggregate
capability is the intersection of component kernel metadata, so unavailable
center derivatives return a structured error. Arithmetic failures are also
structured. Immutable diagnostics report component/background identity,
background policy margin, maximum fixed-anisotropy condition number, and
allocation-free pointwise squared-weight coverage.

Independent tests cover deterministic random SPD Gram matrices, strict
background and policy rejection, independent finite-difference gradient and
Hessian truth, center Hessian capability, coverage after local-weight
underflow, CPD rejection, input boundaries, D=1/D=2/D=3, compile-time D=4
rejection, and `Send + Sync`. A runnable example and deterministic focused
Hessian benchmark cover the public workflow and hot path.

Rust is implemented. CLI is N/A because versioned schemas and the complete
data CLI arrive in M8. C, C++, and Python are N/A because bindings follow API
and schema freeze in M9. Trend-control compilation, region semantics,
reference-field directions, fitted-field integration, orientation estimation,
automatic component selection, persistence, and adapters are excluded. This
change adds no dependency, unsafe code, hidden regularization, jitter,
pseudoinverse, solver change, or production adapter. Independent Review and
integration remain required before the registry may become `integrated`.

The deterministic release-mode smoke baseline on the development machine was
approximately 230 ns/evaluation for D=1, 490 ns/evaluation for D=2, and
1.16 us/evaluation for D=3 over 10,000 two-component Hessian evaluations per
dimension. The benchmark prints a deterministic checksum and is a regression
signal rather than a cross-machine performance promise.
