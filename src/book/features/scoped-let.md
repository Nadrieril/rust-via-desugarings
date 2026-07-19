# Scoped Let

This feature adds `let(in 'a) pattern = ...;` statements.
This is just like the same `let pattern = ...;` statement, except that the place lives until the end
of the named scope instead of the end of the current scope.

```rust
'a: {
    let x = thing();
    {
        let(in 'a) y = thing();
        let z = thing();
        // `z` dropped here
    }
    // `x` and `y` dropped here
}
```
