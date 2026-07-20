# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Implement
- Requirement: REQ-NORMAL-001, Issue #87
- Branch: `codex/req-normal-001-observations`
- Pull request: pending initial push; must remain Draft for independent Review
- Registry state: `in_progress` until the Draft PR number is recorded
- Dependencies: REQ-ORIENT-001 and REQ-CONVEX-001 are `integrated`

## Implemented scope

- Five immutable normal modes lower to the shared semantic IR with one unique
  provenance per generated scalar relation and public deterministic roles.
- Gradient components, complement equalities, oriented projection bounds, and
  angular Lorentz cones preserve explicit hard/soft enforcement. No geological
  term reaches the canonical solver.
- D=2/D=3 complements are deterministic, orthonormal to roundoff, and
  binary-exact under axial sign reversal. Vacuous or angle-insensitive D=1
  modes are rejected; the two meaningful D=1 modes remain available.
- Degree/radian angular domains and nonnegative minimum projection are checked
  without clipping or fallback. Near-zero fitted gradients use an explicit
  same-unit scale and threshold and remain diagnostics only.
- Rustdoc, normative mathematics, eight independent tests, an example, a
  mixed-mode benchmark and Ready/main CI smoke entry are present. CLI is N/A
  until M8; C, C++, and Python are N/A until M9.

## Validation state

- `cargo test -p georbf --test normal_observations` passed all 8 tests.
- `cargo run -p georbf --example normal_observations` passed and reported one
  cone plus projection lower bound `0.05`.
- `cargo bench -p georbf --bench normal_observation_compilation -- --smoke`
  passed with checksum `11088`.
- The 2,000-iteration benchmark passed at 87.47 microseconds per iteration with
  checksum `11088000` on the recorded Windows baseline.
- Focused GeoRBF all-target/all-feature warning-denying Clippy passed.
- The stable implementation tree passed the complete standard gate: workspace
  format, warning-denying all-target/all-feature Clippy, all-feature workspace
  tests, workspace Rustdoc, all 58 requirement checks, and `git diff --check`.

## Next task boundary

Complete this Implement task only: run the standard gate on the final stable
implementation, commit and push, open the Draft PR, record its number and
truthful `implemented` registry state, rerun the required final checks for that
registry head, push, and stop. Independent mathematical Review must start in a
fresh task and must not inherit this implementation reasoning.

## Durable evidence

- Acceptance criteria and exclusions: GitHub Issue #87
- Requirement summary: `changes/REQ-NORMAL-001.md`
- Focused tests: `crates/georbf/tests/normal_observations.rs`
- Normative behavior: `docs/math/NORMAL_AND_TANGENT.md`
- Benchmark: `docs/benchmarks/REQ-NORMAL-001.md`

## Checks not yet available

`cargo-nextest`, `cargo-deny`, `cargo-audit`, and `cargo-semver-checks` are not
installed. Miri is unavailable for pinned Rust 1.96.1. Sanitizers, executable
fuzzing, mutation testing, general allocation instrumentation, and API/ABI/
schema snapshot checks are tracked by later requirements and release gates.
Local `actionlint` is unavailable. No unavailable check is claimed as passed.
