# Benchmarks

Performance-sensitive requirements add deterministic benchmark cases with
fixed data, thread settings, version, and hardware metadata. Stage 0 has no
runtime mathematical path to benchmark.

`REQ-KCALC-001` adds a dependency-free, single-thread benchmark for the full
radial-to-Cartesian jet expansion in D=1, D=2, and D=3. Run the quick check or
the recorded baseline with:

```text
cargo bench -p georbf --bench radial_kernel_calculus -- --smoke
cargo bench -p georbf --bench radial_kernel_calculus
```

The fixed inputs, iteration count, environment, and measured baseline are
recorded in `benches/REQ-KCALC-001.md`.

`REQ-FUNC-001` adds a dependency-free benchmark for sample, complete-polynomial,
and prebuilt-kernel-jet functional actions in D=1, D=2, and D=3:

```text
cargo bench -p georbf --bench atomic_functionals -- --smoke
cargo bench -p georbf --bench atomic_functionals
```

The fixed workload and measured local baseline are recorded in
`docs/benchmarks/REQ-FUNC-001.md`.

`REQ-CPD-001` adds a deterministic 64-center D=3 workload covering polynomial
action assembly, equilibration, RRQR/SVD review, null-space construction, and
verification:

```text
cargo bench -p georbf --bench cpd_rank_nullspace -- --smoke
cargo bench -p georbf --bench cpd_rank_nullspace
```

The recorded baseline and dependency-size context are in
`docs/benchmarks/REQ-CPD-001.md`.

`REQ-IR-001` adds a deterministic 96-constraint semantic-to-canonical
compilation workload for D=1, D=2, and D=3:

```text
cargo bench -p georbf --bench problem_ir -- --smoke
cargo bench -p georbf --bench problem_ir
```

The fixed workload and local baseline are recorded in
`docs/benchmarks/REQ-IR-001.md`.
