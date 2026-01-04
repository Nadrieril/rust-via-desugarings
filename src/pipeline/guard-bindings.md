# Match Guard Mutable Bindings

Match guards are only allowed shared-reference access to the variables bound in the branch.
In the previous step we dealt with by-value bindings.
In this step we deal with by-mutable-reference bindings, using [`let
place`](../features/let-place.md) and [If Let Guards](../features/if-let-guards.md) again.

Let `$pat` be a pattern that has a `ref mut x` binding.
We desugar this as follows:
```rust
match .. {
    $pat if $guard => $arm,
    ..
}

// becomes
match .. {
    // To give only shared-access but have `x` keep its type, we use the little trick again:
    $pat if let x1 = &x && let place x = *x1 && $guard => {
        $arm
    }
    ..
}
```

After this step, guards no longer need special handling around bindings.

---

## Discussion

### Modifying discriminants in match guards

Guards actually have another mutability restriction: for soundness they must not be allowed to
modify any discriminant that participates in the match.

In rustc this is enforced using borrow-checker tricks.
This desugaring ignores that entirely; see also [Borrow Checking](borrow-checking.md).
