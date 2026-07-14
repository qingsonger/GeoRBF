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

Derivative coefficients and their integer radial powers are evaluated as one
scaled product for range classification. In particular, an intermediate
`r^q` underflow must not erase a nonzero derivative or expansion coefficient
when multiplication by its analytic coefficient brings the final result back
into the representable subnormal range. The ordinary finite path remains a
direct integer power and product; only an otherwise zero or non-finite extreme
product is re-evaluated in the log domain.

## Smooth global-support kernels

Every smooth global-support family has exactly one configured
`length_scale = ell`, a positive finite coordinate length. Values are
dimensionless and all families support D=1, D=2, and D=3. Construction also
requires the reciprocal derivative scale through third order to be finite;
this prevents a successfully constructed kernel from promising derivatives
that necessarily overflow near its characteristic radius. No amplitude,
variance, arbitrary smoothness, or generic shape parameter is included.

Let `alpha = 1/ell`, `q = alpha r`, and `h = sqrt(1+q^2)`. The strictly
positive-definite Gaussian uses the squared-exponential convention

```text
phi = exp(-q^2/2)
phi'   = -alpha q phi
phi''  = alpha^2 (q^2-1) phi
phi''' = alpha^3 q (3-q^2) phi
a      = -alpha^2 phi
b      = alpha^3 q phi.
```

This length-scale convention is the `nu -> infinity` limit of the Matérn
scaling below. Gaussian has value one, zero gradient, Hessian
`-alpha^2 I`, and zero third tensor at the center.

The strictly positive-definite inverse multiquadric is

```text
phi = h^-1
phi'   = -alpha q h^-3
phi''  = alpha^2 (2q^2-1) h^-5
phi''' = 3 alpha^3 q (3-2q^2) h^-7
a      = -alpha^2 h^-3
b      = 3 alpha^3 q h^-5.
```

Its implementation uses `hypot(1,q)`, the bounded ratio `q/h`, and scaled
products so `q` or an intermediate inverse power cannot erase a representable
subnormal final result.

The conventional positive square-root multiquadric is conditionally negative
definite on the constant-zero subspace. GeoRBF's conditional-definiteness
contract requires positive projected Gram energy, so the implemented member
has the mandatory opposite sign and CPD order one:

```text
phi = -h
phi'   = -alpha q h^-1
phi''  = -alpha^2 h^-3
phi''' = 3 alpha^3 q h^-5
a      = -alpha^2 h^-1
b      = alpha^3 q h^-3.
```

The later polynomial side space is therefore constants. Inverse
multiquadric and signed multiquadric have the same full third-order center
capability as Gaussian, with center Hessian `-alpha^2 I`; their center values
are one and negative one respectively. The multiquadric value can be
non-representable at a finite extreme radius and then fails explicitly even
when an individual higher derivative would remain finite.

The supported Matérn members use the standard
`t = sqrt(2 nu) r / ell = beta r` convention:

```text
nu=1/2: phi = exp(-t)
nu=3/2: phi = (1+t) exp(-t)
nu=5/2: phi = (1+t+t^2/3) exp(-t).
```

For `nu=1/2`, with `E=exp(-t)`,

```text
phi'   = -beta E
phi''  = beta^2 E
phi''' = -beta^3 E
a      = -beta^2 E / t
b      = beta^3 E (t+1) / t^2.
```

For `nu=3/2`,

```text
phi'   = -beta t E
phi''  = beta^2 (t-1) E
phi''' = beta^3 (2-t) E
a      = -beta^2 E
b      = beta^3 E.
```

For `nu=5/2`,

```text
phi'   = -beta t(1+t) E / 3
phi''  = beta^2 (t^2-t-1) E / 3
phi''' = beta^3 t(3-t) E / 3
a      = -beta^2 (1+t) E / 3
b      = beta^3 t E / 3.
```

Matérn `1/2` is continuous at the center but has no Euclidean first
derivative there, so its center capability is value only. Matérn `3/2`
supports center derivatives through second order, with Hessian
`-beta^2 I`, but its finite one-sided radial third derivative does not create
a spatial third derivative. Matérn `5/2` supports the complete third-order
center jet, with Hessian `-(beta^2/3) I`. All three supply analytic derivatives
through third order at every positive radius.

Exponential products use a direct ordinary path and re-evaluate only a zero
or non-finite extreme product from its combined log magnitude. Rational
products similarly combine the reciprocal scale and `hypot` power before
range classification. These paths preserve representable tails while still
returning structured errors for genuinely non-finite final derivatives or
expansion coefficients.

The SPD and multiquadric sign classifications are cross-checked against
[Micchelli's conditional-positive-definiteness result](https://pages.stat.wisc.edu/~wahba/stat860public/pdf1/micchelli.interpolation.86.pdf).
The half-integer Matérn formulas and `sqrt(2 nu) r/ell` scaling follow
[Rasmussen and Williams, section 4.2](https://gaussianprocess.org/gpml/chapters/RW.pdf).

## Wendland compact-support kernels

The compact catalog uses exactly one configured `support_radius = rho`, a
positive finite coordinate length. Let `alpha = 1/rho`, `q = r/rho`, and
`t = max(1-q,0)`. All three members are dimensionless, normalized to one at
the center, strictly positive definite in D=1, D=2, and D=3, and exactly zero
for `r >= rho`:

```text
C2: phi = t^4 (1 + 4q)
C4: phi = t^6 (1 + 6q + 35q^2/3)
C6: phi = t^8 (1 + 8q + 25q^2 + 32q^3).
```

The C4 polynomial is divided by its conventional constant factor three so
that every member has center value one; positive scalar normalization does not
change strict positive definiteness. These are the dimension-three functions
constructed by [Wendland (1995)](https://doi.org/10.1007/BF02123482), whose
positive-definiteness result also covers the lower supported dimensions.

The implementation evaluates these analytic radial terms without numerical
differentiation:

```text
C2:
  phi'   = -20 alpha q t^3
  phi''  = 20 alpha^2 (4q-1) t^2
  phi''' = 120 alpha^3 (1-2q) t
  a      = -20 alpha^2 t^3
  b      = 60 alpha^3 t^2

C4:
  phi'   = -(56/3) alpha q(1+5q) t^5
  phi''  = -(56/3) alpha^2 (1+4q-35q^2) t^4
  phi''' = 560 alpha^3 q(3-7q) t^3
  a      = -(56/3) alpha^2 (1+5q) t^5
  b      = 560 alpha^3 q t^4

C6:
  phi'   = -22 alpha q(1+7q+16q^2) t^7
  phi''  = -22 alpha^2 (1+6q-15q^2-160q^3) t^6
  phi''' = 1584 alpha^3 q(1+5q-20q^2) t^5
  a      = -22 alpha^2 (1+7q+16q^2) t^7
  b      = 528 alpha^3 q(1+6q) t^6.
```

Here `a=phi'/r` and `b=(phi''-a)/r` are direct forms for D=2/D=3 Cartesian
expansion. Their boundary factors show that the value, first, second, and
third derivatives agree with the exact zero extension at `r=rho`. The C2
center supports spatial derivatives through second order with Hessian
`-20 alpha^2 I`; its nonzero one-sided radial third derivative does not define
a Euclidean third tensor. C4 and C6 have complete third-order center jets with
zero odd tensors and Hessians `-(56/3) alpha^2 I` and `-22 alpha^2 I`.

For `0 < r < rho`, the center half forms `t=1-q`, while the boundary half uses
`t=(rho-r)/rho` so neither an enormous support radius nor subtraction of a
rounded near-one `q` erases representable information. Factored products use
a direct ordinary path and re-evaluate only a zero or non-finite extreme
product from its combined log magnitude. Construction requires reciprocal
scales through third order to be finite, and genuinely non-representable final
results return structured errors. The numerical kernel does not own a sparse
index or assembly policy; those remain gated by the compact-sparse spike.

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
