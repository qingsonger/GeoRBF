# REQ-TANGENT-001

Added immutable D=1/D=2/D=3 Rust tangent semantics. Each validated unit tangent
at a finite point lowers independently to `t^T grad f(x) = 0` through the
existing directional-derivative functional and solver-neutral semantic IR.
Hard enforcement remains hard; scalar SquaredL2, AbsoluteL1, and Huber losses
retain their explicit positive residual scale. Multiple independently
identified tangents may share a point and preserve caller order and complete
provenance. The canonical solver sees only equality rows or soft equality
objectives, never tangent vocabulary.

`TangentProblem<D>` collects a nonempty tangent-only field and requires one
caller-supplied `DerivativeGaugeAnchor<D>`. The anchor is the hard equality
`f(x) = value`, is appended after all tangent rows, and records its point,
finite value, and stable identifier. Omitting it returns the existing
source-aware `GEORBF-E4001` missing-gauge diagnostic against the first tangent.
No automatic point or zero value is selected, and no hard constraint is
relaxed. Duplicate tangent/gauge identifiers, invalid soft metadata,
non-finite gauge values, count overflow, and allocation failure return
structured errors without partial success.

Eight independent integration tests cover analytic directional rows, two
independent tangents at one point, all three scalar soft losses, deterministic
gauge recording, the missing-gauge source and error code, invalid metadata,
duplicate identifiers, dimensional bounds, immutability, and `Send + Sync`.
They also prove that a missing gauge inspects only the first item of an
unbounded tangent iterator before returning `GEORBF-E4001`, while the same
iterator with an explicit gauge retains structured count overflow. Two unit
regressions cover the stable diagnostic display and fallible final problem
storage. A runnable example and deterministic D=3 tangent-plus-gauge compilation
benchmark accompany Rustdoc and the normative mathematics.

Rust is implemented. CLI is N/A because the stage-0 command exposes only help
and version and complete project/schema commands belong to M8. C, C++, and
Python are N/A because their M9 requirements follow Rust API and schema freeze;
no adapter may reimplement tangent or gauge semantics. The focused benchmark
is implemented. No automatic gauge, normal, thickness, multi-field,
persistence, local anisotropy, adapter, release, or later-requirement work was
added.
