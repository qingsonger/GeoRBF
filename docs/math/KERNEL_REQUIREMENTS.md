# Kernel Requirements

Kernel modules contain no geological vocabulary. Each public kernel supplies a
radial value/derivative implementation and metadata covering:

- strictly positive definite or conditionally positive definite with order;
- supported dimensions;
- maximum derivative order and center behavior;
- compact support radius when applicable; and
- named, unit-documented parameters.

The shared metadata model represents strict positive definiteness separately
from conditional positive definiteness. A CPD declaration carries a positive
integer order; order zero is rejected rather than used as an alias for SPD.
This requirement records the order only. Construction of the complete
polynomial space and enforcement of CPD side conditions belong to later
requirements.

Dimension support is a nonempty subset of D=1, D=2, and D=3 and can only be
queried through the sealed compile-time dimension bound. Derivative support is
hierarchical and records a maximum away-from-center order plus an optional
maximum smooth-center order. The center maximum cannot exceed the away maximum.
For any demanded order, the result is exactly one of:

```text
SupportedEverywhere
SupportedAwayFromCenters
Unsupported.
```

Matrix demand is observation order plus center-functional order. Query demand
is requested output order plus center-functional order. A sum above the
zero-through-third calculus range is unsupported. This classification is
metadata, not permission to fabricate a center limit or an unconditional
fitted-model Hessian.

Family parameter definitions use unique lower-snake-case names, a nonempty
description, a physical dimension (`Dimensionless`, `CoordinateLength`, or
`InverseCoordinateLength`), and an explicit finite, nonnegative, or positive
scalar constraint. The ambiguous generic name `shape_parameter` is forbidden.
Configured values are separate from static family metadata and must be finite
and satisfy their definition before a concrete kernel accepts them. Global
support has no radius. Compact support references a declared, strictly
positive coordinate-length radius parameter, so the configured support radius
is validated by that definition.

The v1 catalog includes polyharmonic/surface-spline families, Gaussian, inverse
multiquadric, a multiquadric only after its sign and CPD classification are
verified, supported Matérn smoothness levels, and Wendland C2, C4, and C6.
Classic polyharmonic splines have no redundant shape parameter. Other kernels
use explicit names such as length scale, support radius, core radius,
smoothness, power, or range; there is no generic `shape_parameter`.

Before exposure, every kernel documents its formula, parameter dimensions,
definiteness, CPD order, origin limit, required derivatives, and dimension
range. Tests use high-precision or independent finite differences, origin and
support-boundary cases, point-exchange signs, Hessian symmetry, random Gram SPD
or projected CPD checks, and pathological parameters.

For an isotropic stationary kernel `k(x, y) = phi(r)`, with `d = x - y`,
`r = ||d||`, and `u = d/r`, the non-center calculus is

```text
grad_d k = phi'(r) u
H_dd k = phi''(r) u u^T + (phi'(r) / r) (I - u u^T)

D3_dd d[i,j,l] = phi'''(r) u_i u_j u_l
  + ((phi''(r) - phi'(r) / r) / r)
    (delta_ij u_l + delta_il u_j + delta_jl u_i - 3 u_i u_j u_l).
```

An away radial jet supplies finite derivatives. For D=2 and D=3 it additionally
supplies the two expansion coefficients

```text
a(r) = phi'(r) / r
b(r) = (phi''(r) - a(r)) / r.
```

Concrete radial formulas compute `a` and `b` directly in cancellation-resistant
form. Reconstructing `b` by subtracting independently rounded `phi''` and `a`
near the center can destroy all significant digits or amplify roundoff by
`1/r`; the calculus therefore never performs that reconstruction. Tensor
components use fused multiply-add expansion and reject non-finite results. D=1
uses only `phi'`, `phi''`, and `phi'''` away from the center and never requires
either quotient.

Each query-argument derivative is a displacement derivative and each
center-argument derivative contributes one minus sign. Therefore an order-`n`
mixed tensor has sign `(-1)^m`, where `m` is the number of center arguments;
in particular `grad_y k = -grad_x k` and `H_xy k = -H_xx k`.

The separation radius is computed with maximum-component scaling so a finite,
representable norm does not overflow or underflow merely while it is formed.
Non-finite coordinate subtraction and a radius outside the finite `f64` range
are structured errors. The quotient formulas are never evaluated at `r=0`.
When a radial jet explicitly promises the required smooth Euclidean center
extension through third spatial order, the analytic limits are
`grad_d k = 0`, `H_dd k = phi''(0) I`, and `D3_ddd k = 0`. For
`r_A^2 = d^T B d`, `B = A^T A`, replace `I` in the Hessian center limit by
`B`. Kernels without a required center extension report an
away-from-centers or unsupported capability instead of fabricating a finite
value.

An SPD weighted sum remains available when every component and weight policy
preserves positive definiteness. Spatially local mixtures follow the stricter
contract in `docs/architecture/ANISOTROPY.md`; CPD kernels are rejected there.
