# Rust via Desugarings

This book proposes that the meaning of a piece of Rust code can be explained by successively
desugaring it into into a simpler subset of Rust.
At the end of this process we reach a subset simple enough that it can hopefully be described formally,
for example with [MiniRust](https://github.com/minirust/minirust).

My aim with this book is to help bridge what I perceive to be a "specification gap" in Rust. I'm
a big fan of MiniRust, I think it's a robust answer to "what really happens when I run Rust code".
What remains is to get from source Rust to MiniRust.
The [Rust Reference](https://doc.rust-lang.org/reference/introduction.html) is a wonderful resource
but I don't find that the way it is structured helps me answer the question "I have this piece of
code, how do I understand what it means".
This book is my answer to that question.

In order for each step to produce valid Rust, I'm taking the liberty to add some features to the
language. See the "Extra language features" chapter for details (TODO).

This book focuses on the bodies of functions: statements, expressions, control-flow. It does
not explain e.g. how typechecking works, or anything about traits. This is assumed to be understood
or explained separately.

While I do my best to be precise and correct, this is just a fun project I'm doing with my current
knowledge of Rust. This book will contain mistakes, imprecisions and omissions; please [open an
issue](https://github.com/Nadrieril/rust-via-desugarings/issues) if you notice any!


[^1]: These desugarings are not real! By which I mean, the Rust compiler doesn't necessarily represent
things like I propose; I have to go through hoops so that the output of each step stays valid Rust
code. But it's not that far from how rustc works either.
