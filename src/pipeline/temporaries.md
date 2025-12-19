# Desugaring Temporaries and Intermediate Subexpressions

In this step, we store every sub-expression that is a value into a new temporary variable.

This is a rather subtle step, because the rules for how long the temporary lives are complex. For
more details, you may enjoy [this blog post](https://blog.m-ou.se/super-let/); for even more
details, check out the
[Reference](https://doc.rust-lang.org/reference/destructors.html#r-destructors.scope.temporary).

I'm not at all up-to-date on this topic, but to the best of my knowledge, in edition 2024, this is
for example how a temporary inside an `if let` behaves:
```rust
let opt: RwLock<Option<u32>> = ...
if let Some(x) = opt.read().unwrap().as_ref() {
    ...
} else {
    ...
}
// first becomes, because of autoref+autoderef:
if let Some(x) = Option::as_ref(&*opt.read().unwrap()) {
    ...
} else {
    ...
}
// which desugars to:
{
    let guard = opt.read().unwrap()
    if let Some(x) = Option::as_ref(&*guard) {
        ...
    } else {
        drop(tmp); // The temporary is dropped here
        ...
    }
}
// The temporary is also gone here, if it wasn't explicitly dropped.
```

Borrowed subexpressions can even become statics (this is called "constant promotion"):
```rust
let x = &1 + 2;
// desugars to
static TMP: u32 = 1 + 2;
let x = &TMP; // this allows `x` to have type `&'static u32`
```

This step also desugars every nested value expression:
```rust
let x = 1 + 2 + Some(3).as_ref().unwrap();
// becomes, before this step:
let x = <u32 as Add<u32>>::add(1, <u32 as Add<&u32>>::add(2, Option::unwrap(Option::as_ref(&Some(3)))));
// becomes, after this step:
let tmp1 = Some(3);
let tmp2 = &tmp1;
let tmp3 = Option::as_ref(tmp2);
let tmp4 = Option::unwrap(tmp3);
let tmp5 = <u32 as Add<&u32>>::add(2, tmp4);
let x = <u32 as Add<u32>>::add(1, tmp5);
```

The only nested expressions that remain are place expressions:
```rust
let x = &(0, (1, 2)).1.1;
// becomes:
let tmp = (0, (1, 2));
let x = &tmp.1.1; // we can't assign `tmp.1` to a temporary in general
```
