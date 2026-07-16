# REQ-MODEL-001 Immutable Fitted-Field Evaluation Baseline

Command:

```text
cargo bench -p georbf --bench fitted_field_evaluation
```

Each dimension builds one immutable fitted Gaussian model before timing. The
model has twelve value centers, checked Cholesky solve evidence, identity
normalization, and no polynomial block. Each timed iteration evaluates 128
prebuilt queries and requests the fused scalar value, Cartesian gradient, and
Cartesian Hessian. The checksum includes every returned component. No fitting,
assembly, solve, query construction, thread-pool work, or polynomial scratch
allocation occurs in the timed loop.

Baseline environment: Microsoft Windows NT 10.0.26200.0, 12th Gen Intel Core
i7-1260P, `x86_64-pc-windows-msvc`, Rust 1.96.1, one benchmark process and one
thread. Four consecutive 100-iteration runs on 2026-07-16 produced:

| Dimension | Nanoseconds per fused evaluation | 100-iteration checksum |
| --- | ---: | ---: |
| D=1 | 1307.62--1895.98 | `9.37398492496215295e3` |
| D=2 | 1435.74--2347.29 | `9.30024875183164113e3` |
| D=3 | 1702.76--3079.91 | `7.58451538048664042e3` |

Checksums were bit-identical across all four runs for each dimension. The
optimized benchmark executable was 864,768 bytes. Timings and size are local
regression evidence, not cross-machine or final-library promises. Ready PR and
`main` CI run the shorter two-iteration `--smoke` workload on Windows, Ubuntu,
and macOS.
