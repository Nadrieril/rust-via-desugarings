# Unchecked Indexing

`$place[$index]` is a place operation that has the side effect of panicking on
out-of-bounds access.

This feature introduces the place expression `unchecked_index!($place, $index)` that does the same
but without the bounds check.
Just like built-in indexing, this supports range indexing.

It is of course unsafe to use.
