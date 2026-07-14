# REQ-FUNC-001

Added the two v1 atomic scalar-field functionals, `Value` and
`DirectionalDerivative`, plus nonempty finite `FunctionalExpr` combinations
for exactly D=1, D=2, and D=3. Every term retains an opaque stable caller
provenance identifier and deterministic insertion order. Points and unit
directions reuse the validated geometry layer; coefficients, scalar-field
samples, allocations, shapes, polynomial actions, and accumulated results fail
with structured diagnostics instead of partial success.

The Rust API applies expressions to explicitly aligned value/gradient samples,
to every member of the integrated complete polynomial space, and to Cartesian
kernel jets supplied per term pair. Query and center derivatives delegate their
signs to the shared kernel calculus, including value/derivative,
derivative/value, derivative/derivative, exchange-identity, and analytic center
limits. Distinct `ObservationFunctional` and `CenterRepresenter` wrappers keep
the two architectural collections type-separated without introducing an
observation relation or solver row.

Independent analytic-field, polynomial, linearity, derivative-sign, center,
exchange, provenance, allocation/error, unsupported-dimension, and
thread-safety tests accompany synchronized rustdoc, a runnable example, and a
deterministic D=1/D=2/D=3 benchmark. CLI, C, C++, Python, schemas, and model
persistence are N/A because no problem schema, fitted model, or stable binding
surface exists. Semantic observations, constraints, CPD rank enforcement,
assembly, fitting, and solvers remain outside this atomic requirement.
