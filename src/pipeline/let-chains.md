# Let chains

"Let chains" are what we call expressions like `if let $pat1 = $expr1 && let $pat2 = $expr2`.
In this step, we desugar these. Note that we also support [`||` in let
chains](../features/extended-let-chains.md).

<!-- After the previous desugarings, the only patterns left are bindings, which makes our task easy. -->

In what follows, `$expr1`/`$expr2` are expressions made of boolean expressions, `let $binding
= $place`, and `&&` and `||` operators.
```rust
if $expr1 && $expr2 {
    $then
} else {
    $else
}

// becomes
'exit: {
    if $expr1 {
        if $expr2 {
            $then
            break 'exit
        }
    }
    $else
}
```

```rust
if $expr1 || $expr2 {
    $then
} else {
    $else
}

// becomes
if $expr1 {
    $then
} else if $expr2 {
    $then
} else {
    $else
}
```

```rust
if let $binding = $place {
    $then
} else {
    $else
}

// becomes
if true {
    let $binding = $place;
    $then
} else {
    $else
}
```

After this step, the only remaining branching construct is `if` on booleans.

---

## Discussion

Once more we have patterns + alternations causing us to duplicate user code.
It is surprisingly tricky to avoid, because we need to make sure that drop order is correct on failure of any
condition and on unwind at any point.
Until I find a better solution, this duplication has the benefit of being immensely simple.

Unlike here, when desugaring or-patterns in matches we didn't need to duplicate the arm body.
That's because the drop order of or-patterns is defined to not depend on which alternative is
chosen.
This may prove to be trouble when mixing or-patterns and if-let guard patterns however,
so my bet is on changing or-patterns to drop depending on which alternative succeeded.
