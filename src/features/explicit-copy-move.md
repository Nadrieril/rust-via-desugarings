# Explicit Copy/Move

This feature adds an explicit syntax to explicitly copy or move the contents of a place:
- `copy!($place)` copies the contents of the place. It is a type error if applied to a non-`Copy`
  type;
- `move!($place)` moves the contents out of the place, regardless of whether the value implements
  `Copy`. The place is considered uninitialized afterwards.
