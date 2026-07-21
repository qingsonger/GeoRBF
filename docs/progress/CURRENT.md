# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Implement
- Requirement: REQ-TREND-001, Issue #102
- Branch: `codex/req-trend-001-positive-definite-local-trends`
- Draft pull request: pending first push
- Dependencies: REQ-KERNEL-003, REQ-ANISO-001, and REQ-MODEL-001 are integrated
- Registry state: `in_progress` until the Draft PR is linked

## Implemented scope

- Added an immutable D=1/D=2/D=3 local mixture of fixed anisotropic kernels and
  analytic spatial-basis products.
- Enforced strict-positive-definite kernel metadata, explicit CPD rejection,
  one finite nonzero constant background, a declared operational domain, and a
  positive explicit lower-bound policy.
- Added complete query value, gradient, and Hessian product rules with
  component-intersection capability checks and structured finite-arithmetic
  errors.
- Added immutable construction diagnostics and allocation-free point coverage,
  plus Rustdoc, a runnable example, and a deterministic Hessian benchmark.
- CLI is N/A until M8 schemas and the complete data CLI. C, C++, and Python are
  N/A until M9 API/schema freeze and bindings.

## Validation state

- Focused local-trend tests: 8 passed, including deterministic random Gram SPD,
  independent finite differences, strict background policy, Hessian capability,
  coverage, CPD rejection, dimensions, and input errors.
- The runnable example completed and the release-mode benchmark smoke passed in
  D=1, D=2, and D=3.
- Warning-denying georbf all-target/all-feature Clippy passed during development.
- The complete stable-head standard gate passed: workspace format,
  warning-denying all-target/all-feature Clippy, all-feature workspace tests,
  workspace Rustdoc (including D=4 compile rejection), all 58 requirement
  checks, and complete diff whitespace validation.

## Next task boundary

After this implementation task pushes its stable head and opens the Draft PR,
stop. A fresh Review task must inspect only REQ-TREND-001 and that PR, use an
isolated read-only `math_reviewer`, and must not repair production code or begin
another requirement.

## Durable evidence

- Acceptance criteria and exclusions: GitHub Issue #102
- Requirement summary and benchmark baseline: `changes/REQ-TREND-001.md`
- Independent property/error tests: `crates/georbf/tests/local_trend.rs`
- Public implementation and Rustdoc: `crates/georbf/src/local_trend.rs`
- Runnable example: `crates/georbf/examples/local_trend_mixture.rs`
- Focused benchmark: `crates/georbf/benches/local_trend_mixture.rs`
- Mathematical contract: `docs/architecture/ANISOTROPY.md`, ADR-0005, ADR-0008

## Checks not yet available

`cargo-nextest`, `cargo-deny`, `cargo-audit`, and `cargo-semver-checks` are not
installed. Miri is unavailable for pinned Rust 1.96.1. Sanitizers, executable
fuzzing, mutation testing, general allocation instrumentation, and API/ABI/
schema snapshot checks are tracked by later requirements and release gates.
Local `actionlint` is unavailable. No unavailable check is claimed as passed.
