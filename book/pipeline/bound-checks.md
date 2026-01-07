# Bound Checks

After desugaring temporaries, the remaining place expressions
are _mostly_ side-effect free. The exception is bounds checks.
In this step we make bounds checks explicit.

After the previous desugarings, all place expressions are broken into `let place` bindings, so the
only way an indexing expression may show up is as `let place q = p[i];` where `p` and `i` are
bindings.

We desugar this as follows, using [Unchecked Indexing](../features/unchecked-indexing.md):
```rust
let place q = p[i];

// becomes:
let len = core::slice::length(&raw const p);
assert!(i < len, "appropriate message");
let place q = unchecked_index!(p, i);
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
    let len = x.len();
    assert!(i < len);
    let p = unchecked_index!(x, i);
    let j = {x = &[]; 0};
    &mut p[j] // out of bounds access
};
```

Rustc rejects this code using borrow-checking tricks.
See [Borrow Checking](borrow-checking.md).

`let place` should probably have a way to disallow
invalidating a place alias.
