# Rust via Desugarings

This book proposes to explain the meaning of a piece of Rust code by successively desugaring it into
into a simpler subset of Rust.
At the end of this process we reach a subset simple enough that it can hopefully be described
formally.

I'm writing this book in the context of two projects that are working towards formalizing Rust:
- [MiniRust](https://github.com/minirust/minirust) aims to specify the operational semantics of
  Rust, i.e. what it means to execute Rust, including a precise delineation of what is and isn't UB;
- [a-mir-formality](https://github.com/rust-lang/a-mir-formality) aims to describe, among other
  things, the trait and type system of Rust (including borrow-checking).

Both of these share the limitation of working only with function bodies in a very simplified+precise
form ([MIR](https://rustc-dev-guide.rust-lang.org/mir/index.html)). I'm writing this book as
a complement to those two, filling the gap of how to get from real Rust code to this
simplified+precise form[^1].

## Goals

I have three goals in writing this, in order of importance:

1. Explanatory: I would like Rust users to reach for this book when they don't understand a piece of Rust
   code or an error message. I care more to show where and what kind of transformations happen
   rather than exactly how they happen.

2. Specificatory: I want this book to be sufficiently complete (if not precise) to be the basis of
   a specification for Rust. There should be no unknown unknowns when reading this, and when details
   are skimmed over there should ideally be a link to a source of truth about what really happens.

3. Implementablory: I want this to be close enough to the reality of the compiler that we could make
   rustc output the outcome of some of these desugaring steps. As we've learned many times, the best
   teaching tool is one you can interact with.

## Non-Goals

This book focuses on the meaning of function bodies: statements, expressions, control-flow. It does
not explain e.g. how typechecking works, anything about traits, type layouts, constant evaluation,
linking crates, etc. In fact it heavily relies on types and trait data being known-facts we can make
use of.

This book does not aim to really be a specification document: reaching the level of precision that
the Rust Reference has would take an amount of work I am not prepared for. I do think this way of
presenting things has value, and in writing this am hoping to inform how the Reference may end up
filling its existing gaps.

## Caveats

In order for each step to produce valid and understandable Rust code, I took the liberty to assume
the existence of a few choice language features that don't exist in real Rust.
See the ["Extra language features"](language-features.md) chapter for details.

While I do my best to be precise and correct, this is just a fun project I'm doing with my current
knowledge of Rust. This book will contain mistakes, imprecisions and omissions; please [open an
issue](https://github.com/Nadrieril/rust-via-desugarings/issues) or PR if you notice any!

[^1]: The majority of the info in this book is present in one way or another in the [Rust
Reference](https://doc.rust-lang.org/reference/introduction.html). The point of this book is in part
to know where to start and which Reference sections are relevant to a given piece of Rust code. Also
the Reference isn't executable even in theory, unlike this book.
