# Drop Elaboration

Our last desugaring step is to make drops explicit. Drops happen at the end of a scope, and when
a place is written to.

To desugar drops, I introduce the following macro[^1] :
```rust
macro_rules! drop_in_place {
    ($place:expr) => {{
        unsafe {
            std::ptr::drop_in_place(&raw mut $place);
            std::mem::forget($place);
        }
    }};
}
```

We can't just call `drop($place)` because drop actually happens in-place, which is
soundness-critical for pinned types. This is what this macro does: it causes the appropriate drop
code to run (using the compiler-generated `drop_in_place`), and calls `mem::forget` so that the
borrow-checker can tell that the place no longer needs to be dropped. 

Now for any local or part of a local that hasn't been explicitly moved out, we insert calls to
`drop_in_place!`. This can require adding extra booleans ("drop flags") if different branches
haven't moved the same places:
```rust
let x = Struct {
    a: String::new(),
    b: String::new(),
};
if foo() {
    drop(x.a);
} else {
    drop(x.b);
}
x.a = "some other string".to_owned();

// becomes:
let need_drop_a = true;
let need_drop_b = true;
if foo() {
    need_drop_a = false;
    drop(x.a);
} else {
    need_drop_b = false;
    drop(x.b);
}
if need_drop_a {
    drop_in_place!(x.a);
}
x.a = "some other string".to_owned();
if need_drop_b {
    drop_in_place!(x.b);
}
drop_in_place!(x.a);
```

[^1]: Actually `&raw mut` requires a local to be declared `let mut`, so we should do `(&raw const $place).cast_mut()`. I'm hiding that for simplicity.
