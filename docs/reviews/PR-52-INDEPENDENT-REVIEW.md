# PR #52 Independent Review

- Requirement: REQ-IR-001
- Issue: https://github.com/qingsonger/GeoRBF/issues/51
- Pull request: https://github.com/qingsonger/GeoRBF/pull/52
- Branch: `codex/req-ir-001-semantic-canonical-ir`
- Reviewed head: `dc88b999f02e31934dc1daa06a4909a87aed69ab`
- Fresh re-reviewed head: `133c8680cbd32e539dd855b7c59e1f374cc15f43`
- Base head: `46a6d48115c5a01d2f200cc956a41a1dcc3158fa`
- Review date: 2026-07-15
- Result: P2-1 closed; P2-2 found; bounded Repair required; PR must remain Draft

## Scope and independence

A fresh read-only `math_reviewer` received only the bounded requirement and
dependency summaries, Issue #51 acceptance criteria, the M3 plan, relevant
architecture, constraint, and no-hidden-regularization contracts, the complete
PR diff, and the recorded validation and benchmark evidence. It did not inherit
the implementation reasoning and made no repository or remote changes.

The reviewer independently checked equality, bound, and SOC mappings; signs,
dimensions, constants, sparse shapes, provenance, hard enforcement, source-
aware errors, deterministic ordering, finite and allocation paths, explicit
identity scaling, capability metadata, immutability and safety, interface
dispositions, benchmark and CI routing, and requirement evidence. It found one
P2 test-evidence defect and no P0, P1, or P3 issue.

## Finding

### P2-1: exact canonical-mapping test does not verify sparse coefficients

`crates/georbf/tests/problem_ir.rs:148-177` verifies shifted constants,
provenance fragments, collection counts, scaling, capabilities, and the memory
estimate, but it never compares a canonical `AffineTerm` variable index or
coefficient. Deleting a coefficient, reversing a sign, permuting an index, or
moving terms between the two sides of an SOC could therefore leave this test
green.

The authoritative mappings require

```text
a^T z + q = t                 -> a^T z = t - q
lower <= a^T z + q <= upper   -> lower-q <= a^T z <= upper-q
||F z + f||_2 <= c^T z + d    -> preserve F, f, c, and d
```

The implementation at `crates/georbf/src/problem_ir.rs:525-610` preserves
these coefficients and constants correctly by inspection. The missing
independent assertions nevertheless fail Issue #51's explicit requirement for
exact equality, bound, and SOC mapping tests.

Required repair: extend
`equality_bounds_and_cones_map_with_exact_constant_shifts` to compare the
complete `(variable, coefficient)` sequence for the equality row, bounded row,
both SOC left components, and SOC right expression. The existing fixture
already supplies distinct indices and negative coefficients, so no production
change or new fixture is expected. Compare complete canonical provenance for
at least one row or cone in the same regression.

## Independently verified evidence

- The local, remote branch, and Draft PR heads all matched
  `dc88b999f02e31934dc1daa06a4909a87aed69ab` before review.
- The complete 14-file `origin/main...HEAD` diff was reviewed, including the
  Rust IR, public exports, tests, runnable example, benchmark, CI routing,
  architecture and benchmark records, registry entry, and bounded handoff.
- Constant shifting, sparse validation, deterministic class ordering, hard-
  constraint preservation, source provenance, identity scaling, checked
  numeric-storage estimates, and solver-neutral type boundaries are otherwise
  consistent with the scoped contracts.
- No unsafe code, user-input panic, hidden scaling, jitter, regularization,
  pseudoinverse, hard-to-soft conversion, third-party public geometry type, or
  out-of-scope geological or solver implementation was found.
- Focused problem-IR tests passed 11/11; the crate doctests, runnable example,
  and D=1/D=2/D=3 benchmark smoke passed with the documented checksums.
- The independent reviewer ran the complete stable-head standard gate:
  formatting, warning-denying workspace Clippy for all targets and features,
  all-feature workspace tests, workspace doctests, all 58 requirement checks,
  and `git diff --check`; all passed.
- Draft CI run 29410313417 passed the Ubuntu correctness job on the exact
  reviewed head. The Ready-only Windows, Ubuntu, macOS, and benchmark-smoke
  matrix correctly remained unexecuted.

## Repair evidence pending fresh re-review

Repair code/test head `4562a20d565bc541ffd06a37220378c41229a627`
addresses only P2-1. The existing exact canonical-mapping regression now
compares every sparse `(variable, coefficient)` sequence for the equality row,
linear-bound row, both SOC left expressions, and the SOC right expression. It
also compares the cone's complete canonical observation identifier, source
path and one-based line, original units, field path, and constraint group.

The focused problem-IR test file passed all 11 tests. The first full-gate
attempt exposed only Clippy's test-function length limit; the repeated term
comparison was moved into a test helper without reducing the asserted mapping
or provenance evidence. After that final test-code change, the complete stable-
head standard gate passed formatting, warning-denying workspace Clippy for all
targets and features, all-feature workspace tests, workspace doctests, all 58
requirement checks, and `git diff --check` on the exact repair code/test head.

No production code, public API, manifest, schema, build input, dependency,
benchmark, or interface disposition changed. The requirement remains
`documented`, and this repair evidence does not independently close P2-1.

## Fresh independent re-review of the P2-1 repair

A new read-only `math_reviewer` independently reviewed exact head
`133c8680cbd32e539dd855b7c59e1f374cc15f43` against base
`46a6d48115c5a01d2f200cc956a41a1dcc3158fa`. It received only the bounded
requirement summary and integrated dependency closure, Issue #51 acceptance
criteria, scoped normative documents, the complete 15-file PR diff, validation
and benchmark evidence, and the exact repaired head. It made no repository or
remote changes and did not inherit implementation or Repair reasoning.

The reviewer confirmed P2-1 is closed. The exact regression now compares the
complete equality, linear-bound, both SOC-left, and SOC-right sparse
`(variable, coefficient)` sequences, including negative coefficients and
noncontiguous variable indices. It also compares complete cone provenance.
Independent arithmetic reconfirmed the equality right-hand side of 8, the
closed bound `[0, 5]`, and retained SOC constants of 1, 1, and 3.

### P2-2: canonicalization provenance copies cannot report allocation failure

`crates/georbf/src/problem_ir.rs:544`, `:580`, and `:609` use
`constraint.provenance.clone()`. Although the canonical row collections use
fallible reservations, cloning `SemanticProvenance` deep-copies its owned
strings through ordinary infallible allocation. An allocation failure there
therefore reaches the global allocation-error handler instead of returning the
documented structured `AllocationFailed` error from canonicalization. This
violates Issue #51's explicit allocation-failure acceptance criterion and the
fallible compilation contract.

Required repair: replace these provenance copies with a fallible deep-copy
path and use it consistently for equality, linear-bound, and SOC
canonicalization. Add isolated allocation-failure regressions at each of the
three provenance-copy paths and verify that compilation returns
`CanonicalizationError::Ir(ProblemIrError::AllocationFailed { .. })` without
abort, panic, or partial result. Keep the repair bounded to P2-2 and do not
begin REQ-FIELD-001.

No P0, P1, or P3 finding was found. SPD/CPD classification, center limits,
polynomial and rank decisions, solver infeasibility, rotation invariance,
anisotropy and positive definiteness, Hessian capability, and actual QP/SOCP
solution remain explicitly outside this solver-neutral IR requirement.

## Bounded P2-2 repair evidence pending fresh re-review

Repair code/test head `1e782a73ab758ea93f0e71e5dba250cf3a03e7aa`
addresses only P2-2. Equality, linear-bound, and SOC canonicalization now copy
every owned provenance string through an exact fallible reservation before
inserting the completed row or cone. Reservation failure returns
`CanonicalizationError::Ir(ProblemIrError::AllocationFailed)` with the new
`CanonicalProvenance` storage category; no infallible provenance string clone
remains in the three canonicalization branches.

Three isolated unit regressions use a thread-local, provenance-copy-only
reservation failpoint while exercising the public `try_compile` entrypoint.
The equality, linear-bound, and SOC cases each return the structured allocation
error without abort, panic, global-allocator replacement, or canonical result.
The focused regressions passed 3/3, the complete problem-IR integration file
passed 11/11, and focused warning-denying Clippy passed.

After the last production/test change, the complete stable-head standard gate
passed formatting, warning-denying workspace Clippy for all targets and
features, all-feature workspace tests, workspace doctests, all 58 requirement
checks, and `git diff --check` on exact head `1e782a7`. The later evidence
update changes only this review record and the bounded handoff, so that
immutable-head gate remains applicable. The repair does not independently
close P2-2; PR #52 remains Draft and REQ-IR-001 remains `documented`.

## Disposition

Keep PR #52 Draft. A fresh independent re-review must confirm P2-2 is closed on
exact repair head `1e782a7` and inspect the bounded PR for new findings. This
Repair evidence must not mark the PR ready, merge it, or begin REQ-FIELD-001.

SPD/CPD classification, center limits, polynomial and rank decisions, solver
infeasibility, rotation invariance, anisotropy and positive definiteness, and
Hessian capability are outside this solver-neutral IR requirement. Allocation
instrumentation, nextest, cargo-deny, cargo-audit, semver checks, Miri,
sanitizers, fuzzing, mutation testing, API/ABI/schema snapshots, and local
actionlint remain unavailable or deferred as documented; none is claimed as
executed.
