# Intermediate Subexpression Elimination

In this step we add intermediate variables for every subexpression.

Specifically, if an expression that isn't a simple binding can be written `expr!($subexpr)` where
`$subexpr` is a
[value expression](https://nadrieril.github.io/blog/2025/12/06/on-places-and-their-magic.html),
we rewrite it to `{ let tmp = $subexpr; expr!(tmp) }`.
We do this in an order that preserves normal left-to-right evaluation order.
We skip subexpressions that are constants.

```rust
let mut vec = Vec::new();
vec.push(42);
vec[0] += 1;

// becomes before this step:
(*<Vec<_> as DerefMut>::deref_mut(&mut vec))[0] += 1;

// becomes after this step:
{
    let tmp1 = &mut vec;
    let tmp2 = <Vec<_> as DerefMut>::deref_mut(tmp1);
    *tmp2 += 1;
}
```

At the end of this step, every [value
context](https://nadrieril.github.io/blog/2025/12/06/on-places-and-their-magic.html)
contains either a constant or a variable.

---

## Discussion

We don't really have to skip constants. I just did it that way because MIR allows constants
a operands. I think it's a practical matter of not ending up with billions of variables just for
constants.
