# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Implement
- Requirement: REQ-SPIKE-003, Issue #114
- Branch: `codex/req-spike-003-sparse-backends`
- Draft pull request: #115
- Initial implementation head: `7cae556`
- Dependencies: REQ-KERNEL-004 and its complete closure are integrated
- Registry state: `implemented`

## Scoped implementation

- Added an excluded exact-version harness comparing rstar 0.13.0 and
  kiddo 5.3.2 against independent brute-force strict-radius truth, and
  faer 0.24.4 against sprs 0.11.4 plus sprs-ldl 0.10.0 on identical
  canonical CSC Wendland C2 systems.
- Eight combined-feature regressions cover exact support-boundary exclusion,
  deterministic pair ordering, sparse symmetry and locality, independent
  matrix-vector and analytic-solution truth, finite original-unit residuals,
  singular and malformed failures, repeated reports, scaling, and kiddo's
  valid-input default-bucket panic.
- ADR-0012 selects rstar, CSC, and faer sparse LLT for later private production
  adoption. It rejects a fixed kiddo bucket as a user-input safety policy and
  rejects sprs-ldl's LGPL-2.1 production dependency.
- CI now covers all four minimal feature cross-products, all features, both
  missing-capability compile failures, formatting, warning-denying Clippy, and
  the release smoke workload on the existing Draft/Ready platform policy.
- No production dependency, sparse adapter, fitted-field behavior, public API,
  schema, regularization, pseudoinverse, densification, or fallback is added.

## Evidence state

- The all-feature harness and all four minimal index/backend combinations pass.
  Candidate index pair sets match brute force exactly; both solver paths recover
  analytic truth and reject the singular inconsistent fixture.
- Three optimized Windows runs cover 216, 512, and 1,000 points. Pair and
  nonzero counts and per-candidate checksums are deterministic. Faer is
  materially faster at the largest solve; kiddo is faster at the largest index
  case but its public default alias panics on that valid axis-aligned input.
- Exact-version dependency, license, MSRV, maintenance, unsafe/native exposure,
  archive size, binary size, OSV, and repository-advisory evidence is recorded
  in ADR-0012 and `docs/benchmarks/REQ-SPIKE-003.md`.
- The combined 76-package OSV query found only RUSTSEC-2024-0436 for
  unmaintained `paste 1.0.15`; no memory-safety severity is reported. Rstar,
  sprs, and kiddo repositories report no security advisory.

## Next task boundary

Run the complete standard workspace gate on the final stable implementation
tree, commit and push the registry and bounded handoff evidence, and stop.
Independent Review of Draft PR #115 must occur in a fresh task with a
read-only reviewer; do not start REQ-SPARSE-001.

## Durable evidence

- Acceptance criteria and exclusions: GitHub Issue #114
- Draft implementation: GitHub PR #115
- Requirement summary: `changes/REQ-SPIKE-003.md`
- Reproducible harness: `spikes/sparse-backends/`
- Selection decision:
  `docs/adr/ADR-0012-rstar-faer-compact-sparse-backends.md`
- Scaling and size evidence: `docs/benchmarks/REQ-SPIKE-003.md`
- Numerical policy: `docs/architecture/SOLVER_POLICY.md`

## Checks not yet available

`cargo-nextest`, `cargo-deny`, `cargo-audit`, and `cargo-semver-checks` are not
installed. Miri is unavailable for pinned Rust 1.96.1. Sanitizers, executable
fuzzing, mutation testing, and API/ABI/schema snapshot checks are tracked by
later requirements and release gates. Local `actionlint` is unavailable. No
unavailable check is claimed as passed.
