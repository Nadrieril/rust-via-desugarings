# Name Resolution & Macro Expansion

Name Resolution is the process by which Rust figures out what every identifier (local variable,
function call, module, import, etc) refers to. Macro expansion is the process of running a macro and
replacing the macro call with its output.

The two are intertwined: a macro may emit new macros, which will affect the
state of name resolution for macros. For instance:

```rust
macro_rules! define_bump {
    ($name:ident) => {
        macro_rules! $name { ($x:expr) => { $x + 1 } }
    };
}

// Name resolution figures out that this points to the macro above.
define_bump!(bump);

fn main() {
    // We can't resolve this name until after expanding the `define_bump!` call above.
    let _ = bump!(3);
}
```

In fact the whole process is stateful: we must expand items in declaration order and I think
even the order in which we explore modules can have consequences.

To represent the output of name resolution and hygiene, as part of our desugaring we expand all
names to full paths, for any name where there could be ambiguity we rename identifiers to make them
unique, and we insert [Explicit Hygiene Markers](../features/hygiene-markers.md) as appropriate.

For example:
```rust
mod foo {
    fn bar(x: u32) {}
}
use foo::*;
fn main() {
    let x = 4;
    let x = x + 1;
    bar(x);
}

// becomes:
mod foo {
    fn bar() {}
}
fn main() {
    let x1 = 4;
    let x2 = x1 + 1;
    crate::foo::bar(x2);
}
```

This also deals with macro hygiene:

```rust
fn foo() -> u32 {
    let x = 1;
    macro_rules! check {
        () => { x == 1 }; // Uses `x` from the definition site.
    }
    let x = 2;
    if check!() {
        x
    } else {
        0
    }
}

// becomes:
fn foo() -> u32 {
    let x1 = 1;
    let x2 = 2;
    if x1 == 1 {
        x2
    } else {
        0
    }
}
```

See the Reference for details on [macro
expansion](https://doc.rust-lang.org/reference/macros-by-example.html) and [name
resolution](https://doc.rust-lang.org/reference/names.html).

At the end of this step, there are no macros left, no `use` statements, every item is referred to by
its full path, and all variables have unique names.
