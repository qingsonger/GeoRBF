# PR #55 Independent Review

- Requirement: REQ-FIELD-001
- Issue: https://github.com/qingsonger/GeoRBF/issues/54
- Pull request: https://github.com/qingsonger/GeoRBF/pull/55
- Branch: `codex/req-field-001-hard-equality-assembly`
- Reviewed head: `ab28b3c49820e3bb05bc10201fdab7fcec9ba84f`
- Repair code/test head: `b8c1367c019c4891a2f7b1ef20a453f07d96ebf4`
- Clean re-reviewed head: `e55ad1689f6eebb17b1e0b962729e323c8cbe840`
- Base head: `5c5acfcbf11172eddc0c4390b1207e02db9ded33`
- Review dates: 2026-07-15 and 2026-07-16
- Result: P2-1 through P2-4 and P3-1 closed; no P0-P3 finding remains

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

## Repair evidence pending fresh re-review

Repair code/test head `b8c1367c019c4891a2f7b1ef20a453f07d96ebf4`
addresses only P2-1 through P2-4 and P3-1:

- the field assembly error now stores `KernelActionError` inline, so evaluator
  and contraction failures retain both atomic-term provenances and their field
  row/column without allocating on the error-mapping path; an armed test-only
  allocation failpoint remains unconsumed when the evaluator fails;
- CPD observation polynomial actions write directly into their final row and
  reuse one fallibly allocated value scratch and one gradient scratch across
  every observation; test-only counters remain exactly `(1, 1)` for both three
  and seventeen rows, while the independent augmented-matrix truth is unchanged;
- the coincident Matérn 1/2 rejection now covers D=1, D=2, and D=3, reaches the
  nonzero directional term indices `(1, 1)`, compares both first-order demands,
  the away-only classification and coincidence flag, and proves zero evaluator
  dispatches in every dimension;
- the D=1 `phi(r)=r^3` fixture independently compares the complete 5-by-5
  augmented matrix, `P=Q=[[1,-1],[0,1],[1,1]]`, RRQR and SVD ranks of two,
  full-rank decision, `Q^T Z`, unit orthogonality and sign-independent alignment
  with `[1,2,-1]/sqrt(6)`, and `Z^T K Z=4/3`; and
- variable-block collection reservation has its own `VariableBlocks` storage
  category, with a targeted failpoint regression proving the exact requested
  block count of two rather than reporting sparse affine-term storage.

The five public field integration tests and three private allocation/error-path
regressions pass. Warning-denying all-target/all-feature focused Clippy, the
runnable example, and D=1/D=2/D=3 optimized benchmark smoke passed; benchmark
checksums remained `1.61140766744821190e3`, `1.35292609211012223e3`, and
`1.19221674654189542e3` respectively.

On exact repair code/test head `b8c1367`, the complete standard gate passed:

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo test --doc --workspace
cargo xtask requirements check
git diff --check
```

The registry checker reports all 58 v1 requirements valid. No manifest, schema,
dependency, CI route, benchmark input, adapter disposition, or requirement
status changed. PR #55 remains Draft and REQ-FIELD-001 remains `documented`;
this repair evidence does not independently close the findings. A fresh
read-only re-review must inspect the exact repaired PR head before the ready-CI
integration sequence. Do not begin REQ-SOLVE-001.

## Fresh independent re-review of the repaired head

A new read-only project `math_reviewer` independently reviewed the complete
16-file PR diff on exact evidence head
`e55ad1689f6eebb17b1e0b962729e323c8cbe840` against base
`5c5acfcbf11172eddc0c4390b1207e02db9ded33`. It received only the bounded
requirement and integrated dependency summaries, Issue #54 acceptance
criteria, the M3 plan, scoped mathematical, architecture, CPD, regularization,
and rank-backend contracts, the complete diff, validation and benchmark
evidence, and the original findings. It made no repository or remote changes
and did not inherit the Repair task's reasoning.

The reviewer independently confirmed every original finding is closed:

- P2-1: `KernelActionError` is stored inline, evaluator-error mapping performs
  no allocation, and the isolated failure regression retains the field row,
  center column, and both atomic-term provenances.
- P2-2: one fallibly allocated polynomial value scratch and one gradient
  scratch are reused across all CPD observation rows. Constant-allocation
  regressions cover three and seventeen observations, while independent matrix
  truth verifies unchanged polynomial actions.
- P2-3: the coincident Mat茅rn 1/2 rejection covers D=1, D=2, and D=3, checks
  both first-order demands, term indices `(1, 1)`, away-only capability and
  coincidence, and proves zero evaluator dispatches.
- P2-4: the D=1 `phi(r)=r^3` fixture compares the complete 5-by-5 augmented
  matrix and `P=Q`, both rank decisions of two, `Q^T Z`, unit orthogonality,
  alignment with `[1,2,-1]/sqrt(6)`, and `Z^T K Z=4/3`.
- P3-1: variable-block collection reservation uses the distinct
  `VariableBlocks` category, and its isolated regression verifies the exact
  requested count of two.

Independent Gaussian differentiation confirmed
`grad_x k=-d k/ell^2`, `grad_y k=d k/ell^2`, and the mixed directional action
`(u dot v/ell^2 - (u dot d)(v dot d)/ell^4) k`. The CPD fixture independently
gives `null(Q^T)=span([1,2,-1])` and projected energy `8/6=4/3`. The reviewer
found complete polynomial augmentation, hard equalities, symmetry, derivative
capability rejection, and the absence of solves, jitter, regularization,
pseudoinverse, constraint softening, or center selection to be intact. No new
P0, P1, P2, or P3 finding was found.

The reviewer passed the five public field integration tests, the three private
allocation and error-path regressions, formatting, warning-denying focused
Clippy, the runnable example, D=1/D=2/D=3 benchmark smoke with the recorded
checksums, all 58 requirement checks, and `git diff --check`. Draft Ubuntu CI
run 29463442762 also passed on exact reviewed head `e55ad16`. The complete
standard workspace gate passed on repair code/test head `b8c1367`; the later
reviewed head changes only the requirement summary, bounded handoff, and review
evidence, so that immutable code/test-head gate remains applicable.

## Clean re-review disposition

PR #55 may enter the mandatory integration sequence. Synchronize this clean
review evidence, mark the resulting exact head Ready, and wait for the complete
Windows, Ubuntu, and macOS correctness matrix with every benchmark smoke
workload. Merge exactly once only when that full gate is green, then record
truthful integration state in an isolated change. Do not begin REQ-SOLVE-001.

## Integration evidence

- Clean-review evidence was pushed in documentation-only commit
  `eb914ebebdc66956769a03bb983be52a95c691ce`, and PR #55 was marked Ready
  without changing the independently reviewed implementation.
- Ready-head CI run 29464034282 passed Windows, Ubuntu, and macOS on exact head
  `eb914ebebdc66956769a03bb983be52a95c691ce`, including every benchmark smoke
  workload and the requirement-registry gate.
- PR #55 squash-merged exactly once as
  `aea272cedf43dfa8fd7b59ed31324fa582fcc858`; Issue #54 closed as completed.
- Post-merge `main` CI run 29464518016 passed the complete three-platform
  correctness, benchmark-smoke, and registry gate on exact merge commit
  `aea272cedf43dfa8fd7b59ed31324fa582fcc858`.
- Integration-state PR #56 contains only the registry and evidence changes
  described below and remains Draft until its exact final evidence head is
  green.

The isolated integration-state change records REQ-FIELD-001 as `integrated`
and advances the bounded handoff without starting REQ-SOLVE-001. It changes no
production code, tests, manifests, schemas, CI, build inputs, APIs, numerical
behavior, dependencies, tags, or releases.
