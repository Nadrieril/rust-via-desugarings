# Automatic Drop

This feature adds a `ensure_dropped!($place)` built-in macro.

This compiles to appropriate calls to [`drop_in_place!($place)`](./in-place-drop.md)
for any parts of the place still initialized.
