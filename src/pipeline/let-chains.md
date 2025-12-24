# Lazy Boolean Operators and Let chains

The boolean operators `&&` and `||` are lazy, which means that they only evaluate their rhs if it is
can influence the final value of the operation.

```rust
$lhs || $rhs
// desugars to:
if $lhs { true } else { $rhs }
```

```rust
$lhs && $rhs
// desugars to:
if $lhs { $rhs } else { false }
```

Inside an `if` expression, the `&&` operator is also allowed to be mixed with `if let`. This is
called "let chains" and looks like `if let $pat1 = $expr1 && let $pat2 = $expr2`.

To desugar this, we'll make use of "block `break`", which enables cool control-flow tricks:
```rust
if let $pat1 = $expr1 && let $pat2 = $expr2 {
    $then_expr
} else {
    $else_expr
}

// becomes
'exit: {
    if let $pat1 = $expr1 {
        if let $pat2 = $expr2 {
            break 'exit $then_expr
        }
    }
    $else_expr
}
```

This covers all the uses of `&&`/`||`[^1]. After this step, there are no `&&` or `||` operators
left.

[^1]: This will no longer be true once the `if_let_guard` feature gets [stabilized](https://github.com/rust-lang/rust/pull/141295). Desugaring it looks quite tricky in fact: to desugar match guards we need to have handled match guard bindings, which requires expanding or-patterns, but then the `let` guards themselves might contain or-patterns! For that reason I decided not to cover `if_let_guard`s for now.
