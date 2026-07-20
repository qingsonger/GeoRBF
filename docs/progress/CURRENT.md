# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Implement
- Requirement: REQ-NORMAL-001, Issue #87
- Branch: `codex/req-normal-001-observations`
- Pull request: #88 (Draft; independent Review has not started)
- Registry state: `implemented`, not `integrated`
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
- Implementation commit `84779ad` was pushed and Draft PR #88 was opened. The
  final PR-link registry/handoff tree then passed the same complete standard
  gate. Only this validation wording changed afterward; no production, test,
  manifest, schema, CI, build input, API, or numerical behavior changed.

## Next task boundary

After the final PR-link registry/handoff head is pushed, stop. Open a fresh
Review task for only Draft PR #88 and REQ-NORMAL-001. That task must create the
project `math_reviewer` independently from bounded requirement/dependency,
normative, diff, test, benchmark, registry, handoff, and validation evidence;
it must not inherit this implementation reasoning or start REQ-TANGENT-001.

## Durable evidence

- Acceptance criteria and exclusions: GitHub Issue #87
- Draft implementation pull request: GitHub PR #88
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
