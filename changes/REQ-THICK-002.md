# REQ-THICK-002

Added immutable post-fit sampled geometric thickness validation for exactly
D=1, D=2, and D=3. `SampledThicknessRequest<D>` owns distinct lower/upper
level identifiers and strictly increasing fitted scalar values, a positive
minimum thickness, selected original-coordinate points with provenance,
caller-requested quantiles, an explicit proposal opt-in, and finite positive
search, refinement, value, distance, and gradient thresholds.

For each location, the Rust implementation evaluates the fitted value and
original-coordinate gradient, forms a scale-safe unit normal, uniformly
brackets the lower target along the negative normal and the upper target along
the positive normal, then refines sign-changing brackets by deterministic
bisection. Complete measurements retain both intersections, the oriented
normal, line distance, sample index, point, and provenance. Low gradients,
missing intersections, and exhausted refinement limits are per-location
failures; fitted evaluation, allocation, geometry, overflow, and invalid input
remain structured top-level errors.

The validator prepares fitted-field polynomial scratch once per complete
validation and reuses it for every selected point, bracket step, and bisection
step. The sampled batch therefore performs no polynomial scratch allocation
per evaluation; report and sorting storage use checked bounded reservations.

Reports expose the minimum, caller-ordered deterministic type-7 quantiles,
violation locations, and optional proposed `LocalNormalThickness<D>` values.
Proposals are values for a separate explicit refit only. Validation never
mutates a field or problem, inserts a constraint, chooses a solver, relaxes a
hard constraint, regularizes, solves, or refits. The diagnostic label
`SampledGeometricValidation / SampledGeometricEvidence` remains distinct from
`ScalarLevelGap / ScalarOnly` and
`SampledLocalNormalCone / SampledLocalFirstOrder`.

Independent tests cover an exactly reproduced fitted parallel-level field,
analytic curved level sets with independently solved normal-line roots,
no-intersection reporting, minimum and quantile aggregation, invalid input,
violation and proposal provenance, repeat determinism, and D=1/D=2/D=3
`Send + Sync`. Module tests also cover scale-safe extreme gradient norms. A
documented Rust example and a deterministic 32-location fitted-field benchmark
are included, with Ready/main three-platform benchmark-smoke wiring.

Rust is implemented. CLI is N/A because the stage-0 command exposes only help
and version and complete project/schema commands belong to M8. C, C++, and
Python are N/A because their M9 requirements follow Rust API and schema freeze;
no adapter may reimplement search, quantiles, or proposal semantics. The
focused benchmark is implemented. This change adds no global minimum-distance
claim, automatic refit, new dependency, project/multi-field behavior,
persistence, adapter, release, or subsequent requirement work.
