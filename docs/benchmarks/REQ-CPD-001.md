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

Four consecutive 100-iteration runs on 2026-07-15 took 53.3802, 83.1340,
74.6411, and 78.8284 ms, or an observed 0.534--0.831 ms per complete assembly.
Every run produced the bit-identical checksum
`-4.71844785465691530e-12`. The optimized benchmark executable was 283,648
bytes. These timings and size are local regression evidence, not cross-machine
or final artifact promises.

The benchmark constructs owned action and basis outputs and backend workspaces,
but action assembly reuses one polynomial value/gradient scratch allocation
across all centers and performs no per-center or per-element allocation. Ready
PR and `main` CI run the one-iteration `--smoke` workload on Windows, Ubuntu,
and macOS.
