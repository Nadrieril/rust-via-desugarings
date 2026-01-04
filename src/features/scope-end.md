# Explicit End Of Scope

When a binding goes out of scope, any parts of it that remained initialized are dropped,
and any borrows of it are invalidated.

This feature introduces a special macro `scope_end!($binding)` which has this same effect.

Operationally, this compiles to 1. appropriate calls to `core::ptr::drop_in_place` for any parts of
the place still initialized, followed by 2. a built-in operation that makes the borrow-checker
consider the place uninitialized.

Unlike move outs, this is unconditional: it is an error to re-initialize `x` after `scope_end!(x)`.

Note: by this definition, `scope_end!(x); scope_end!(x);` is the same as `scope_end!(x);`.
