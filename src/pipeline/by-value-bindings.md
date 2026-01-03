# By-Value Bindings

By-value bindings are special because they need to work in two steps: first we check that the whole
pattern matches, then we can set the by-value bindings.
To get flexibility in the coming steps, we transform these bindings using [`let
place`](../features/let-place.md) and [If Let Guards](../features/if-let-guards.md).

Let `pat!(x)` stand for a pattern that involves a by-value binding `x`.

Inside a let-chain, we turn `let pat!(x) = $expr` into `let pat!(place p) = $expr if let x = p`.

For matches without guards, we desugar as follows:
```rust
match .. {
    pat!(x) => $arm,
    ..
}

// becomes:
match .. {
    pat!(place p) => {
        let x = p;
        $arm
    }
    ..
}
```

For match guards, we need to make sure that any moves happen only when the guard succeeds.
To make this work, we only give guards shared access to the value.
In order for the binding to have the right type, we use a clever trick:
```rust
match .. {
    pat!(x) if $guard => $arm,
    ..
}

// becomes:
match .. {
    pat!(place p) if let x1 = &p && let place x = *x1 && $guard => {
        let x = p;
        $arm
    }
    ..
}
```

After this step, patterns no longer have by-value bindings.
