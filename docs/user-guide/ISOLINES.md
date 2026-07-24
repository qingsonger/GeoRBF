# Two-Dimensional Isolines

`FittedField<2>::try_isolines` extracts evidence for `f(x, y) = h` from one
finite rectangle in the fitted field's original-coordinate convention. It
evaluates the immutable Rust model directly. The contour code does not refit
the field, change coefficients, approximate derivatives, or duplicate kernel
mathematics.

## Request and methods

An `IsolineRequest` supplies a finite target level, strict lower-left and
upper-right corners, one `IsolineMethod`, and `IsolineSettings`. The settings
make the numerical resolution explicit:

- nonzero X and Y cell counts;
- a nonzero maximum bisection count for each crossed edge;
- positive finite fitted-value and original-coordinate distance tolerances.

`MarchingSimplices` is the reference path. It splits every cell along the
lower-left to upper-right diagonal and marches the two resulting triangles.
`DisambiguatedMarchingSquares` is the regular-grid path. For an
alternating-sign cell, it evaluates the scale-normalized bilinear saddle when
that saddle lies inside the cell. If it does not, it uses the bilinear center.
An exactly zero decider uses the documented positive-connectivity tie. Every
ambiguous cell records the normalized decider, decision kind, and selected
edge pairing.

The grid is evidence, not a completeness proof. A component that is smaller
than a cell or crosses an edge an even number of times can remain unseen.
Increase the explicit grid resolution when application scale or independent
validation requires it.

## Intersections and degeneracy

The extractor evaluates every grid node through `FittedField<2>::try_value`.
An edge is retained only when one endpoint is exactly on the level or the two
finite endpoint residuals have opposite signs. A sign bracket is refined by
bisection until the fitted-value tolerance or maximum-coordinate bracket-width
tolerance is satisfied. Exhausting the explicit iteration count is an error
and returns no partial report.

Shared horizontal, vertical, diagonal, and exact grid-vertex intersections
receive canonical identities. GeoRBF deduplicates those identities before
building topology and removes only exactly repeated undirected segments. It
does not merge distinct nearby components through a hidden geometric radius.

If both endpoints of a complete grid edge are exactly on the target level,
ordinary one-crossing marching topology is underdetermined. Those finite
samples do not prove that a nonlinear field equals the level throughout the
edge, so GeoRBF returns a conservative structured `DegenerateGridEdge` error
rather than inventing either an isolated crossing or a continuous segment.
Exact-vertex patterns that cannot form the ordinary two-intersection square or
triangle cases are likewise explicit degeneracy errors.

## Returned topology

`IsolineReport::vertices()` contains original-coordinate points, fitted
values, and target residuals. `polylines()` contains indices into that one
deduplicated vertex array. A closed polyline lists each vertex once and
implicitly connects its last vertex back to its first. An open polyline lists
both endpoints.

Topology is accepted only when every unique vertex has degree one or two.
Degree greater than two is a structured non-manifold error. Every degree-one
vertex must lie on the requested rectangle within the explicit coordinate
tolerance; otherwise the component is internally incomplete and extraction
fails. Diagnostics retain every accepted open endpoint and all boundary sides
it touches, including both sides at a rectangle corner.

Diagnostics also record the method, grid dimensions, actual fitted-field
evaluation count, raw and unique segment counts, exact duplicate removal,
deduplicated vertex count, ambiguous cells, and open and closed component
counts.

## Execution behavior and interface scope

The convenience method is deterministic and serial. The controlled method
accepts `ExecutionOptions` and `ExecutionControl`, rejects a requested thread
count above one before evaluation, and checks cancellation around every
analytic value query. Cancellation, fitted-field failure, allocation failure,
nonrepresentable work, degeneracy, refinement exhaustion, or invalid topology
returns no partial report.

Preparation and point-evaluation failures retain the exact
`FittedFieldEvaluationError<2>` as their `Error::source`. Forming either
structured isoline diagnostic stores that source inline and performs no heap
allocation of its own.

The CLI cannot consume an in-memory fitted model until the M8 versioned
project/model schema and loading workflow exist. C, C++, and Python parity are
M9 work; all later adapters must call this Rust implementation.
