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
For order `m`, the required side space is the complete polynomial space of
total degree at most `m-1` in each declared supported dimension; order one
therefore means constants. This requirement records that contract only.
Construction of the space, term-count overflow and size diagnostics, and
enforcement of CPD side conditions belong to later requirements.
The normative polynomial and null-space definition is in
[`CPD_AND_POLYNOMIALS.md`](CPD_AND_POLYNOMIALS.md).

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
fitted-model Hessian. Away-from-center capability covers every strictly
positive separation. For compact support this includes the support boundary,
not only the interior formula.

Family parameter definitions use unique lower-snake-case names, a nonempty
description, a physical dimension (`Dimensionless`, `CoordinateLength`, or
`InverseCoordinateLength`), and an explicit finite, nonnegative, or positive
scalar constraint. The ambiguous generic name `shape_parameter` is forbidden.
Configured values are separate from static family metadata and must be finite
and satisfy their definition before a concrete kernel accepts them. Global
support has no radius. Compact support references a declared, strictly
positive coordinate-length radius parameter, so the configured support radius
is validated by that definition. For radius `rho`, compact support means the
kernel uses the exact zero extension for `r >= rho`. The interior formula and
its one-sided derivatives at `rho` must match that extension through the
declared away derivative order; otherwise the family must declare a lower
capability or be rejected. Sparse neighbor exclusion may rely on this exact
boundary contract only after a concrete kernel verifies it.

The v1 catalog includes polyharmonic/surface-spline families, Gaussian, inverse
multiquadric, a multiquadric only after its sign and CPD classification are
verified, supported Matérn smoothness levels, and Wendland C2, C4, and C6.
Classic polyharmonic splines have no redundant shape parameter. Other kernels
use explicit names such as length scale, support radius, core radius,
smoothness, power, or range; there is no generic `shape_parameter`.

## Polyharmonic and surface splines

For an integer power `p >= 1`, the CPD-positive polyharmonic normalization is

```text
s_p = (-1)^(floor(p / 2) + 1)

phi_p(0) = 0
phi_p(r) = s_p r^p          for r > 0 and odd p
phi_p(r) = s_p r^p log(r)   for r > 0 and even p.
```

`PolyharmonicSpline(p)` supports D=1, D=2, and D=3 and declares the minimal
CPD order `floor(p/2)+1`. The integer power selects a family member; it is not
a tunable scalar parameter, has no physical parameter unit, and is never
represented as a redundant shape parameter.

Here `r` is the numeric radius in the active coordinate representation. For
an even power, `log(r)` therefore uses one active coordinate unit as its fixed
reference; it does not introduce a tunable length scale. Coordinate metadata
and any normalization are preserved separately by the coordinate contract.

`SurfaceSpline<D>(m)` is the dimension-specific Sobolev parameterization. It
is valid exactly when `2m > D`, selects `p = 2m-D`, supports only its
compile-time dimension, and declares CPD order `m`. Thus its later side space
is the complete polynomials through total degree `m-1`, even where the same
signed radial formula could also satisfy a lower-order generic PHS side
condition. Zero or insufficient order and overflow while forming `2m` are
structured construction errors.

Every positive radius supplies derivatives through third order. For odd `p`,

```text
phi'   = s_p p r^(p-1)
phi''  = s_p p(p-1) r^(p-2)
phi''' = s_p p(p-1)(p-2) r^(p-3)
a      = s_p p r^(p-2)
b      = s_p p(p-2) r^(p-3).
```

For even `p`, with `L = log(r)`,

```text
phi'   = s_p r^(p-1) [p L + 1]
phi''  = s_p r^(p-2) [p(p-1) L + 2p - 1]
phi''' = s_p r^(p-3) [p(p-1)(p-2) L + 3p^2 - 6p + 2]
a      = s_p r^(p-2) [p L + 1]
b      = s_p r^(p-3) [p(p-2) L + 2(p-1)].
```

The implementation evaluates `a` and `b` from these closed forms; it never
reconstructs `b` by subtracting rounded values. At the origin the Euclidean
spatial capability is exact through `min(p-1, 3)`: value only for `p=1`, first
order for `p=2`, second order for `p=3`, and third order for `p>=4`. A finite
one-sided radial derivative does not widen this spatial center capability.
Requests for a complete third-order center jet below `p=4` fail explicitly.
Negative or non-finite radii and non-representable derivatives or expansion
coefficients also return structured errors.

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
