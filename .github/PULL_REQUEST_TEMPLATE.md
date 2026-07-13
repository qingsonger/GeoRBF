## Requirement

- Requirement ID:
- Issue:
- Milestone:
- Execution mode: Implement / Repair / Review / Release

## What changed and why

Describe the complete diff, design basis, user impact, and explicit exclusions.

## Mathematical and architecture review

- Formulae, signs, dimensions, and units:
- SPD/CPD, center limits, polynomial space, and rank policy:
- Hard constraints, infeasibility, gauge, contrast, and regularization:
- API, schema, ABI, and cross-language impact:
- Allocation and performance impact:

Use `N/A: reason` where a category genuinely does not apply.

## Verification

List exact commands and results, including independent truth or property tests,
error paths, applicable D=1/D=2/D=3 coverage, and benchmarks.

## Checks not run

List every skipped check and the reason. Do not write "none" unless all checks
defined for the current milestone ran.

## Completion checklist

- [ ] Scope is one atomic requirement or one repair.
- [ ] Dependencies are integrated.
- [ ] No hard constraint is silently changed and no hidden regularization exists.
- [ ] Applicable interfaces, tests, docs, examples, diagnostics, and benchmarks are updated.
- [ ] `requirements/v1.yaml`, `docs/progress/CURRENT.md`, and a changelog fragment are updated.
- [ ] Standard local checks pass.
- [ ] Independent review is complete before merge.
