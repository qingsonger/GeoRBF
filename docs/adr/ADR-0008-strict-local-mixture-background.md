# ADR-0008: Strict Background for Local Kernel Mixtures

- Status: Accepted
- Date: 2026-07-13
- Clarifies: ADR-0005

## Context

ADR-0005 requires a global background component in a spatial mixture, but the
mere presence of a component proves only positive semidefiniteness if its
spatial weight can vanish. GeoRBF requires a strictly positive-definite local
kernel and honest conditioning diagnostics.

## Decision

The v1 local-mixture path uses a fixed strictly positive-definite background
kernel whose finite spatial weight is nonzero everywhere. On a declared
operational domain the weight must additionally satisfy a policy lower bound
for conditioning. All other fixed-kernel components are positive definite or
positive semidefinite under their diagonal weight congruence. Weight
differentiability is capability-checked, with at least C2 weights required for
Hessian evaluation. CPD components remain incompatible with this path.

## Consequences

Every finite Gram matrix at distinct points is strictly positive definite by
the background congruence term. Coverage and lower-bound diagnostics can catch
near-degenerate configurations without hidden jitter. This record narrows the
background precondition in ADR-0005 and does not change its mixture form.
