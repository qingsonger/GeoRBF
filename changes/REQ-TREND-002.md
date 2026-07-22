# REQ-TREND-002

Issue #108 defines the acceptance criteria and exclusions for regional and
reference-field local trend controls. The Rust core now exposes an immutable
compiler for exactly D=1, D=2, and D=3. It lowers one deterministic ordered
control list into the existing strictly positive-definite
`LocalTrendMixture<D>` with the caller's explicit constant background retained
as component zero.

Each control selects a configured kernel and either a spheroidal orientation
with one principal direction and axial/transverse lengths or an ellipsoidal
orientation with ordered orthogonal axes and paired lengths. Every metric is
constructed through `GlobalAnisotropy` under explicit condition policy. The
compiler does not sort or repair axes, infer lengths, or bypass the primitive's
structured CPD rejection.

Directions may be explicit unit values or normalized Cartesian gradients
sampled from one immutable `GeoProject` field at the control location. Stable
norm formation, a positive rejection threshold, a separate low-confidence
threshold, field identifiers, original gradient norms, and evaluation failures
are all retained explicitly. Missing or unknown fields and zero,
below-threshold, unavailable, or unrepresentable gradients fail without a
fallback direction or implicit refit.

Each control also supplies a nonzero representable signed strength, a positive
representable Gaussian influence radius, and an optional closed axis-aligned
region. The region uses the quintic smootherstep on both sides of each axis and
their product across axes. The compiled basis is exactly zero outside and on
the boundary, has zero boundary gradient and Hessian, and is C2 through all
faces, edges, corners, and plateau joins. The existing local-mixture evaluator
then supplies every kernel/weight product-rule term through query Hessian order.

Compiler diagnostics preserve resolved axes and lengths, explicit/reference
provenance, reference-gradient confidence, radii, strengths, regions,
per-control anisotropy condition numbers, sign-invariant adjacent direction
jumps and exceedances. Existing immutable background, condition-number, and
allocation-free point coverage diagnostics remain available from the compiled
mixture. The result and diagnostics are deterministic, immutable, and
`Send + Sync`.

Independent tests cover rotated spheroidal and ellipsoidal metrics against
hand-formed `B` matrices, deterministic control order, control coverage, exact
C2 region boundaries, diagonal and mixed finite-difference regional product
rules, explicit excessive-condition rejection, reference-gradient
normalization, provenance, confidence, and structured unknown/unavailable/zero/
unrepresentable failures, CPD rejection, D=1/D=2/D=3, compile-time D=4
rejection, and `Send + Sync`. Private numerical regressions verify that narrow
region transitions retain representable physical first and second derivatives
and that an exactly zero compact gate short-circuits an otherwise overflowing
Gaussian displacement through Hessian order. Further regressions retain an
amplitude-scaled regional value, gradient, and Hessian after the corresponding
unscaled gate underflows; accept a transition whose attained maximum C2
curvature remains finite; and skip an irrelevant overflowing fixed anisotropic
kernel when either the demanded compact query jet or compact center factor is
exactly zero. A separate public regression retains a represented regional
derivative whose ungated Gaussian value underflows. A runnable example
demonstrates regional compilation and Hessian evaluation.

Rust is implemented. CLI and versioned schemas are N/A until M8 defines the
persisted control representation and complete data CLI. C, C++, and Python are
N/A until the M9 adapter/API-freeze work. Model refit and persistence are N/A:
the compiler returns a local-mixture input and diagnostics without mutating or
refitting any field.

This change adds no dependency, unsafe code, arbitrary location-dependent
metric, hard region indicator, automatic control/axis/length/kernel selection,
solver work, hidden jitter, regularization, pseudoinverse, eigenvalue clipping,
or CPD side-condition workaround. Independent Review and integration remain
required before the registry may become `integrated`.

The deterministic release-mode smoke baseline on the development machine was
approximately 10.7 us/compilation for four controls and 38.7 us/compilation for
sixteen controls over 200 compilations per case. The benchmark prints a
deterministic checksum and is a regression signal rather than a cross-machine
performance promise.
