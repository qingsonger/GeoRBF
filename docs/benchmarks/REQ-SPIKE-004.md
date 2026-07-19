# REQ-SPIKE-004 Convex Backend Spike

Command:

```text
cargo run --manifest-path spikes/convex-backends/Cargo.toml --release
```

Baseline environment: Microsoft Windows NT 10.0.26200.0, 12th Gen Intel Core
i7-1260P, `x86_64-pc-windows-msvc`, Rust 1.96.1, one benchmark process. The
harness pins Clarabel 0.11.1 and OSQP 1.0.1. Each timed iteration constructs and
solves deterministic canonical data, requires an exact solved status, and
reviews analytic truth and original-unit equality, bound, and cone residuals.

Three consecutive runs on 2026-07-19 produced these total times for three
iterations:

| Problem | Size | Clarabel 0.11.1 | OSQP 1.0.1 |
| --- | ---: | ---: | ---: |
| Box QP | 16 | 0.3950--0.4451 ms | 0.3436--0.5514 ms |
| Box QP | 32 | 0.4925--0.5317 ms | 0.4681--0.5882 ms |
| Box QP | 64 | 0.7306--0.8912 ms | 1.0583--1.1584 ms |
| SOCP | 16 | 0.2874--0.5157 ms | N/A: OSQP has no second-order cone |
| SOCP | 32 | 0.5382--0.7393 ms | N/A: OSQP has no second-order cone |
| SOCP | 64 | 0.7903--1.0695 ms | N/A: OSQP has no second-order cone |

Checksums were bit-identical across all three repeats for every backend,
problem, and size. Checksums are not compared between QP backends because their
valid rounding paths differ. Independent tests compare both QP solutions with
analytic truth and review the returned infeasibility certificates.

Minimal-feature release binaries were 499,200 bytes for Clarabel alone and
349,696 bytes for OSQP alone. The exact reachable x86_64 Windows graphs
contained 34 and 7 external packages, with cached crate-archive totals of
1,996,202 and 3,480,139 bytes. OSQP's archive total includes its substantially
larger vendored native implementation even though the optimized harness binary
is smaller.

This benchmark is a dependency-selection probe, not a stable performance API.
Draft CI runs the shorter 8 and 16 workload on Ubuntu. Ready PR and `main` CI
run the same smoke workload and all feature configurations on Windows, Ubuntu,
and macOS.
