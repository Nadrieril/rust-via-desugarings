# Coercions

Type coercions are implicit operations that change the type of a value.
They happen automatically at certain locations when the expected type
doesn't match the actual type of an expression.

The locations where locations can happen are called "coercion sites" and listed in [this Reference
section](https://doc.rust-lang.org/reference/type-coercions.html?#r-coerce.site).
The allowed coercions are then listed
[here](https://doc.rust-lang.org/reference/type-coercions.html?#r-coerce.types).

In this step, we desugar these coercions into explicit conversions.
The outcome is either an `as`-cast `$expr as $ty` or a reborrow making use of
`Deref`/`DerefMut`.

For example:
```rust
fn foo(s: &str) { .. }
let x: String = ...;
let string_ref: &String = &x;
foo(string_ref);

// becomes:
foo(&**string_ref);
```

```rust
let x = 42u32;
let dyn_x: &dyn Debug = &x;
let meta = core::ptr::metadata(dyn_x);

// becomes:
let dyn_x: &dyn Debug = &x as &dyn Debug;
let meta = core::ptr::metadata(dyn_x as *const dyn Debug);
```

After this step, expressions have the type expected of them.
