# Drop Elaboration

Values are dropped when locals go out of scope, and when a place is written to.

To desugar drops, I introduce the following macro[^1] :
```rust
macro_rules! drop_in_place {
    ($place:expr) => {{
        unsafe {
            std::ptr::drop_in_place((&raw const $place).cast_mut());
            std::mem::forget(move!($place));
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
    drop(move!(x.a));
} else {
    drop(move!(x.b));
}
x.a = "some other string".to_owned();

// becomes:
let need_drop_a = true;
let need_drop_b = true;
if foo() {
    need_drop_a = false;
    drop(move!(x.a));
} else {
    need_drop_b = false;
    drop(move!(x.b));
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

See the [corresponding Reference section](https://doc.rust-lang.org/reference/destructors.html) for
details.

One tricky case is assignment through a mutable reference:
```rust
let a: &mut String = ...;
*a = String::new(); // this drops the previous string

// becomes:
drop_in_place!(*a);
*a = String::new(); // borrowck knows there's no previous string to drop
```

This is not allowed in today's Rust, but would be with the [Moving Out Of
`&mut`](../features/moving-out-of-mut.md) feature.

After this step, all assignments of `!Copy` types are to statically uninitialized places (hence
won't cause implicit drops), and all drops are explicit.

---

## Discussion

One massive limitation of our current approach is that we're missing information about which drops
run when unwinding. In `rustc`, drop elaboration runs on MIR, which makes the unwinding control-flow
explicit. In particular, it can express "if the function call `foo()` panics, then we drop this or
that place". Expressing this in surface Rust looks tricky.

[^1]: The `cast_mut` dance is there because `&raw mut local` would require the local to be declared `let mut`.
