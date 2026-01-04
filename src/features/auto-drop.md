# Automatic Drop

This feature adds a `ensure_dropped!($place)` builtin macro.
When executed, this drops any part of `$place` that hasn't already been moved out of.
