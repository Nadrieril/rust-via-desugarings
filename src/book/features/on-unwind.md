# Cleanup On Unwinding

This feature adds a new control-flow construct `on_unwind $expr { $block }`.

This evaluates `$expr` and returns its value,
unless `$expr` causes unwinding to occur.
In that case, the statements in `$block` are executed before
unwinding continues.

This is used to make cleanup code explicit.
This could look like:

```rust,example
let x = String::new();
function_call()

// becomes:
let x = String::new();
on_unwind function_call() {
  scope_end!(x);
};
```
(Where `scope_end!` is defined [here](./scope-end.md)).

This is different from `catch_unwind` because one cannot go back to normal function execution;
when the end of the `on_unwind` block is reached, unwinding continues into the parent stack frame.
(Tho maybe we could allow `return`ing from such a block, and `catch_unwind` could just be
implemented in terms of that?).
