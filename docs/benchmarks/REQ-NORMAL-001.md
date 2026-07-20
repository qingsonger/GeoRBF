# REQ-NORMAL-001 Normal Observation Compilation Baseline

Command:

```text
cargo bench -p georbf --bench normal_observation_compilation
```

Baseline environment: Microsoft Windows NT 10.0.26200.0, 12th Gen Intel Core
i7-1260P, `x86_64-pc-windows-msvc`, Rust 1.96.1, single benchmark process. Each
iteration constructs 30 mixed D=3 observations, six of each implemented mode,
with complete unique provenance and explicit soft L2 enforcement. The 72
resulting scalar relations are canonicalized over three synthetic gradient
variables into equality, linear-bound, and ordered cone objectives with
identity scaling and a checked numeric-storage estimate.

The 2,000-iteration run on 2026-07-20 measured 87.47 microseconds per complete
build and compilation and produced checksum `11088000`, corresponding to a
bit-identical checked numeric estimate of 5,544 bytes per iteration. The
two-iteration `--smoke` run completed at 191.70 microseconds per iteration with
checksum `11088`. Timings are a local regression baseline, not a cross-machine
performance promise. CI uses the shorter smoke workload on Ready PRs and
`main`.
