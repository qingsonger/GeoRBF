# Anisotropy Architecture

## Global metrics

Global anisotropy evaluates radial distance through an invertible linear map

```text
r_A(x, y) = ||A(x - y)||.
```

Isotropic, spheroidal, ellipsoidal, and validated user metrics are supported.
`A` must be finite and have a finitely representable inverse, so `B = A^T A`
is SPD. The Rust core stores all three fixed-size objects: `A`, `A^-1`, and
`B`. It rejects a metric component that cannot be represented even if the
input entries of `A` are individually finite.

The convenience families use these conventions:

- isotropic length `ell`: `A = I/ell`;
- spheroidal principal unit direction `u`, axial length `ell_a`, and transverse
  length `ell_t`: `B = u u^T/ell_a^2 + (I-u u^T)/ell_t^2`; `A` is a stable
  orthonormal-frame factor with principal row `u^T/ell_a` and transverse rows
  scaled by `1/ell_t`, avoiding a large-minus-large projector evaluation;
- ellipsoidal unit axes `u_i` and lengths `ell_i`: row `i` of `A` is
  `u_i^T/ell_i`; and
- a user supplies either an arbitrary `A` or an exactly symmetric SPD `B`.

Lengths are positive values in the active coordinate length unit and their
reciprocals must be representable. Therefore `A(x-y)` is a nondimensionalized
displacement. A concrete radial kernel used with physical ellipsoid lengths
is evaluated in that transformed coordinate system; a transformed-coordinate
kernel length scale of one leaves the supplied ellipsoid lengths unchanged.
Alternatively, a caller may supply a dimensionless `A` and retain a physical
kernel length parameter. The application must choose one composition
explicitly and record it; GeoRBF does not infer or combine scales.

Ellipsoidal axes are already validated `UnitDirection` values. Their pairwise
dot products are checked against a caller-supplied finite tolerance in
`[0,1)`. The constructor neither changes nor orthogonalizes them. A user metric
must be componentwise finite and exactly symmetric. Its unregularized Cholesky
factorization must have strictly positive computed pivots. Before factorization,
power-of-two congruence equilibration and fixed-size floating expansions certify
the exact signs of all leading principal minors of the supplied `f64` matrix.
The same certification is applied to a metric derived from `A`; construction
fails if rounding `A^T A` loses SPD. A strict necessary diagonal-maximum bound
rejects only non-SPD off-diagonal entries that would overflow exact-product
formation; determinant boundaries remain exact-expansion decisions. There is
no symmetrization, eigenvalue clipping, diagonal adjustment, jitter, or
pseudoinverse.

Exact partial-pivot inversion decides singularity and representability.
Deterministic fixed-sweep one-sided Jacobi SVD supplies positive finite
singular values and the Euclidean condition number; it is diagnostic and does
not replace the exact inversion decision. No condition threshold is implicit.
A caller may select an explicit finite maximum at least one, in which case the
error records that maximum and the rejected singular-value diagnostics.

The transformed separation is formed as `A(x-y)`, not as `Ax-Ay`, so a common
large origin cannot overflow or erase a representable displacement. Stable
maximum-component radius construction and analytic center dispatch remain in
kernel calculus. Given a transformed-coordinate spatial jet, the original-
coordinate query derivatives are

```text
g_x       = A^T g_z,
H_x       = A^T H_z A,
T_x,ijk   = sum_abc A_ai A_bj A_ck T_z,abc.
```

The returned jet keeps the existing center-argument rule: each center
derivative contributes exactly one minus sign. Direct fixed-size sums through
third order preserve exact Hessian and third-tensor permutation symmetry;
non-representable finite-input results are structured errors. Construction and
evaluation allocate no heap memory and use no dynamic dispatch.

Global metrics do not estimate axes, select ratios, construct observations,
assemble kernels, or fit fields. They are compatible with either SPD or CPD
radial families because an invertible change of coordinates preserves the
family classification; later CPD assembly still owns polynomial side
conditions.

## Deferred orientation estimation

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

### V1 positive-definite mixture primitive

`REQ-TREND-001` implements the mathematical mixture primitive independently of
the later control compiler. `LocalTrendMixture<D>` retains a finite ordered set
of `LocalTrendComponent<D>` values. Every component combines one existing
`KernelDefinition<D>`, one fixed validated `GlobalAnisotropy<D>`, and one
analytic `SmoothSpatialWeight<D>`. Construction accepts only kernels whose
metadata declares strict positive definiteness and identifies the rejected
component and CPD order otherwise. It does not attach a polynomial side space
or reinterpret a CPD kernel as SPD.

The v1 strict background policy is deliberately concrete: the selected
background weight is a finite nonzero constant. This is stronger than merely
sampling a spatially varying function and proves that its diagonal congruence
is invertible everywhere. A caller declares a finite closed axis-aligned
operational domain and a positive finite minimum absolute background weight;
construction rejects a background below that policy. Because the background
is constant, the proved lower bound holds on the entire coordinate space and
therefore on the declared domain. Other components may use signed constant or
Gaussian weights. Every accepted nonzero weight amplitude must have a nonzero
finite represented square; this prevents floating-point underflow from erasing
the strict background diagonal contribution. A Gaussian value may still
underflow to represented zero far from its center without weakening strict
positive definiteness, because it supplies only an additional positive-
semidefinite congruence term. Its demanded derivatives are evaluated with a
scaled exponential product when direct multiplication would underflow or
overflow, so a rounded-zero or subnormal value does not erase a representable
gradient or Hessian.

For query derivatives through Hessian order, the implementation evaluates

```text
grad_x (b_x b_y k) = b_y (grad(b_x) k + b_x grad_x(k)),

H_x (b_x b_y k) = b_y (
    H(b_x) k
    + grad(b_x) grad_x(k)^T
    + grad_x(k) grad(b_x)^T
    + b_x H_x(k)).
```

Every term is included and checked for finite representability. Aggregate
capability is the intersection of every fixed kernel's metadata: a center
Hessian is rejected if any member provides it only away from centers. The
primitive exposes component/background identity, maximum fixed-anisotropy
condition number, lower-bound policy margin, and allocation-free pointwise
`sum_r b_r(x)^2` coverage. Weight evaluation is demand-bounded: coverage and
center factors compute values only, and query weights compute no derivative
above the caller's request. It applies no jitter, regularization, clipping,
pseudoinverse, automatic component selection, or implicit refit.

Local geological controls, regions, reference-field gradients, direction-jump
and confidence policy, and fitted-field integration belong to subsequent
requirements. Versioned schemas and the complete CLI remain M8 work; C, C++,
and Python adapters remain M9 work.
