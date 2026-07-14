# GeoRBF Repository Instructions

These instructions apply to the entire repository. More-specific `AGENTS.md`
files may narrow them but must not weaken the mathematical, safety, or release
contracts recorded here.

## Mission and source of truth

Develop GeoRBF incrementally toward a stable v1.0.0. Each change must be
reviewable, reproducible, and tied to exactly one atomic v1 requirement, one
repository-workflow repair Issue, or one repair of an existing pull request.

- `requirements/v1.yaml` is the only machine-readable completion registry.
- `docs/progress/CURRENT.md` is the handoff state for the next run.
- `docs/progress/HISTORY.md` indexes completed work without loading it into
  every run.
- `V1_SCOPE.md`, `docs/math/`, `docs/architecture/`, and accepted ADRs are the
  normative design contracts.
- Chat history is not project state.

## Mandatory preflight

Before editing:

1. Confirm the repository, worktree, origin, branch, and tags.
2. Fetch the remote and inspect open issues, pull requests, reviews, and CI.
3. Read this file and `docs/progress/CURRENT.md`. Run
   `cargo xtask requirements next` or
   `cargo xtask requirements show <REQ-ID>`, then
   `cargo xtask requirements deps <REQ-ID>`. Read only the selected
   requirement, its dependency closure, its listed documents, relevant ADRs,
   and the current milestone plan. Read the complete registry only in Release
   mode or when changing the registry validator.
4. Preserve unrelated or uncommitted user changes.
5. Select Repair, Review, Implement, or Release mode in that priority order.

Never commit directly to `main`. Normal feature branches use
`codex/<requirement-id>-<slug>`; the initial baseline uses
`bootstrap/specification`. Repository-workflow repairs use
`codex/issue-<number>-<slug>`.

## Task and context boundaries

- One Codex task handles exactly one mode and one atomic requirement or PR
  repair. Never continue directly into another requirement.
- Implement mode ends after the scoped implementation, focused checks, one
  final standard-check pass on the stable head, handoff update, push, and Draft
  PR update. Independent Review starts in a fresh task.
- Review mode uses a fresh task and a read-only independent reviewer supplied
  only the requirement summary, dependency closure, normative documents, PR
  diff, and validation evidence. It records findings but does not repair
  production code in the same task.
- Repair mode addresses only selected review or CI findings, adds regressions,
  reruns focused checks and the final standard checks after the last code
  change, pushes, and stops for fresh re-review.
- After a clean re-review, a fresh Review task may mark the PR ready, merge it,
  wait for the complete ready-head CI, merge only when it is green, record
  truthful integration state, and stop. The next requirement always starts in
  a new task.
- If a task needs a second context compaction or its active scope is no longer
  explainable from `CURRENT.md` and the PR diff, stop at a safe committed
  boundary, refresh the bounded handoff, and resume in a new task.
- Keep tool output compact. Query only required GitHub fields, summarize
  successful checks, and expand logs only around failures. Do not paste old
  CI histories into the active handoff.
- Exact new-task, Review, Repair, re-review, integration, next-requirement, and
  safe-resume prompts are maintained in `docs/CODEX_WORKFLOW.md`.

## Non-negotiable architecture

- GeoRBF has one scalar-field core: `f: R^D -> R` for exactly D=1, D=2, or
  D=3.
- Rust contains the sole implementation. CLI, C, C++, and Python layers remain
  thin adapters to it.
- A project may own multiple independent scalar fields, but they all use the
  same `FieldProblem<D>` design.
- Kernel code knows mathematical values and derivatives, never geological
  terms.
- Observations and center representers are distinct architectural objects.
- The solver receives only canonical equality, linear-bound, and cone forms;
  it never knows levels, normals, tangents, or lithology.
- The core crate forbids unsafe code. Unsafe code is confined to the smallest
  future FFI boundary with documented invariants, pointer checks, panic
  containment, and dynamic-analysis coverage.

GeoRBF is a new design. Do not copy, translate, or mechanically rewrite Surfe,
implement Surfe compatibility, introduce its five model types, or use Surfe
golden output as a correctness oracle. Commercial software may inspire public
workflow capabilities only; do not reverse engineer proprietary behavior.

## Mathematical and numerical rules

- Support only D=1, D=2, and D=3 through compile-time dimension bounds.
- The v1 atomic functionals are Value and DirectionalDerivative; finite linear
  combinations form higher semantic constraints.
- Level observations use explicit level variables. Order is a DAG and must be
  checked for cycles, conflicts, isolation, missing gauge, and missing
  contrast.
- Hard constraints stay hard. Never silently drop conflicts, add jitter,
  regularize, use a pseudoinverse to hide rank loss, or convert hard
  constraints to soft ones.
- Every regularization policy and automatic solver adjustment is explicit and
  recorded in diagnostics.
- CPD systems generate their complete polynomial space, diagnose rank with a
  scale-aware RRQR and SVD review, and enforce the polynomial side condition
  through a null-space or mathematically equivalent method.
- Direction-only and axial normals use the normal orthogonal complement. An
  unknown-polarity direction does not imply a nonzero gradient magnitude.
- Keep scalar level gaps, local first-order thickness cones, and sampled
  geometric thickness validation distinct in APIs and diagnostics.
- A local anisotropy construction must preserve positive definiteness. The v1
  construction is a smooth weighted sum of fixed SPD kernels and includes all
  weight product-rule derivatives. It rejects CPD kernels explicitly.
- Hessian support is capability-checked from kernel smoothness and observation
  and center derivative orders; never promise it unconditionally.

Before selecting a production numerical dependency, complete the corresponding
spike and ADR. Do not implement a production QP or SOCP interior-point solver
inside GeoRBF.

## Implementation rules

- Core code does not panic on user input, print progress, use global mutable
  state, or depend on adapters.
- Production code contains no placeholder macros, placeholder success paths,
  or unresolved task markers.
- Public geometry types do not expose third-party linear-algebra types.
- Fitted models are immutable, `Send + Sync`, deterministic to serialize, and
  independent of their builders.
- Batch hot paths avoid per-element allocation, repeated kernel cloning, and
  unnecessary full-matrix work.
- New dependencies require a written need, maintenance and license review,
  MSRV and platform review, unsafe audit, size assessment, alternatives, and an
  ADR when numerically significant.

## Requirement workflow

For one selected requirement:

1. Create or read its issue and write explicit acceptance criteria.
2. Confirm all dependencies are `integrated`.
3. Add independent truth or mathematical-property tests before or alongside
   implementation.
4. Implement the smallest complete change without crossing milestones.
5. Update applicable Rust, CLI, C, C++, and Python surfaces; use `N/A: reason`
   only where genuinely inapplicable.
6. Update documents, examples, schemas or API snapshots, diagnostics, and a
   changelog fragment.
7. During implementation run relevant focused checks. After the final code
   change run the standard workspace checks once on the stable head; rerun a
   full gate only if that head changes or a required check was not executed.
8. Update `requirements/v1.yaml` and the bounded
   `docs/progress/CURRENT.md` accurately. Put durable completed evidence in the
   requirement change fragment, review record, benchmark record, or
   `docs/progress/HISTORY.md`, not in an append-only handoff.
9. Commit conventionally, push without force, and open or update a Draft PR.

A requirement becomes `integrated` only after its implementation, tests,
documentation, applicable interfaces, diagnostics, benchmark obligations,
independent review, green CI, and merged PR are all complete. Never label an
experimental, partial, or deferred v1 capability as integrated.

## Standard checks

Run when available:

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo test --doc --workspace
cargo xtask requirements check
```

Focused checks are development feedback, not a substitute for this final
gate. A green full gate for an immutable commit may be referenced instead of
rerunning it when a later commit changes documentation or review evidence only;
the task must prove that no production, test, manifest, schema, or build input
changed. Ready PRs and `main` retain the complete three-platform CI and
benchmark-smoke gate. Draft PRs may use the documented one-platform correctness
gate until they are marked ready.

As the repository gains the corresponding tooling, also run nextest,
cargo-deny, vulnerability auditing, semver, API, ABI, schema, and benchmark
smoke checks. Record every skipped check and its reason; never claim an
unexecuted check passed.

## Review and release

Mathematical or numerical changes require an independent review in a fresh
task or the project `math_reviewer` agent, covering
formulae, signs, dimensions, units, SPD or CPD classification, center limits,
polynomial space, scale-aware rank decisions, hard constraints, infeasibility,
rotation invariance, positive definiteness, Hessian capabilities, independent
truth, allocations, hidden regularization, interface parity, and requirement
updates. The independent reviewer must not inherit the implementation task's
reasoning transcript.

Release mode is forbidden until every mandatory v1 requirement is integrated,
all release gates in `docs/release/RELEASE_CHECKLIST.md` pass, at least one full
release-candidate rehearsal succeeds, and publishing credentials are available.
Never claim a tag, registry publication, artifact, or release exists unless it
actually does.
