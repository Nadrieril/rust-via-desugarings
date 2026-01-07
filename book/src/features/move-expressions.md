# Move Expressions for Closure Captures

Closures and async blocks can "capture" places from their surrounding environment, which is an
implicit operation. To make this explicit I propose to use a feature called "move expressions"
that's been discussed lately.

This feature adds a `move($expr)` expression, valid in a closure, that:
1. Evaluates `$expr` in the parent of the closure;
2. Stores the result inside the closure;
3. Acts as a place alias for the place where that result is stored (i.e. the field of the closure object).

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

Move expressions can be nested: `move(move($expr))` evaluates the inner `move($expr)` when the outer
closure is created, then the outer `move(move($expr))` when the inner closure is created:
```rust
let mut x = Some(42);
let generate_replacer = || {
  do_some_stuff();
  |new: u32| Option::replace(&mut move(move(x)), new)
};
// is equivalent to:
let generate_replacer = || {
  do_some_stuff();
  let inner_x = move(x);
  |new: u32| Option::replace(&mut move(inner_x), new)
};
```
