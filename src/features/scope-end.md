# Explicit End Of Scope

When a binding goes out of scope, any parts of it that remained initialized are dropped,
and any borrows to it are invalidated.

This feature introduces a special macro `scope_end!($binding)` which has this same effect.

Operationally, this compiles to 1. appropriate calls to `core::ptr::drop_in_place` for any parts of
it that require it, followed by 2. a built-in operation that makes the borrow-checker consider the
place uninitialized.
