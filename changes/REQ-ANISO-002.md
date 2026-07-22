# REQ-ANISO-002

Issue #105 defines the acceptance criteria and exclusions for robust global
orientation-tensor estimation. The Rust core now accepts finite nonnegative
weighted `UnitDirection<D>` samples for exactly D=1, D=2, and D=3, requires at
least one strictly positive weight, and forms the normalized symmetric tensor
from axial outer products. Stable maximum-relative weight normalization and
compensated tensor sums accept common finite rescaling through extreme weights
without attaching meaning to direction sign. Represented trace normalization,
fixed-capacity exact dyadic principal-minor certification, and bounded maximum
uniform off-diagonal retention preserve trace one and PSD when independent
component rounding would otherwise cross the semidefinite boundary, including
when an exact product lies below the minimum binary64 subnormal.

The immutable result exposes the finite tensor, nonincreasing eigenvalues,
normalized eigenvalue shares, canonical-sign principal axes, normalized
adjacent eigengaps, per-axis confidence, and an explicit caller-thresholded
isotropy decision. The private symmetric eigendecomposition of the certified
PSD tensor reuses the already pinned nalgebra 0.35.0 dependency. If that
backend returns a negative roundoff value, a bounded SVD of the same certified
matrix supplies right singular vectors and nonnegative singular values equal
to the PSD eigenvalues; diagnostics record the selected path and uniform
off-diagonal retention scale. Both use machine-epsilon convergence resolution
and a recorded 64-iteration limit.
Non-convergence, non-finite results, and impossible negative fallback spectral
values are errors; no eigenvalue clipping, diagonal jitter, pseudoinverse, or
hidden rank decision is applied. No nalgebra type crosses the public API.
This adds no dependency, feature, lockfile, license, MSRV, binary-graph, or
unsafe-audit surface beyond the accepted ADR-0009/ADR-0010 production pin.

`PrincipalAxisRatios<D>` are relative lengths in descending-eigenvalue order.
They must be finite, nonincreasing, at least one, exactly normalized to a final
value of one, and capable of forming represented normalized squared shares.
An estimator either retains fixed ratios or searches a caller-supplied finite
candidate set within an explicit finite maximum. It rejects empty, duplicate,
unordered, unnormalized, non-representable, or out-of-bound candidates rather
than sorting, scaling, clipping, or generating them. Validation covers both
the maximum-scaled square and its normalized share after division by the
represented square sum, so a positive axis contribution cannot silently
underflow out of candidate scoring.

Deterministic leave-one-out cross-validation fits axes without each positive
sample and scores candidate expected squared direction shares against held-out
squared projections. The lowest weighted Brier-style score wins; exact score
ties choose the lexicographically smaller ratios independent of candidate
order. At least two positive samples are required. Every candidate score is
retained.

Diagnostics also retain positive sample count, maximum normalized weight,
tensor correlation scale, spectral path, selection kind, selected maximum
ratio, and every sample's rotation-invariant leave-one-out tensor influence.
Zero-weight samples have zero influence;
removing the only positive sample is explicitly assigned influence one because
no reduced estimate exists. The maximum influence and first matching sample
are reported for outlier review.

Independent public tests cover D=1/D=2/D=3 analytic truth, sign invariance,
rotation covariance away from degenerate eigenspaces, common and extreme
weight scaling, minimum-subnormal PSD preservation, invalid weights and ratio-
share underflow, exact and near isotropy, eigengap confidence, outlier
influence, bounded and duplicate candidates, concentrated candidate selection,
exact tie-breaking, insufficient leave-one-out data, compile-time D=4
rejection, and `Send + Sync`. A serial internal audit covers sample-count-
independent allocation behavior. A runnable example and deterministic D=3
cross-validation benchmark cover the public workflow.

The PR #106 Repair makes leave-one-out selection invariant when a training
fold contains a repeated or numerically unresolved eigenspace: candidate loss
compares the total observed and expected share within each maximal adjacent
eigenvalue group resolved at `64 D epsilon`, rather than projections onto an
arbitrary basis inside that group. A global-rotation regression checks all
candidate scores and the selected ratios. The Repair also enforces the public
influence range through an explicit `64 D^2 epsilon` upper roundoff band:
values inside the band are recorded as one and larger overshoots are structured
errors. An extreme finite-weight regression checks every per-sample influence
and the aggregate maximum. A subsequent Repair preserves exact total mass in
the represented grouped loss by assigning both observed and expected residual
mass to the final eigenspace group. A public D=3 regression with one
`f64::MAX`-weighted direction and three unit-weight coordinate axes verifies
the independently derived candidate-score ordering and selection when the
dominant leave-one-out fold is fully unresolved.

The ANISO002-REV-004 Repair adds a public single-sample D=2 regression for the
valid direction proportional to `[1,30]`. Before repair, independently rounded
outer-product entries had determinant `-2.168404344971009e-19`, and the former
symmetric eigendecomposition rejected the valid sample with eigenvalue
`-1.1089908126111444e-16`. The repaired construction restores represented
trace one, certifies every principal minor exactly, and only when required
retains the maximum bounded uniform off-diagonal
correlation factor. When symmetric-eigensolver roundoff remains negative for
that certified matrix, the bounded PSD SVD obtains nonnegative eigenvalues
without clipping or jitter and the diagnostic records the fallback. The
regression checks successful estimation, represented trace one, a nonnegative
D=2 determinant, nonnegative spectral values, a sub-unit correlation scale,
and the explicit fallback path.

The ANISO002-REV-005/006/007 Repair preserves the exact dyadic sign of every
D=2/D=3 represented principal minor by accumulating bit-decomposed binary64
significands in fixed-capacity integer limbs across the complete product and
triple-product exponent range. A public `[1, f64::from_bits(1)]` regression
now forces the represented off-diagonal to the maximum retained PSD boundary
instead of accepting determinant `-2^-2148`. Ratio validation rejects
`[2^537,2^537,1]` because division by its represented square sum would erase
the positive final normalized share. Finally, weight normalization uses
allocation-free two-pass scalar state, fixed D=1/D=2/D=3 nalgebra matrices
replace heap-backed fold matrices, and fixed arrays replace spectral sorting
and axis-collection vectors. Only a fixed number of owned
result/candidate-work vectors remain.

The ANISO002-REV-008/009 Repair replaces fixed-count real-number correlation
bisection with a finite search over the ordered positive binary64 scale bit
patterns from zero through one. The public D=2 `[1,2^-538]` regression reaches
the greatest certified represented scale `2^-537`, rounds both off-diagonal
entries to zero, retains trace one, and returns a nonnegative spectrum. A fresh
re-review then showed that exact represented D=3 determinant acceptance can be
nonmonotone because the three correlations round independently. The final
ANISO002-REV-008 Repair therefore bisects only the monotone order-two minors,
then searches the remaining D=3 bit interval from high to low with exact dyadic
determinant upper bounds. It prunes an interval only when its greatest possible
determinant is proved negative, so the first accepted candidate is maximal
without assuming determinant monotonicity. The reviewed single-sample direction
`[0.2929103819395529, 0.39358823180141855, -0.3403261034581484]` now retains
scale `1.0.to_bits() - 1`; exact tests cover accepted, rejected, accepted, and
rejected states at the final four scale patterns.

The ANISO002-REV-008/009 Repair also removes every manual allocation annotation.
A dedicated single-test integration binary constructs and warms the inputs
outside the measured region, then observes actual allocator calls around only
`try_estimate`: fixed-ratio estimation performs two allocations for both four
and sixteen samples, while cross-validated estimation performs five for both.
This supplies the remaining independent regression evidence for
ANISO002-REV-007 without changing the production allocation path.

Actual allocation observation adds the exactly pinned test-only
`allocation-counter` 0.8.1 crate. It has no transitive dependencies, is dual
MIT/Apache-2.0 licensed, is 29.58 kB of packaged source, uses Rust 2021, and
builds on the pinned Rust 1.96.1 toolchain; it declares no lower MSRV. The
repository is unarchived but low-maintenance (last pushed 2023-09-21, one open
issue, no open pull request when reviewed) and the stable API is narrowly
scoped to synchronous thread-local measurement. The unsafe audit found one
private `GlobalAlloc` implementation that counts through thread-local fixed
state and delegates allocation and deallocation directly to
`std::alloc::System`; no unsafe code enters GeoRBF. Focused tests pass on
Windows; Draft CI will exercise Ubuntu, while the later ready gate will cover
macOS. The crate does not enter production artifacts or change their size.
Handwritten allocator instrumentation was rejected because it would violate
GeoRBF's unsafe policy; OS heap telemetry is not portable, and the removed
manual annotations could not observe unmarked allocations. This non-numerical
test helper needs no ADR.

The initial optimized Windows smoke measured approximately 7.39 us per
four-sample, three-candidate D=3 estimate over 2,000 estimates, with checksum
`1.00428812046557887e4`. This is a local regression signal, not a cross-machine
performance promise. The ANISO002-REV-004 exact-sign certification and explicit
spectral fallback policy retained that checksum at approximately 9.20 us per
estimate locally. The ANISO002-REV-005/006/007 fixed-size spectral and
allocation repair retained the per-estimate checksum contribution at
approximately 5.15 us per estimate over 100,000 estimates locally.

Rust and the focused benchmark are implemented. CLI is N/A until the M8
versioned schema and complete CLI requirements define persisted estimator
inputs. C, C++, and Python are N/A until M9 API freeze and adapter work. Model
persistence is not changed. Absolute correlation-length inference,
`GlobalAnisotropy` construction, local control compilation, local-trend
changes, observation assembly, field fitting, solvers, stochastic search,
schemas, persistence, and adapters are excluded. Independent Review and
integration remain required before the registry may become `integrated`.
