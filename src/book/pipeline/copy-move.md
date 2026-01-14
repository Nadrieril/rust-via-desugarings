# Explicit Copies/Moves

When a place expression is used where a value is needed, the contents of the place are copied or
moved out, depending on the type of the place (this is called a "place-to-value coercion").

We'll use the `copy!` and `move!` operators proposed in [Explicit
Copy/Move](../features/explicit-copy-move.md).
This step adds a `copy!` or `move!` to every place-to-value coercion:
```rust
let x = (String::new(), 42);
let y = x.0;
let z = x.1;

// becomes:
let y = move!(x.0);
let z = copy!(x.1);
```

After this step, the use of each place is explicit: copy, move, borrow, etc.
