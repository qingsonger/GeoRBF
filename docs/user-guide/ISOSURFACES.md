# Three-Dimensional Isosurfaces

`FittedField<3>::try_isosurface` extracts an indexed triangle mesh for
`f(x, y, z) = h` from one finite box in the fitted field's original-coordinate
convention. The extractor evaluates the immutable Rust model directly. It
does not refit the field, change coefficients, approximate derivatives, or
duplicate kernel mathematics.

## Request and methods

An `IsosurfaceRequest` contains a finite target level, strict lower and upper
box corners, one `IsosurfaceMethod`, and `IsosurfaceSettings`. The settings
make all resolution decisions explicit:

- nonzero X, Y, and Z cell counts;
- a nonzero maximum bisection count for each crossed edge;
- positive finite fitted-value and original-coordinate tolerances.

`MarchingSimplices` is the reference path. It uses a globally conforming
Freudenthal split into six tetrahedra around the lower-to-upper body diagonal
of every cube. `TopologyAwareMarchingCubes` uses only regular cube edges. It
connects their crossings on cube faces and applies a scale-normalized bilinear
asymptotic decision to every alternating-sign face. Both cells adjacent to a
shared face therefore use the same samples and the same decision. An exact
zero decision has a deterministic positive-connectivity tie.

The grid is finite evidence, not a proof of completeness. Components smaller
than a cell or even-crossing behavior along one edge can remain unseen.
Increase the explicit resolution when application scale or independent
validation requires it.

## Intersections and degeneracy

Grid nodes and refinement points are evaluated through the fitted field. An
edge intersection exists only for an exact target endpoint or a true endpoint
sign change. Sign brackets use bracket-preserving bisection until the value or
coordinate tolerance is met. Exhausting the iteration limit returns an error
and no partial mesh.

Global grid vertices and grid-edge endpoint pairs are canonical identities.
Repeated cell and simplex observations of the same crossing are deduplicated
by identity. Only exactly repeated undirected triangles are removed; distinct
nearby vertices are never merged through an implicit geometric radius.

Two exact target endpoints make a sampled edge underdetermined: finite
endpoint samples cannot prove that the nonlinear edge lies entirely on the
target. GeoRBF reports `DegenerateGridEdge` instead of inventing an arbitrary
surface. Unsupported exact-vertex face or tetrahedron cases, non-cyclic cube
boundary graphs, and multiple cube-boundary loops whose interior connectivity
is not proved by face samples are structured failures. A center sample alone
is not treated as a general trilinear-topology proof. Collapsed triangles and
zero-gradient retained vertices are also rejected.

## Normals, winding, and topology

Every `IsosurfaceVertex` contains the refined original-coordinate point,
fitted value, target residual, and a unit normal obtained from the fitted
field's analytic original-coordinate gradient. A zero gradient cannot define
a normal and is rejected. `IsosurfaceTriangle` indices are wound so their
cross product points toward the positive-gradient side.

Topology validation counts every undirected triangle edge:

- incidence above two is non-manifold and rejected;
- incidence two requires opposite traversal by its two triangles;
- incidence one is accepted only when both vertices lie on a common requested
  box face within the coordinate tolerance.

Triangle connectivity produces deterministic `IsosurfaceComponent` records.
A closed component has edge incidence two everywhere. An open component
retains the sorted requested-box faces touched by its boundary edges.
Diagnostics include actual fitted evaluations, raw and unique triangle
counts, exact duplicates removed, deduplicated vertices, ambiguous face
decisions, boundary-edge count, and open/closed component counts.

## Execution behavior and interface scope

The convenience method is deterministic and serial. The controlled method
accepts `ExecutionOptions` and `ExecutionControl`, rejects a requested thread
count above one before field evaluation, and checks cancellation around every
analytic query. Allocation, arithmetic, cancellation, capability, refinement,
degeneracy, normal, or topology failure returns no partial report.

The stage-0 CLI cannot load a fitted model. Versioned model/project input and
mesh export belong to `REQ-SCHEMA-001`, `REQ-IO-001`, and `REQ-CLI-001`.
C, C++, and Python parity belongs to M9. Every future adapter must delegate to
this Rust implementation.
