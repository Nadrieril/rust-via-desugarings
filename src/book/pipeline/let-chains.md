# Let chains

"Let chains" are what we call expressions like `if let $pat1 = $expr1 && let $pat2 = $expr2`.
In this step, we desugar these. Note that we also support [`||` in let
chains](../features/extended-let-chains.md).

<!-- After the previous desugarings, the only patterns left are bindings, which makes our task easy. -->

In what follows, `$expr1`/`$expr2` are expressions made of `&&`, `||`, `let $binding
= $place`, `let binding;`, and boolean expressions.

First, the base cases:
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

```rust
if let $binding; {
    $then
} else {
    $else
}

// becomes
if true {
    let $binding;
    $then
} else {
    $else
}
```

Then the `&&` case, using block-`break` to jump over the `else` branch:
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
            break 'exit $then;
        }
    }
    $else
}
```

And finally the `||` case, which we specify as follows, with a duplication:
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
    $then // duplicated :/
} else {
    $else
}
```

After this step, the only remaining branching construct is `if` on booleans.


---

## Discussion

### Avoiding duplication

We'd like to avoid duplicating user code, especially as in this case
this can lead to exponential blowup of code size.
The tricky part is preserving drop order, particularly on unwind.
This section proposes a solution;
it is quite involved, yet it's the simplest I found.

First we get rid of non-`place` bindings in two steps:
1. Move binding declarations to the left by turning every `$bool_expr && let x;` into `let x; &&
   $bool_expr`;
2. Move binding declarations out of `||` as follows, where `x1` is a fresh name.
    (This uses [`scope_end!`](../features/scope-end.md))
    ```rust
    (let x; && $expr1) || $expr2
    // becomes
    let x1; && (let place x = x1 && $expr1 || { scope_end!(x1); true } && $expr2)
    ```
    ```rust
    $expr1 || (let x; && $expr2)
    // becomes
    let x1; && ({ scope_end!(x1); true } && $expr1 || let place x = x1 && $expr2)
    ```

This produces new top-level `&&`-chains, onto which we recursively apply the `&&` case above.
This takes care to declare a series of bindings in the correct order for each branch,
which ensures correct drop order even on unwind.

Now the only bindings left are `let place` bindings.
We first move these to the right by transforming `let place p = $place && $expr`
into `$expr && let place p = $place`.
If `$expr` mentioned `p`, we substitute `$place` in its stead.
This even allows swapping two `let place` bindings.

By the above the order of the `let place` bindings is unimportant,
so for any place `p` that has a `let place` alias
in both `||` alternatives, we can write the condition as follows:
```rust
($expr1 && let place p = $place1) || ($expr2 && let place p = $place2)
```
then transform it using [conditional place aliases](../features/let-place.md):
```rust
let branch;
    && ($expr1 && { branch = true; true } || $expr2 && { branch = false; true })
    && let place p = if_place!(branch, $place1, $place2)
```

At the end of this, the remaining `||`-chains involve only boolean expressions,
which we can desugar like in [Lazy Boolean Operators](boolean-operators.md)
without needing to care about binding scopes.

Worked example:
```rust
if (let Some(a) = foo() && let Some(b) = a.method())
    || (let Some(b) = bar() && let Some(a) = b.method()) {
  ..
}

// becomes:
{
    let foo_left;
    let a_left;
    let method_left;
    let b_left;
    let bar_right;
    let b_right;
    let method_right;
    let a_right;
    let branch;
    if ({ foo_left = foo(); true }
         && foo_left.is_some()
         && { a_left = foo_left.Some.0; true }
         && { method_left = a_left.method(); true }
         && method_left.is_some()
         && { b_left = method_left.Some.0; true }
         && { branch = true; true }
      ) || ({ scope_end!(b_left); true }
         && { scope_end!(method_left); true }
         && { scope_end!(a_left); true }
         && { scope_end!(foo_left); true }
         && { bar_right = bar(); true }
         && bar_right.is_some()
         && { b_right = bar_right.Some.0; true }
         && { method_right = b_right.method(); true }
         && method_right.is_some()
         && { a_right = method_right.Some.0; true }
         && { branch = false; true }
      ) {
        let place a = if_place!(branch, a_left, a_right);
        let place b = if_place!(branch, b_left, b_right);
        ..
    }
}
```

The hoops we have to jump through to avoid duplicating code are not great.
The thing we're trying to express is (somewhat) simple, but expressing it
in surface Rust is hard.

An alternative to the proposed approach would be to make all the scope ends and unwind paths
explicit beforehand, which would give us full control over
the order in which locals are dropped.
Attempts at writing this in a compositional way have so far failed,
hence the conditional `let place` approach.

An in-between solution could be a feature to control drop order of bindings dynamically.

Spec-wise we can possibly just keep the version that duplicates;
it has the benefit of utmost simplicity.
