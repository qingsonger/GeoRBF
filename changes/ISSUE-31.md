# Issue 31 -- Codex workflow context efficiency

Bounded Codex work to one mode and one atomic requirement or PR repair, reduced
the project default reasoning effort from `xhigh` to `high`, and added a narrow
read-only `xhigh` mathematical reviewer. Added compact requirement selection,
summary, and dependency-closure commands to `xtask`, replaced the append-only
current handoff with a bounded state file plus a historical evidence index, and
documented exact Chinese prompts for every task transition.

Draft pull requests now run a one-platform correctness gate without benchmark
smoke. Ready pull requests and `main` retain the complete three-platform gate,
all standard checks, and every benchmark smoke workload. Mathematical,
numerical, interface, requirement-integration, and release contracts are
unchanged.

Independent Review found one merge-order contradiction, P1-1. An isolated
repair established the canonical Ready -> complete exact-ready-head CI -> one
green-gated merge -> truthful integration-state sequence and added a focused
consistency regression. Fresh independent re-review found no P0-P3 finding.

Ready CI run 29319187658 passed on Windows, Ubuntu, and macOS for exact PR head
`17b7d16`, including all five benchmark smoke workloads. PR #32 then
squash-merged once as `cc68f0e`, closing Issue #31. Post-merge `main` CI run
29319281151 passed the same complete three-platform gate.
