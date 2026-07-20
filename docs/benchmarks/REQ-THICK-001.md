# REQ-THICK-001 Local Thickness Compilation Baseline

Command:

```text
cargo bench -p georbf --bench local_thickness_compilation
```

Baseline environment: Microsoft Windows NT 10.0.26200.0, 12th Gen Intel Core
i7-1260P, `x86_64-pc-windows-msvc`, Rust 1.96.1, single benchmark process. Each
iteration constructs two fixed explicit levels, two memberships, one scalar
minimum-gap linear bound, and 32 D=3 sampled local thickness values. It compiles
96 Cartesian derivative rows into 32 hard ordered Lorentz cones while retaining
unique provenance, identity scaling, and checked numeric-storage estimates.

The 2,000-iteration run on 2026-07-20 measured 40.17 microseconds per complete
build and compilation and produced checksum `8304000`, corresponding to a
bit-identical checked numeric estimate of 4,152 bytes per iteration. The
two-iteration `--smoke` run measured 112.50 microseconds per iteration with
checksum `8304`. Timings are a local regression baseline, not a cross-machine
performance promise. CI uses the shorter smoke workload on Ready PRs and
`main`.
