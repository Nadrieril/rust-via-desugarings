# Desugaring Steps

Here is a birds-eye view of the transformations we'll be doing:

- Resolve names and expand macros;
- Lower surface control-flow sugar (`for`, `while`, `?`, `if let`/`let else`, etc.) into a handful
  of constructs (`loop`, `if`, `break`/`continue`/`return`);
- Make implicit operations explicit: autoderef/autoref, coercions, method resolution, operator
  overloading, match ergonomics;
- Materialize temporaries so that every intermediate value gets a name and a lifetime.
- Turn closures into plain structs;
- Make drops explicit.

Each step must produce an equivalent program, i.e. the desugared program compiles if and only if the
original one does, and both have the same semantics.
Some of the desugaring steps fails to enforce that; this is noted in their Discussion section.

At the end of all that, we get a program in a very limited and precise subset of Rust.
See [The Final Language](final-language.md) for details and discussion.
