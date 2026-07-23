# REQ-SPIKE-003 Compact Sparse Backend Spike

Command:

```text
cargo run --manifest-path spikes/sparse-backends/Cargo.toml --release --all-features
```

Baseline environment: Microsoft Windows NT 10.0.26200.0, 12th Gen Intel Core
i7-1260P, `x86_64-pc-windows-msvc`, Rust 1.96.1, one benchmark process. The
harness pins rstar 0.13.0, kiddo 5.3.2, faer 0.24.4, sprs 0.11.4, and
sprs-ldl 0.10.0 with default features disabled at the root dependency boundary.

Each fixture is a D=3 unit grid with Wendland C2 support radius 1.75. Independent
brute force defines the strict-interior neighbor truth. Candidate index results
are remeasured, strict-filtered, sorted, and deduplicated. Both solver paths
receive the same full symmetric sparse triplets and known truth solution,
construct CSC directly without a dense intermediate, factor once, solve once,
and pass independent original-unit residual and analytic-solution review.

Kiddo's public default 32-entry leaf bucket panics on the 1,000-point
axis-aligned fixture. The timed comparison uses an explicit 128-entry bucket
only so this fixed fixture can finish; ADR-0012 rejects that bounded workaround
for arbitrary production data.

Three consecutive runs on 2026-07-23 produced these total times for three
iterations:

| Operation | Points | Stored pairs/nonzeros | Candidate A | Candidate B |
| --- | ---: | ---: | ---: | ---: |
| Fixed-radius index | 216 | 2,156 pairs | kiddo: 1.0149--1.3044 ms | rstar: 0.9615--1.0887 ms |
| Fixed-radius index | 512 | 5,580 pairs | kiddo: 2.2709--2.9989 ms | rstar: 2.2309--2.7713 ms |
| Fixed-radius index | 1,000 | 11,476 pairs | kiddo: 4.8760--6.0248 ms | rstar: 6.5732--8.0464 ms |
| CSC factor and solve | 216 | 4,096 nonzeros | faer: 1.6899--2.0256 ms | sprs: 1.3485--1.6228 ms |
| CSC factor and solve | 512 | 10,648 nonzeros | faer: 5.4346--6.6258 ms | sprs: 5.2117--6.4961 ms |
| CSC factor and solve | 1,000 | 21,952 nonzeros | faer: 9.3249--13.1647 ms | sprs: 16.9863--21.7024 ms |

Every index candidate produced the same pair count and bit-identical checksum:
`5.19426000000000000e5`, `3.16567350000000000e6`, and
`1.27125270000000000e7` at increasing sizes. Faer and sprs use different valid
floating-point paths, so their solution checksums are reviewed independently
rather than compared bitwise between backends. Each backend's checksum and
residual were bit-identical across all repeats. The largest residual was
`3.33066907387546962e-15`.

Minimal-feature x86_64 Windows evidence:

| Configuration | External packages | Cached archives | Release harness |
| --- | ---: | ---: | ---: |
| faer + rstar | 47 | 3,518,941 bytes | 2,808,832 bytes |
| sprs + rstar | 25 | 1,399,464 bytes | 262,144 bytes |
| faer + kiddo | 55 | 3,739,064 bytes | 2,807,808 bytes |
| sprs + kiddo | 39 | 2,111,761 bytes | 261,632 bytes |

The benchmark is dependency-selection evidence, not a stable performance API.
`-- --smoke` runs 64- and 216-point cases for CI. Draft CI runs the complete
feature matrix and smoke workload on Ubuntu. Ready PR and `main` CI run them on
Windows, Ubuntu, and macOS.
