# Move Expressions for Closure Captures

Closures and async blocks can "capture" places from their surrounding environment, which is an
implicit operation. To make this explicit I propose to use a feature called "move expressions"
that's been discussed lately.

The way I understand it, it works as follows: `move($expr)` is an expression valid in a closure
that:
1. Evaluates `$expr` in the parent of the closure;
2. Stores the result inside the closure;
3. The `move(..)` expression itself is a place expression that corresponds to where that result is
   stored (basically it's a field of the closure).

For example, a function that increments a captured variable can be expressed as:
```rust
// Implicit capture:
let mut increment = || x += 1;

// Explicit capture:
let mut increment = || *move(&mut x) += 1;
```

To see why it's important that `move(..)` be a place expression, consider:
```rust
let mut x = Some(42);
// This moves the whole of `x` inside the closure; this closure could be returned from
// the current function.
let mut replace = move |new: u32| x.replace(new);

// Explicit capture:
let mut replace = |new: u32| Option::replace(&mut move(x), new);
```

Here the `&mut move(..)` directly borrows the place where we stored the initial value, and modifies
it on each call. That would not work if `move(..)` was a value expression.
