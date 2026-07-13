# Kernel Requirements

Kernel modules contain no geological vocabulary. Each public kernel supplies a
radial value/derivative implementation and metadata covering:

- strictly positive definite or conditionally positive definite with order;
- supported dimensions;
- maximum derivative order and center behavior;
- compact support radius when applicable; and
- named, unit-documented parameters.

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

For an isotropic stationary kernel `k(x, y) = phi(r)`, with `d = x - y` and
`r = ||d||`, the non-center sign convention is

```text
a(r) = phi'(r) / r
grad_x k =  a(r) d
grad_y k = -a(r) d
H_xx k = a(r) I + (phi''(r) / r^2 - phi'(r) / r^3) d d^T
H_xy k = -H_xx k.
```

These quotient formulas are never evaluated at `r=0`. When the radial kernel
has the required twice differentiable Euclidean center extension, the analytic
limits are `grad_x k = grad_y k = 0`, `H_xx k = phi''(0) I`, and
`H_xy k = -phi''(0) I`. For `r_A^2 = d^T B d`, `B = A^T A`, replace `I` in
the Hessian center limit by `B`. Kernels without a required center extension
report an away-from-centers or unsupported capability instead of fabricating a
finite value.

An SPD weighted sum remains available when every component and weight policy
preserves positive definiteness. Spatially local mixtures follow the stricter
contract in `docs/architecture/ANISOTROPY.md`; CPD kernels are rejected there.
