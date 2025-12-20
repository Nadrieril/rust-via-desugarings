# Corountine Transformation

`async` blocks are similar to closures: they get desugared to a new ADT that implements an
appropriate trait; this transformation is quite a bit more complex than for closure though.

I called this section "coroutine transform" because the compiler supports a slightly more general
transformation than just `async`, which may get used in the future to get a similar desugaring for
e.g. iterators/generators.

The intuition is as follows:
- Every `async` block becomes a state machine, represented roughly as an enum with one variant per state;
- Every suspension point (`.await`) corresponds to a possible state, and the data stored for that
  state is the value of all locals that are live at that point.
- That state machine then implements `Future`, where `poll` resumes from the current state and
  attempts to make progress.

TODO: example
