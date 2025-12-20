# Explicit Copies/Moves

When a place expression is used where a value is needed, the contents of the place are copied or
moved out, depending on the type of the place (this is called a "place-to-value coercion").

Rust does not have an explicit syntax to distinguish these two, so allow me to pretend there are
builtin operations that do that; I'll write them `copy!` and `move!`.

This step adds a `copy!` or `move!` to every place-to-value coercion:
```rust
let x = (String::new(), 42);
let y = x.0;
let z = x.1;
// becomes:
let y = move!(x.0);
let z = copy!(x.1);
```

The only different between these two if for the borrow-checker today, but the underlying semantic
model of Rust reserves the right to care about the difference eventually.
