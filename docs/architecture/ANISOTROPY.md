# Anisotropy Architecture

## Global metrics

Global anisotropy evaluates radial distance through an invertible linear map

```text
r_A(x, y) = ||A(x - y)||.
```

Isotropic, spheroidal, ellipsoidal, and validated user metrics are supported.
`A` must be finite and invertible, so `B = A^T A` is SPD; singular or
policy-rejected ill-conditioned transforms fail with recorded singular values
and condition estimates. The same chain rule supplies original-coordinate
derivatives.

An orientation tensor

```text
C = sum_i weight_i n_i n_i^T
```

uses unit directions, finite nonnegative weights, and at least one strictly
positive weight. It is sign-invariant and positive semidefinite and estimates
axes, not correlation lengths. Axis ratios are user-provided or selected by
bounded deterministic candidates and cross-validation. Diagnostics include
eigenvalue gaps, isotropy, maximum ratio, confidence, and outlier influence.

## Local positive-definite trends

Arbitrary location-dependent point-pair metrics are forbidden. Local structure
uses

```text
k(x, y) = sum_r b_r(x) b_r(y) k_A_r(x, y),
```

where every `k_A_r` is a fixed SPD anisotropic kernel, `b_r` is smooth, and a
global background component prevents uncovered regions. More precisely, the
background kernel is strictly PD and its finite weight is nonzero everywhere;
on a declared operational domain it must also satisfy a policy lower bound for
conditioning. For any distinct sample points, with
`D_r = diag(b_r(x_i))`, the Gram quadratic form is

```text
a^T K a = sum_r (D_r a)^T K_A_r (D_r a) > 0 for every a != 0,
```

because every term is nonnegative and the background term is strictly
positive. Merely including a background component whose weight may vanish is
insufficient. Weights must have the derivatives demanded by the model
capability (at least C2 for Hessians). Gradient and Hessian calculations include
every product-rule derivative of the weights.

Controls may carry location, normal, principal tangent, influence radius,
strength, and region. A normalized gradient from another already fitted field
may generate a control direction. Diagnostics export axes, lengths, weights,
background, condition number, coverage, direction jumps, and low-confidence
areas. CPD kernels produce an explicit incompatibility error in this path.
