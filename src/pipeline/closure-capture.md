# Closure capture

Closures and async blocks can refer to the places in their environment:
```rust
let mut x = 0;
let mut increment = || x += 1; // `&mut x` is captured
increment();
increment();
assert_eq!(x, 2);
```

This gets eventually compiled to a data type that stores references to, or the contents of, the
captured places. The compiler determines automatically how to capture each place depending on how it
is used in the closure/async block.

In this step, we use [`move` expressions](../features/move-expressions.md) to make all these
captures explicit (note that this is very different from `move!(..)` introduced in [Explicit
Copies/Moves](copy-move.md)).

Our initial example becomes:
```rust
let mut increment = || x += 1;

// desugars to:
let mut increment = || *move(&mut x) += 1;
```

Another example:
```rust
let mut x = Some(42);
let mut replace = move |new: u32| Option::replace(&mut x, new);

// desugars to:
let mut replace = |new: u32| Option::replace(&mut move(x), new);
```

This final example uses a unique immutable borrow (which we introduce in [Unique-Immutable
Borrow](../features/uniq-borrow.md)) since a `&mut` borrow would require `let mut xs`:
```rust
let mut x = 42;
let rx = &mut x;
let mut increment = || *rx += 1;

// desugars to:
let mut increment = || **move(&uniq rx) += 1;
```

See [the Reference](https://doc.rust-lang.org/reference/types/closure.html#r-type.closure) for
details about what gets captured and how.
Thanks to previous desugarings all place uses are explicit, which makes the analysis
straightforward.

After this step, all closure and async block captures are explicit, using `move` expressions.
