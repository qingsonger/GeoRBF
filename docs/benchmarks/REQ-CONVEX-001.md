# REQ-CONVEX-001 benchmark

Run with:

```text
cargo bench -p georbf --bench convex_solver
```

The production-adapter benchmark constructs deterministic sparse bounded QPs
with 16, 32, and 64 original variables. Each variable has a two-sided hard
bound and an independently chosen squared-L2 target, so the compiler introduces
one explicit nonnegative violation variable per target. Each timed iteration
includes canonical-to-conic compilation inside the private adapter, Clarabel
setup and solve, and GeoRBF-owned KKT, cone, objective, hard-residual, and
provenance review. Problem construction is outside the timed loop.

On 2026-07-19, Windows x86_64 MSVC, Rust 1.96.1, the one-iteration release smoke
workload completed the 8-variable case in 0.4392 ms and the 16-variable case in
0.3477 ms. Checksums were `4.00000000000000444` and
`7.99999999999999911`. The optimized benchmark executable was 617,984 bytes.
These local, small-problem observations validate the delivery path; they are
not stable performance promises or evidence for the later sparse-field path.

Ready PR and `main` CI run the 8/16 `--smoke` workload on Windows, Ubuntu, and
macOS. Full local 16/32/64 results and exact Ready-head CI evidence remain
review and integration obligations.
