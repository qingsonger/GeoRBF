# REQ-CONTOUR-001

Issue #132 defines the acceptance criteria and exclusions for one-dimensional
level points. The Rust core adds an immutable `FittedField<1>` extraction API
over an explicit finite original-coordinate domain. It midpoint-splits the
requested uniform scan, retains analytic fitted-value and original-coordinate
derivative brackets, and refines them with bracket-preserving bisection under
explicit value, coordinate, derivative, and iteration tolerances.

Returned isolated points are sorted and deterministically deduplicated. Every
point retains fitted value, target residual, analytic derivative, and
boundary/crossing/stationary classification. A separate ordered stationary
view retains at-level tangencies and non-level stationary evidence. Adjacent
segments whose endpoint values and derivatives both satisfy the requested
tolerances are merged into degenerate level intervals; no arbitrary point is
emitted for that non-isolated solution set.

The operation uses the fitted model's existing analytic value-and-gradient
path and original-coordinate chain rule. It performs no finite differences,
implicit fit, coefficient mutation, hidden adjustment, topology
reconstruction, schema I/O, or adapter-side mathematics. Its controlled form
is deterministic and serial, rejects unsupported thread counts and gradients
that are not supported everywhere before evaluation, reports monotonic
progress, checks cancellation around every fitted-field query, and returns no
partial report on failure. Stationary diagnostics contain only actual
derivative sign brackets or exact-zero scan nodes; a merely tolerance-small
sample remains candidate evidence without being mislabeled as a bracket.

Independent CPD-polynomial truth tests cover transformed crossing roots,
reflected original-coordinate derivative signs, non-level stationary evidence,
an at-level tangency, exact domain-boundary deduplication, a constant
degenerate level interval, invalid settings, work-budget overflow,
away-from-centers gradient-capability rejection, a complete CPD cubic with a
tolerance-small but strictly positive derivative, refinement exhaustion,
cancellation, serial-policy rejection, and progress semantics. The
`georbf.level_points.v1` benchmark exercises a fixed quadratic model and is
routed to Ready/main three-platform smoke CI.

Rust and benchmark surfaces are implemented. CLI is N/A until an M8 versioned
model/project input can supply a fitted field. C, C++, and Python are N/A until
their M9 adapter and parity requirements. Two-dimensional isolines,
three-dimensional isosurfaces, schemas, persistence, contour exports, and
topology are excluded.
