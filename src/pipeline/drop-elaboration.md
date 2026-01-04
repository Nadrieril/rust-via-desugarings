# Drop Elaboration

We know where drops may happen, so we now only need to decide which drops to run at each relevant
program point.

Dropping happens in-place by running the compiler-generated `core::ptr::drop_in_place` function on
the place to be dropped.
That function takes care to recursively call `Drop::drop` for all the subtypes that require it. For
our purposes we introduce the following macro[^1], which calls `core::ptr::drop_in_place` then calls
`mem::forget` so that the borrow-checker can tell that the place has been deinitialized.
```rust
macro_rules! drop_in_place {
    ($place:expr) => {{
        unsafe {
            core::ptr::drop_in_place((&raw const $place).cast_mut());
            core::mem::forget(move!($place));
        }
    }};
}
```

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

(I'm hiding unwind paths in these examples for legibility.)

After this step, all the code involved in dropping values is explicit.

[^1]: The `cast_mut` dance is there because `&raw mut local` would require the local to be declared `let mut`.
