# `let place` Desugaring

In this step, we desugar [`place p` bindings](../features/let-place.md).

All the patterns have been desugared away, so the only `place p` bindings are directly in the form
`let place p = $expr;`. Moreover we also desugared `$expr` so that it is a "pure place expression",
in particular side-effect-free.

We can therefore simply substitute `$expr` for `p` syntactically.
For example:
```rust
let place p = x.field;
something(&p);
something_else(p);

// becomes:
something(&x.field);
something_else(x.field);
```

After this step, there are no `place p` bindings left.
