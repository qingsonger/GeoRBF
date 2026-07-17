# PR #64 Independent Review

- Requirement: REQ-DIAG-001
- Issue: https://github.com/qingsonger/GeoRBF/issues/63
- Pull request: https://github.com/qingsonger/GeoRBF/pull/64
- Branch: `codex/req-diag-001-structured-diagnostics`
- Reviewed head: `872837e0f97114b9e0368e5c670ab41eaeea8f6c`
- Stable implementation head: `9ef9a22`
- Base head: `c55c4cf5a5bee65f13444d596e8b48bf98ef0118`
- Review date: 2026-07-16; fresh re-review: 2026-07-17
- Result: P1-1 and P2-1 closed; no P0-P3 finding remains

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

## Repair disposition

Repair implementation head `193ee44` addresses both findings:

- P1-1: `DiagnosticPath::try_source` fallibly copies a validated
  `SourceLocation` and field path while accepting observation, level, and
  constraint-group identifiers independently. Table-driven regressions cover
  source-plus-field with neither identifier and source-plus-field-plus-level
  without an observation, assert every getter and exact display, and reject
  empty field and group text.
- P2-1: the ten-category contract table now asserts every exact numeric code,
  symbolic identifier, complete display string, and symbolic-identifier
  uniqueness.

The six focused diagnostics tests, focused warning-denying Clippy, focused
Rustdoc, and `git diff --check` passed. The exact implementation, test,
architecture, and change-fragment head `193ee44` passed the complete standard
workspace gate: formatting, warning-denying workspace Clippy for all targets
and features, all-feature workspace tests, workspace Rustdoc, all 58
requirement checks, and `git diff --check`.

This Repair task has not independently re-reviewed or closed the findings.
PR #64 must remain Draft and REQ-DIAG-001 remains `implemented`. A fresh
independent re-review must confirm P1-1 and P2-1 are closed and check the repair
diff for new P0-P3 findings. Do not begin REQ-EXEC-001 or another requirement.

## Fresh independent re-review

A fresh read-only independent reviewer examined exact evidence head
`c838de45e32b1a6673cdb2b62b1eb6b28f9a1d7c` against base
`c55c4cf5a5bee65f13444d596e8b48bf98ef0118`, with focused attention on repair
diff `13bee8c..193ee44`. The reviewer received only the bounded requirement and
integrated dependency summaries, Issue #63 criteria and exclusions, the M3
plan, scoped architecture and ADR contracts, complete PR and repair diffs,
original findings, and exact-head validation evidence. It made no repository
or remote changes.

P1-1 is closed. `DiagnosticPath::try_source` accepts a validated
`SourceLocation` and field path while keeping observation, level, and
constraint-group identifiers independently optional. The table-driven
regression covers source-plus-field without either identifier and
source-plus-field-plus-level without an observation, checks every getter and
the complete display text, and rejects empty field and supplied group text.
The existing complete semantic-provenance constructors remain covered.

P2-1 is closed. The ten-category independent contract table now locks every
category, numeric code, exact symbolic identifier, complete deterministic
display string, and symbolic-identifier uniqueness. The expected table is
independent of the production mapping and would reject a symbolic remap or
lost category-specific display evidence.

The regression review found no new provenance, stable-code, validation,
hard-constraint, hidden-recovery, formatting, allocation, API-ownership,
`Send + Sync`, interface-disposition, test-truth, or requirement-evidence
defect. The repair introduces no schema or language binding, numerical
backend, solver recovery, pseudoinverse, regularization, or hard-constraint
relaxation. Mathematical formulae, signs, dimensions, units, SPD/CPD
classification, center limits, polynomial spaces, RRQR/SVD decisions,
rotation invariance, positive definiteness, and Hessian capabilities are not
applicable because this requirement changes only the public diagnostic
taxonomy and provenance boundary.

Independent validation used an isolated target directory: all six diagnostics
integration tests passed, the complete PR `git diff --check` passed, and the
working tree remained clean. Draft Ubuntu CI run 29547332906 passed the
repository correctness gate on exact reviewed head `c838de4`. That head differs
from complete-local-gate repair implementation head `193ee44` only through the
review record and bounded handoff.

No P0, P1, P2, or P3 finding remains.

## Integration disposition

The integration task confirmed exact Ready head
`8d265f1fdc523199b09a8c1b28d2e32f14288940` differs from independently
reviewed evidence head `c838de45e32b1a6673cdb2b62b1eb6b28f9a1d7c` only through
this review record and the bounded handoff. Those evidence-only changes alter
no production code, tests, manifests, schemas, CI, build input, registry, API,
numerical behavior, dependency, tag, or release.

Exact Ready head `8d265f1` passed the complete Windows, Ubuntu, and macOS
correctness matrix with every benchmark-smoke workload in CI run 29547848410.
PR #64 then squash-merged exactly once as
`654cb60c786a095768e2b9b0430d8208ab88808a`, and Issue #63 closed as
completed. Post-merge `main` run 29548328531 passed the same complete
three-platform correctness, benchmark-smoke, and requirement-registry gate on
that exact merge commit.

The isolated integration-state change updates only the registry, this review
evidence, the history index, and the bounded handoff. It changes no production
code, tests, manifests, schemas, CI, build inputs, APIs, numerical behavior,
dependencies, tags, or releases. REQ-DIAG-001 may therefore be recorded as
`integrated`; the next requirement must begin only in a fresh task after the
isolated integration-state pull request is green and merged.

The isolated integration-state registry tree passed the complete local
standard gate and `git diff --check`. The subsequent validation note is
documentation-only and changes no production, test, manifest, schema, CI, or
build input. The isolated integration-state evidence is published in PR #65.
