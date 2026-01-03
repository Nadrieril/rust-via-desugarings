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
'exit: {
    // TODO: how to get the drop order right
    if $expr1 {
        // uhh somehow forward the bindings in the right order
    } else if $expr2 {
        // uhh somehow forward the bindings in the right order
    } else {
        $else
        break 'exit
    }
    $then
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
