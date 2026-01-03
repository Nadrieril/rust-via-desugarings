# Lazy Boolean Operators

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

Inside an `if` expression, the `&&` and [`||`](../features/extended-let-chains.md)
operators are also allowed to be mixed with `if let` (this is called "`let`-chains").
We do not touch these at this stage, they will be dealt with in a later pass.

After this step, the only `&&` and `||` operators left are involved in `let`-chains.
In particular, they're all directly inside an `if`.
