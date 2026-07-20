# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Clean independent re-review complete; Ready integration sequence in progress / REQ-CONVEX-001
- Requirement: REQ-CONVEX-001, Issue #81
- Implementation branch: `codex/req-convex-001-canonical-solver`
- Implementation pull request: #82 (Draft)
- R82-008 repair code/test head: `c1753bdb98e6abec69486c36713d887491204f67`
- Repair evidence head: `26da57b86d8865604285e3e1cfeea8f124329763`
- Clean re-reviewed head: `ad677e33ea2e4d99b0f6f3f93c66743dd98e8cac`
- Review record: `docs/reviews/PR-82-INDEPENDENT-REVIEW.md`
- Registry state: `implemented`; it is not `integrated`
- Direct dependencies REQ-SOFT-001, REQ-LINEQ-001, and REQ-SPIKE-004 are all
  `integrated`

## Final re-review disposition

- A fresh read-only project `math_reviewer` independently reviewed the exact
  repaired PR head using only bounded requirement, dependency, normative-
  document, diff, test, benchmark, registry, handoff, and CI evidence. It did
  not inherit implementation or Repair reasoning.
- R82-001 through R82-008 are closed. No P0, P1, P2, or P3 finding remains.
- Structurally zero objectives now use a recorded dimensionless objective-unit
  reference. Original row values convert it to componentwise gradient units;
  no raw dimensioned floor or tolerance multiplier was added.
- The adapter positively infinity-normalizes independent zero/nonnegative rows
  and whole Lorentz blocks before dispatch, maps backend slack and dual values
  back to original units, and records the complete scaling vector.
- The public hard-only `x >= 1` regression succeeds at row scales `1e-12`, `1`,
  and `1e12`, with every normalized KKT and hard-relation review at or below the
  exact requested `1e-9` tolerance. A synthetic nonstationary dual is still
  rejected.
- PR #82 may proceed to Ready CI. REQ-INFEAS-001 has not begun.

## Validation state

- Focused warning-denying all-target/all-feature Clippy, all six private convex
  tests, all ten convex integration tests, the runnable example, the 8/16
  benchmark smoke workload, and `git diff --check` passed after the final
  production/test change. Smoke checksums remained
  `4.00000000000000444` and `7.99999999999999911`; timings are not performance
  promises.
- Exact code/test head `c1753bdb98e6abec69486c36713d887491204f67`
  passed the complete standard gate: workspace format, warning-denying
  all-target/all-feature Clippy, all-feature workspace tests, workspace
  Rustdoc, all 58 requirement checks, and `git diff --check`.
- The later solver-policy, requirement change fragment, review evidence, and
  registry test-name update change no production, test, manifest, schema, CI,
  build, API, numerical, or dependency input. The updated registry separately
  passed all 58 requirement checks and exact whitespace checks.
- Draft CI run 29711320592 passed the configured Ubuntu job on exact cleanly
  re-reviewed head `ad677e33`. Ready-only Windows, Ubuntu, macOS, and benchmark-
  smoke CI has not run for the final evidence head.
- This re-review task changes only the review record and bounded handoff; no
  production, test, manifest, schema, CI, build, registry, API, numerical, or
  dependency input changes.

## Next task

Synchronize this evidence-only re-review and handoff commit on PR #82, mark the
PR Ready, and wait for the complete Windows, Ubuntu, and macOS correctness
matrix plus every benchmark-smoke workload on that exact final evidence head.
Merge exactly once only when green, wait for green post-merge `main` CI, then
record truthful integration state in an isolated follow-up change. If this task
is interrupted, resume only this integration sequence; do not begin
REQ-INFEAS-001.

## Checks not yet available

`cargo-nextest`, `cargo-deny`, `cargo-audit`, and `cargo-semver-checks` are not
installed. Miri is unavailable for pinned Rust 1.96.1. Sanitizers, executable
fuzzing, mutation testing, general allocation instrumentation, and API/ABI/
schema snapshot checks are tracked by later requirements and release gates.
Local `actionlint` is unavailable. Exact OSV and repository advisory API
queries are the performed dependency review; unavailable tools are not claimed.
