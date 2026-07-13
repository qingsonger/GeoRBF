# Core Mathematical Specification

## Domain and dimensions

The sole mathematical product is a scalar field

```text
f: R^D -> R, D in {1, 2, 3}.
```

Public APIs admit exactly those three dimensions through a sealed compile-time
dimension bound. D=0 and arbitrary unverified dimensions are invalid. A project
may contain several independent fields; this does not create several core
algorithms.

## Geometry primitives

`Point<D>`, `Vector<D>`, `Direction<D>`, and `UnitDirection<D>` exist only for
`D` in `{1, 2, 3}`. Every stored component is a finite `f64`; constructors
reject NaN and either infinity with the zero-based component index. A zero
`Vector` is valid, while `Direction` and `UnitDirection` reject a vector whose
components are all positive or negative zero. Their representation is private,
so safe callers cannot bypass these invariants.

`Direction` preserves the finite, nonzero magnitude supplied by the caller.
For its components `v_i`, `UnitDirection` computes

```text
s   = max_i |v_i|
q_i = v_i / s
n   = sqrt(sum_i q_i^2)
u_i = q_i / n.
```

The direction invariant guarantees `s > 0`. At least one `|q_i|` is one and
every `|q_i| <= 1`, so forming the norm cannot overflow or collapse to zero in
the supported dimensions. The stored `u` has Euclidean norm one up to floating
point roundoff. This requirement assigns no physical units, coordinate axes,
CRS, or orientation semantics; those are introduced by later requirements.

## Field representation

For center representers `M_j` at centers `c_j`, weights `w_j`, polynomial basis
`p`, and coefficients `beta`, a fitted field is

```text
f(x) = sum_j w_j M_j^(y) k(x, c_j) + p(x)^T beta.
```

An observation functional `L_i` produces the matrix action

```text
A_ij = L_i^(x) M_j^(y) k(x_i, c_j).
```

`DirectionalDerivative(x, u)` means `u^T grad_x` for a validated unit
direction `u`; the same definition is applied in the center variable for
`M_j^(y)`. No adapter or assembly path inserts an additional sign. For a
stationary symmetric kernel, this convention gives

```text
grad_x k(x, y) = -grad_y k(x, y).
```

Kernel calculus must use analytic center limits rather than evaluate formulas
containing `1/r` at `r=0`. Exchange-identity tests cover value/derivative and
derivative/derivative actions, and an all-representer matrix must be symmetric
to its scale-derived numerical tolerance.

Observations and center representers are separate collections. This distinction
is retained even when an all-representer strategy initially gives them the same
points and functionals.

The only v1 atomic functionals are value at a point and directional derivative
at a point. A finite linear expression of atoms represents differences,
coordinate derivatives, directional gradients, tangents, normal complements,
monotonicity, and polarity.

## Coordinates

Fitting uses normalized coordinates

```text
x_tilde = S^-1 (x - mu),
```

where `S` is invertible and carries the coordinate scaling policy. The model
stores `mu` and `S`. With `g_tilde` and `H_tilde` evaluated in normalized
coordinates, original-coordinate derivatives are

```text
g = S^-T g_tilde
H = S^-T H_tilde S^-1.
```

Coordinate length units must be compatible. CRS metadata is preserved but the
core does not reproject coordinates. All internal angles are radians.

## Derivative capability

The matrix derivative demand is observation order plus center-functional order.
The query demand is requested output order plus center-functional order. A
model reports value, gradient, and Hessian capability as supported everywhere,
supported only away from centers, or unsupported. Kernel smoothness and center
limits decide the result; Hessian support is never unconditional. A local
mixture additionally requires the corresponding derivatives of every spatial
weight, including all product-rule terms.

## Correctness policy

Truth comes from analytic fields, high-precision evaluation, independent finite
differences, invariance properties, and documented SPD or CPD properties. No
external geological implementation is a correctness oracle. Numerical
tolerances are scale-derived and recorded by the future `NumericalPolicy`.
