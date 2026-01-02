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
if matches!($place, $pat1) && $guard1 {
    $arm1
} else if matches!($place, $pat2) && $guard2 {
    $arm2
} else if matches!($place, $pat3) {
    $arm3
} else {
    unsafe { unreachable_unchecked() }
}
```

## Unnesting patterns

Of course this gains us nothing if `matches!` just expanded right back to a `match` expression.
Instead, we'll compile each `matches!` expression down to built-in comparisons by recursively
simplifying the expressions.

By way of example:
- `matches!($x, _)` => `true`;
- `matches!($x, 42u32)` => `$x == 42u32`;
- `matches!($x, 42u32..=73u32)` => `42u32 <= $x && $x <= 73u32`;
- `matches!($x, &$p)` => `matches!(*$x, $p)`;
- `matches!($x, ($p0, $p1))` => `matches!($x.0, $p0) && matches!($x.1, $p1)`;
- `matches!($x, Struct { a: $pa, b: $pb })` => `matches!($x.a, $pa) && matches!($x.b, $pb)`;
- `matches!($x, Enum::Variant { a: $pa, b: $pb }))` => `$x.enum#discriminant == discriminant_of!(Enum, Variant) &&
  matches!($x.Variant.a, $pa) && matches!($x.Variant.b, $pb)`;
- `matches!($x, [$pa, .., $pz])` => `$x.len() >= 2 && matches!($x[0], $pa) && matches!($x[x.len() - 1], $pz)`.

Note that we use [Enum Projections](../features/enum-projections.md) and [Enum Discriminant
Access](../features/enum-discriminant.md) for enums. Note also that we don't deal with or-patterns
because they've been dealt with already.

Note that the left-to-right order is important here; these are lazy boolean operators. In fact the
outcome of this desugaring step must go back to [Control-flow Desugarings](control-flow.md) to fix
that up.

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

[^1]: At least one difference is that rustc tests or-pattern alternatives after other patterns to
reduce duplicate work. So `matches!($x, (true|true, false))` is actually compiled to `matches!($x.1,
false) && matches!($x.0, true|true)`. There are also details around enums with only one variant.
Also constants in patterns get turned into patterns, which may behave differently than plain `==`
comparison does in terms of exact UB.
