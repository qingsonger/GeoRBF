# Normal and Tangent Semantics

All input directions are validated and normalized. Orientation conversions
preserve polarity metadata and use radians internally.

For a unit normal `n`, the complement basis is
`T in R^(D x (D-1))` with `T^T T = I`, `T^T n = 0`, and
`T T^T = I - n n^T`. Basis construction is deterministic up to signs that do
not change the compiled constraints.

## Normal modes

- GradientVector imposes `grad f(x) = g`.
- DirectionOnly chooses an orthonormal basis `T` for the complement of unit
  normal `n` and imposes `T^T grad f(x) = 0`.
- DirectionWithPolarity adds `n^T grad f(x) >= g_min`, where `g_min` may be
  zero only when explicitly requested.
- AngularCone imposes
  `||T^T grad f(x)||_2 <= tan(theta) n^T grad f(x)` and
  `n^T grad f(x) >= g_min`.
- AxialDirection or UnknownPolarity uses only the complement equations and is
  invariant under `n -> -n`.

For oriented modes, `g_min` is finite and nonnegative. An angular cone requires
`0 <= theta < pi/2`; this domain is what makes the displayed cone convex and
prevents an undefined or sign-reversing tangent. In 2D the complement has one
column; in 3D it has two independent columns.

In 1D the complement has zero columns. `DirectionOnly` and `AxialDirection`
would therefore add no constraint, while `AngularCone` would be independent of
`theta`; the compiler rejects all three as semantically empty or misleading in
D=1. `GradientVector` and `DirectionWithPolarity` remain valid in D=1.
Unknown polarity does not justify the non-convex claim
`|n^T grad f| >= g_min`. A fitted near-zero gradient is reported as a
diagnostic, not prevented by a fictitious convex constraint.

## Tangents and derivative-only gauges

An exact tangent `t` imposes `t^T grad f(x) = 0`; soft tangents use an explicit
loss and scale. Several independent tangents may share a point. A field with
only derivative observations has an additive constant freedom. The default is
to require an explicit anchor. Any future opt-in automatic anchor must be
reported in diagnostics and persisted in the model.
