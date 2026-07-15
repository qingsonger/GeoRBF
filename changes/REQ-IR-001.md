# REQ-IR-001

Added immutable provenance-preserving semantic and solver-neutral canonical
problem intermediate representations for exactly D=1, D=2, and D=3. Every
semantic constraint retains a stable observation identifier, source path and
one-based line, original units, field path, optional group, compiled
`ObservationFunctional`, relation, enforcement/loss metadata, and problem-level
execution options. Constructors reject empty or duplicate observations,
malformed bounds and cones, invalid soft parameters, non-finite values, count
overflow, and allocation failure without partial success.

An explicit caller linearizer maps each functional expression to a finite,
strictly ordered sparse affine expression over named variable blocks. This
keeps basis and kernel assembly outside the IR layer. Deterministic
canonicalization shifts constants exactly into equality and linear-bound right
hand sides, retains affine cone constants, and preserves complete row or cone
provenance. Canonical output records variable offsets, explicit identity
scaling, required equality/bound/cone capabilities, and a checked numeric
memory estimate without exposing nalgebra or any geological type.

Soft enforcement and its SquaredL2, AbsoluteL1, or Huber metadata are retained
but objective/epigraph compilation fails explicitly until later approved
requirements. The implementation introduces no solver, objective, kernel
matrix, polynomial augmentation, level DAG, geological observation compiler,
scaling, regularization, jitter, pseudoinverse, hidden variable, or constraint
relaxation.

Independent tests cover exact equality/bound/cone mappings, affine constant
shifts, provenance, deterministic order, D=1/D=2/D=3, malformed metadata and
relations, sparse ordering and indices, source-aware linearizer failures,
unsupported soft paths, allocation/count overflow, isolated fallible
provenance-copy failures for equality/bound/cone canonicalization,
unsupported-dimension compile failure, and `Send + Sync`. Canonical provenance
deep copies reserve every owned string fallibly and return structured
`AllocationFailed` diagnostics without exposing a partial result. Rustdoc, a
runnable example, normative architecture documentation, and a deterministic
benchmark are synchronized. CLI, C, C++, Python, schemas, and persistence are
N/A because no fitting, schema, fitted-model, or stable binding surface exists
yet.
