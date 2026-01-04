# Explicit Drop Locations

This step makes explicit where drops may happen, using the [`ensure_dropped!`
macro](../features/auto-drop.md).

We add `ensure_dropped!` statements in two locations: at end of scopes and before variable
assignments.

```rust
scope_end!($local);

// becomes:
ensure_dropped!($local);
scope_end!($local);
```

```rust
$place = $expr;

// becomes:
ensure_dropped!($place);
$place = $expr;
```

One tricky case is assignment through a mutable reference:
```rust
let a: &mut String = ...;
*a = String::new(); // this drops the previous string

// becomes:
ensure_dropped!(*a); // this drops the previous string
*a = String::new(); // borrowck knows there's no previous string to drop
```

This is not allowed in today's Rust, but is legal for us thanks to the [Moving Out Of
`&mut`](../features/moving-out-of-mut.md) feature.

After this step, all assignments are to statically uninitialized places (hence won't cause implicit
drops), and every `scope_end` is a no-op because the place is already known to be uninitialized.
