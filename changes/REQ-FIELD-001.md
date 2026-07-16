# REQ-FIELD-001

Added public `FieldProblem<D>` and immutable dense hard-equality assembly for
exactly D=1, D=2, and D=3. Provenance-bearing semantic observations and center
representers remain separate types and collections. The all-representer path
requires exact same-index expression alignment and hard equality enforcement,
then evaluates only the upper kernel triangle and reflects it explicitly.

Assembly uses kernel metadata to reject unsupported combined value or
directional-derivative demand before evaluation, including away-only
derivatives at coincident points. The evaluator callback receives the exact
query/center points and demand, so concrete radial kernels and accepted global
anisotropy remain outside the assembly module. Kernel, contraction,
polynomial, canonicalization, allocation, count, dimension, and capability
failures return structured diagnostics without partial success.

Strictly positive-definite kernels produce the symmetric center-weight system.
CPD metadata automatically generates the complete polynomial space, assembles
distinct observation and center polynomial actions, performs the existing
scale-aware RRQR/SVD rank and null-space review, appends `Q^T w=0` rows, and
records `Z^T K Z`. The complete augmented matrix receives a recorded
`64*n*eps` normalized symmetry review. No solver, jitter, regularization,
pseudoinverse, hidden constraint relaxation, center selection, sparse path, or
fitted-model API is introduced.

Independent tests compare mixed-functional D=1/D=2/D=3 matrices with closed-
form Gaussian value, gradient, and mixed-Hessian actions; verify analytic linear
field right-hand sides, bit-symmetric reflection, CPD complete-polynomial and
null-space evidence, role-mismatch rejection, query/center signs, and
nonsmooth-center capability rejection before evaluator dispatch. Rustdoc, a
runnable example, deterministic benchmark, CI smoke route, mathematical and
architecture contracts, registry, and bounded handoff are synchronized.

Review repair removes allocation from kernel-action error mapping, reuses one
fallibly allocated value/gradient scratch pair across every CPD observation,
and precisely distinguishes variable-block collection allocation failures.
Regressions cover allocation failpoints and constant scratch allocation,
coincident invalid capability in D=1/D=2/D=3, the complete independent 5-by-5
CPD augmented matrix, both rank decisions, `Q^T Z`, orthonormality, and the
projected scalar `Z^T K Z=4/3`.

CLI, C, C++, and Python are N/A because REQ-SOLVE-001 has not yet introduced a
solver, REQ-MODEL-001 has not introduced fitted models, and schema/binding
requirements have not frozen a user-facing fitting surface. All mathematics
remains in the Rust core.
