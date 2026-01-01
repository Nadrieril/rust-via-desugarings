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
The compiler determines automatically how to capture each place depending on how it is used.

In this step, we use [`move` expressions](../features/move-expressions.md) to make all closure captures
explicit.

Our initial example becomes:
```rust
let mut increment = || x += 1;

// desugars to:
let mut increment = || *move(&mut x) += 1;
```

Another example:
```rust
let mut x = Some(42);
// This moves the whole of `x` inside the closure; this closure could be returned from
// the current function.
let mut replace = move |new: u32| x.replace(new);

// desugars to:
let mut replace = |new: u32| Option::replace(&mut move(x), new);
```

After this step, all closure captures are done with `move` expressions.

## Discussion

There's a small caveat to this way of doing things: this desugaring may introduce new "variable
should be declared `mut`" errors. For example:

```rust
let mut x = 42;
let rx = &mut x;
let mut increment = || *rx += 1;

// desugars to:
let mut increment = || **move(&mut rx) += 1; // requires `let mut rx;`
```

There's actually a bit of a hack in the compiler to avoid exactly this: the compiler internally
supports a [unique-but-not-mutable
borrow](https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/mir/enum.MutBorrowKind.html#variant.ClosureCapture)
for this exact case.

A solution would be to expose this borrow to users; it could be named `&uniq T`.
