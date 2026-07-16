# PR #64 Independent Review

- Requirement: REQ-DIAG-001
- Issue: https://github.com/qingsonger/GeoRBF/issues/63
- Pull request: https://github.com/qingsonger/GeoRBF/pull/64
- Branch: `codex/req-diag-001-structured-diagnostics`
- Reviewed head: `872837e0f97114b9e0368e5c670ab41eaeea8f6c`
- Stable implementation head: `9ef9a22`
- Base head: `c55c4cf5a5bee65f13444d596e8b48bf98ef0118`
- Review date: 2026-07-16
- Result: one P1 and one P2 finding; repair required

## Scope and independence

A fresh read-only `xhigh` independent reviewer received only the bounded
REQ-DIAG-001 requirement and integrated dependency summaries, Issue #63
acceptance criteria and exclusions, the M3 plan, scoped architecture and ADR
contracts, the complete PR diff, tests, registry evidence, and exact-head
validation state. It did not inherit the implementation task's reasoning and
made no repository or remote changes.

The reviewer independently checked source and provenance semantics, stable
numeric and symbolic code mappings, validation invariants, hard-constraint and
infeasibility truthfulness, hidden recovery, deterministic formatting,
allocation and API ownership, immutable `Send + Sync` use, public interface
completeness, tests, interface dispositions, and requirement evidence.

No numerical formula, derivative sign, D-dependent geometry, physical-unit
conversion, SPD or CPD classification, center limit, polynomial space,
RRQR/SVD decision, rotation invariance, positive definiteness, or Hessian
behavior changed in this requirement, so those mathematical checklist areas
were not applicable.

## Findings

### P1-1: diagnostic paths cannot retain source-located fields and levels independently

`crates/georbf/src/diagnostics.rs:36-42` privately stores source path, source
line, field path, observation ID, level ID, and constraint group as independent
optional components. The public construction API does not expose the same
independence:

- `crates/georbf/src/diagnostics.rs:64-65` copies a complete
  `SemanticProvenance`, always attaching an observation ID;
- `crates/georbf/src/diagnostics.rs:74-78` adds a level to that complete
  observation provenance; and
- `crates/georbf/src/diagnostics.rs:86-97` creates a field-plus-level path but
  cannot retain source path or source line.

There is therefore no public valid representation for a parse-time source
location plus field path before an observation or level ID exists, or for a
source-located level declaration without an observation ID. This violates
Issue #63's acceptance criterion that source location, field path, stable
observation ID, and stable level ID can be retained independently or together.

The existing all-category test also uses complete observation provenance for
an unrelated `kernel.length` input field at
`crates/georbf/tests/diagnostics.rs:35-44`, which hides the missing path shape
instead of proving it.

Required repair: add the smallest fallible public construction boundary that
can preserve a validated source location and semantic field path with optional
observation, level, and constraint-group identifiers without exposing mutable
fields or schema types. Add a table-driven regression for at least
source-plus-field with neither identifier and source-plus-field-plus-level
without an observation identifier, asserting every getter and exact display.

### P2-1: stable symbolic codes and complete display text are not regression-locked

`crates/georbf/tests/diagnostics.rs:89-110` asserts exact numeric codes and
broad categories, but symbolic identifiers are checked only for the
`GEORBF-E` prefix. `crates/georbf/tests/diagnostics.rs:111` then checks display
text only against whichever identifier the implementation currently returns.

Those assertions would accept an incorrect symbolic remapping such as
`MissingGauge` to `GEORBF-E4999`. They would also accept removal of most
category-specific display evidence because only one input-source substring is
checked later at `crates/georbf/tests/diagnostics.rs:120`. The tests therefore
do not meet Issue #63's independent stable-code and deterministic-formatting
coverage criterion even though the current implementation mappings are
internally correct.

Required repair: extend the ten-category expected table with every exact
symbolic identifier and complete expected display string, assert equality
rather than prefixes, and assert identifier uniqueness.

## Independently verified evidence

- Local HEAD, the remote branch, and Draft PR #64 matched exact reviewed head
  `872837e0f97114b9e0368e5c670ab41eaeea8f6c`; the worktree was clean.
- Draft Ubuntu CI run 29503574991 passed the repository correctness gate on
  the exact reviewed head. The Ready-only Windows, Ubuntu, macOS, and benchmark
  smoke matrix correctly did not run.
- Exact implementation, test, documentation, and registry head `9ef9a22`
  passed the complete local standard gate recorded in the requirement
  evidence. The later reviewed head changes only `docs/progress/CURRENT.md`.
- The independent reviewer reran all five focused diagnostics tests, all 58
  requirement checks, and `git diff --check`; they passed.
- All ten current category, numeric-code, symbolic-code, and error-variant
  mappings are unique and internally consistent.
- Rank dimensions, conditioning estimate and limit, memory byte evidence,
  gauge count, contrast identity, version mismatch, and infeasibility-source
  invariants reject invalid construction without panic or hidden recovery.
- CLI, C, C++, Python, and benchmark dispositions match the later M8/M9
  adapter and schema boundaries and the non-hot-path nature of construction.

## Disposition

PR #64 must remain Draft and REQ-DIAG-001 remains `implemented`. A fresh Repair
task must address only P1-1 and P2-1, add the required independent regressions,
rerun focused checks and the complete stable-head standard gate, push, and stop
for a fresh independent re-review. Do not begin REQ-EXEC-001 or another
requirement.
