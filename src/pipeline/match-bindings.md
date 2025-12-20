# Match Bindings

Patterns operate in two steps: we first check if the pattern matches, and then assign each binding
of the pattern. In this step we make the assignments explicit, leaving patterns that don't contain
bindings.

To make this work, we need the capability to refer to places inside enum variants. I propose the
syntax `$place.$variant_name`, e.g. `place.Some`:

```rust
let res: Result<&u32, Error> = ...;
match res {
   Ok(&x) => ..,
   Err(ref e) => ..,
}
// desugars to:
match res {
   Ok(&_) => {
      let x = unsafe { *res.Ok.0 };
      ..
   }
   Err(_) => {
      let e = unsafe { &res.Err.0 };
      ..
   }
}
```

That operation is unsafe because it is only valid if the enum is indeed in the right variant.

This is straightforward enough, except for match guards: for match guards we do a little hack so
that they only get shared-ref access to the binding, yet get a binding of the expected type:
```rust
let opt: Option<T> = ...;
match opt {
   Some(x) if { foo!(x) } => ..,
   Some(ref x) if { bar!(x) } => ..,
   Some(ref mut x) if { baz!(x) } => ..,
   _ => ..,
}
// desugars to:
match opt {
   // Note the extra deref in `foo!(*x)`
   Some(_) if { let x = &opt.Some.0; foo!(*x) } => {
      let x = opt.Some.0;
      ..
   }
   // Nothing surprising here.
   Some(_) if { let x = &opt.Some.0; bar!(x) } => {
      let x = &opt.Some.0;
      ..
   }
   // Note the `*&&mut $place` to get a place of type `&mut T` but with shared access.
   Some(_) if { let x = &mut opt.Some.0; let x = &x; baz!(*x) } => {
      let x = &mut opt.Some.0;
      ..
   }
   _ => ..,
}
```

After this step, all patterns are free of bindings.
