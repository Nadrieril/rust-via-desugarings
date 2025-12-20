# Closure capture

Closures and async blocks can refer to the places in their environment:
```rust
let mut x = 0;
let mut increment = || x += 1; // `&mut x` is captured
increment();
increment();
assert_eq!(x, 2);
```

What this compiles to is an ADT that stores references to, or the values of, the captured places.
This operation involves a bit of magic, as it has to infer whether to move a value into the
closure, or whether to take a reference to it, and if so with which mutability.

To make this explicit I propose to add a feature called "move expressions" that's been discussed
lately. The way I understand it, `move($expr)` is an expression valid in a closure that
1. Evaluates `$expr` in the parent of the closure;
2. Stores the result inside the closure;
3. The `move(..)` expression itself is a place expression that corresponds to where that result is
   stored (basically it's a field of the closure).

Our initial example becomes:
```rust
let mut increment = || x += 1;

// desugars to:
let mut increment = || *move(&mut x) += 1;
```

To see why it's important that `move(..)` be a place expression, consider:
```rust
let mut x = Some(42);
// This moves the whole of `x` inside the closure; this closure could be returned from
// the current function.
let mut replace = move |new: u32| x.replace(new);

// desugars to:
let mut replace = |new: u32| Option::replace(&mut move(x), new);
```

Here the `&mut move(..)` directly borrows the place where we stored the initial value, and modifies
it on each call.

After this step, all closure captures are done with `move` expressions[^1].

[^1]: There's a small detail: with this desugaring, we may get some "variable should be declared
`mut`" errors that we didn't get with the original closure. The reason is that MIR actually has
a special unique-but-not-mutable borrow for when a closure captures a `x: &mut T` and uses it
without mutating `x` itself. Our desugaring would capture `&mut x`, triggering the error.
