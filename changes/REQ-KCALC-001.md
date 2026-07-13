# REQ-KCALC-001

Added a geology-free radial kernel calculus for exactly D=1, D=2, and D=3.
Validated point pairs use an overflow/underflow-resistant Euclidean separation,
and caller-supplied radial jets expand through value, gradient, Hessian, and
third Cartesian derivative tensors. Query and center argument derivatives use
the exact stationary-kernel sign convention.

Coincident points take a separate analytic path: a smooth center jet produces
its supplied value, zero gradient, `phi''(0) I`, and a zero third tensor without
evaluating any radial quotient. Non-finite radial inputs, non-representable
separations or tensor components, and center/away mismatches return structured
errors.

Independent polynomial truth, finite differences, center limits, exchange
signs, exact tensor symmetries, numerical extremes, error paths, compile-fail
dimension checks, and thread-safety traits are covered. A runnable Rust example
and a deterministic dependency-free D=1/D=2/D=3 benchmark document the public
path and performance baseline.

Concrete kernel families, kernel metadata, anisotropy, geological orientation,
functionals, assembly, solvers, language bindings, and Surfe compatibility
remain out of scope.
