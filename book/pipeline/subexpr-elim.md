# Intermediate Subexpression Elimination

In this step we add intermediate variables for every subexpression.

Specifically, if an expression that isn't a simple binding can be written `expr!($subexpr)`,
we rewrite it.
If `$subexpr` is a [value expression](https://nadrieril.github.io/blog/2025/12/06/on-places-and-their-magic.html),
we rewrite it to `{ let tmp = $subexpr; expr!(tmp) }`;
if it is a place expression,
we rewrite it to `{ let place tmp = $subexpr; expr!(tmp) }`;

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

```rust
let x = 1 + 2 + Some(3).as_ref().unwrap();

// becomes, before this step:
let x = {
    let tmp1 = Some(3);
    1 + <u32 as Add<&u32>>::add(2, Option::unwrap(Option::as_ref(&tmp1)))
};

// becomes, after this step:
let x = {
    let tmp1 = Some(3);
    let tmp2 = &tmp1;
    let tmp3 = Option::as_ref(tmp2);
    let tmp4 = Option::unwrap(tmp3);
    let tmp5 = <u32 as Add<&u32>>::add(2, tmp4);
    1 + tmp5
};
```

An example that will be important for [Bounds Checks](./bound-checks.md):
```rust
expr!($place[$i][$j])

// becomes
{
    let place p = $place;
    let i = $i;
    let place q = p[i]; // order is important because the bound check happens here
    let j = $j;
    let place r = q[j];
    expr!(r)
}
```

At the end of this step, every [value
context](https://nadrieril.github.io/blog/2025/12/06/on-places-and-their-magic.html)
contains either a constant or a variable and every place expression is non-nested.

---

## Discussion

We don't really have to skip constants. I just did it that way because MIR allows constants
a operands. I think it's a practical matter of not ending up with billions of variables.
