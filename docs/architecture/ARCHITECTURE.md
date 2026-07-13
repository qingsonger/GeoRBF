# Software Architecture

## Workspace boundaries

- `crates/georbf`: safe Rust mathematical core and all domain compilation.
- `crates/georbf-cli`: command and file adapter only.
- `crates/georbf-ffi`: smallest stable C ABI boundary and the only future
  location for narrowly justified unsafe code.
- `crates/georbf-python`: PyO3/NumPy adapter to the core.
- `xtask`: repository checks and release engineering.

The C++ API is a header-only RAII wrapper around the C ABI. No adapter
reimplements constraint compilation, assembly, solving, or evaluation.

## Core layers

The planned core modules are dimension, geometry, units, coordinates,
orientation, transform, kernel, kernel calculus, polynomial, functional,
observation, levels, anisotropy, problem, semantic and canonical IR, basis,
assembly, solver, model, contour, I/O, diagnostics, and structured errors. They
remain in one strong crate until evidence justifies a split.

Dependencies point inward: geometry and kernel calculus know no geology;
coordinate metadata and transforms depend only on validated geometry and units;
problem compilation depends on functionals and semantic concepts; assembly
depends on the canonical representation; solvers know only numerical forms;
models own immutable coefficients and transforms; adapters depend on public
core APIs.

## Runtime behavior

Long operations accept cancellation, progress, explicit thread count, and
determinism through interfaces. The core emits no stdout or stderr output.
User input returns structured errors rather than panicking. A fitted model is
immutable, `Send + Sync`, independent of its builder, and deterministic to
serialize.

Dense assembly computes only required symmetric work in blocks and reuses
per-thread storage. Compact-support paths use a neighborhood index and sparse
storage. Performance changes are accepted only with fixed-data baselines and
documented hardware and thread settings.
