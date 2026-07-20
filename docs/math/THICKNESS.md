# Thickness Contracts

GeoRBF separates three concepts that must not share one capability label.

## Minimum scalar gap

For levels `a` and `b`,

```text
h_b - h_a >= delta_h
```

is a scalar level-value relation. It does not by itself state a geometric
distance between complete level sets. [`LevelOrder`](../../crates/georbf/src/levels.rs)
keeps this relation as a hard canonical linear bound. Its
`thickness_diagnostics()` value is `ScalarLevelGap / ScalarOnly`; neither the
canonical relation nor its diagnostic is renamed as a distance.

## Local normal thickness constraint

At a specified sample point, the convex first-order sufficient condition is

```text
T_min ||grad f(x)||_2 <= h_b - h_a.
```

It is compiled as an SOCP constraint. Its guarantee is local and sampled; it is
not a proof of the global minimum Euclidean distance between two curved level
sets.

`LocalNormalThickness<D>` is available for exactly D=1, D=2, and D=3. It owns
distinct lower and upper `LevelId` values, a finite `Point<D>`, a positive
finite `T_min`, and complete caller provenance. The point, thickness, and
gradient linearizer must use one caller-consistent coordinate system: if the
point coordinates are measured in metres, `T_min` is in metres and the
linearized gradient is per metre. Coordinate conversion or normalization is
explicitly outside this semantic constructor.

After a `LevelProblem` has introduced explicit `h_k` variables,
`try_compose_local_normal_thickness` requests the D Cartesian directional
derivatives in zero-based axis order. For each affine derivative row `g_j`, it
forms the checked row `T_min g_j`. The ordered cone is

```text
||(T_min g_0), ..., (T_min g_(D-1))||_2
    <= (+1) h_upper + (-1) h_lower.
```

The compiler owns both signs and the positive scale. It rejects unknown or
equal endpoint levels, zero or non-finite thickness, field linearizations that
refer to level variables, duplicate stable observation identifiers, and
coefficient or constant products that overflow or underflow to zero. The
constraint is always hard; no soft loss, automatic scaling, hidden
regularization, or relaxation is introduced. The canonical solver receives
only affine rows and one Lorentz cone, with the original provenance.

Diagnostics label this relation as
`SampledLocalNormalCone / SampledLocalFirstOrder`. That label is intentionally
different from `ScalarLevelGap / ScalarOnly` and does not imply that the
sampled geometric validator below has run.

## Sampled geometric validation

After fitting, an independent validator searches from selected locations along
local normals for adjacent level-set intersections, refines each intersection,
computes geometric distances, and returns minimum, quantiles, failures, and
violation locations. It may produce an explicit proposed set of new local
constraints, but refitting is a separate user-visible action. Diagnostics label
local convex constraints and sampled validation separately.

This validator belongs to `REQ-THICK-002` and is not implemented by the local
cone API. Constructing or solving local cones never performs intersection
search, generates extra constraints, or refits automatically.
