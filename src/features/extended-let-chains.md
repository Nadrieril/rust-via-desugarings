# Extended Let Chains

## Let Chain Disjunctions

This feature enables `if let $pat = $expr || let $pat = $expr`.
This is similar to let chains in the kind of syntax allowed.

This can be mixed with normal boolean conditions and `&&`-based let chains.
If any binding is bound in both alternatives, it must have the same type.
Such a binding can then be referred to in the arm body.

```rust
if let Some(y) = foo()
  || (let Some(x) = bar() && let Some(y) = baz(x) && cond()) {
    // `x` is accessible here
    ..
}
```

The drop order of the bindings depends on which branch succeeded.
```rust
if (let Some(a) = foo() && let Some(b) = a.method())
  || (let Some(b) = bar() && let Some(a) = b.method()) {
    ..
}
```

## Let Chain Forward Declarations

This feature also enables `let x;` in the middle of a let chain.
This declares the variable without initializing it.
Its drop order is determined by this declaration site just like a normal `let x = ...`.
This is used to get the right temporary lifetimes and drop order when desugaring.

```rust
if let Some(x) = foo() && let y; && let Some(z) = if bar() { y = Some(thing()); &y } else { &None } {
    ..
}
```
