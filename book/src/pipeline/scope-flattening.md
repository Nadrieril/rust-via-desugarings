# Scope Flattening

Now that all ends of scope are explicit, we can remove any blocks that aren't the target of a `break
'label`.

```rust
{
    let x;
    x = String::new();
}

// becomes
let x;
x = String::new();
```
