# Bound Checks

After desugaring temporaries, the remaining place expressions
are _mostly_ side-effect free. The exception is bounds checks.
In this step we make bounds checks explicit.

The tricky part is where to place the check.
Given a place expression `$place` that contains an indexing subexpression,
let `expr!($place)` be the smallest value expression that contains it.

Now write `$place` as `place!($base[$index])` where `$base` contains no bound-checked indexing (this
is to make sure we do the checks in the right order).

Then we desugar as follows, using [Unchecked Indexing](../features/unchecked-indexing.md):
```rust
expr!(place!($base[$index]))

// becomes:
{
    let len = core::slice::length(&raw const $base);
    assert!($index < len, "appropriate message");
    expr!(place!(unchecked_index!($base, $index)))
}
```

We do something similar for range indexing.

At the end of this step, there are no checked indexing place expressions left.

---

## Discussion

This desugaring is actually unsound if we don't run borrow-checking before doing it:
```rust
let mut x: &[[u32; 1]] = &[[42]];
let _ = &mut x[0][{x = &[]; 0}];

// becomes:
let _ = {
    let i = 0;
    assert!(i < x.len());
    let j = {x = &[]; 0};
    &mut unchecked_index!(unchecked_index!(*x, 0), j); // out of bounds access
};
```

Rustc rejects this code using borrow-checking tricks.
We should probably find a way to emulate them.
See [Borrow Checking](borrow-checking.md).
