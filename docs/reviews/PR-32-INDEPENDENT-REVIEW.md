# PR 32 Independent Review

## Scope and verdict

This fresh Review task examined Issue #31 and PR #32 at head `3363d79`
against `main` at `a2be099`. The review was limited to the Issue acceptance
criteria, the pull-request diff, CI behavior, compact requirement queries,
Codex configuration, bounded progress handoff, history index, and task prompts.
It did not review or change GeoRBF mathematics or product behavior.

Verdict: one P1 finding blocks marking the Draft PR ready. No P0, P2, or P3
finding was identified.

## Findings

### P1-1 -- Mandatory instructions permit merging before ready-head CI

`AGENTS.md:56-57` currently orders the clean re-review path as marking the PR
ready, merging it, waiting for complete ready-head CI, and then merging only
when CI is green. The first merge instruction permits exactly the premature
merge that the following clause and the Issue #31 acceptance criteria forbid.
Because `AGENTS.md` is the mandatory repository instruction source, the later
correct sequence in `docs/CODEX_WORKFLOW.md:88-91` does not remove the
contradiction.

Required repair: state one unambiguous sequence: mark the PR ready, wait for
the complete Windows/Ubuntu/macOS and benchmark-smoke ready-head CI, merge only
after that exact head is green, then record the truthful integration state.

Smallest regression evidence: a focused consistency check must show that both
`AGENTS.md` and `docs/CODEX_WORKFLOW.md` express the same ready -> complete
ready-head CI -> merge order and contain no pre-CI merge instruction.

## Evidence reviewed

- Issue #31 acceptance criteria and PR #32 diff, metadata, commits, reviews,
  comments, mergeability, and current checks.
- `cargo xtask requirements next`, `show REQ-POLY-001`, and
  `deps REQ-POLY-001`, including dependency-first closure output.
- `cargo test -p xtask`: 13 passed.
- `cargo clippy -p xtask --all-targets -- -D warnings`: passed.
- Focused command checks for `next`, `show`, `deps`, unknown IDs, and excess
  arguments: valid commands succeeded and invalid commands failed.
- `codex --strict-config --version`: project and custom-agent TOML parsed with
  Codex CLI 0.142.5. The current Codex manual confirms the project-scoped
  custom-agent fields and the `agents.max_threads` / `agents.max_depth`
  settings used by this PR.
- Draft CI run 29312598987 passed the Ubuntu correctness job on current head
  `3363d79`; the three-platform workspace job was skipped as designed.
- The implementation head's recorded complete local standard gate remains
  applicable: `3363d79` differs from tested code/config head `f09ff13` only in
  `docs/progress/CURRENT.md`. This Review task made evidence-only documentation
  changes and therefore did not repeat the unchanged full gate.
- UTF-8 inspection confirmed that the exact Chinese prompts are intact;
  earlier mojibake was only a PowerShell display-decoding artifact.

## Disposition

P1-1 was repaired in a fresh Repair task without product or mathematics
changes. `AGENTS.md` and `docs/CODEX_WORKFLOW.md` now share the canonical
ready -> complete exact-ready-head CI -> one green-gated merge -> integration
state sequence, and the contradictory pre-CI merge instruction was removed.

The new
`workflow_docs_require_green_ready_head_ci_before_one_merge` regression reads
both documents, requires the same canonical sequence in each, and rejects the
original premature-merge wording. Its focused run passed, as did focused
warning-denying `xtask` Clippy. The final standard gate then passed on the
stable repaired code/test worktree: formatting, warning-denying workspace
Clippy, 119 workspace tests, 21 doctests, and all 58 requirement checks. Only
this review record and the bounded handoff changed after that gate.

Keep PR #32 in Draft. A fresh independent re-review must verify P1-1 is closed
and check for regressions before the PR can be marked ready. Do not begin
REQ-POLY-001.
