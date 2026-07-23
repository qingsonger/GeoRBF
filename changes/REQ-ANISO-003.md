# REQ-ANISO-003

Issue #111 defines the acceptance criteria and exclusions for exportable local
anisotropy diagnostics. The Rust core now produces one deterministic owned
`AnisotropyDiagnosticExport<D>` from an immutable
`CompiledTrendControls<D>` for exactly D=1, D=2, and D=3. Export does not alter
controls, refit a field, regularize a metric, or change the strict-background
positive-definiteness proof.

Control records preserve caller order, control and mixture-component indices,
positions, signed strengths, influence radii, optional compact C2 regions,
fixed-metric condition numbers, and sign-invariant adjacent direction jumps
with explicit policy flags. The tagged orientation record preserves a
spheroid's unique principal axis plus axial/transverse lengths without
inventing transverse axes, while ellipsoids retain every ordered resolved axis
and paired length. Direction provenance and confidence remain attached.

Strict-background evidence includes the component index, constant weight
magnitude, caller minimum, policy ratio, and background fixed-metric condition
number. Summary evidence includes the mixture-wide maximum condition number,
maximum adjacent jump, low-confidence reference-direction count, and jump
exceedance count.

Caller-supplied sample positions produce signed component weights in mixture
order plus the existing aggregate squared-weight coverage, background square,
active-component count, and operational-domain membership. Non-representable
weight or coverage evaluation retains sample/component context in a structured
error. All owned output vectors use fallible reservation.

Low-confidence reference-field directions export source-aware region records
in control/axis order with positions, optional compact regions, field IDs, and
original gradient norms. Explicit directions do not produce such records, and
an absent compact region remains explicit instead of becoming an arbitrary
box.

Independent tests cover the public diagnostic record contract, hand-computed
signed Gaussian weights and coverage, strict-background and condition evidence,
spheroidal/ellipsoidal fidelity, antipodal jump invariance and threshold
exceedance, source-aware low-confidence compact regions, explicit-direction
exclusion, structured evaluation failure, D=1/D=2/D=3, compile-time D=4
rejection, and `Send + Sync`. A runnable example demonstrates format-neutral
point sampling. The benchmark obligation is N/A because diagnostic export is
an explicit owned operation outside evaluation hot paths.

Rust is implemented. CLI and versioned schemas are N/A until M8 persistence
and complete-CLI requirements. C, C++, and Python are N/A until M9 adapter and
API-freeze requirements. GUI, VTK, persistence encoders, topology inference,
new numerical dependencies, unsafe code, hidden jitter, clipping,
regularization, and pseudoinverses are outside this requirement. Independent
Review and integration remain required before the registry may become
`integrated`.
