# Problem Intermediate Representations

## SemanticProblemIR

The semantic form retains observation and level identifiers, source file and
line, original units, field path, constraint group, compiled functional
expression, relation, enforcement, loss, and execution options. Validation and
diagnostics operate here so errors remain traceable to user input.

## CanonicalProblem

The numerical form is

```text
minimize 0.5 z^T H z + g^T z
subject to
    A_eq z = b_eq
    lower <= A_lin z <= upper
    ||F_j z + f_j||_2 <= c_j^T z + d_j.
```

It records variable blocks, row provenance, scaling, solver capabilities, and
memory estimates but contains no level, horizon, normal, tangent, stratigraphy,
or lithology type. Compilation is deterministic for a fixed input and policy.

Centers and observations remain separate through both forms. Canonicalization
checks finite values, units, normalization, direction norms, duplicate and
conflicting functionals, polynomial rank, kernel derivative capability,
anisotropy validity, gauge, contrast, and memory bounds before allocation or
solution.
