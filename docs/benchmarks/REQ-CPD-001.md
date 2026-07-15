# REQ-CPD-001 CPD Rank and Null-Space Baseline

Command:

```text
cargo bench -p georbf --bench cpd_rank_nullspace
```

Baseline environment: Windows NT 10.0.26200.0, 12th Gen Intel Core i7-1260P,
`x86_64-pc-windows-msvc`, Rust 1.96.1, single benchmark process. Each iteration
assembles a deterministic 64-by-4 D=3 polynomial-action matrix, performs eight
equilibration passes, column-pivoted QR and bounded SVD rank review, constructs
the 64-by-60 orthonormal null-space basis, and verifies its side condition and
orthonormality.

After the scaled-null-space repair, four consecutive 100-iteration runs on
2026-07-15 took 112.5480, 94.3384, 87.2701, and 70.6042 ms, or an observed
0.706--1.125 ms per complete assembly. Every repaired run produced the
bit-identical checksum `-4.97657470788226419e-12`. The optimized benchmark
executable was 291,840 bytes. These timings and size are local regression
evidence, not cross-machine or final artifact promises.

The benchmark constructs owned action and basis outputs and backend workspaces,
but action assembly reuses one polynomial value/gradient scratch allocation
across all centers and performs no per-center or per-element allocation. Ready
PR and `main` CI run the one-iteration `--smoke` workload on Windows, Ubuntu,
and macOS.
