# Normal and Tangent Semantics

## Geological orientation conversion

Planar and linear orientations are distinct immutable values for exactly D=2
and D=3. A planar orientation owns a validated unit normal; a linear
orientation owns a validated unit direction. Neither value is an observation
or constraint, and conversion alone never asserts a nonzero gradient.

Every orientation preserves one explicit polarity:

- `Positive` keeps the reported or convention-derived reference direction.
- `Negative` stores its antipode.
- `Unknown` keeps the deterministic reference representative but declares its
  sign axial. Downstream direction-only semantics must therefore be invariant
  under `n -> -n`.

Angular inputs carry an explicit degree or radian unit and are converted to
radians before trigonometric evaluation. Negative zero is canonicalized to
positive zero. Exact horizontal, vertical, and compass-quadrant boundaries use
exact zero components so an irrelevant azimuth cannot perturb a pole.

The D=2 canonical vertical-section frame is X horizontal and Y up. Signed dip
`d` is positive when a plane descends toward +X, and signed plunge `p` is
positive downward. The reference directions are

```text
plane normal = [sin(d), cos(d)]
lineation    = [cos(p), -sin(p)].
```

Both angles are restricted to `[-pi/2, pi/2]`.

The D=3 canonical local frame is right handed: X is easting, Y is northing,
and Z is up. Azimuth is measured clockwise from +Y toward +X. For down-dip or
lineation azimuth `a`, unsigned plane dip `d`, and signed plunge `p` positive
downward,

```text
plane normal = [sin(d) sin(a), sin(d) cos(a),  cos(d)]
lineation    = [cos(p) sin(a), cos(p) cos(a), -sin(p)].
```

Strike/dip uses the right-hand rule: down-dip azimuth is
`strike + pi/2 (mod 2pi)`. Strike, dip direction, and azimuth lie in
`[0, 2pi)`; plane dip lies in `[0, pi/2]`; plunge lies in
`[-pi/2, pi/2]`. Degree inputs use the exact corresponding intervals. A
non-identity axis order, downward-positive vertical axis, left-handed frame,
or different bearing origin must be converted explicitly to this canonical
frame before orientations and coordinates are combined; metadata alone does
not guess that conversion.

These conventions follow the planar/linear distinction and explicit
convention metadata in [OGC GeoSciML 4.1, sections 8.4.6.2 and
8.4.6.3](https://docs.ogc.org/is/16-008/16-008r1.html). The compass azimuth and
right-hand-rule meaning (down dip to the right when facing along strike) agree
with [USGS OFR 01-223](https://pubs.usgs.gov/of/2001/of01-223/richard2.html).

## Normal constraints

All input directions are validated and normalized.

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
