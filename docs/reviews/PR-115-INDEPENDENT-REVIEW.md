# PR #115 Independent Review

- Requirement: REQ-SPIKE-003
- Issue: https://github.com/qingsonger/GeoRBF/issues/114
- Pull request: https://github.com/qingsonger/GeoRBF/pull/115
- Branch: `codex/req-spike-003-sparse-backends`
- Reviewed head: `2ad68e530d11a9486f3a48e3437b15115c32329e`
- Base head: `244e8877ad3833b02bb32c0a8e3ea1e729119f74`
- Review date: 2026-07-23
- Result: one P1 and one P2 finding require Repair

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
