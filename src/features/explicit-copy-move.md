# Explicit Copy/Move

Rust does not have an explicit syntax to explicitly copy or move the contents of a place, so allow me
to pretend there are builtin operations that do that; I'll write them `copy!` and `move!`.

- `copy!($place)` copies the contents of the place. It is a type error if applied to a non-`Copy`
  type;
- `move!($place)` moves the contents out of the place, regardless of whether the value implements
  `Copy`.
