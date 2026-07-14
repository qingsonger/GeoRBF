# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Repair / Codex workflow efficiency
- Issue: #31, `[WORKFLOW] Reduce Codex context and verification overhead`
- Branch: `codex/issue-31-context-efficiency`
- Pull request: #32 (Draft),
  `https://github.com/qingsonger/GeoRBF/pull/32`
- Scope: task boundaries, bounded context, requirement query tooling,
  project reasoning defaults, independent-review configuration, tiered CI,
  and exact user prompts
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

Open a fresh Review task for the workflow-efficiency Draft PR #32.
Review only Issue #31 acceptance criteria, the PR diff, CI behavior, xtask
selection/query logic, Codex configuration, bounded handoff, and usage prompts.
Do not begin REQ-POLY-001 in that Review task.

If review finds a defect, start a fresh Repair task for only those findings.
After a clean review and truthful integration-state update, start a new
Implement task for REQ-POLY-001.

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
- Draft CI run 29312539834 passed the Ubuntu correctness gate for commit
  `f09ff13`; the 3-OS benchmark matrix was correctly skipped while Draft
- Local `actionlint` is unavailable

## Checks not yet available

`cargo-nextest`, `cargo-deny`, `cargo-audit`, and `cargo-semver-checks` are not
installed. Miri is unavailable for pinned Rust 1.96.1. Sanitizers, executable
fuzzing, mutation testing, allocation instrumentation, and API/ABI/schema
snapshot checks are tracked by later requirements and release gates.
