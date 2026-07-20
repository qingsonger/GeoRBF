# REQ-INFEAS-001

Added an immutable, source-aware canonical constraint review for hard affine
equalities and linear-bound rows. Each nonzero sparse row is independently
infinity-normalized and compared in both direct and sign-reversed orientations.
Exactly proportional rows are diagnosed as duplicates. Distinct rows whose
dimensionless infinity distance is at most the explicit `128 * epsilon`
threshold are diagnosed as near duplicates. Pair ordering is deterministic;
each entry retains both relation families, both complete source provenances,
the selected orientation, measured distance, and comparison threshold.

The review is deliberately non-mutating. It does not delete, merge, reorder,
rescale, perturb, regularize, or soften a canonical relation. Soft objectives
are counted and excluded from hard-feasibility decisions. Hard ordered cones
are counted but are not reinterpreted as affine functionals. Allocation and
provenance-copy failures remain structured `ProblemIrError` values rather than
partial reports.

Canonical exact-conflict review now covers all hard affine relation pairs.
Constant equalities with nonzero right-hand sides fail with their complete
source. An equality is treated as a singleton closed interval, a linear bound
as its supplied interval, and exactly proportional rows are transformed only
for comparison into the earlier row's orientation. A disjoint intersection
returns source-aware interval evidence. Positive row scales `1e-12`, `1`, and
`1e12` produce the same conflict decision. Approximate rows never prove
infeasibility. General multi-row and cone infeasibility remains on the existing
Clarabel path and is accepted only after GeoRBF's independent original-unit
dual-certificate review.

Eight independent integration tests cover exact duplicates, sign-reversed near
duplicates, deterministic ordering, unchanged canonical inputs, equality/bound
and constant-equality conflicts, positive scalar-unit rescaling, soft-objective
exclusion, D=1/D=2/D=3, `Send + Sync`, and a three-row infeasible system with a
source-complete independently reviewed certificate. Existing problem-IR,
linear-semantic, and convex-solver suites remain green. A runnable example and
a deterministic 96-constraint benchmark accompany Rustdoc and the normative
constraint and architecture documents.

The bounded repair for review findings R85-001 and R85-002 replaces rounded
divide-and-multiply proportionality with exact cross-products of the binary
coefficient values. The same representation compares proportional interval
endpoints without materializing a quotient, so finite overflow and underflow
examples remain exact conflicts. Independent public-canonicalization
regressions cover the one-ULP nonparallel pair, the exactly proportional
`[1, 7, 13]` and `[49, 343, 637]` pair, and both extreme single-variable
interval conflicts. No canonical row, solver policy, dependency, or interface
was changed.

Rust is implemented. CLI is N/A because the stage-0 command exposes only help
and version and project/schema commands belong to M8. C, C++, and Python are
N/A because their M9 requirements follow Rust API and schema freeze; no adapter
may reimplement constraint classification or certificate review. The focused
benchmark is implemented. No dependency, backend setting, schema, unsafe code,
geological cone semantic, solver fallback, hidden regularization, constraint
relaxation, release, or later-requirement work was added.
