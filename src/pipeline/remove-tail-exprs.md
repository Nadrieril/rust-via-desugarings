# Removing Tail Expressions

After the previous desugarings, any block that returns a value is the target of an assignment.
In this step we move the assignment inside the block so as to remove all tail expressions.

```rust
$place = {
    $statements;
    $expr
};

// becomes:
{
    $statements;
    $place = $expr;
}
```

```rust
$place = if $bool {
    $then
} else {
    $else
};

// becomes:
if $bool {
    $place = $then;
} else {
    $place = $else;
}
```

```rust
$place = loop {
    $statements;
    if $bool {
        break $expr;
    }
};

// becomes
loop {
    $statements;
    if $bool {
        $place = $expr;
        break;
    }
}
```

After this step, all blocks end in a statement rather than an expression, and all blocks and
control-flow expressions have type `()`.
