# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Implement / next atomic product requirement
- Active repository repair: none
- Integrated workflow repair: Issue #31 via PR #32
- Integration-state branch: `codex/issue-31-integration-state`
- Integration-state pull request: #33
- Product and mathematics impact of Issue #31: none

## Product delivery state

- Current milestone: M2 / v0.2.0 -- polynomial spaces, CPD, and atomic
  functionals
- Next product requirement: REQ-POLY-001
- REQ-POLY-001 status: `planned`
- REQ-POLY-001 issue, branch, and pull request: not yet created
- Dependencies: REQ-DIM-001 is `integrated`; no blocker
- `cargo xtask requirements next` selects REQ-POLY-001

## Next task

Open a fresh Implement task for only REQ-POLY-001. Perform the mandatory
preflight, rerun the compact `next`, `show`, and `deps` queries, read only the
listed mathematical documents plus relevant ADRs and the M2 milestone plan,
then create the Issue with explicit acceptance criteria and the required
`codex/req-poly-001-<slug>` branch. Complete only that atomic requirement to a
Draft PR and stop for a fresh independent Review.

## Context for the next task

1. Read `AGENTS.md` and this file.
2. Inspect remote Issue, PR, review, and CI state before selecting the mode.
3. For product work, run `cargo xtask requirements next`, then
   `cargo xtask requirements show <REQ-ID>` and `requirements deps <REQ-ID>`.
4. Read only the documents listed by that compact summary plus relevant ADRs
   and the current milestone plan.
5. Consult `docs/progress/HISTORY.md` only when historical evidence is needed.

## Latest known gates

- Synchronized `main`: `cc68f0e`
- Latest `main` CI: run 29319281151 passed on Windows, Ubuntu, and macOS with
  all five benchmark smoke workloads
- Last integrated product requirement: REQ-ANISO-001 via implementation PR
  #29 and integration-state PR #30
- Issue #31 closed when PR #32 squash-merged as `cc68f0e`; the Ready head was
  `17b7d16`
- Independent Review found P1-1 in the mandatory merge sequence; the isolated
  repair and focused consistency regression closed it
- Fresh independent re-review found no P0, P1, P2, or P3 finding
- Final repair gate passed on the stable code/test worktree: formatting,
  warning-denying workspace Clippy, 119 workspace tests, 21 doctests, and all
  58 requirement checks; only review evidence and this handoff changed after
  the gate
- Draft CI run 29317847992 passed the Ubuntu correctness gate for exact repair
  head `a17e0cd`; the full matrix was correctly skipped while Draft
- Ready CI run 29319187658 passed on Windows, Ubuntu, and macOS for exact head
  `17b7d16`, including all five benchmark smoke workloads
- Local `actionlint` is unavailable

## Checks not yet available

`cargo-nextest`, `cargo-deny`, `cargo-audit`, and `cargo-semver-checks` are not
installed. Miri is unavailable for pinned Rust 1.96.1. Sanitizers, executable
fuzzing, mutation testing, allocation instrumentation, and API/ABI/schema
snapshot checks are tracked by later requirements and release gates.
