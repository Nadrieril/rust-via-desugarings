# Cleanup On Unwinding

This feature adds a new control-flow construct `on_unwind $expr { $block }`.

This evaluates `$expr` and returns its value,
unless `$expr` causes unwinding to occur.
In that case, the statements in `$block` are executed before
unwinding continues.

This is used to make cleanup code explicit.
This could look like:

```rust
let x = String::new();
function_call()

// becomes:
let x = String::new();
on_unwind function_call() {
  scope_end!(x);
};
```
