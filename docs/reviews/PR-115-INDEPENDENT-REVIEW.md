# PR #115 Independent Review

- Requirement: REQ-SPIKE-003
- Issue: https://github.com/qingsonger/GeoRBF/issues/114
- Pull request: https://github.com/qingsonger/GeoRBF/pull/115
- Branch: `codex/req-spike-003-sparse-backends`
- Reviewed head: `2ad68e530d11a9486f3a48e3437b15115c32329e`
- Repair head re-reviewed:
  `7257e67873b1fddd22d6a122f57e5cb92c354bda`
- Final Repair head re-reviewed:
  `83ba364da69477d13510f7304c255e3c28cbdae6`
- Base head: `244e8877ad3833b02bb32c0a8e3ea1e729119f74`
- Review date: 2026-07-23
- Result: all three findings closed; no P0-P3 finding remains

## Scope and independence

A fresh read-only project `math_reviewer` received only the bounded
REQ-SPIKE-003 summary and integrated dependency closure, Issue #114 acceptance
criteria and exclusions, the M7 plan, solver policy and ADR-0012, the complete
PR diff, harness and benchmark evidence, exact validation results, and Draft
CI state. It inherited no Implement reasoning and made no repository or remote
change.

The reviewer independently checked the Wendland C2 formula and SPD
classification, support boundary, deterministic neighborhood truth, canonical
CSC evidence, independent matrix-vector and analytic-solution truth, residual
units, explicit failure, hidden regularization or fallback, allocation and
densification risk, dependency audit, benchmark interpretation, interface
dispositions, registry state, and CI coverage.

## Findings

### P1 SPIKE003-REV-001: matrix-vector truth is circular and does not verify candidate CSC contents

`spikes/sparse-backends/src/main.rs:136-142` assembles the sparse entries and
creates the expected right-hand side with `sparse_matrix_vector_product`.
The only matrix-vector assertion at
`spikes/sparse-backends/src/main.rs:706-720` calls that same function again
and compares it with the right-hand side it previously produced. The faer and
sprs paths at `spikes/sparse-backends/src/main.rs:486-527` return only a
solution and stored-nonzero count; no test inspects either candidate's CSC
column pointers, row indices, stored values, or candidate-storage
matrix-vector product.

A wrong Wendland coefficient, assembled entry, or consistently wrong
matrix-vector implementation could therefore generate both `A` and `b`, after
which each backend would still recover the planted solution. This does not
establish Issue #114's independent Wendland and matrix-vector oracle or its
canonical CSC verification, while
`docs/adr/ADR-0012-rstar-faer-compact-sparse-backends.md:18-22,72-78` claims
that evidence.

Required Repair: add a hand-derived small Wendland C2 system and inspect each
candidate's actual CSC representation and matrix-vector product. The smallest
regression can use collinear points at `0`, `R/2`, and `R`, independently
specify `phi(0)=1`, `phi(1/2)=3/16`, and `phi(1)=0`, and therefore require

```text
A = [[1, 3/16, 0], [3/16, 1, 3/16], [0, 3/16, 1]]
x = [1, 2, 3]
A*x = [11/8, 11/4, 27/8].
```

The expected values must not be created with `wendland_c2`,
`assemble_wendland_c2`, or `sparse_matrix_vector_product`. For both candidates,
the regression must check CSC shape, monotone column pointers, sorted unique
row indices, exact stored values and symmetry, candidate-storage
matrix-vector output, and recovery of `x`.

### P2 SPIKE003-REV-002: published factor-and-solve timings include unreported work

`spikes/sparse-backends/src/main.rs:596-616` times the complete `solve` call.
That call includes residual recomputation and analytic-truth review at
`spikes/sparse-backends/src/main.rs:456-477`, while each backend also allocates
triplets and constructs CSC inside the timed region before factorization at
`spikes/sparse-backends/src/main.rs:486-527`.
`docs/benchmarks/REQ-SPIKE-003.md:29-36` and
`docs/adr/ADR-0012-rstar-faer-compact-sparse-backends.md:107-111` instead label
the measurements as CSC factor-and-solve totals.

The measurements therefore conflate backend-specific triplet and CSC
construction, factorization, solve, residual review, and truth checking. They
cannot support a factorization-speed conclusion under the published label.

Required Repair: either relabel the numbers everywhere as end-to-end
construct/factor/solve/review totals and avoid factorization-speed conclusions,
or time CSC construction, factorization, solve, and review separately and
update the decision evidence. Add a benchmark-output schema regression
requiring explicit phase fields or an explicit `end_to_end` label, rerun the
fixed three-trial workload, and update the benchmark report and ADR from those
outputs.

No other P0, P1, P2, or P3 finding was identified.

## Independent truth review

- The implemented `(1-q)^4(4q+1)` formula is Wendland C2 for D=3, with center
  value one and strict support value zero. Distinct grid centers therefore
  form an SPD Gram fixture.
- The full-neighborhood bound of 27 grid points is consistent with the
  support radius, and `(stored_nonzeros + point_count)/2` reproduces the
  reported upper-triangle pair counts.
- The residual is a dimensionless normwise backward error,
  `||b-Ax||_inf / (||A||_inf ||x||_inf + ||b||_inf)`.
- No jitter, pseudoinverse, regularization, densification, fallback, or
  backend switch appears.
- CPD polynomial spaces, rank decisions, Hessians, hard constraints, and
  public interface parity are genuinely inapplicable to this excluded
  dependency spike.

## Validation and disposition

- Local and remote branch heads matched exact reviewed head `2ad68e5`; the
  base was `244e887`, and the worktree remained clean throughout review.
- Draft CI run 29979880254 passed the complete configured Ubuntu correctness,
  candidate-feature, negative-configuration, and sparse smoke gate on exact
  reviewed head `2ad68e5`. The Ready-only Windows, Ubuntu, macOS, and benchmark
  smoke matrix was skipped as designed and is not claimed as passed.
- The independent reviewer passed all eight locked all-feature harness tests,
  the locked release smoke workload, the complete PR whitespace check,
  exact-version metadata and dependency-tree review, and requirement
  show/dependency checks. The locked combined current-target normal graph
  contains the recorded 76 external packages.
- The parent Review task independently passed the same eight all-feature
  harness tests, all 58 requirement checks, and the complete PR whitespace
  check.
- Stable implementation head `255bac8` retains the complete standard local
  gate recorded by Implement. The tail through reviewed head `2ad68e5` changes
  only the bounded handoff.
- This Review change updates only this review record, the requirement's
  document index, and the bounded handoff. It changes no production or spike
  code, test, manifest, schema, CI, build, dependency, benchmark result, or
  numerical behavior.

PR #115 remains Draft and REQ-SPIKE-003 remains `implemented`, not
`integrated`. A fresh Repair task must address only SPIKE003-REV-001 and
SPIKE003-REV-002, add the required regressions, rerun focused checks and the
complete stable-head gate, update this review evidence and the bounded handoff,
push, and stop for a fresh independent re-review. Do not begin REQ-SPARSE-001.

## Repair evidence pending fresh independent re-review

- Repair implementation and benchmark-evidence head:
  `7257e67873b1fddd22d6a122f57e5cb92c354bda`
- Reviewed implementation head before Repair:
  `2ad68e530d11a9486f3a48e3437b15115c32329e`
- Scope: SPIKE003-REV-001 and SPIKE003-REV-002 only

SPIKE003-REV-001 is repaired with a hand-derived three-point fixture at
`0`, `R/2`, and `R`. Its expected values explicitly require
`phi(0) = 1`, `phi(1/2) = 3/16`, and `phi(1) = 0`, with
`A * [1, 2, 3] = [11/8, 11/4, 27/8]`. The expected CSC arrays and product do
not use `wendland_c2`, `assemble_wendland_c2`, or
`sparse_matrix_vector_product`. The regression separately exercises the
harness kernel and assembly, then inspects each candidate's actual CSC shape,
column pointers, sorted unique row indices, exact stored values, explicit
symmetry, storage-level matrix-vector result, and recovered solution.

SPIKE003-REV-002 is repaired by adding an explicit `phase` field to every
benchmark row. Solver rows are labeled
`construct_factor_solve_review_checksum_end_to_end`; the report, ADR, change
fragment, and harness README now state that these totals include triplet
allocation, CSC construction, factorization, solve, residual and analytic
review, and checksum accumulation. They make no isolated factorization-speed
claim. Three consecutive optimized Windows runs of the repaired harness
refreshed the recorded 216-, 512-, and 1,000-point ranges. A schema regression
requires the explicit end-to-end phase names.

After the last code/test change, focused validation passed:

- sparse-harness formatting and warning-denying all-target/all-feature Clippy;
- all 10 combined-feature tests;
- all four minimal index/backend feature cross-products;
- both required missing-capability negative configurations; and
- the optimized all-feature smoke workload.

The fixed full three-trial workload also passed before the final test-only
strengthening of the hand-derived symmetry assertions. That strengthening is
compiled only under `cfg(test)` and does not change the release benchmark
binary or its output schema.

Exact Repair head `7257e67` then passed the complete stable-head gate:

- `cargo fmt --all -- --check`;
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`;
- `cargo test --workspace --all-features`;
- `cargo test --doc --workspace`;
- `cargo xtask requirements check` for all 58 requirements; and
- `git diff --check`.

This is Repair evidence, not independent closure. PR #115 remains Draft and
REQ-SPIKE-003 remains `implemented`, not `integrated`. A fresh isolated
mathematical and numerical re-review must inspect the complete repaired PR diff
and exact Repair head before the PR can be marked ready. This Repair does not
begin REQ-SPARSE-001.

## Fresh independent re-review

A new read-only project `math_reviewer` independently reviewed exact Repair
implementation head `7257e67873b1fddd22d6a122f57e5cb92c354bda`
against base `244e8877ad3833b02bb32c0a8e3ea1e729119f74`. It received
only the bounded requirement summary and integrated dependency closure, Issue
#114 acceptance criteria and exclusions, the M7 plan, normative solver policy
and ADR, the complete exact-head diff, harness and benchmark evidence, and
validation state. It inherited no Implement or Repair reasoning and made no
repository or remote change.

The reviewer closed SPIKE003-REV-001. The hand-derived regression independently
specifies the exact matrix, CSC arrays, and right-hand side at
`spikes/sparse-backends/src/main.rs:830-916`. It reads both candidates' actual
CSC pointers, row indices, and values, computes the product from candidate
storage, and solves through the candidate factorization paths at
`spikes/sparse-backends/src/main.rs:546-619`.

The reviewer also closed SPIKE003-REV-002. Solver measurements are accurately
labeled as end-to-end construction, factorization, solve, residual and analytic
review, and checksum work at `spikes/sparse-backends/src/main.rs:19,701-721`.
The ADR and benchmark report make no isolated-factorization conclusion.

### P2 SPIKE003-REV-003: index construction is omitted from the explicit phase name

The repaired phase constant at
`spikes/sparse-backends/src/main.rs:18` is
`query_filter_canonicalize_checksum_end_to_end`. Each timed iteration at
`spikes/sparse-backends/src/main.rs:681-693`, however, calls `indexed_pairs`,
which constructs the complete Kiddo tree at
`spikes/sparse-backends/src/main.rs:263-268` or bulk-loads the Rstar tree at
`spikes/sparse-backends/src/main.rs:296-302` before querying it.

The same omission appears in `spikes/sparse-backends/README.md:65-66`,
`docs/benchmarks/REQ-SPIKE-003.md:26-40`, and
`docs/adr/ADR-0012-rstar-faer-compact-sparse-backends.md:108-113`. The recorded
numbers are valid construct-plus-query measurements, but the explicit phase
name and prose describe only query, filtering, canonicalization, and checksum
work. Construction costs differ between the two candidate data structures, so
the omission makes the benchmark interpretation materially incomplete.

Required Repair: relabel the unchanged timing region everywhere as an explicit
construct/query/filter/canonicalize/checksum end-to-end phase. Strengthen the
schema regression at `spikes/sparse-backends/src/main.rs:1030-1037` to require
the construction component rather than only an `_end_to_end` suffix. The
existing timing numbers may be retained if the timed code and phase boundary
remain unchanged; rerun them if the phase boundary changes.

No other P0, P1, P2, or P3 finding was identified.

## Re-review truth and validation

- For `phi(q) = (1-q)^4(4q+1)`, the independent values are
  `phi(0) = 1`, `phi(1/2) = 3/16`, and `phi(1) = 0`. The hand matrix has
  eigenvalues `1` and `1 +/- 3*sqrt(2)/16`, all positive, and maps
  `[1, 2, 3]` to `[11/8, 11/4, 27/8]`.
- The strict squared-distance predicate and zero value at normalized distance
  one agree at the support boundary. The unit-grid support radius 1.75 permits
  at most 27 full-row nonzeros and 14 stored upper pairs per point.
- The reported backward error is the dimensionless original-unit quantity
  `||b-Ax||_inf / (||A||_inf ||x||_inf + ||b||_inf)`.
- Independent faer source inspection confirmed that its default LLT dynamic
  regularization threshold and replacement value are both zero. The harness
  contains no jitter, pseudoinverse, densification, or backend fallback.
- The independent reviewer passed all 10 locked all-feature tests, a locked
  smoke run, exact direct-version review, and the complete base-to-Repair-head
  whitespace check. Two cold-build attempts timed out before the same locked
  test command completed successfully with a separate temporary target.
- The parent Review task independently passed the same 10 locked all-feature
  tests, the optimized locked release smoke workload, all 58 requirement
  checks, and the complete base-to-Repair-head whitespace check.
- Exact Repair head `7257e67` retains the complete stable standard gate and
  focused cross-product evidence recorded above. Ready-only Windows, Ubuntu,
  macOS, and benchmark-smoke CI remains unexecuted and is not claimed.
- This re-review changes only this evidence record and the bounded handoff. It
  changes no production or spike code, test, manifest, schema, CI, build input,
  dependency, benchmark result, or numerical behavior.

PR #115 remains Draft and REQ-SPIKE-003 remains `implemented`, not
`integrated`. Open a fresh Repair task limited to SPIKE003-REV-003, update the
phase label, documentation, and regression, run focused and stable-head checks,
push, and stop for another fresh independent re-review. Do not mark the PR
ready, merge it, or begin REQ-SPARSE-001.

## SPIKE003-REV-003 Repair evidence pending fresh independent re-review

- Repair implementation and stable full-gate head:
  `83ba364da69477d13510f7304c255e3c28cbdae6`
- Re-reviewed implementation head before this Repair:
  `7257e67873b1fddd22d6a122f57e5cb92c354bda`
- Scope: SPIKE003-REV-003 only

SPIKE003-REV-003 is repaired by relabeling the index timing phase as
`construct_query_filter_canonicalize_checksum_end_to_end`. The schema
regression now requires that exact phase, including its construction
component, rather than checking only an `_end_to_end` suffix. The ADR,
benchmark report, harness README, and change fragment explicitly state that
each timed index iteration constructs the complete Kiddo tree or bulk-loads
the complete Rstar tree before query, strict filtering, canonicalization, and
checksum work.

The `time_index` control flow, `Instant` boundary, candidate construction and
query calls, fixture, iteration count, and checksum are unchanged. The
previously recorded three-trial Windows timing ranges therefore remain the
measurements for this now-correctly named phase; this Repair does not present
them as query-only evidence and did not rerun the unchanged workload.

After the final code and test change, focused validation passed:

- sparse-harness formatting and warning-denying all-target/all-feature Clippy;
- all 10 combined-feature tests;
- all four minimal feature cross-products;
- both required missing-capability negative configurations; and
- the optimized all-feature smoke workload, whose index rows emitted the new
  explicit construction phase.

Exact Repair head `83ba364` then passed the complete stable-head gate:

- `cargo fmt --all -- --check`;
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`;
- `cargo test --workspace --all-features`;
- `cargo test --doc --workspace`;
- `cargo xtask requirements check` for all 58 requirements; and
- `git diff --check`.

This is Repair evidence, not independent closure. PR #115 remains Draft and
REQ-SPIKE-003 remains `implemented`, not `integrated`. A fresh isolated
mathematical and numerical re-review must inspect exact Repair head `83ba364`
before the PR can be marked ready. This Repair does not merge the PR or begin
REQ-SPARSE-001.

## Fresh independent closure of SPIKE003-REV-003

A new isolated read-only project `math_reviewer` independently reviewed exact
Repair implementation head
`83ba364da69477d13510f7304c255e3c28cbdae6` against base
`244e8877ad3833b02bb32c0a8e3ea1e729119f74`. It received only the bounded
requirement summary and integrated dependency closure, Issue #114 acceptance
criteria and exclusions, the M7 plan, normative solver policy and ADR, the
complete exact-head diff, harness and benchmark evidence, the three prior
findings, and recorded validation. It inherited no Implement or Repair
reasoning and made no repository or remote change.

SPIKE003-REV-003 is closed. The index phase is exactly
`construct_query_filter_canonicalize_checksum_end_to_end` at
`spikes/sparse-backends/src/main.rs:18`, and the schema regression requires
that complete label at `spikes/sparse-backends/src/main.rs:1030-1042`. The
timed interval at `spikes/sparse-backends/src/main.rs:681-693` begins before
`indexed_pairs`, which constructs Kiddo at lines 263-268 or bulk-loads Rstar
at lines 296-302, and ends after checksum accumulation. ADR-0012, the
benchmark report, harness README, and change fragment all explicitly include
construction. The diff from `7257e67` to `83ba364` does not change the timing
control flow or boundary, so the existing fixed measurements remain valid.

No new P0, P1, P2, or P3 finding was identified. Exact comparison with the
unique brute-force pair oracle, explicit pair sorting and deduplication, and
sorted-unique CSC row checks cover duplicate prevention. A separate
coincident-coordinate fixture is not an additional Issue #114 acceptance
item.

Independent truth and validation included:

- `phi(q) = (1-q)^4(4q+1)` has
  `phi(0) = 1`, `phi(1/2) = 3/16`, and `phi(1) = 0`; the hand matrix has
  eigenvalues `1` and `1 +/- 3*sqrt(2)/16`, all positive, and maps
  `[1, 2, 3]` to `[11/8, 11/4, 27/8]`.
- Both candidate CSC paths expose the expected arrays, storage-level product,
  symmetry, and recovered solution. Strict squared-distance filtering excludes
  the support boundary, and the unit-grid radius permits at most 27 full-row
  nonzeros and 14 stored upper pairs per point.
- The reported backward error is the dimensionless original-unit quantity
  `||b-Ax||_inf / (||A||_inf ||x||_inf + ||b||_inf)`. Faer's default LLT
  dynamic-regularization threshold and replacement value are zero. No jitter,
  pseudoinverse, densification, regularization, or backend fallback appears.
- The reviewer passed all 10 locked all-feature tests, the locked optimized
  all-feature smoke workload with both repaired index labels, exact direct
  dependency review, compact requirement and dependency checks, and complete
  base-to-Repair-head whitespace validation.
- The parent Review task independently passed the same 10 all-feature tests,
  optimized release smoke workload, all 58 requirement checks, and complete
  base-to-Repair-head whitespace validation.

Exact Repair head `83ba364` retains its recorded complete stable standard gate.
The tail through this clean re-review conclusion changes only Markdown evidence
and does not invalidate that gate. REQ-SPIKE-003 remains `implemented` until
PR #115 passes the complete exact Ready-head Windows, Ubuntu, and macOS matrix
with every benchmark smoke, merges exactly once, and the isolated integration
state is recorded.

## Integration evidence

- Exact Ready evidence head:
  `60527129d29dd9eab0e2e6a2d1fcad2dfd5e7e82`
- Ready CI run: 29984412613
- Squash merge: `97fabfa213721309bf5431d1f041fe0a66a4011b`
- Post-merge `main` CI run: 29985301861
- Integration-state branch: `codex/req-spike-003-integration-state`
- Integration-state pull request: pending creation

Ready CI run 29984412613 passed the complete Windows, Ubuntu, and macOS
workspace gate on exact Ready head `6052712`, including every configured
backend combination, every benchmark-smoke workload, and requirement
validation. PR #115 then squash-merged exactly once as `97fabfa`; Issue #114
closed as completed. Post-merge `main` CI run 29985301861 passed the same
complete three-platform gate on exact merge commit `97fabfa`.

The isolated integration-state change updates only the requirement registry,
this review evidence, the completed-history index, and the bounded handoff. It
changes no production code, test, manifest, schema, CI, build input, public
API, numerical behavior, dependency, tag, or release. The requirement may be
recorded as `integrated` in this change because implementation, tests,
documentation, applicable interfaces, diagnostics, independent review, exact
Ready-head CI, the implementation merge, and post-merge `main` CI are complete.
The integration-state pull request itself must still pass its complete local
standard gate and exact Ready-head CI before it merges.
