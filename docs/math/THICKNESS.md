# Thickness Contracts

GeoRBF separates three concepts that must not share one capability label.

## Minimum scalar gap

For levels `a` and `b`,

```text
h_b - h_a >= delta_h
```

is a scalar level-value relation. It does not by itself state a geometric
distance between complete level sets.

## Local normal thickness constraint

At a specified sample point, the convex first-order sufficient condition is

```text
T_min ||grad f(x)||_2 <= h_b - h_a.
```

It is compiled as an SOCP constraint. Its guarantee is local and sampled; it is
not a proof of the global minimum Euclidean distance between two curved level
sets.

## Sampled geometric validation

After fitting, an independent validator searches from selected locations along
local normals for adjacent level-set intersections, refines each intersection,
computes geometric distances, and returns minimum, quantiles, failures, and
violation locations. It may produce an explicit proposed set of new local
constraints, but refitting is a separate user-visible action. Diagnostics label
local convex constraints and sampled validation separately.
