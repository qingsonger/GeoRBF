# REQ-MODEL-001

Added a public immutable `FittedField<D>` for exactly D=1, D=2, and D=3. One
high-level fit call consumes a normalized-coordinate `FieldProblem<D>`,
coordinate metadata, affine normalization, a concrete configured kernel,
optional constant global anisotropy, and an explicit dense-solve policy. The
same retained kernel definition drives assembly and later evaluation, so a
callback cannot assemble one kernel and persist another.

The fitted field owns center representers, one deterministic coefficient
vector, the complete CPD polynomial space when required, exact
value/gradient/Hessian capabilities, and complete assembly and solve
diagnostics. It retains no semantic builder, canonical problem, dense matrix,
right-hand side, factorization, or solver workspace. All public model state is
immutable and composed of `Send + Sync` data.

Original-coordinate query points are normalized with
`x_tilde = S^-1 (x - mu)`. Kernel and optional global-anisotropy derivatives
are evaluated in normalized model coordinates. Gradients return through
`S^-T`, and Hessians through `S^-T H S^-1`. Center value representers use
query derivatives directly. Center directional derivatives preserve the exact
center-argument minus sign and contract mixed query/center second and third
derivatives without finite differences.

Capabilities combine requested output order with the maximum retained center
functional order. `SupportedAwayFromCenters` is preserved rather than
overstated: exact query/center coincidences return provenance-bearing
structured failures. Complete polynomial spaces now evaluate Hessians with
falling-factorial derivatives and no coordinate division. Nonfinite kernel,
polynomial, weighted-contribution, transform, allocation, assembly, and solve
failures remain structured.

`KernelDefinition<D>` records every implemented configured kernel family.
`FittedFieldRecord` exposes build version, coordinate metadata, normalization,
kernel and anisotropy definitions, centers, weights, polynomial basis and
coefficients, capabilities, and diagnostics in deterministic order. This is a
borrowed input view for the later schema requirement; it does not declare enum
discriminants, debug text, or Rust memory layout as a wire format and adds no
reader, writer, checksum, or migration behavior.

Independent tests cover analytic Gaussian value/gradient/Hessian truth under
normalization, combined anisotropy and normalization chain rules, exact
directional-center signs through third order, a pure CPD quadratic with known
polynomial coefficients and Hessian, Matérn center-capability boundaries,
builder independence, deterministic record ordering, compile-time
`Send + Sync`, and concurrent bit-identical reads. Rustdoc, a runnable example,
a deterministic twelve-center D=1/D=2/D=3 evaluation benchmark, three-platform
benchmark smoke routing, registry, and bounded handoff are synchronized.

CLI, C, C++, and Python are N/A in this requirement. The public persistence
schema and CLI file/command contract arrive in M8, while stable C/C++/Python
model handles and parity are M9 requirements. All fitting and evaluation
mathematics remain solely in the Rust core.
