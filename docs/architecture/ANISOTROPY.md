# Anisotropy Architecture

## Global metrics

Global anisotropy evaluates radial distance through an invertible linear map

```text
r_A(x, y) = ||A(x - y)||.
```

Isotropic, spheroidal, ellipsoidal, and validated user metrics are supported.
The same chain rule supplies original-coordinate derivatives.

An orientation tensor

```text
C = sum_i weight_i n_i n_i^T
```

is sign-invariant and estimates axes, not correlation lengths. Axis ratios are
user-provided or selected by bounded deterministic candidates and
cross-validation. Diagnostics include eigenvalue gaps, isotropy, maximum ratio,
confidence, and outlier influence.

## Local positive-definite trends

Arbitrary location-dependent point-pair metrics are forbidden. Local structure
uses

```text
k(x, y) = sum_r b_r(x) b_r(y) k_A_r(x, y),
```

where every `k_A_r` is a fixed SPD anisotropic kernel, `b_r` is smooth, and a
global background component prevents uncovered regions. Gradient and Hessian
calculations include every product-rule derivative of the weights.

Controls may carry location, normal, principal tangent, influence radius,
strength, and region. A normalized gradient from another already fitted field
may generate a control direction. Diagnostics export axes, lengths, weights,
background, condition number, coverage, direction jumps, and low-confidence
areas. CPD kernels produce an explicit incompatibility error in this path.
