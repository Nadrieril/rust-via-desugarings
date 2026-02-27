# Explicit Hygiene Markers

Macro hygiene is not only there to avoid identifier name clashes; it affects language semantics in
at least one other way: editions. When the behavior of a program depends on the edition, the edition
to use is taken from an appropriate token; for example for the [if let
rescope](https://doc.rust-lang.org/edition-guide/rust-2024/temporary-if-let-scope.html) changes
I believe the relevant edition is that of the `if` token. 

When a crate in edition `N` calls a macro from a crate in edition `M`, each token recalls which
edition it comes from. So the following code may or may not panic depending on the edition of the
crate that defines the `if_let` macro.
```rust
use core::cell::Cell;

// Possibly defined in another crate.
macro_rules! if_let {
    (($pat:pat = $expr:expr) { $($then:tt)* } else  { $($else:tt)* }) => {
        if let $pat = $expr { $($then)* } else  { $($else)* }
    }
}

fn main() {
    let x = Cell::new(0);
    x.set(42);
    if_let!((None = Some(&ZeroOnDrop(&x))) {
        assert_eq!(x.get(), 42);
        // In edition 2024, value is dropped here
    } else {
        assert_eq!(x.get(), 0); // Fails in edition 2021
    });
    // In edition 2021, value is dropped here
    assert_eq!(x.get(), 0);
}

/// Sets the value to `0` on drop.
struct ZeroOnDrop<'b>(&'b Cell<u32>);
impl<'b> Drop for ZeroOnDrop<'b> {
    fn drop(&mut self) {
        self.0.set(0);
    }
}
```

Naively expanding macros could therefore change program semantics.
To be able to expand the macro in a semantics-preserving way, this feature adds a `#[edition
= "..."]` attribute that changes the edition of the following token.
So the macro above would expand to `#[edition = "2021"] if let ...` or `#[edition = "2024"] if let
...` depending on the edition of the crate that declares it.

---

## Discussion

Prior art: [this MCP](https://github.com/rust-lang/compiler-team/issues/692).

I do not know if there are other hygiene considerations that can affect program semantics. If so
we'll need to add similar attributes.

This attribute proposal is actually incomplete: attributes are not allowed between any two tokens.
Maybe the edition-relevant tokens can all be preceded by an attribute? Otherwise we'll need to
invent more syntax like `x#edition#2024` or such horrors.
