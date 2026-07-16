# REQ-DIAG-001

Added a GeoRBF-owned structured diagnostic boundary for input, capability,
rank, gauge, contrast, infeasibility, conditioning, memory, cancellation, and
version failures. `GeoRbfError` carries validated category-specific evidence
and returns explicit `ErrorCode` values with stable numeric and symbolic
representations. Deterministic display text starts with the symbolic code, but
Rust enum layout, `Debug` output, and memory layout are not declared as a wire
format.

`DiagnosticPath` fallibly copies complete semantic observation provenance and
can attach a stable `LevelId`, or identify a level independently through its
semantic field path. Source paths retain the original input path and one-based
line, field path, stable observation and level identifiers, and optional
constraint group. Infeasibility evidence can retain multiple conflicting
sources without deleting or relaxing a hard constraint.

Category-specific evidence validates nonempty text, rank dimensions, nonzero
gauge components, distinct contrast levels, nonempty infeasibility sources,
finite positive conditioning limits and exceeded estimates, explicit memory
limits, and genuinely different versions. The common taxonomy is independent
of numerical backends; existing detailed CPD, dense-rank, residual,
factorization, assembly, and model diagnostics remain authoritative detailed
evidence in their owning layers.

Independent tests cover all ten error categories, stable codes and formatting,
complete observation-plus-level paths, invalid diagnostic construction,
`Send + Sync`, and a recursive source check that forbids stdout/stderr/debug
output macros in the safe Rust core. Rustdoc and the architecture contract are
synchronized.

Rust is implemented. CLI is N/A because the current stage-0 adapter has no
core operation whose diagnostics cross the command boundary. C, C++, and
Python are N/A until their M9 ABI and binding requirements; they will map the
stable core codes without reimplementing error policy. Benchmarking is N/A
because diagnostic construction is not a hot path. This requirement adds no
level DAG, gauge or contrast solver, infeasibility detector, execution-control
implementation, persistence schema, ABI symbol, Python module, numerical
dependency, hidden regularization, pseudoinverse, or constraint relaxation.
