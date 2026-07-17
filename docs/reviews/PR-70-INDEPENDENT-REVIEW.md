# PR #70 Independent Review

- Requirement: REQ-LEVEL-001
- Issue: https://github.com/qingsonger/GeoRBF/issues/69
- Pull request: https://github.com/qingsonger/GeoRBF/pull/70
- Branch: `codex/req-level-001-explicit-level-variables`
- Reviewed head: `7d8d2834a539850ee73b0124faba6d4d88b20f27`
- Base head: `2904c64c8d99e0b6a3183dc6c232953a969922ad`
- Review date: 2026-07-17
- Result: five P1, three P2, and one P3 finding; Repair required

## Scope and independence

A fresh read-only project `math_reviewer` received only the bounded
REQ-LEVEL-001 summary and integrated dependency closure, Issue #69 acceptance
criteria, the M4 plan, the constraint-semantics, ADR-0003, and level-layer
architecture contracts, the complete PR diff, tests, benchmark, registry
evidence, and exact-head validation state. It did not inherit the implementation
task's reasoning and made no repository or remote changes.

The reviewer checked level-row formulae, signs, constants and indices; fixed,
unknown, and prior semantics; dimensional validity; DAG and connected-component
logic; transitive gap arithmetic; conservative floating-point boundaries; hard
constraint preservation; source provenance; structured errors; D=1/D=2/D=3
bounds; allocations; interface dispositions; benchmark scope; and registry
truth.

## Findings

There are no P0 findings. The following P1-P3 findings are actionable.

### P1 R70-001: memberships accept dimensionally and gauge-invalid functionals

`LevelMembership` accepts an unrestricted `ObservationFunctional<D>` at
`crates/georbf/src/levels.rs:164-190`, while validation at lines 332-339 never
requires the contract's unit-weight Value functional `f(x_i)`. A derivative,
`2 f(x)`, or a value difference can therefore compile as `L(f) - h_k = 0`.
This can compare `f/length` with `f`, and the unconditional union at lines
1300-1302 then applies the ordinary joint-shift gauge argument even though a
derivative is shift-invariant and `2 f` shifts by `2 c`.

Repair must reject every membership that is not one coefficient-1 Value atom
unless the normative contract is explicitly changed. Regressions must reject a
directional derivative and a non-unit or multi-value expression with structured
semantic errors, while accepting one unit Value atom.

### P1 R70-002: extreme finite fixed endpoints hide a hard order conflict

At `crates/georbf/src/levels.rs:1225-1230`, lower fixed `f64::MAX`, upper fixed
`-f64::MAX`, and zero gap produce `fixed_gap = -infinity`; the scale and
tolerance also become infinity, so `infinity <= infinity` incorrectly accepts
the contradiction. All user inputs are finite.

An overflow-safe comparison must preserve the definite conflict. A regression
with separately located memberships must require `FixedOrderConflict` and all
endpoint and edge sources for this exact extreme case.

### P1 R70-003: accumulated-gap overflow rejects a feasible hard system

At `crates/georbf/src/levels.rs:1204-1208`, overflow in a derived path sum is
returned as a semantic error. Levels fixed at `-f64::MAX` and `f64::MAX`, an
unknown middle level, and two gaps of `f64::MAX` are feasible with the middle
level equal to zero: each original hard row has a finite difference of exactly
`f64::MAX`. The validator instead sums the two gaps to infinity and rejects the
problem.

Repair needs overflow-safe path comparison without changing the original hard
rows. The feasible three-level chain must validate, alongside a separate
overflow-safe contradictory-endpoint regression.

### P1 R70-008: functional provenance masks mathematical fixed conflicts

`validate_fixed_memberships` at `crates/georbf/src/levels.rs:1101-1103` uses
derived `PartialEq` on `ObservationFunctional`. That equality includes
`FunctionalProvenance` and expression insertion order, so independently sourced
unit Value evaluations at the same point compare unequal. The hard equations
`f(x) = a` and `f(x) = b` remain contradictory for `a != b` regardless of
provenance, but the precheck misses them. The current test at
`crates/georbf/tests/levels.rs:225-237` clones one functional and cannot expose
the defect.

Repair must compare mathematical functionals independently of source identity.
A regression must construct the same Value evaluation with distinct functional
provenance and require `FixedMembershipConflict`; a reordered-expression case
is also required if multi-term memberships remain permitted.

### P1 R70-009: graph connectivity does not prove field contrast

The union-find and contrast checks at `crates/georbf/src/levels.rs:1349-1382`
accept any positive order gap or distinct anchors in the component containing a
field membership. With only `f(x_A) = h_A`, a membershipless `B` can absorb
`h_B - h_A >= delta` while the field remains constant. A distinct fixed but
membershipless `B` has the same defect.

Repair must prove contrast on at least two membership-coupled field
functionals, not merely on connected level vertices. Regressions must require
`MissingContrast` for one membership plus a positive-gap membershipless level,
and for one membership plus a distinct fixed membershipless level joined by a
zero-gap edge; each has a constant-field feasibility witness.

### P2 R70-004: cycle diagnostics include downstream non-cycle edges

Kahn's unemitted vertices at `crates/georbf/src/levels.rs:1155-1167` include
cycle vertices and vertices reachable from a cycle. Collecting every edge whose
endpoints are unemitted therefore reports downstream DAG edges as cyclic,
contrary to `changes/REQ-LEVEL-001.md:21-22`. The pure-cycle test at
`crates/georbf/tests/levels.rs:173-191` asserts only a count.

A regression with `A <-> B` and `B -> C` must identify exactly the two cycle
edge observation IDs in stable order.

### P2 R70-005: fixed-membership conflicts omit fixed-definition sources

The source vector at `crates/georbf/src/levels.rs:1104-1125` contains only the
two memberships even though the contradiction also depends on the two fixed
definitions. This is incomplete compared with the error's complete-source
contract and with fixed-order conflict evidence.

A regression must require both fixed-level definition sources and both
membership sources.

### P2 R70-006: missing-contrast diagnostics cite unrelated levels

When contrast is absent, `crates/georbf/src/levels.rs:1382-1385` always reports
`levels[0]` and `levels[1]`. Those may be isolated anchored components and may
themselves have distinct values, so they are not evidence for the failing field
component required by `docs/math/CONSTRAINT_SEMANTICS.md:47-51`.

A regression must place two distinct isolated fixed anchors first and a later
field-connected pair without contrast, then require component-specific level or
equivalent source evidence.

### P3 R70-007: test and registry evidence overstate independent coverage

The registry list at `requirements/v1.yaml:395` and the claims at
`changes/REQ-LEVEL-001.md:35-40` overstate the tests. The DAG case has a unique
topological order and does not prove deterministic tie-breaking; the cycle test
checks only a source count; and the fixed-functional test clones one functional
rather than independently constructing the same mathematical evaluation.

Repair must add a DAG with simultaneously eligible nodes and assert insertion-
order tie-breaking, assert exact source identifiers, add the provenance-
independent conflict regression from R70-008, and then make the registry and
change fragment no stronger than the resulting evidence.

## Independent truth and unaffected contracts

- Membership, fixed-value, and order canonical rows have the documented signs,
  constants, variable offsets, and insertion order for valid inputs.
- Priors remain explicit metadata and are neither hardened nor claimed as
  solved. Individual hard rows are not dropped, softened, jittered,
  regularized, pseudoinverted, or otherwise repaired.
- Compile-time D=1/D=2/D=3 bounds and the tested `Send + Sync` properties are
  consistent with the requirement.
- Rust is the only current interface. CLI deferral to M8 and C, C++, and Python
  deferral to M9 match the milestone plan.
- SPD/CPD classification, polynomial spaces, center limits, positive
  definiteness, and Hessian capability are not applicable to this semantic
  layer.
- Fixed-path validation remains a residual performance risk: it is
  `Theta(F * V * E)` and allocates two work vectors per fixed source, while the
  64-level benchmark has only one fixed source and no acceptance threshold.

## Independently verified evidence

- The worktree was clean before review evidence was added, and local HEAD and
  remote PR head matched reviewed commit `7d8d283`.
- Exact-head Draft Ubuntu CI run 29561377945 passed its complete correctness
  gate. The Ready-only Windows, Ubuntu, macOS, and benchmark-smoke matrix did not
  run, as expected for a Draft PR.
- The parent Review task independently reran the focused level suite (10
  passed), core Rustdoc, and the complete PR whitespace check; all passed.
- The stable implementation tree retains the recorded complete standard local
  gate. This Review task changes only review and bounded-handoff documentation.

## Repair disposition

PR #70 must remain Draft and REQ-LEVEL-001 remains `implemented`. A fresh
Repair task must address only R70-001 through R70-009, add the specified
independent regressions, rerun focused checks during development, and run the
complete standard gate after the final production or test change. This Review
task does not repair production code, mark the PR ready, merge it, integrate
the requirement, or begin another requirement.
