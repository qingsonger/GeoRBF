# ADR-0001: Single Unified Scalar-Field Core

- Status: Accepted
- Date: 2026-07-13

## Context

GeoRBF must combine scalar values, level membership, differences, derivatives,
normals, tangents, bounds, cones, and thickness while serving several language
interfaces. Treating observation combinations as separate model families would
duplicate assembly, solving, evaluation, and error behavior.

## Decision

Use one `FieldProblem<D>` mathematical core for `f: R^D -> R`, with D restricted
to 1, 2, or 3. Semantic observation types compile to finite expressions of
value and directional-derivative functionals and then to one canonical
numerical problem. A `GeoProject` may own multiple independent fields, each
using this core.

## Consequences

All constraints share kernel calculus, rank handling, solvers, models, and
diagnostics. Cross-language parity has one numerical source. Geological-body
topology and arbitrary vector fields remain outside v1.
