# The Final Language

We have now fully desugared our Rust program. The resulting program uses a very limited subset of
Rust, described here.

An "constant value" `$const` is:
- A constant literal `true`, `42u32`, `3.14f64`, `"str literal"`, etc;
- A named constant `CONSTANT`;
- A [function item](https://doc.rust-lang.org/reference/types/function-item.html#function-item-types) ZST.

A "place expression" `$place` is:
- A local variable `x`;
- A named static or thread local `STATIC`;
- A dereference `*$place`;
- A field access `$place.$field_name`;
- An enum field access `$place.$variant_name.$field_name`;
- A discriminant access `$place.enum#discriminant` (see [Enum Discriminant
  Access](../features/enum-discriminant.md));
- Indexing into a slice of array `$place[$operand]`.

An "operand" `$operand` is:
- A place access `copy!($place)` or `move!($place)` (see [Explicit Copy/Move](../features/explicit-copy-move.md));
- A constant `$const`.

An "value expression" `$val_expr` is:
- An operand `$operand`;
- A borrow `&$place`/`&mut $place`/`&raw const $place`/`&raw mut $place`;
- A cast `$operand as $ty`;
- A built-in operation `$operand + $operand`, `$operand >= $operand`, `!$operand`, etc;
- A repeat expression `[$operand; $const]`.

A "statement" `$statement` is:
- Assignment `$place = $val_expr`;
  <!-- - Place mention `let _ = $place;` (needs to be kept for accurate borrow-checking); -->
- Function call `$place = $operand($operand..)`;
- If expression `if $operand { $block } else { $block }`;
- Loop expression `'a: loop { $block }`;
- Unwind cleanup expression `on_unwind { $block } { $block }` (see [Cleanup On Unwinding](../features/on-unwind.md));
- Named block `'a: { $block };`;
- Jumps `break 'a`/`continue 'a`;
- Return `return $operand`;
- `inline_asm!(..)`.

A "block" `$block` is a list of `;`-terminated statements. It is always of type `()`.

A fully desugared program is a series of variable declarations `let x: $ty;`/`let mut x: $ty;`
followed by a block.

## Difference with MIR

This target language is intentionally very close to
[MIR](https://rustc-dev-guide.rust-lang.org/mir/index.html)[^2]. The main differences are:
- Our language has structured control-flow whereas MIR has a graph of blocks with `goto`s;
- MIR explicitly tracks what cleanups happen on unwinding;
- MIR has `StorageLive`/`StorageDead` statements to track allocation/deallocation of locals; we
  instead have `let x;` to allocate and `scope_end!(x)` (see [Explicit End Of
  Scope](../features/scope-end.md)) that marks where the local is deinitialized. This may not be
  exactly equivalent;
- Instead of `return value;`, MIR has a return place that must be written to before returning.
  That's easy to recover from what we have;
- Bounds checks?;

The biggest missing piece is without a doubt the info about drops on unwind.
As discussed in [Drop Elaboration](drop-elaboration.md), this defeats part of the point of doing
drop elaboration early, since it will have to be done again to know what drops happen on unwind.
We might need to come up with a language feature that can express cleanup-on-unwind.

I also expect there to be a lot more hidden subtleties I haven't accounted for, e.g. around constant
evaluation or opaque types.

## Difference with MiniRust

MiniRust is intentionally quite close to MIR[^3]. Beyond the differences with MIR we already saw, to
get valid MiniRust we'd also need the following:
- Corountine transform, which transforms `async` blocks into state machines; I didn't know where to
  fit it in the desugarings;
- Change a bunch of intrinsic calls like `ptr::read`, `u32::add_with_overflow` to built-in
  operations.
- Anything else?

<!-- This is the level at which we can start to talk about precise semantics. The state-of-the art for -->
<!-- this, that exists today, is [MiniRust](https://github.com/minirust/minirust). MiniRust is a tiny -->
<!-- language that resembles MIR, with a formal and executable semantics. -->

TODO

---

## Discussion

- missing info for borrowck
- slice patterns
- monomorphization
- I end up duplicating user code

By far the trickiest part of all this was the handling of temporaries.
It infected everything else I tried to do.
The second trickiest was `let` chains + `if let` guards.
Third was or-patterns.

These all depend on each other in non-trivial ways.

TODO

Things to add/try before publishing:
- `on_unwind function_call(..) { $cleanup_blocks; }`
- try accurate borrowck

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
[^2]: MIR is actually a bunch of languages in a trenchcoat. The MIR I'm talking about here is a MIR
post-drop elaboration but pre-coroutine transform.
[^3]: In this case, a different MIR than I was talking about. MiniRust is closer to [runtime
MIR](https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/mir/enum.MirPhase.html#variant.Runtime).
