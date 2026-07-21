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

`SampledThicknessRequest<D>` makes the fitted scalar values `h_lower <
h_upper`, the positive minimum geometric thickness, selected original-
coordinate locations, quantile probabilities, and every search/refinement
limit explicit. The request is available for exactly D=1, D=2, and D=3. A
selected location owns complete provenance; it need not be a fitting center or
an earlier observation.

`try_validate_sampled_thickness_with_control` accepts explicit
`ExecutionOptions` and one borrowed `ExecutionControl`. The current serial
implementation accepts an absent thread count or an explicit count of one and
rejects any larger count before fitted-field evaluation storage is prepared or
the field is evaluated. Progress reports the caller's determinism selection and
one effective worker. It checks cancellation before work, after reusable
fitted-field evaluation storage is prepared, and before and after every field
evaluation in the location, bracketing, and refinement loops. Progress uses a
checked maximum evaluation budget and deterministic input/search order. A
cancelled call returns a typed execution error and no partial report; neither
the execution metadata, control, token, nor sink is retained by the fitted
field. The convenience method uses default execution metadata and an empty
control.

For each selected location `x`, the validator evaluates the immutable fitted
field once for `(f(x), grad f(x))`. A scale-safe gradient norm below the
explicit threshold is reported as a per-location failure. Otherwise the
oriented unit normal is

```text
n(x) = grad f(x) / ||grad f(x)||_2.
```

The lower target is searched on `x - t n(x)` and the upper target on `x + t
n(x)`, for `0 <= t <= t_max`. Each side uses a fixed uniform number of
bracketing steps. An endpoint is accepted when its absolute scalar residual is
within the supplied value tolerance; a sign-changing bracket is refined by
bisection until the value tolerance or original-coordinate distance tolerance
is met. Exhausting the search radius or the refinement-iteration limit is
reported, never converted to a distance. A tangential contact that neither
lands within the value tolerance nor changes sign is therefore truthfully
reported as not found.

When both intersections exist, the reported quantity is the Euclidean length
between the two returned original-coordinate points,

```text
d_sample = ||x_upper - x_lower||_2.
```

In exact arithmetic this equals `t_lower + t_upper`. The returned-point
formula is authoritative in finite precision so coordinate rounding cannot
turn a larger stored separation into a false violation or proposed constraint.

It is sampled geometric evidence, not a proof of the global minimum distance
between curved level sets. The distinct diagnostic classification is
`SampledGeometricValidation / SampledGeometricEvidence`; scalar gaps and local
first-order cones retain their existing labels.

Successful distances are sorted with IEEE total ordering. Each requested
probability `q` in `[0, 1]` uses the deterministic type-7 convention: rank
`q(n - 1)` and linear interpolation between the adjacent sorted values. When no
location succeeds, the minimum and all requested quantile distances are
`None`, while the location failures remain available.

A successful distance strictly below the requested minimum is reported with
its sample index, point, measured distance, threshold, and provenance. If the
request opts in, each violation also produces one proposed
`LocalNormalThickness<D>` at that selected point. These are returned values
only: the validator never adds them to a problem, changes hard/soft semantics,
selects solver settings, solves, or refits. The user must explicitly construct
and execute any later refit.
