# Unique-Immutable Borrow

`&uniq T` is a new reference type that like `&mut T` is guaranteed to be the only reference pointing
to a given place. Unlike `&mut T` however, it doesn't allow mutating the pointed value.

The point of this borrow is that a `&uniq &mut T` is allowed to mutate the underlying `T` but not
the `&mut T` itself. This is used to [desugar closure captures](../pipeline/closure-capture.md).

---

## Discussion

This borrow actually [exists in the
compiler](https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/mir/enum.MutBorrowKind.html#variant.ClosureCapture)
for exactly the same reason we need it: for capturing `&mut` references.
It is just not exposed to users.

I'm not sure if this is useful for anything else. I recall from [discussions on true
reborrowing](https://haibane-tenshi.github.io/rust-reborrowing/) that possibly a function like `fn
reborrow<'a, 'b>(x: &'a uniq &'b mut T) -> &'a mut T` is more true to what reborrowing does than the
same function with a `&mut &mut T` argument, but I haven't thought this through in detail.
