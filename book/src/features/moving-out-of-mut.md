# Moving out of &mut

This proposes a (hopefully) simple modification to the borrow-checker: we should allow moving out of
`&mut T` references, as long as we put a value back before anyone else can see it.

In practice, that means that we must put the value back before any function call since any function
call may unwind (unless compiled with panic=abort I guess).

```rust
fn foo(x: &mut Vec<u32>) {
    let vec: Vec<_> = *x; // move out
    // can't call any functions oops
    *x = vec; // write a value back
    // now we're back to normal
}
```

The purpose of this feature in this document is to [Make dropping
explicit](../pipeline/drop-elaboration.md).
If we wanted it as a general feature, we may need things like `nopanic` functions to make it usable.
