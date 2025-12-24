# Match Unnesting

Operationally, pattern matching expressions just stand for a series of comparisons of discriminants
or integers. So in principle we could desugar a `match` to a big series of `if`s. However that would not
give us the same MIR as we get today: the lowering of patterns to MIR is a bit sophisticated[^1], to
emit more performant code. I also find that more pleasing. So let's try to preserve that.


At the end of this step the only `match`es left will be on integer, char or boolean constants. As
a shorthand we'll allow `if b` to stand for the obvious corresponding `match` on booleans. This will
be the only remaining branching construct.

## Match guards

Let's start with match guards:
```rust
match $expr {
    Some(_) if $guard => ..,
    None => ..,
    Some(_) => ..,
}
```
In that match, if `$expr` returns `false` we keep trying the arms below. I propose to add a syntax
for that:
```rust
'a: match $expr {
    Some(_) => if $guard {
        ..
    } else {
        continue 'a; // tries the arms after this one
    },
    None => ..,
    Some(_) => ..,
}
```

To be precise, this `continue` statement would mean: "continue trying to match the arms below this
one". Making this into a fully-fledged feature is out of scope of this document but would
have to be treated specifically by match exhaustiveness

## Shallow patterns

The basic building block we'll desugar everything to is `match`ing on integers and booleans. As
a shorthand we'll allow `if b` to stand for the obvious corresponding `match` on booleans. The
allowed patterns in these matches are constant patterns and the catch-all `_` pattern. Everything
else will desugar to those.

Non-nested patterns then desugar as follows:
- Boolean and integer literal patterns stay as-is;
- Float literals become `if $val == $literal` comparisons;
- Float and integer range patterns become `if $start < $val && $val < end` comparisons;
- Enum patterns become `match`es on the enum discriminant;
- Slice patterns become length checks.

TODO: more detail, also explain discriminants, and subplaces, and enum projections

## Nested patterns

Now we're left with nested patterns. A nested pattern can be decomposed as 1. a "outer test", which
corresponds to one of the case we just saw, and 2. its subpatterns, that each apply to a subplace.

For example, `Some(42)` decomposes into a test that `$scrutinee.discriminant == Some`, and a nested
pattern match `matches!($scrutinee.Some.0, 42)`. By repeatedly applying this decomposition, in the
end we get only shallow tests like in the previous section.

TODO: explain that we skip into the inside of irrefutable constructors

The way the whole desugaring works is that we take the leftmost test of the first match arm, do that
test, and figure out what remains to be done in each branch. We make heavy use of the
`match`-`continue` feature:
```rust
match val {
    (Some(_), None) => .., // branch 1
    (_, Some(_)) => .., // branch 2
    (None, None) => .., // branch 3
}

// desugars to:
'a: match val.0.enum#discriminant {
    discriminant_of!(Option, Some) => match val {
        (_, None) => .., // branch 1
        (_, Some(_)) => continue 'a,
    },
    _ => match val {
        (_, Some(_)) => .., // branch 2
        (None, None) => .., // branch 3
    }
}

// then to:
'a: match val.0.enum#discriminant {
    discriminant_of!(Option, Some) => match val.1.enum#discriminant {
        discriminant_of!(Option, None) => {
            // branch 1
        }
        discriminant_of!(Option, Some) => continue 'a,
    },
    _ => match val {
        (_, Some(_)) => .., // branch 2
        (None, None) => .., // branch 3
    }
}

// and finally to:
'a: match val.0.enum#discriminant {
    discriminant_of!(Option, Some) => match val.1.enum#discriminant {
        discriminant_of!(Option, None) => {
            // branch 1
        }
        discriminant_of!(Option, Some) => continue 'a,
    },
    _ => match val.1.enum#discriminant {
        discriminant_of!(Option, Some) => {
            // branch 2
        }
        discriminant_of!(Option, None) => match val.0.enum#discriminant {
            discriminant_of!(Option, None) => .., // branch 3
            _ => unsafe { unreachable_unchecked() },
        },
    },
}
```

This way of doing things might look weird but it has the benefit of not duplicating the body of
branches. That's also what rustc does today[^2].


[^1]: It's not _that_ sophisticated actually, e.g. on the second example below it could figure out
that matching on `val.1` first produces better code. But we don't do that kind of reasoning yet.
[^2]: There's a subtle point here spec-wise: it's not yet settled in Rust what the compiler is or
isn't allowed to do with patterns. This desugaring is roughly what the compiler does today, but we
may decide to specify something less precise to leave space for future optimizations.
