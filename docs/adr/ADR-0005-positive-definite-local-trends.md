# ADR-0005: Positive-Definite Local Trend Mixtures

- Status: Accepted
- Date: 2026-07-13

## Context

Putting an arbitrary location-dependent metric into a point-pair distance does
not generally preserve kernel positive definiteness. Local structural trends
must retain a valid optimization problem and differentiable evaluation.

## Decision

Use a finite mixture

```text
k(x, y) = sum_r b_r(x) b_r(y) k_A_r(x, y),
```

where each `k_A_r` is a verified fixed-metric SPD kernel, weights are smooth,
and a global background component is present. Derivatives include every weight
product-rule term. The compiler rejects CPD kernels in this v1 path.

## Consequences

Local directions, ellipsoids, influence radii, regions, and reference-field
controls are possible with a positive-definiteness guarantee. Evaluation cost
scales with active components and requires coverage and conditioning
diagnostics.
