# REQ-CONTOUR-002

Issue #135 defines the acceptance criteria and exclusions for two-dimensional
isolines. The Rust core adds an immutable `FittedField<2>` extraction API over
an explicit finite original-coordinate rectangle. Callers select either a
fixed lower-left-to-upper-right marching-simplices reference or disambiguated
marching squares and provide nonzero grid/refinement limits plus positive
finite value and coordinate tolerances.

Every grid node is evaluated through the retained fitted-field value path.
Only exact endpoint hits and true value-sign-change brackets form
intersections. Brackets use explicit bracket-preserving bisection, and failure
to satisfy value or coordinate tolerance within the iteration limit returns no
partial report. Shared crossings are deduplicated by canonical grid-edge or
grid-vertex identity; only exact repeated undirected segments are removed.

Alternating-sign square cells use a scale-normalized bilinear asymptotic
decider. An interior saddle supplies the decision value, a non-interior saddle
falls back to the bilinear center, and an exactly zero value uses deterministic
positive connectivity. Each decision retains its cell, normalized value,
decision kind, and edge pairing.

The unique segment graph is accepted only with vertex degree at most two.
Every open endpoint must touch the requested rectangle within the explicit
coordinate tolerance and records all touched sides. Returned components are
deterministic open or closed polylines over one deduplicated original-coordinate
vertex array. Edges with two exact target-level endpoint samples are
conservatively underdetermined because the finite samples do not prove the
nonlinear edge interior; those edges, unsupported exact-vertex patterns,
non-manifold vertices, and interior endpoints are structured failures rather
than silently dropped or invented topology.

Independent exact-CPD-polynomial tests cover a transformed open line, a closed
circle, an exactly tied saddle, a nonzero asymptotic decision, the
marching-simplices reference, invalid settings and domains, an exact
non-isolated grid edge, refinement exhaustion, work overflow, cancellation,
serial-policy rejection, and progress semantics. The `georbf.isolines.v1`
benchmark exercises a fixed 64 by 64 circle extraction and is routed to
Ready/main three-platform smoke CI.

Rust and benchmark surfaces are implemented. CLI is N/A until an M8 versioned
model/project input can supply a fitted field. C, C++, and Python are N/A until
their M9 adapter and parity requirements. Three-dimensional isosurfaces,
schemas, persistence, mesh/contour exports, arbitrary unstructured meshes, and
adapter-side mathematics are excluded.
