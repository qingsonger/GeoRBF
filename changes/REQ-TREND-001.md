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

Post-review repair makes the public weight representation opaque so external
callers cannot forge variants or cached Gaussian parameters. It rejects every
nonzero amplitude whose represented square is zero, evaluates Gaussian
derivatives with a stable scaled exponential product when the value or an
intermediate product underflows, and bounds weight arithmetic to the requested
derivative order. Regressions cover the external construction barrier,
strict-background square underflow, a representable extreme-scale Hessian, and
Value/Coverage success when an unused Hessian would overflow.

Fresh re-review repair also evaluates the Gaussian weight value through the
combined amplitude/exponent logarithmic scale whenever the direct exponential
product is not normal, retaining a representable mixture contribution even if
the exponential alone underflows. Gaussian construction now rejects radii
whose reciprocal or reciprocal square rounds to zero, preserving the public
inverse-derivative contract. Public regressions cover both extreme-scale
boundaries.

The subsequent F7-F8 repair no longer forms a scaled displacement or a mixed
scaled-coordinate product before stable derivative combination. Gradients use
the equivalent `b (-r^-2) delta` factorization, while mixed Hessian entries use
`b r^-2 r^-2 delta_i delta_j`; the stable product combines every nonzero
factor before rounding the final result. Canonical axis ordering preserves
bitwise Hessian symmetry. Public regressions retain the independent 120-digit
gradient and mixed-Hessian truths at both underflow boundaries.

The F9 repair avoids diagonal-Hessian cancellation at a Gaussian radius. It
uses the equivalent `b r^-4 (delta-r)(delta+r)` factorization instead of
forming `(delta/r)^2 - 1` from an already rounded scaled displacement. The
public D=1 regression at the successor of radius three retains the independent
mixture-Hessian truth `1.2101577062956176e-17`.

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

The post-review repair smoke on the same development machine reported
approximately 305 ns/evaluation for D=1, 519 ns/evaluation for D=2, and
1.17 us/evaluation for D=3 over 500,000 evaluations per dimension, with the
same deterministic checksums. This records the stable-derivative checks rather
than setting a cross-machine threshold; Value and Coverage now avoid all unused
gradient and Hessian arithmetic.

The F7-F8 repair smoke reported approximately 225 ns/evaluation for D=1,
465 ns/evaluation for D=2, and 1.11 us/evaluation for D=3 over 10,000
evaluations per dimension, with the established deterministic checksums. The
smoke remains a regression signal rather than a cross-machine threshold.

The F9 repair smoke reported approximately 211 ns/evaluation for D=1,
458 ns/evaluation for D=2, and 1.12 us/evaluation for D=3 over 500,000
evaluations per dimension, with the established deterministic checksums. It
likewise records a local regression signal rather than a performance promise.
