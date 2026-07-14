# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Repair / PR #32 review finding P1-1
- Issue: #31, `[WORKFLOW] Reduce Codex context and verification overhead`
- Branch: `codex/issue-31-context-efficiency`
- Pull request: #32 (Draft),
  `https://github.com/qingsonger/GeoRBF/pull/32`
- Review record: `docs/reviews/PR-32-INDEPENDENT-REVIEW.md`
- Selected repair: make `AGENTS.md` require ready -> complete green
  ready-head CI -> merge exactly once, consistent with
  `docs/CODEX_WORKFLOW.md`
- Product and mathematics impact: none

## Product delivery state

- Current milestone: M2 / v0.2.0 -- polynomial spaces, CPD, and atomic
  functionals
- Next product requirement: REQ-POLY-001
- REQ-POLY-001 status: `planned`
- REQ-POLY-001 issue, branch, and pull request: not yet created
- Dependencies: REQ-DIM-001 is `integrated`; no blocker
- Product work remains paused until the higher-priority workflow repair is
  reviewed and integrated.

## Next task

Open a fresh Repair task for only PR #32 finding P1-1. Remove the contradictory
pre-CI merge instruction, add focused consistency evidence for the ready ->
complete ready-head CI -> merge sequence, run the final standard checks on the
stable repaired head, update the review record and this handoff, push, and stop
for fresh re-review. Do not begin REQ-POLY-001.

## Context for the next task

1. Read `AGENTS.md` and this file.
2. Inspect remote Issue, PR, review, and CI state before selecting the mode.
3. For product work, run `cargo xtask requirements next`, then
   `cargo xtask requirements show <REQ-ID>` and `requirements deps <REQ-ID>`.
4. Read only the documents listed by that compact summary plus relevant ADRs
   and the current milestone plan.
5. Consult `docs/progress/HISTORY.md` only when historical evidence is needed.

## Latest known gates

- Synchronized `main`: `a2be099`
- Latest `main` CI: run 29310297567 passed on Windows, Ubuntu, and macOS
- Last integrated product requirement: REQ-ANISO-001 via implementation PR
  #29 and integration-state PR #30
- Local final gate for Issue #31: passed on the final code/config worktree;
  formatting, warning-denying workspace Clippy, 118 workspace tests, 21
  doctests, all 58 requirement checks, `git diff --check`, and strict Codex
  config parsing succeeded
- Independent Review of PR #32 found one merge-blocking P1 in the mandatory
  integration sequence; no P0, P2, or P3 finding was identified
- Review-focused `cargo test -p xtask` passed 13 tests, warning-denying focused
  Clippy passed, compact commands and failures behaved correctly, and Codex
  0.142.5 strictly parsed the project configuration
- Draft CI run 29312598987 passed the Ubuntu correctness gate for reviewed head
  `3363d79`; the 3-OS benchmark matrix was correctly skipped while Draft
- Local `actionlint` is unavailable

## Checks not yet available

`cargo-nextest`, `cargo-deny`, `cargo-audit`, and `cargo-semver-checks` are not
installed. Miri is unavailable for pinned Rust 1.96.1. Sanitizers, executable
fuzzing, mutation testing, allocation instrumentation, and API/ABI/schema
snapshot checks are tracked by later requirements and release gates.
