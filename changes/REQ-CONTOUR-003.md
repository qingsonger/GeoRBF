# REQ-CONTOUR-003

Issue #138 defines the acceptance criteria and exclusions for
three-dimensional isosurfaces. The Rust core adds a deterministic
`FittedField<3>::try_isosurface` API over an explicit finite
original-coordinate box.

The reference method uses a conforming six-tetrahedron Freudenthal split.
The topology-aware regular-grid method constructs cube-boundary loops from
canonical cube-edge crossings. Alternating-sign shared faces use a
scale-normalized bilinear asymptotic decision and a deterministic exact-zero
tie, so adjacent cells select identical face connectivity.

Only exact endpoint hits and true sign brackets form intersections. Brackets
use explicit bounded bisection. Global grid-vertex and grid-edge identities
deduplicate shared intersections without a hidden spatial radius, and only
exactly repeated undirected triangles are removed.

Every retained vertex receives an analytic original-coordinate fitted-gradient
unit normal. Triangle winding faces the positive-gradient side. Mesh edge
incidence, shared-edge traversal, boundary location, and connected components
are validated. Exact target-level sampled edges, unsupported exact-vertex
patterns, non-cyclic or multiple-loop cube graphs with underdetermined interior
connectivity, zero-gradient vertices, collapsed triangles, non-manifold edges,
inconsistent winding, and interior boundaries are structured failures with no
partial mesh.

Independent exact-CPD-polynomial tests cover transformed and identity plane
fields, a closed sphere, a saddle with shared-face ambiguity, both marching
methods, triangle orientation, exact-edge degeneracy, invalid settings and
domains, refinement exhaustion, cancellation, serial execution policy, and
work-budget overflow. The `georbf.isosurfaces.v1` benchmark extracts a fixed
24 by 24 by 24 sphere and is routed to Ready/main three-platform smoke CI.

Rust and benchmark surfaces are implemented. CLI is N/A because the stage-0
command cannot load an immutable fitted model; versioned schemas, persistence,
mesh export, and the complete contour command are owned by REQ-SCHEMA-001,
REQ-IO-001, and REQ-CLI-001. C, C++, and Python are N/A until their M9 adapter
and parity requirements. Arbitrary unstructured meshes, adaptive octrees,
mesh repair/Boolean operations, persistence formats, and adapter-side
mathematics are excluded.
