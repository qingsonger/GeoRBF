# One-Dimensional Level Points

`FittedField<1>::try_level_points` finds evidence for `f(x) = h` on one finite
interval in the fitted field's original coordinate convention. It evaluates
the immutable Rust model directly. No contour code refits the field, changes
coefficients, approximates derivatives, or duplicates kernel mathematics.

## Search contract

A `LevelPointRequest` supplies the finite target level, inclusive lower and
upper coordinates, and `LevelPointSettings`. The settings make every numerical
decision explicit:

- a nonzero uniform scan-interval count;
- a nonzero maximum bisection count per detected bracket;
- positive finite value, original-coordinate distance, and derivative
  tolerances.

Each requested scan interval is split at its midpoint. The report therefore
records twice the requested number of examined segments. A fitted value sign
change is a crossing bracket. A stationary bracket has opposite-sign analytic
original-coordinate derivatives at its endpoints, or collapses to one scan
node whose derivative is exactly zero. A derivative that is merely within the
requested tolerance remains stationary-candidate evidence and is not called a
bracket unless its neighboring derivatives actually change sign.
Bracket-preserving bisection stops only when the matching residual tolerance
or coordinate-width tolerance is satisfied; otherwise the explicit iteration
limit produces an error and no partial report.

The fitted field must declare gradients
`SupportedEverywhere`. GeoRBF rejects `SupportedAwayFromCenters` before the
first evaluation because a domain may contain an unsampled nondifferentiable
center and derivative-sign bisection requires a derivative defined throughout
every retained bracket.

The scan resolution is evidence, not a global root-count proof. A field may
oscillate more than once between adjacent scan nodes. Increase the explicit
interval count when the application's spatial frequency or independent
validation requires it.

## Returned evidence

`LevelPointReport::points()` is sorted by original coordinate and deduplicated
within the requested coordinate tolerance. Each point retains its fitted value,
level residual, analytic derivative, and one classification:

- `Boundary` for an exact requested domain endpoint;
- `Crossing` for an isolated non-stationary value root;
- `Stationary` for an at-level analytic stationary candidate, including a
  tangency.

`stationary_points()` also retains non-level extrema or tolerance-small
stationary candidates. Its `is_at_level` flag is based only on the explicit
value tolerance. `diagnostics()` exposes every value bracket, every justified
stationary bracket, and the actual fitted-field evaluation count.

When both endpoints of adjacent examined segments satisfy the value and
derivative tolerances, GeoRBF merges them into a
`DegenerateLevelInterval`. Such an interval represents a numerically
non-isolated level set at the selected scan resolution. GeoRBF reports its
bounds and maximum residual evidence and emits no arbitrary isolated point
inside it.

## Execution behavior

The convenience method is deterministic and serial. The controlled method
accepts `ExecutionOptions` and `ExecutionControl`, rejects a requested thread
count above one before evaluation, and checks cancellation around every
analytic value-and-gradient query. Cancellation, fitted-field failure,
allocation failure, nonrepresentable work, or refinement exhaustion returns no
partial report.

The CLI cannot consume an in-memory fitted model until the M8 versioned
project/model schema and loading workflow exist. C, C++, and Python parity are
M9 work; all later adapters must call this Rust implementation.
