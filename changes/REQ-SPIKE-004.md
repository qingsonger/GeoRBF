# REQ-SPIKE-004

Evaluated canonical convex QP and SOCP backends and accepted ADR-0011, which
selects the Clarabel 0.11 release line for later private production adoption.
The decision records direct cone mapping, original-unit solution and
certificate review, deterministic behavior, maintenance, license, MSRV,
unsafe and native-code exposure, advisories, dependency and binary size,
three-platform strategy, alternatives, and explicit solver-adjustment policy.

Added an excluded reproducible comparison crate pinned to Clarabel 0.11.1 and
OSQP 1.0.1. Eleven combined-feature cases cover a shared analytic QP, a Lorentz-
cone SOCP with independent truth, linearly and conically infeasible systems,
scale-invariant dual-certificate stationarity and separation, dual-cone
membership, linear-sparse QP fixture semantics, deterministic repeated reports,
and pre-dispatch nonfinite rejection. CI covers both single-backend
configurations, the combined configuration, rejection of an empty backend
selection, and a release smoke workload.

The production workspace gains no solver dependency, convex adapter, public
matrix or cone type, user API, hidden regularization, constraint relaxation, or
fallback. Rust, CLI, C, C++, and Python interface dispositions remain N/A for
this dependency spike. Production canonical solver integration, diagnostics,
and user-visible behavior remain in REQ-CONVEX-001.
