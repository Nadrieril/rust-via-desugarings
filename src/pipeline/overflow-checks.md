# Overflow Checks

Depending on compilation flags, built-in arithmetic operations may introduce overflow checks.
We desugar them here.

```rust
$a + $b

// becomes, in debug mode:
{
    let (res, overflow) = core::intrinsics::add_with_overflow($a, $b);
    if overflow {
        panic!("appropriate message")
    }
    res
}
```

We do similar checks for subtraction, multiplication, division, remainder, and right/left shifts.
Some checks are not optional, like the zero check for division and remainder.

After this step, all built-in arithmetic operations are infallible.
