# If Let Guards

This is [RFC 2294](https://rust-lang.github.io/rfcs/2294-if-let-guard.html) which is to be
stabilized soon: this feature enables `if let` in match guards.

```rust
match ui.wait_event() {
    KeyPress(mod_, key, datum) if let Some(action) = intercept(mod_, key) => act(action, datum),
    ev => accept!(ev),
}
```
