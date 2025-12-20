# Desugaring Steps

Here is a birds-eye view of the transformations we'll be doing:

- Resolve names and expand macros so we talk about the final, macro-free program.
- Lower surface control-flow sugar (`for`, `while`, `?`, `if let`/`let else`, etc.) into a handful
  of constructs (`loop`, `match`, `break`/`continue`/`return`).
- Make implicit conversions explicit: autoderef/autoref, coercions, method and operator resolution.
- Materialize temporaries so every intermediate value gets a name and a lifetime.
- Turn higher-level features into more primitive ones: matches without ergonomics, closures into
  structs plus trait impls, async/await into coroutines.
- Finally, make ownership explicit: copies vs moves, and inserts drops that would have happened
  implicitly.

After all that, the program is reduced to much simpler basic blocks that should map unambiguously
MIR/MiniRust. That's the hope, at least.
