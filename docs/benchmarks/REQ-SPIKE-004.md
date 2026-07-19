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
Both QP paths construct their diagonal quadratic and identity-bound operators
directly in CSC form with O(n) stored entries and O(n) setup work.

Three consecutive runs on 2026-07-19 produced these total times for three
iterations:

| Problem | Size | Clarabel 0.11.1 | OSQP 1.0.1 |
| --- | ---: | ---: | ---: |
| Box QP | 16 | 0.3528--0.6381 ms | 0.2753--0.5791 ms |
| Box QP | 32 | 0.3571--0.5241 ms | 0.3886--0.4335 ms |
| Box QP | 64 | 0.6048--0.7438 ms | 0.6695--0.6983 ms |
| SOCP | 16 | 0.2622--0.3757 ms | N/A: OSQP has no second-order cone |
| SOCP | 32 | 0.3644--0.3866 ms | N/A: OSQP has no second-order cone |
| SOCP | 64 | 0.6217--1.6774 ms | N/A: OSQP has no second-order cone |

Checksums were bit-identical across all three repeats for every backend,
problem, and size. Checksums are not compared between QP backends because their
valid rounding paths differ. The overlapping QP ranges do not establish a
consistent backend ordering. Independent tests compare both QP solutions with
analytic truth, verify the linear-sparse fixtures, and review normalized
infeasibility certificates under positive rescaling.

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
