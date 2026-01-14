# Explicit End Of Scope

When a binding goes out of scope, any parts of it that remained initialized are dropped,
and any borrows of it are invalidated.

This feature introduces a special macro `scope_end!($local)` which has this same effect.

Operationally, this compiles to a call to [`ensure_dropped!($place)`](./auto-drop.md), followed by
a deallocation of the local.

Unlike move outs, this is permanent: it is an error to re-initialize `$local` after
`scope_end!($local)`.

Note: `scope_end!($local); scope_end!($local);` is the same as `scope_end!($local);`.
