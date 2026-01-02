# Match Bindings

Patterns operate in two steps: we first check if the pattern matches, and then assign each binding
of the pattern. In this step we make the assignments explicit, leaving only patterns that don't
contain bindings.

Every subpattern unambiguously refers to a subplace of the scrutinee place.
Using [Enum Projections](../features/enum-projections.md), we can name that place
and bind it explicitly.

For example:
```rust
match $scrutinee {
   Struct { a: Enum::Variant { ref mut x, y: $pat }, b } => $arm,
   ..
}

// becomes
match $scrutinee {
   Struct { a: Enum::Variant { x: _, y: $pat }, b: _ } => {
      let x = unsafe { &mut $scrutinee.a.Variant.x }; // variant access is unsafe
      let b = $scrutinee.b;
      $arm
   }
   ..
}
```

This covers plain patterns.
Match guards behave a bit differently, because they we only give them shared-ref access to the
bindings.
In order for the bindings to still have the expected type, we do a little hack whereby we replace
every use of the binding in the guard[^1] :
```rust
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

---

## Discussion

The special handling of bindings in guards was a major source of headaches in structuring this book.
I wonder if there's a way to make them less annoying, for example with [`let
place`](https://nadrieril.github.io/blog/2025/12/09/postfix-macros-and-let-place.html).

<!-- Note that this desugaring is slightly different than what rustc does today: in the `ref` and `ref -->
<!-- mut` binding cases, rustc keeps the same initial binding between the guard and the match arm. -->
<!-- This can be observed by comparing addresses. -->
<!-- I believe that behavior is a valid refinement of the specification I propose, --> 

[^1]: This is in fact exactly what rustc does internally. I wish there was a cleaner way to do this.
