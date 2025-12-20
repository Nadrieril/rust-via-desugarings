# Rust via Desugarings

This book proposes to explain the meaning of a piece of Rust code by successively desugaring it into
into a simpler subset of Rust.
At the end of this process we reach a subset simple enough that it can hopefully be described formally,
for example with something like [MiniRust](https://github.com/minirust/minirust).

This desugarings are not real! By which I mean, the Rust compiler doesn't do all of them like this,
or in this order. Rather this is my attempt at representing what rustc computes into valid Rust
code.

This book focuses mostly on the bodies of functions: statements, expressions, control-flow. It does
not explain how typechecking works or anything about traits. In fact it actually relies on
typechecking for a lot of these desugarings.

While I do my best to be precise and correct, this is just a fun project I'm doing with my current
knowledge of Rust. This book will contain mistakes, confusions and omissions; please [open an
issue](https://github.com/Nadrieril/rust-via-desugarings/issues) if you notice any!
