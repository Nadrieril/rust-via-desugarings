# Functional Record Update

In this step we desugar [Functional Record
Update](https://doc.rust-lang.org/reference/expressions/struct-expr.html#r-expr.struct.update)
syntax:

```rust
// Assume `Struct` has 4 fields named a,b,c,d
x = Struct { a: $expr_a, b: $expr_b, ..$place };

// becomes:
x = Struct { a: $expr_a, b: $expr_b, c: $place.c, d: $place.d };
```

This is correct because `..$expr` is a place context and we've already made
sure that all place context contain side-effect free place expressions.
