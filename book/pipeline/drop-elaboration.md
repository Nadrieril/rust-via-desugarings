# Drop Elaboration

We know where drops may happen, so we now only need to decide which drops to run at each relevant
program point.

Dropping happens in-place by running the compiler-generated `core::ptr::drop_in_place` function on
the place to be dropped.
Instead of calling that function directly we use the
[`drop_in_place!()`](../features/in-place-drop.md) built-in macro so that the borrow-checker can
tell that the place has been deinitialized.

In this step we'll replace every `ensure_dropped!($place)` with a series of
appropriate calls to `drop_in_place!($subplace)`.

For any subplace of `$place` that hasn't been explicitly moved out, we insert a call to
`drop_in_place!`.
This can require adding extra booleans ("drop flags") if different branches haven't moved the same
places:
```rust
let x = Struct {
    a: String::new(),
    b: String::new(),
};
if foo() {
    drop(move!(x.a));
} else {
    drop(move!(x.b));
}
ensure_dropped!(x.a);
x.a = "some other string".to_owned();
ensure_dropped!(x);
scope_end!(x);

// becomes:
let a_is_initialized = true;
let b_is_initialized = true;
if foo() {
    drop(move!(x.a));
    a_is_initialized = false;
} else {
    drop(move!(x.b));
    b_is_initialized = false;
}
if a_is_initialized {
    drop_in_place!(x.a);
}
x.a = "some other string".to_owned();
if b_is_initialized {
    drop_in_place!(x.b);
}
drop_in_place!(x.a);
scope_end!(x);
```

(unwind paths omitted for legibility)

After this step, all the code involved in dropping values is explicit.
