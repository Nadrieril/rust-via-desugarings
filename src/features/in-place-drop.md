# In-Place Drop

This feature introduces a built-in macro operator `drop_in_place!($place)` that 1. sets the place as
uninitialized for the purposes of borrow-checking, and 2. calls `core::ptr::drop_in_place(&raw mut
$place)` on it.

This is used in [Drop Elaboration](../pipeline/drop-elaboration.md) to make drops explicit.

This can't be desugared to two separate steps because deinitializing first would make the `&raw mut`
borrow invalid, and deinitializing last would cause the place to be double-dropped if the drop code
panics.
