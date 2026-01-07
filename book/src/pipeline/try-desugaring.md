# Try Desugaring

`$expr?` desugars to (see [RFC](https://rust-lang.github.io/rfcs/3058-try-trait-v2.html#desugaring-)):

```rust
match Try::branch($expr) {
    ControlFlow::Continue(v) => v,
    ControlFlow::Break(r) => return FromResidual::from_residual(r),
}
```
