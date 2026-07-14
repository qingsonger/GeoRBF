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
