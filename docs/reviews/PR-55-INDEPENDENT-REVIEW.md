# PR #55 Independent Review

- Requirement: REQ-FIELD-001
- Issue: https://github.com/qingsonger/GeoRBF/issues/54
- Pull request: https://github.com/qingsonger/GeoRBF/pull/55
- Branch: `codex/req-field-001-hard-equality-assembly`
- Reviewed head: `ab28b3c49820e3bb05bc10201fdab7fcec9ba84f`
- Base head: `5c5acfcbf11172eddc0c4390b1207e02db9ded33`
- Review date: 2026-07-15
- Result: four P2 findings and one P3 finding; Repair required

## Scope and independence

A fresh read-only project `math_reviewer` received only the bounded requirement
and integrated dependency summaries, Issue #54 acceptance criteria, the M3
plan, relevant mathematical, architecture, and rank-backend contracts, the
complete PR diff, and the recorded validation and benchmark evidence. It did
not inherit the implementation reasoning and made no repository or remote
changes.

The reviewer independently checked query/center signs, mixed-functional
dimensions, SPD/CPD classification, center capability, complete polynomial
augmentation, RRQR/SVD/null-space use, hard constraints, hidden
regularization, symmetry, allocation behavior, independent truth, interface
dispositions, benchmark routing, and requirement evidence. It found no error
in the production formulae, signs, or CPD augmented-matrix construction, but
found the following evidence, allocation, and diagnostic defects.

## Findings

### P2-1: kernel-action error mapping performs an infallible allocation

`crates/georbf/src/field.rs:618-624` stores every evaluator or contraction
error with `Box::new(source)`. That allocation can abort on failure instead of
returning the structured allocation or provenance-bearing kernel-action error
promised by the module. Under resource pressure the error path can therefore
lose its observation row, center column, and functional-term provenance.

Required repair: remove the allocation by storing the source inline, or make
the indirect storage genuinely fallible and map failure to an exact allocation
diagnostic. Add an isolated evaluator-error regression with an allocation
failpoint and verify that the returned error retains the observation index,
center index, and term provenance without panic, abort, or partial success.

### P2-2: CPD observation polynomial actions allocate per row

`crates/georbf/src/field.rs:648-661` calls `try_apply_polynomial` once per
observation. The implementation at `crates/georbf/src/functional.rs:485-495`
creates output, value, and gradient `Vec` buffers on every call even though the
same module provides a scratch-reusing polynomial-action path and the CPD
center path already reuses scratch storage. A problem with `n` observations
therefore performs about `3n` avoidable allocations in a batch hot path.

Required repair: allocate polynomial scratch once and reuse it across every
observation while preserving structured reservation failures. Add a test-only
reservation or allocation counter that proves scratch allocation count remains
constant as the observation count grows, and independently verify that the P
block remains numerically unchanged.

### P2-3: invalid-capability evidence covers only D=1

`crates/georbf/tests/field.rs:392-419` fixes the nonsmooth coincident-derivative
case to `SpatialKernelJetPrefix<1>`. Issue #54 requires the analytic,
mixed-functional, symmetry, and invalid-capability cases for D=1, D=2, and
D=3. The current test leaves the D=2/D=3 coincident, direction-contraction, and
term-index paths unverified.

Required repair: parameterize the rejection regression for D=1, D=2, and D=3.
For each dimension, construct a coincident directional-derivative pair, prove
that evaluator dispatch remains zero, and compare the combined derivative
order, capability, coincident flag, and observation/center term indices.

### P2-4: CPD integration evidence lacks independent matrix truth

`crates/georbf/tests/field.rs:341-367` checks only augmented dimensions,
null-space shape, and a residual bound. For its D=1 `phi(r)=r^3` fixture,
independent arithmetic gives

```text
P = Q = [[1, -1], [0, 1], [1, 1]],
null(Q^T) = span([1, 2, -1]),
Z^T K Z = 4/3
```

for a normalized null vector. An internally consistent but incorrect
observation/center polynomial action, query/center sign, or energy input could
currently retain the checked shapes and small residual.

Required repair: compare the complete 5-by-5 augmented matrix, both RRQR and
SVD rank decisions of two, `Q^T Z`, null-vector orthogonality, and the projected
scalar `4/3` against independent truth.

### P3-1: variable-block reservation failure is mislabeled

`crates/georbf/src/field.rs:668-674` maps failure to reserve the
`Vec<VariableBlock>` collection to `FieldAssemblyStorage::AffineTerms`. This
misreports the allocation source as sparse row coefficients and prevents exact
diagnostic localization.

Required repair: add a distinct variable-block collection storage category and
use it for this reservation. Add a dedicated failpoint regression that checks
the exact category and requested block count.

## Independently verified evidence

- The local, remote branch, and Draft PR heads matched exact reviewed head
  `ab28b3c49820e3bb05bc10201fdab7fcec9ba84f`; the worktree was clean.
- Draft Ubuntu CI run 29422460418 passed on the exact reviewed head. The
  Ready-only Windows, Ubuntu, macOS, and benchmark-smoke matrix correctly did
  not run.
- The complete 15-file `origin/main...HEAD` diff was reviewed, including the
  public assembly API, independent tests, example, benchmark, CI route,
  normative documents, registry, and bounded handoff.
- Gaussian query/center first-derivative signs and mixed Hessian actions, the
  D=1/D=2/D=3 analytic mixed-functional cases, CPD order-two complete `[1,x]`
  space, hard-equality preservation, analytic center capability, and the
  absence of jitter, regularization, pseudoinverse, or constraint relaxation
  were independently checked.
- The reviewer passed all five field integration tests, the runnable example,
  the D=1/D=2/D=3 benchmark smoke, and all 58 requirement-registry checks.
- Exact implementation head `6687631` already passed the complete stable-code
  standard gate. The later reviewed head changes only bounded progress and
  requirement PR/status evidence; it changes no production code, tests,
  manifest, schema, dependency, CI, or benchmark input.

Upper-triangle reflection is not a finding. Issue #54 and the mathematical
contract explicitly authorize reflection after exact same-index observation
and center expression alignment, which `FieldProblem::try_new` enforces.
Evaluator/metadata consistency remains a documented callback precondition and
is a residual misuse risk rather than a defect in this requirement.

Rotation invariance was reviewed structurally rather than through a dedicated
field regression. Allocation instrumentation, nextest, cargo-deny, cargo-audit,
semver checks, Miri, sanitizers, fuzzing, mutation testing, API/ABI/schema
snapshots, and local actionlint remain unavailable or deferred and are not
claimed as executed.

## Disposition

PR #55 must remain Draft and REQ-FIELD-001 must remain `documented`. A fresh
Repair task should address only P2-1 through P2-4 and P3-1, add the specified
regressions, rerun focused checks and the final stable-head standard gate, push,
and stop for a new independent re-review. Do not begin REQ-SOLVE-001.
