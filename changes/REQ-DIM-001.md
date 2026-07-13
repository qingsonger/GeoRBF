# REQ-DIM-001

Added const-generic `Point`, `Vector`, `Direction`, and `UnitDirection` Rust
types for exactly one, two, and three dimensions. Private representations and
fallible constructors enforce finite components; direction types additionally
enforce a nonzero value.

Unit directions use maximum-component scaling before evaluating the Euclidean
norm, preventing overflow for values near `f64::MAX` and preventing the norm
from collapsing for very small finite values. Integration tests cover all
supported dimensions, invalid values, zero directions, conversions, extreme
magnitudes, scale invariance, and thread-safety traits. Compile-fail doctests
cover unsupported zero- and four-dimensional points, and a runnable example
documents the public construction path.

Coordinate metadata, units, orientation semantics, vector arithmetic, kernels,
solvers, language bindings, and Surfe compatibility remain out of scope.
