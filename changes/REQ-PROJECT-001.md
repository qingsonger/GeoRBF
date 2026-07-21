# REQ-PROJECT-001

Issue #99 defines the acceptance criteria and exclusions for independent
multi-field projects. The Rust core now exposes a stable caller-controlled
`FieldId`, an owned `ProjectField<D>`, and an immutable `GeoProject<D>` for
exactly D=1, D=2, and D=3. Project construction preserves caller insertion
order and returns structured errors for zero fields, duplicate identifiers,
and checked allocation failure without returning partial state.

Every project entry consumes and owns an already independent `FittedField<D>`.
Those fields continue to be produced solely through the existing
`FieldProblem<D>` assembly and `FittedField<D>` fitting path; the project layer
adds no kernel, assembly, solver, or evaluation implementation. Field lookup is
deterministic and immutable, and fields retain their separate coordinate
metadata, normalization, kernel, coefficients, capabilities, and diagnostics.
The project does not reconcile field coordinate conventions or create joint
constraints, so evaluating one field cannot select, refit, mutate, or couple
another.

`GeoProject::try_reference_input` resolves a `FieldId` to a validated borrowed
`ReferenceFieldInput`. An unknown identifier returns a structured error. The
view delegates value, gradient, and Hessian requests to the referenced fitted
field in that field's original-coordinate convention. This is only a typed
identity and delegation boundary for later reference-controlled SPD local
mixtures: it adds no weighting rule, local anisotropy, coordinate reprojection,
geological-body topology, Boolean solid, fault relationship, or persistence
claim.

Independent tests cover distinct two-field interpolation results, stable
identifier values and insertion order, empty and duplicate rejection, valid
and missing reference inputs, pointer identity of the resolved immutable field,
delegated output evaluation, ownership after the source model is dropped,
construction in D=1/D=2/D=3, compile-time rejection of D=4, and `Send + Sync`
public project types. Rustdoc and the architecture contract describe the same
boundary.

Rust is implemented. CLI is N/A because versioned project formats and the
complete data CLI arrive in M8. C, C++, and Python are N/A because stable
project handles and bindings follow Rust API and schema freeze in M9. A focused
benchmark is N/A because projects delegate all fitting and evaluation to the
already benchmarked individual-field paths. This change adds no dependency,
unsafe code, hidden regularization, schema, adapter implementation, automatic
joint fit, or later-requirement local-trend behavior. Independent Review and
integration remain required before the registry may become `integrated`.
