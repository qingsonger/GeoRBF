# FFI Policy

The C ABI uses opaque handles, fixed-width status and version types, explicit
array lengths, caller-provided output buffers, and a thread-local last-error
channel. It never exposes Rust vectors, strings, trait objects, panics, or
third-party matrix types. Both typed common builders and versioned JSON/project
entry points delegate to the same Rust core.

Every exported function validates nullability, lengths, alignment where
required, finite values, enum discriminants, handle type and lifetime, and
output capacity before dereferencing memory. A panic boundary converts any Rust
panic to a stable internal-error status. Safety invariants are documented beside
each unsafe block and covered by unit, C integration, Miri where applicable,
and sanitizer or fuzz tests.

The C++ layer is header-only RAII over this ABI with destruction, move-only
ownership, spans, vectors, and one documented error policy. It contains no
mathematical implementation. ABI headers and symbols are snapshot-tested before
v1 freeze.
