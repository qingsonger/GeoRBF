# Normal and Tangent Semantics

All input directions are validated and normalized. Orientation conversions
preserve polarity metadata and use radians internally.

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

In 2D the complement has one column; in 3D it has two independent columns.
Unknown polarity does not justify the non-convex claim
`|n^T grad f| >= g_min`. A fitted near-zero gradient is reported as a
diagnostic, not prevented by a fictitious convex constraint.

## Tangents and derivative-only gauges

An exact tangent `t` imposes `t^T grad f(x) = 0`; soft tangents use an explicit
loss and scale. Several independent tangents may share a point. A field with
only derivative observations has an additive constant freedom. The default is
to require an explicit anchor. Any future opt-in automatic anchor must be
reported in diagnostics and persisted in the model.
