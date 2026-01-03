# Match Lowering

Operationally, pattern matching expressions stand for a series of comparisons of discriminants
or integers. In this step we do that transformation.

## `match` to `if`

Because we've dealt with all the bindings, all remaining patterns have no bindings. Moreover the
scrutinee of a match has been turned into a side-effect-less place expression.
We can therefore transform:
```rust
match $place {
    $pat1 if $guard1 => $arm1,
    $pat2 if $guard2 => $arm2,
    $pat3 => $arm3,
}
```
into:
```rust
if let $pat1 = $place && $guard1 {
    $arm1
} else if let $pat2 = $place && $guard2 {
    $arm2
} else if let $pat3 = $place {
    $arm3
} else {
    unsafe { unreachable_unchecked() }
}
```

## Unnesting patterns

Of course this gains us nothing if the `if let` to expanded right back to `match` expressions.
Instead, we'll compile each `let` expression down to built-in comparisons by recursively simplifying
the expressions.

By way of example:
- `let _ = $x` => `true`;
- `let 42u32 = $x` => `$x == 42u32`;
- `let 42u32..=73u32 = $x` => `42u32 <= $x && $x <= 73u32`;
- `let &$p = $x` => `let $p = *$x`;
- `let ($p0, $p1) = $x` => `let $p0 = $x.0 && let $p1 = $x.1`;
- `let Struct { a: $pa, b: $pb } = $x` => `let $pa = $x.a && let $pa = $x.b`;
- `let Enum::Variant { a: $pa, b: $pb } = $x` => `$x.enum#discriminant == discriminant_of!(Enum, Variant) &&
  let $pa = $x.Variant.a && let $pb = $x.Variant.b`;
- `let [$pa, .., $pz] = $x` => `let len = core::slice::len(&raw const $x) && len >= 2 && let $pa = $x[0] && let $pz = $x[len - 1]`.

Note that we use [Enum Projections](../features/enum-projections.md) and [Enum Discriminant
Access](../features/enum-discriminant.md) for enums. Note also that we don't deal with or-patterns
because they've been dealt with already.

Note that the left-to-right order is important here; these are lazy boolean operators.
After this step we go back to [Control-flow Desugarings](control-flow.md) to fix that up,
and continue until no new `let` chains are produced.

At the end of this step, the only remaining branching construct is `if`.

---

## Discussion

### Evaluation order

The exact semantics of patterns are not decided yet. What's presented in this section is actually
a proposal I'm putting forward, that happens to mostly[^1] be compatible with what's implemented in
rustc today.

This proposal has the benefit and drawback of setting in stone a particular order of evaluation.
This is useful for unsafe code that may want to know exactly what is accessed in which order, and
detrimental to optimizations.

The `unsafe` code that motivated me to fix the order is manually-implemented tagged unions:
```rust
struct MyOption<T> {
    is_some: bool,
    contents: MyOptionContents<T>,
}
union MyOptionContents<T> {
    uninit: (),
    init: T,
}

impl<T> MyOption<T> {
    fn as_ref(&self) -> Option<&T> {
        unsafe {
            match *self {
                MyOption { is_some: true, contents: MyOptionContents { ref init } } => Some(init),
                MyOption { is_some: false, .. } => None,
            }
        }
    }
}
```

I don't know if this sufficient motivation, nor exactly the extent of optimizations we'd lose if we
set this in stone.

### Modifying discriminants in match guards

Orthogonally to this, this skips over some borrow-checking considerations: today the borrow-checker
prevents match guards from altering discriminants that participate in the match, which is required
for soundness. This desugaring ignores that entirely; see also [Borrow
Checking](borrow-checking.md).

### Why go `if let` -> `match` -> `if let`?

The current series of desugarings looks silly for `if let`: we go through a match then back to `if
let`. The reasoning is that `match`es have a special interaction
between guards and or-patterns, so we must expand or-patterns before turning `match`es to `if let`s.

Now if we didn't first turn `if let`s to `match`es, we'd need to handle or-patterns in `if let`
separately, which would look a bit redundant (also we need to handle or-patterns before we can
separate bindings from patterns).
Hence this choice I made to go through `match`.

And why not then move this whole thing before the first `if let`-chain desugaring? Because before we
turn `match`es to `if let`s we must handle temporary lifetime extension, and before we can handle
temporary lifetime extension we must expand `let` chains, I think.

### Looping back to earlier desugarings

This desugaring has the drawback that it emits code with constructs like `&&` that we should have
already desugared, which requires looping back to earlier desugaring passes.
On the face of it this is unfortunate, but it's secretly in preparation for the [`if let`
guards](https://rust-lang.github.io/rfcs/2294-if-let-guard.html) feature, which I really don't see
how to handle without a desugaring fixpoint.

[^1]: At least one difference is that rustc tests or-pattern alternatives after other patterns to
reduce duplicate work. So `matches!($x, (true|true, false))` is actually compiled to `matches!($x.1,
false) && matches!($x.0, true|true)`. There are also details around enums with only one variant.
Also constants in patterns get turned into patterns, which may behave differently than plain `==`
comparison does in terms of exact UB.
