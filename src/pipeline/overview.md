# Desugaring Steps

Here is a birds-eye view of the transformations we'll be doing:

- Resolve names and expand macros so we talk about the final, macro-free program;
- Lower surface control-flow sugar (`for`, `while`, `?`, `if let`/`let else`, etc.) into a handful
  of constructs (`loop`, `if`, `break`/`continue`/`return`);
- Make implicit conversions explicit: autoderef/autoref, coercions, method resolution, operator
  overloading, match ergonomics;
- Materialize temporaries so every intermediate value gets a name and a lifetime.
- Turn closures into plain structs;
- Finally, make ownership explicit: copies vs moves, and inserts drops that would have happened
  implicitly.

Each step must produce an equivalent program, i.e. the desugared program compiles if and only if the
original one does, and both have the same semantics.

At the end of all that, the resulting program should be much closer to MIR/MiniRust. See [the
corresponding chapter](final-language.md) for discussion.
