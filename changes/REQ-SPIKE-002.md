# REQ-SPIKE-002

Evaluated current Rust-accessible RRQR and SVD options and accepted ADR-0009,
which selects the nalgebra 0.35 release line for later internal production rank
screening and review. The decision records maintenance, license, Rust 1.96.1
compatibility, platform strategy, unsafe exposure, dependency and binary size,
alternatives, deterministic behavior, explicit equilibration and rank
thresholds, and bounded SVD non-convergence handling.

Added an excluded, reproducible comparison crate pinned to faer 0.24.4 and
nalgebra 0.35.0. Six independent truth and property cases cover full rank,
exact deficiency, near-threshold behavior, nonzero diagonal scaling,
repeatability, and invalid inputs. A deterministic scaling workload records
32, 64, and 128 matrix baselines, while CI now runs the shorter spike lint,
test, and benchmark workload on the same platform matrix as the workspace.

The production workspace receives no numerical dependency, solver, public
matrix type, user API, pseudoinverse, hidden regularization, or fallback. Rust,
CLI, C, C++, and Python interface dispositions remain N/A for this dependency
spike. Production CPD rank enforcement, null-space construction, diagnostics,
and threshold-adjacent high-precision review remain in REQ-CPD-001.
