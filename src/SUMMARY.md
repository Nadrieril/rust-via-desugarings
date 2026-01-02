# Summary

- [Introduction](introduction.md)
- [Desugaring Steps](pipeline/overview.md)
  - [Name Resolution & Macro Expansion](pipeline/name-resolution-macro-expansion.md)
  - [Control-flow Desugarings](pipeline/control-flow.md)
    - [Loop Desugaring](pipeline/loop-desugaring.md)
    - [Try Desugaring](pipeline/try-desugaring.md)
    - [TODO: Lazy Boolean Operators and Let chains](pipeline/let-chains.md)
  - [Type-Directed Expression Transformations](pipeline/expr-transforms.md)
    - [TODO: Autoderef](pipeline/autoderef.md)
    - [Method Resolution & Operator Overload](pipeline/method-resolution.md)
    - [Coercions](pipeline/coercions.md)
    - [TODO: Match Ergonomics](pipeline/match-ergonomics.md)
      <!-- TODO: indexing -->
    - [TODO: Overflow and Bounds Checks]()
  - [Temporaries and Subexpressions](pipeline/temporaries.md)
    - [Temporaries and Lifetime Extension](pipeline/value-to-place.md)
    - [Intermediate Subexpression Elimination]()
      <!-- TODO: remove tail expressions; explicit return -->
      <!-- TODO: block unnesting, e.g. `if { ...; cond } {}` -->
    - [Explicit Binding Scopes]()
      <!-- TODO: add types to bindings -->
      <!-- TODO: replace `break val` and `return val` -->
      <!-- TODO: explicit binding scopes: forward-declare all bindings? storage_dead them? -->
      <!-- TODO: how about `scope_end!($place)` before drop elab? -->
      <!-- TODO: explicit unwind paths around calls and scope_end!(). when adding an unwind path,
      need to add a lot of scope_end!()s -->
  - [Pattern Desugarings](pipeline/patterns.md)
    - [Desugaring Patterns to Matches](pipeline/everything-is-match.md)
    - [Or-patterns](pipeline/or-patterns.md)
    - [Match Bindings](pipeline/match-bindings.md)
    - [Match Lowering](pipeline/match-desugaring.md)
  - [Closure Desugarings](pipeline/closures.md)
    - [Closure Capture](pipeline/closure-capture.md)
    - [Closure To Struct Desugaring](pipeline/closure-adt.md)
  - [Ownership Desugarings](pipeline/explicit-ownership.md)
    - [Explicit Copies/Moves](pipeline/copy-move.md)
    - [Drop Elaboration](pipeline/drop-elaboration.md)
      <!-- TODO: replace aggregates with phased init -->
    - [Borrow Checking?](pipeline/borrow-checking.md)
      <!-- [Coroutine Transformation](pipeline/coroutine.md) -->
  - [TODO: The Final Language](pipeline/final-language.md)
- [Extra Language Features](language-features.md)
  - [Enum Discriminant Access](features/enum-discriminant.md)
  - [Enum Projections](features/enum-projections.md)
  - [Explicit Copy/Move](features/explicit-copy-move.md)
  - [Explicit End Of Scope](features/scope-end.md)
  - [Explicit Hygiene Markers](features/hygiene-markers.md)
    <!-- - [TODO: `let place`?](features/let-place.md) -->
    <!-- TODO: on_unwind! ( $expr, { block } ) -->
  - [Move Expressions for Closure Captures](features/move-expressions.md)
  - [Moving Out Of `&mut`](features/moving-out-of-mut.md)
  - [Phased Initialization](features/phased-initialization.md)
  - [Unique-Immutable Borrow](features/uniq-borrow.md)
