# Rust via Desugarings

This book proposes to explain the meaning of a piece of Rust code by successively desugaring it into
into a simpler subset of Rust.
At the end of this process we reach a subset simple enough that it can hopefully be described formally,
for example with something like [MiniRust](https://github.com/minirust/minirust).

These desugarings are not real! By which I mean, the Rust compiler doesn't necessarily represent
things like I propose; I have to go through hoops so that the output of each step stays valid Rust
code. But it's not that far from how rustc works either.

This book focuses mostly on the bodies of functions: statements, expressions, control-flow. It does
not explain e.g. how typechecking works, or anything about traits. This is assumed to be understood
or explained separately.

While I do my best to be precise and correct, this is just a fun project I'm doing with my current
knowledge of Rust. This book will contain mistakes, confusions and omissions; please [open an
issue](https://github.com/Nadrieril/rust-via-desugarings/issues) if you notice any!
