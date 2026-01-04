# Pattern Unnesting

Operationally, pattern matching expressions stand for a series of comparisons of discriminants or
integers.
In this step we'll compile each `if let` expression down to built-in comparisons by recursively
simplifying the expressions.

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

The left-to-right order is important here; these are lazy boolean operators.

At the end of this step, the only remaining patterns are `x`/`ref x`/`ref mut x`/`place x` bindings.

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

### Precise semantics

There are tiny discrepancies between this proposed desugaring and what the lang team has decided to
be true today. For example, non-`#[non_exhaustive]` enums with a single variant don't incur
a discriminant read today but do in this desugaring.
I propose that we should change the language in this case, to make the language simpler.

[^1]: At least one difference is that rustc tests or-pattern alternatives after other patterns to
reduce duplicate work. So `matches!($x, (true|true, false))` is actually compiled to `matches!($x.1,
false) && matches!($x.0, true|true)`. There are also details around enums with only one variant.
Also constants in patterns get turned into patterns, which may behave differently than plain `==`
comparison does in terms of exact UB.
