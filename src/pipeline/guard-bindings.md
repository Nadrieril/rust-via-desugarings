# Match Guard Mutable Bindings

Match guards are only allowed shared-reference access to the variables bound in the branch.
In the previous step we dealt with by-value bindings.
In this step we deal with by-mutable-reference bindings, using [`let
place`](../features/let-place.md) again.

Let `$pat` be a pattern that has a `ref mut x` binding.
We desugar this as follows:
```rust
match .. {
    $pat if $guard => $arm,
    ..
}

// becomes
match .. {
    // To give only shared-access but have `x` be of type `&mut T`, we do a little hack:
    $pat if let x1 = &x && let place x = *x1 && $guard => {
        $arm
    }
    ..
}
```

After this step, guards no longer need special handling around bindings.
