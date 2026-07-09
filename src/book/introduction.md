# Rust via Desugarings

This book proposes to explain the meaning of a piece of Rust code by successively desugaring it into
into a simpler subset of Rust.
At the end of this process we reach a subset simple enough that it can hopefully be described
formally.

I'm writing this book in the context of three projects that are working towards formalizing Rust:
- [MiniRust](https://github.com/minirust/minirust) aims to specify the operational semantics of
  Rust, i.e. what it means to execute Rust, including a precise delineation of what is and isn't
  [Undefined Behavior](https://rust-lang.github.io/unsafe-code-guidelines/glossary.html#undefined-behavior);
- [a-mir-formality](https://github.com/rust-lang/a-mir-formality) aims to describe, among other
  things, the trait and type system of Rust (including borrow-checking);
- the
  [dictionary-passing-style](https://rust-lang.github.io/rust-project-goals/2026/dictionary-passing-style-experiment.html)
  experiment proposes a model for thinking about what trait solving does and how to reason about its
  soundness.

The first two share the limitation of working only with function bodies in a very simplified+precise
form (roughly, [MIR](https://rustc-dev-guide.rust-lang.org/mir/index.html)).
The third speaks only of traits and types.
To get a complete story, we need to tie all of these together and relate them
with source-level Rust code.
New Rust features are commonly described as desugarings into existing features;
in this book I take this idea to its extreme, by describing the whole language
using desugarings[^1].

## Goals

I have three goals in writing this, in order of importance:

1. Explanatory: I would like this book to be a reference to reach for when one doesn't understand
   a piece of Rust code or an error message. I care more to show where and what kind of
   transformations happen rather than exactly how they happen.

2. Specificatory: I want this book to be sufficiently complete (if not precise) that we could
   imagine building a specification for Rust on top of it. There should be no unknown unknowns when
   reading this. When details are skimmed over there should be a link to a source of truth about
   what really happens.

3. Implementablory: I want this to be close enough to the reality of the compiler that we could make
   a tool that outputs the outcome of some of these desugaring steps. As we've learned
   many times, the best teaching tool is one you can interact with.

## Non-Goals

This book focuses on the meaning of function bodies: statements, expressions, control-flow. It does
not explain e.g. how typechecking works, how to solve traits, type layouts, constant evaluation,
linking crates, etc. In fact it heavily relies on types and trait data being known-facts we can make
use of.

This book does not aim to be an actual specification document[^2] :
duplicating the contents of the Reference would be a waste of effort;
I see this book more as a guide for how to read the Reference,
and a vision for how a formal & legible & executable spec could be structured.

## Caveats

In order for each step to produce valid and understandable Rust code, I took the liberty to assume
the existence of a whole bunch of language features that don't exist in real Rust.
See the [Extra Language Features](language-features.md) chapter for details.

This also mostly doesn't include `async`, in large parts because I'm not very familiar with the
details of how it's implemented.

While I do my best to be precise and correct, this is just a fun project I'm doing with my current
knowledge of Rust. This book will contain mistakes, imprecisions and omissions; feel free to [open an
issue](https://github.com/Nadrieril/rust-via-desugarings/issues) or PR if you notice any!

## Executable Experiment

I am currently experimenting with making this book executable, by making it a literate Rust program
that can parse, desugar and execute real Rust code.
Sections that are part of this experiment are indicated by a ▶️ symbol.

## LLM Disclaimer

The tooling used in this repo (`./tools`) is heavily LLM-assisted. While this book is an experiment,
I won't deeply inspect that code as long as it looks reasonable.

The contents of the book itself are carefully handcrafted (except when indicated otherwise).
LLMs can be used for routine tasks but shall not be trusted[^3].
This book aims for utmost precision; even an "obvious" tweak can turn out to be load-bearingly
incorrect; hence careful human scrutiny is demanded.

[^1]: The majority of the info in this book is present in one way or another in the [Rust
Reference](https://doc.rust-lang.org/reference/introduction.html). The point of this book is in part
to know where to start and which Reference sections are relevant to a given piece of Rust code. Also
the Reference isn't executable even in theory, unlike this book.
[^2]: I do wonder if it would make sense for the Reference to be structured in that way.
[^3]: Especially not for changes that move code around, as it's easy to overlook subtle changes there.
