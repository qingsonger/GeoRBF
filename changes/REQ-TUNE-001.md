# REQ-TUNE-001

Issue #126 defines the acceptance criteria and exclusions for deterministic
bounded parameter tuning. The Rust core exposes one GeoRBF-owned
`TuningProblem<D>` for exactly D=1, D=2, and D=3. It owns immutable observation
locations, explicit candidates, and inclusive bounds without exposing a
third-party numerical type or mutating a semantic problem.

Each candidate may tune kernel length, compact-support radius, explicit
regularization, axis ratio, and local-control influence radius. Active
parameters must appear in every candidate and have finite physical-domain
bounds. Missing, unbounded, out-of-bound, and exact duplicate candidates fail
before scoring. The engine never generates an unrequested candidate or
regularization amount.

Five strategies are complete. Fixed selection returns one exact caller index
without evaluating alternatives. The distance heuristic computes a stable
Euclidean nearest-neighbor distance for every observation, takes their median,
and minimizes mean squared log-ratio error over active length-like values using
finite log differences rather than an overflow-prone quotient. Cross-validation
requires between two folds and the observation count, builds seeded nonempty
round-robin folds, and minimizes total weighted held-out squared error divided
by total weight. Its diagnostics retain each exact weighted squared error and
weight. Generalized cross-validation minimizes canonical
`n * RSS / (n - effective_dof)^2` after enforcing `0 <= effective_dof < n` and
one common observation count across candidates. Power-function selection
minimizes the maximum nonnegative squared power over a positive sample count.

The caller implements `TuningEvaluator` so the already-established field,
kernel, and solver APIs remain the sole source of actual fitting. An evaluator
failure rejects the whole search: no candidate is skipped and no alternative
criterion is selected. Every evidence scalar and derived score is checked for
finiteness and physical validity. Exact best-score ties use SplitMix64 keys
derived only from the explicit seed and candidate index. Complete diagnostics
retain the criterion, seed, bounds, folds, every candidate score and
criterion-specific evidence, and exact tie count.

Independent integration tests cover the five strategy truths, a known
cross-validation optimum, auditable unequal fold weights, canonical GCV
arithmetic and common-count rejection, extreme finite distance scales, seeded
repeatability and seed-controlled ties, every supported parameter and bound,
D=1/D=2/D=3 parity, malformed bounds and candidates, duplicate locations,
insufficient or excessive folds, nonfinite or invalid evaluator evidence, and
fail-fast evaluator errors. An allocation-counting unit regression proves the
pre-reserved nearest-distance and fold-order sorts allocate no auxiliary
storage. A runnable rustdoc example and deterministic five-strategy release
benchmark are included, and Ready/main CI includes its smoke workload.

Rust is implemented. CLI and versioned schemas are N/A until the M8 complete
CLI and persistence requirements. C, C++, and Python are N/A until the M9
adapter, parity, and API-freeze requirements; the C++ wrapper depends on the
future C ABI. Continuous optimization, implicit fitting, automatic
regularization, probabilistic GP features, and interface-milestone work remain
excluded. Independent Review, Ready-head CI, merge, and isolated integration-
state evidence remain required before the registry status can become
`integrated`.
