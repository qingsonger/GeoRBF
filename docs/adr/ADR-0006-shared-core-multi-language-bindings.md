# ADR-0006: Shared Rust Core for All Language Interfaces

- Status: Accepted
- Date: 2026-07-13

## Context

Rust, CLI, C, C++, and Python must expose consistent fitting and evaluation.
Independent implementations would drift in formulas, tolerances, solvers,
serialization, and error handling.

## Decision

Rust is the sole mathematical implementation. The CLI and PyO3 adapters call
the public core. The C ABI uses opaque handles and stable boundary types. C++ is
a header-only RAII wrapper around that ABI. No adapter compiles constraints,
assembles matrices, solves systems, or evaluates kernels independently.

## Consequences

Cross-language tests compare one computation through several adapters. API
freeze proceeds core first, then C ABI, C++, and Python. Binding crates remain
thin but may own validation required solely by their language boundary.
