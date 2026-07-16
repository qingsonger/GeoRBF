# REQ-SOLVE-001 Dense Equality Solver Baseline

Command:

```text
cargo bench -p georbf --bench dense_equality_solver
```

Each iteration solves one fixed 64-by-64 system and includes eight-pass
dimensionless equilibration, column-pivoted QR screening, bounded SVD rank
review, condition estimation, symmetric congruence scaling, one explicitly
selected factorization, at most four refinement corrections, and scaled plus
exact-binary original-unit residual review. The SPD workload selects checked
Cholesky. The indefinite workload consists of 32 coupled two-variable blocks
with a zero leading diagonal and selects symmetric-pivoted Bunch--Kaufman LBLT.

Baseline environment: Microsoft Windows NT 10.0.26200.0, 12th Gen Intel Core
i7-1260P, `x86_64-pc-windows-msvc`, Rust 1.96.1, one benchmark process and one
thread. Four consecutive 100-iteration runs on 2026-07-16 produced:

| Workload | Microseconds per complete solve | 100-iteration checksum |
| --- | ---: | ---: |
| 64x64 checked Cholesky | 1285.08--2215.76 | `6.79953343557435619e3` |
| 64x64 pivoted LBLT | 1251.93--1812.49 | `-3.12168044453656239e3` |

Checksums were bit-identical across all four runs for each workload. The
optimized benchmark executable was 301,568 bytes. Timings and size are local
regression evidence, not cross-machine or final-library promises. Ready PR and
`main` CI run the shorter two-iteration `--smoke` workload on Windows, Ubuntu,
and macOS.
