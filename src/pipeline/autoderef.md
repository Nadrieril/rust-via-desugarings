# Autoderef

TODO: every place context has an associated mutability. for patterns, comes from bindings. for `let
place`, comes from inference.

Autoderef is what allows using `&T`/`&mut T`/`SmartPtr<T>` values as it they were of type `T`. In
this phase we only concern ourselves with a limited set of autoderef cases: those that come from
explicit place operations.

The place operations in question are deref (`*x`) and field access (`x.field`).

- Field access:

    Given the expression `x.field`, TODO

Autoderef turns `&T` into `T` by inserting repeated `*` operations. In surface Rust, you get
autoderef on field access and on explicit `*x` (the latter is only a type-directed sugar that works
the same way as method resolution). For example:
```rust
struct Wrapper(Box<String>);
impl Wrapper { fn first(&self) -> &str { &self.0[..1] } }

let w = Wrapper(Box::new("hi".to_owned()));
let c = w.first().chars().next().unwrap();
let len = (*w.0).len();
// becomes
let c = Wrapper::first(&w).chars().next().unwrap();
let len = (*(*w).0).len();
```

This keeps deref coercions separate from coercions proper: here we only insert actual deref ops that
will panic if the `Deref` impl does. Method-call autoderef lives in the method resolution step
because it interacts with autoref and trait lookup.
