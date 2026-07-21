# REQ-ANISO-002

Issue #105 defines the acceptance criteria and exclusions for robust global
orientation-tensor estimation. The Rust core now accepts finite nonnegative
weighted `UnitDirection<D>` samples for exactly D=1, D=2, and D=3, requires at
least one strictly positive weight, and forms the normalized symmetric tensor
from axial outer products. Stable maximum-relative weight normalization and
compensated tensor sums accept common finite rescaling through extreme weights
without attaching meaning to direction sign.

The immutable result exposes the finite tensor, nonincreasing eigenvalues,
normalized eigenvalue shares, canonical-sign principal axes, normalized
adjacent eigengaps, per-axis confidence, and an explicit caller-thresholded
isotropy decision. The private symmetric eigendecomposition reuses the already
pinned nalgebra 0.35.0 dependency with machine-epsilon convergence resolution
and a recorded 64-iteration limit. Non-convergence, non-finite results, and
negative returned eigenvalues are errors; no clipping, jitter, pseudoinverse,
or hidden rank decision is applied. No nalgebra type crosses the public API.
This adds no dependency, feature, lockfile, license, MSRV, binary-graph, or
unsafe-audit surface beyond the accepted ADR-0009/ADR-0010 production pin.

`PrincipalAxisRatios<D>` are relative lengths in descending-eigenvalue order.
They must be finite, nonincreasing, at least one, exactly normalized to a final
value of one, and capable of forming represented normalized squared shares.
An estimator either retains fixed ratios or searches a caller-supplied finite
candidate set within an explicit finite maximum. It rejects empty, duplicate,
unordered, unnormalized, non-representable, or out-of-bound candidates rather
than sorting, scaling, clipping, or generating them.

Deterministic leave-one-out cross-validation fits axes without each positive
sample and scores candidate expected squared direction shares against held-out
squared projections. The lowest weighted Brier-style score wins; exact score
ties choose the lexicographically smaller ratios independent of candidate
order. At least two positive samples are required. Every candidate score is
retained.

Diagnostics also retain positive sample count, maximum normalized weight,
selection kind, selected maximum ratio, and every sample's rotation-invariant
leave-one-out tensor influence. Zero-weight samples have zero influence;
removing the only positive sample is explicitly assigned influence one because
no reduced estimate exists. The maximum influence and first matching sample
are reported for outlier review.

Independent public tests cover D=1/D=2/D=3 analytic truth, sign invariance,
rotation covariance away from degenerate eigenspaces, common and extreme
weight scaling, invalid weights, exact and near isotropy, eigengap confidence,
outlier influence, bounded and duplicate candidates, concentrated candidate
selection, exact tie-breaking, insufficient leave-one-out data, compile-time
D=4 rejection, and `Send + Sync`. A runnable example and deterministic D=3
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

The initial optimized Windows smoke measured approximately 7.39 us per
four-sample, three-candidate D=3 estimate over 2,000 estimates, with checksum
`1.00428812046557887e4`. This is a local regression signal, not a cross-machine
performance promise.

Rust and the focused benchmark are implemented. CLI is N/A until the M8
versioned schema and complete CLI requirements define persisted estimator
inputs. C, C++, and Python are N/A until M9 API freeze and adapter work. Model
persistence is not changed. Absolute correlation-length inference,
`GlobalAnisotropy` construction, local control compilation, local-trend
changes, observation assembly, field fitting, solvers, stochastic search,
schemas, persistence, and adapters are excluded. Independent Review and
integration remain required before the registry may become `integrated`.
