# Temporaries and Lifetime Extension

A "value-to-place coercion" occurs when a value expression is used in a context where a place is
needed, e.g. because it is borrowed, matched on, or has a field accessed.
See [this blog post](https://nadrieril.github.io/blog/2025/12/06/on-places-and-their-magic.html)
for more details about place/value expressions and place/value contexts.

Whenever that happens, the value will get stored in a temporary variable. In this step, we make
these temporaries explicit.

The rules that determine the scope of these temporaries are complex; they're described in [the
Reference](https://doc.rust-lang.org/reference/destructors.html#temporary-scopes).
You may also enjoy [this blog post](https://blog.m-ou.se/super-let/) with a more explanatory style.

In this step, for each expression `$expr` to be coerced, we first add a `let tmp;` statement,
then assign it `tmp = $expr;` (these two steps can be merged), then use `tmp` where the expression was.
The placement of the `let x;` determines how long the value will live since it affects drop order.
To get the right scope, extra blocks `{ .. }` may be added.

For example:
```rust
let s = if Option::is_some(&Option::clone(&opt)) {
    let _x = &42;
    &String::new()
} else {
    &String::new()
};

// becomes:
let tmp3;
let tmp4;
let s = if { let tmp1 = Option::clone(&opt); Option::is_some(&tmp1) } {
    let tmp2 = 42;
    let _x = &tmp2;
    tmp3 = String::new();
    &tmp3
} else {
    tmp4 = String::new();
    &tmp4
};
```
Or:
```rust
let opt: RwLock<Option<u32>> = ...
if let Some(x) = Option::as_ref(&*Result::unwrap(RwLock::read(&opt))) {
    ...
} else {
    ...
}

// becomes (in edition 2024):
if let tmp = Result::unwrap(RwLock::read(&opt)) && let Some(x) = Option::as_ref(&*tmp) {
    ...
} else {
    ...
}
```

Note how in let chains we may introduce the temporaries as part of the let chain to get the
right scope. Our [Extended Let Chains](../features/extended-let-chains.md) allow forward declarations
`let x;` in the middle of a let chain for that purpose.

Taking an example from the [edition
book](https://doc.rust-lang.org/edition-guide/rust-2024/temporary-tail-expr-scope.html):

```rust
fn f() -> usize {
    let c = RefCell::new("..");
    c.borrow().len()
}

// Becomes, after method resolution:
fn f() -> usize {
    let c = RefCell::new("..");
    str::len(*<Ref<_> as Deref>::deref(&RefCell::borrow(&c)))
}

// Before 2024, this becomes:
fn f() -> usize {
    let tmp1; // Added at the start of scope so that it drops after the other locals.
    let tmp2;
    let c = RefCell::new("..");
    tmp1 = RefCell::borrow(&c); // error[E0597]: `c` does not live long enough
    tmp2 = <Ref<_> as Deref>::deref(&tmp1);
    str::len(*tmp2)
}

// After 2024, this becomes:
fn f() -> usize {
    let c = RefCell::new("..");
    let tmp1; // drops before other locals
    let tmp2;
    tmp1 = RefCell::borrow(&c);
    tmp2 = <Ref<_> as Deref>::deref(&tmp1);
    str::len(*tmp2)
}
```

There is an exception to the above: temporaries can, [when
sensible](https://doc.rust-lang.org/reference/destructors.html#r-destructors.scope.const-promotion),
become statics instead of local variables. This is called "constant promotion":
```rust
let x = &1 + 2;

// becomes:
static TMP: u32 = 1 + 2;
let x = &TMP; // this allows `x` to have type `&'static u32`
```

After this step, all place contexts contain place expressions.



<!-- This step also desugars every nested value expression: -->
<!-- ```rust -->
<!-- let x = 1 + 2 + Some(3).as_ref().unwrap(); -->
<!-- // becomes, before this step: -->
<!-- let x = <u32 as Add<u32>>::add(1, <u32 as Add<&u32>>::add(2, Option::unwrap(Option::as_ref(&Some(3))))); -->
<!-- // becomes, after this step: -->
<!-- let tmp1 = Some(3); -->
<!-- let tmp2 = &tmp1; -->
<!-- let tmp3 = Option::as_ref(tmp2); -->
<!-- let tmp4 = Option::unwrap(tmp3); -->
<!-- let tmp5 = <u32 as Add<&u32>>::add(2, tmp4); -->
<!-- let x = <u32 as Add<u32>>::add(1, tmp5); -->
<!-- ``` -->

<!-- The only nested expressions that remain are place expressions: -->
<!-- ```rust -->
<!-- let x = &(0, (1, 2)).1.1; -->
<!-- // becomes: -->
<!-- let tmp = (0, (1, 2)); -->
<!-- let x = &tmp.1.1; // we can't assign `tmp.1` to a temporary in general -->
<!-- ``` -->
