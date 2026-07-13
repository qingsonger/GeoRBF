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
