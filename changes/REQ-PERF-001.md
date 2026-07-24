# REQ-PERF-001

Issue #129 defines the acceptance criteria and exclusions for the dense and
sparse performance baseline. Dense all-representer assembly now traverses
fixed 32-by-32 upper-triangle representer blocks. It evaluates each diagonal
or upper entry exactly once, reflects every off-diagonal value exactly once,
and retains the same exact symmetry review and hard-constraint mathematics.
Assembly diagnostics record the block edge, upper-block count, unique kernel
actions, and reflected entries.

The Rust core adds a GeoRBF-owned reusable
`FittedFieldEvaluationWorkspace<D>` and ordered value/gradient batch APIs for
exactly D=1, D=2, and D=3. A caller may reuse one workspace and output vector
for serial batches, or request a fallible convenience batch with explicit
nonzero thread count and memory limit. Multithreaded batches split the input
into deterministic contiguous ranges, give each scoped worker its own scratch,
write disjoint output ranges, preserve input order, and never configure a
global thread pool. A worker creation failure or panic is structured; a query
failure retains its exact input index and returns no partial batch.

Sparse workspaces reserve the complete retained-center capacity before the
first query, so exact rstar candidate lookup, independent support filtering,
sorting, and deduplication cannot grow the center-index vector per point.
Dense batches visit every center. Sparse batches visit only exact supported
centers and retain their existing center-count evidence in every result.
Repeated serial calls with warmed workspace and output capacity allocate zero
times. The convenience path makes a constant two allocations for a dense
single-worker batch, independent of whether the input contains four or 256
queries.

`BatchEvaluationMemoryDiagnostics` checks every byte product and sum before
allocation. It records output bytes, scratch bytes per worker, total worker
scratch, effective nonempty workers, the logical peak, and the explicit
caller limit. The portable estimate intentionally excludes operating-system
thread stacks and standard-library thread bookkeeping; no excluded memory is
misreported as GeoRBF-owned payload. A limit below the estimate fails before
workspace or output allocation.

Independent integration tests cover D=1/D=2/D=3 dense scalar parity,
bit-identical one/four-worker results, exact 33-center block counts and
symmetry, sparse locality and thread determinism, exact dense and sparse
workspace formulas, pre-allocation memory rejection, empty batches,
incompatible workspace rejection, zero-allocation warmed reuse, and
batch-length-independent allocation counts. The versioned
`georbf.performance.v1` benchmark uses fixed D=3 dense and compact-sparse
models, fixed ordered queries, one/two/four workers, exact center-visit counts,
checked memory, and full value/gradient checksums. Ready and `main` CI run its
smoke workload on Windows, Ubuntu, and macOS.

Rust is implemented. CLI is N/A until the M8 complete fitting and evaluation
CLI. C, C++, and Python are N/A until the M9 adapter, parity, and API-freeze
requirements; the C++ wrapper depends on the future C ABI. The benchmark
surface is implemented. No backend, dependency, solver policy, regularization,
schema, CLI, adapter, contour, or release behavior changes. Independent
Review, exact Ready-head CI, merge, and isolated integration-state evidence
remain required before the registry status can become `integrated`.
