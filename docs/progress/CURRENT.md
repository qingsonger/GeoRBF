# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Implement / REQ-SPIKE-002
- Requirement: REQ-SPIKE-002, Issue #40
- Branch: `codex/req-spike-002-rrqr-svd-backend`
- Draft pull request: #41
- Initial implementation commit: `69c5a06`
- Registry state in this change: `documented`
- Dependency: REQ-BOOTSTRAP-001 is `integrated`
- Production dependency state: unchanged; the comparison crate is excluded
  from the production workspace

## Implemented decision

- ADR-0009 selects the nalgebra 0.35 release line for later internal RRQR
  screening and SVD rank review. REQ-CPD-001 must repeat the patch-level audit
  before adding a production dependency.
- The explicit spike policy uses eight deterministic alternating infinity-norm
  equilibration passes and dimensionless `max(m,n) * eps * leading` RRQR and
  SVD thresholds. SVD is the review classification; RRQR disagreement remains
  diagnostic evidence.
- The isolated harness compares nalgebra 0.35.0 and faer 0.24.4 with six
  analytic/property/error cases, fixed-size scaling probes, optional backend
  features, exact lockfile, and no pseudoinverse or solve fallback.
- CI now lints, tests, and smoke-runs the spike in Draft Ubuntu and the complete
  ready/main Windows, Ubuntu, and macOS matrix.

## Validation state

- Focused all-feature tests pass all 6 rank-spike cases.
- All-feature and each single-backend build/test paths pass on pinned Rust
  1.96.1 `x86_64-pc-windows-msvc`.
- Warning-denying all-target/all-feature spike Clippy passes.
- Three deterministic release benchmark runs and the smoke workload pass; the
  exact ranges and size evidence are in `docs/benchmarks/REQ-SPIKE-002.md`.
- The final stable-head standard workspace gate passed: formatting,
  warning-denying workspace Clippy with all targets/features, workspace tests
  with all features, workspace rustdoc, and all 58 requirement checks.

## Next task

Open a fresh Review task for only Draft PR #41 and REQ-SPIKE-002. Supply the
requirement summary, dependency closure, ADR-0009, solver policy, PR diff, and
exact validation evidence to an independent reviewer. Do not implement
REQ-CPD-001 in the same task.

## Durable evidence

- Acceptance criteria and exclusions: GitHub Issue #40
- Decision: `docs/adr/ADR-0009-nalgebra-rank-review-backend.md`
- Change summary: `changes/REQ-SPIKE-002.md`
- Benchmark and size baseline: `docs/benchmarks/REQ-SPIKE-002.md`
- Reproducible harness: `spikes/rank-backends/`

## Checks not yet available

`cargo-nextest`, `cargo-deny`, `cargo-audit`, and `cargo-semver-checks` are not
installed. Miri is unavailable for pinned Rust 1.96.1. Sanitizers, executable
fuzzing, mutation testing, allocation instrumentation, and API/ABI/schema
snapshot checks are tracked by later requirements and release gates. Local
`actionlint` is unavailable.
