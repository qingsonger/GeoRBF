# REQ-CENTER-001

Issue #120 defines the acceptance criteria and exclusions for rank-safe center
selection; Draft PR #121 carries the implementation for independent Review.
The Rust core now exposes one GeoRBF-owned
`CenterSelectionProblem<D>` for exactly D=1, D=2, and D=3. It validates finite
candidate locations, an aligned finite target-residual vector, and a finite
exact-symmetric row-major Gram matrix without exposing nalgebra types. The
constructor requires an explicit `KernelDefiniteness` classification and
accepts only strictly positive-definite input. A CPD declaration is rejected
with its positive order before selection or generic numerical review because
the raw matrix alone lacks the required `Q`, null-space, and projected-energy
evidence.

Five strategies are complete. All-representer preserves every input index.
User-provided preserves one explicit, nonempty, unique, in-range index
sequence. Seeded farthest-point traversal starts at `seed mod candidate_count`
and repeatedly maximizes minimum stable Euclidean separation. Seeded
residual-greedy chooses the largest current absolute interpolation residual;
seeded power-greedy chooses the largest current squared power. Exact score ties
use deterministic SplitMix64 keys derived only from the explicit seed and
candidate index, with no global state.

Residual- and power-greedy share a checked incremental Newton--Cholesky update.
Every candidate pivot must be finite and strictly greater than its own local
threshold `candidate_count * epsilon * abs(K_ii)`. Under an equivalent
nonzero basis scaling `K -> D K D`, the pivot and threshold both scale by
`D_ii^2`; an unrelated large diagonal therefore cannot reject an independent
basis member. A deficient pivot returns `InsufficientBasisRank` with selected
rank, requested rank, candidate, pivot, and the applied local threshold; the
implementation never jitters, substitutes a diagonal, regularizes, skips a
requested step, calls a pseudoinverse, or changes factorization.

Every returned strategy result receives a second, authoritative review. Its
selected principal Gram matrix passes through the existing eight-pass
equilibration, scale-aware RRQR screen, bounded SVD review, factor-16 ambiguity
guard band, and checked Cholesky under the caller's nonzero explicit memory
limit. Complete `DenseRankDiagnostics` are retained on success. Rank
deficiency, ambiguity, non-convergence, failed Cholesky, nonfinite arithmetic,
allocation failure, and a memory-limit failure remain structured errors with
their underlying evidence.

Selection is intentionally separate from semantic constraints, field
assembly, and fitted models. It returns stable indices and never drops an
observation, softens a hard relation, mutates `FieldProblem<D>`, fits
coefficients, or changes solver policy. This preserves the architecture
contract that field assembly does not select centers and prevents selection
from becoming an implicit semantic rewrite.

Thirteen independent integration tests cover all five analytic strategies,
stable user order, one-dimensional farthest truth, seeded exact-distance ties
and repeatability, diagonal residual and power truth, congruent basis-scaling
rank invariance, typed CPD rejection on the two-center `-r` fixture,
duplicate-basis Schur-pivot rejection, complete RRQR/SVD rank rejection,
Gram/target length mismatch, nonfinite Gram/target input, malformed indices,
insufficient candidate count, D=1/D=2/D=3 parity, and pre-dispatch memory
rejection. A runnable rustdoc example and deterministic five-strategy release
benchmark are included; Ready and `main` CI run the smoke workload on Windows,
Ubuntu, and macOS.

The final stable implementation head passed the complete standard workspace
gate on 2026-07-23: format, warning-denying all-target/all-feature Clippy,
all-feature workspace tests, workspace doctests, and the 58-requirement
registry check. The release benchmark smoke also passed after the last
production-code change.

Rust is implemented. CLI and versioned schemas are N/A until the M8 complete
CLI and persistence requirements. C, C++, and Python are N/A until the M9
adapter, parity, and API-freeze requirements; the C++ wrapper depends on the
future C ABI. Parameter tuning, topology, persistence, and general performance
work remain excluded. Fresh independent re-review, Ready-head CI, merge, and
isolated integration-state evidence remain required before the registry status
can become `integrated`.
