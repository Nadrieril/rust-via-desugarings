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
- Unchecked indexing `unchecked_index!($place, $operand)`, `unchecked_index!($place,
  $operand..=$operand)` (see [Unchecked Indexing](../features/unchecked-indexing.md)).

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
- A variable declaration `let x: $ty;`/`let mut x: $ty;`.
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

A fully desugared function body is a block.

## Difference with MIR

This target language is intentionally very close to
[MIR](https://rustc-dev-guide.rust-lang.org/mir/index.html)[^2]. The main differences are:
- Our language has structured control-flow whereas MIR has a graph of blocks with `goto`s;
- MIR has `StorageLive`/`StorageDead` statements to track allocation/deallocation of locals; we
  instead have `let x;` to allocate and `scope_end!(x)` (see [Explicit End Of
  Scope](../features/scope-end.md)) that marks where the local is deinitialized. This may not be
  exactly equivalent;
- Instead of `return value;`, MIR has a return place that must be written to before returning.

There a probably a ton of subtle differences I haven't noticed,
but overall going from this to MIR looks pretty straightforward.

## Difference with MiniRust

MiniRust is intentionally quite close to MIR[^3]. Beyond the differences with MIR we already saw,
from what I know to get valid MiniRust we'd also need at least the following:
- Corountine transform, which transforms `async` blocks into state machines; I didn't know where to
  fit that in the desugarings;
- Change a bunch of intrinsic calls like `ptr::read`, `u32::add_with_overflow` to built-in
  operations.

But I haven't investigated in detail.

---

## Discussion

I feel like this accomplished the goals I set out in the introduction.
I noted decisions made and caveats throughout the book.

The most important caveat to note is the question of borrow-checking.
In [the relevant section](borrow-checking.md), I highlight how borrow-checking after our desugarings
would allow unsound code to compile.
We therefore need to either borrow-check somewhere in the middle or desugar differently.
I am leaving this question open for now.

I am also left unsatisfied with [the desugaring of `||`-chains](let-chains.md).
It seems we have two bad choices: duplicate user code (and risk exponential blowup),
or emit nasty code.

## Conclusion

I am pretty pleased with the shape this took.
I am now convinced that this way of presenting things is fruitful.
In an idealized universe, this book would be combined with MiniRust and a-mir-formality to make
a reference interpreter written in literate programming style;
if I had to choose I would quite like that as an official spec for Rust.

By far the trickiest part of all this was the interaction between temporaries and patterns[^4].
I'd like to thank @dianne for helping me figure out a pass ordering that doesn't loop onto itself
all over the place and helping me get temporary lifetimes right.

I don't know what will become of this book now. I'd like it to become some form of official document
that is kept up-to-date as the language evolves. Only time will tell if this will be deemed
a worthy investment.

Thanks for reading, and please let me know[^1] if you found this useful!
If you find mistakes or missing details please open an issue or a PR!

[^1]: I'm @Nadrieril on the [rust-lang Zulip](https://rust-lang.zulipchat.com), that's the easiest way to reach me.
[^2]: MIR is actually a bunch of languages in a trenchcoat. The MIR I'm talking about here is a MIR
post-drop elaboration but pre-coroutine transform.
[^3]: In this case, a different MIR than I was talking about. MiniRust is closer to [runtime
MIR](https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/mir/enum.MirPhase.html#variant.Runtime).
[^4]: Desugaring or-patterns required temporaries to be handled, but these seemed to require let
chains to be desugared, but we can't desugar let chains in if let guards before desugaring match
guards in some form, etc etc. It was all a big fun mutually-dependent knot.
