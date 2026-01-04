# Unchecked Indexing

Indexing `$place[$index]` is a place operation that has the side effect of panicking on
out-of-bounds access.
There is no equivalent place expression for indexing that does not do a bounds check.

This feature introduces the place expression `unchecked_index!($place, $index)` that does just that.
Just like builtin indexing, this also supports range indexing.
