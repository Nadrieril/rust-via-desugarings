# The Final Language

We have now fully desugared our Rust program. The resulting program uses a very limited set of basic
operations, as follows:

- All locals are individually pre-declared as `let x;`/`let mut x;` at the start of their scope;
- The only compound expressions are place expressions, made of locals `x`, builtin deref `*x`, field
  access `x.field` and builtin indexing `x[i]`;
- The only control-flow constructs are blocks, `if` and `loop`, along with `break` (valid for
  loops and blocks), `continue` (valid for loops only) and `return`;
- Statements are limited to the following:
    - `$place = $expr`;
    - `$place = call($expr, $expr..)`;
    - `let _ = $place` (needs to be kept for accurate borrow-checking);
    - control-flow constructs: `if`/`loop`/`return`/`break`/`continue`;
    - `drop!($place)`;
    - `inline_asm!(..)`;

TODO: how about `conditional_drop!($place)` before drop elab?
TODO: need storagelive/storagedead if we're getting rid of scopes
TODO: desugar async to general coroutines, at least

## Difference with MIR

- Control-flow;
- Explicit unwinding cleanup blocks;
- `StorageLive`/`StorageDead`;
- Bounds checks;

Biggest missing piece: unwind blocks & drops on unwind

## MiniRust

Missing:
- Corountine transform
- Bunch of intrinsics that look like function calls today, e.g. `ptr::read`.

This is the level at which we can start to talk about precise semantics. The state-of-the art for
this, that exists today, is [MiniRust](https://github.com/minirust/minirust). MiniRust is a tiny
language that resembles MIR, with a formal and executable semantics.

---

## Discussion

- edition hygiene
- missing info for borrowck
- slice patterns

TODO

## Conclusion

<!-- should only use a limited set of -->
<!-- basic operations. At that level, type-checking ideally is really just a check that doesn't change -->
<!-- program behavior. -->

My hope with these desugarings is that we can reach a subset of Rust that is as precise as MiniRust
is today, thus bridging the gap from source Rust to MiniRust. I don't think we're quite there yet
but hopefully this is a good step in that direction!

Thanks for reading, please open issues if you find mistakes or missing details, and let me know[^1] if
you found this useful!

[^1]: I'm @Nadrieril on the [rust-lang Zulip](https://rust-lang.zulipchat.com), that's the easiest way to reach me.
