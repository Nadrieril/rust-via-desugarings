# Summary

- [Introduction](introduction.md)
- [Desugaring Steps](pipeline/overview.md)
  - [Name Resolution & Macro Expansion](pipeline/name-resolution-macro-expansion.md)
  - [TODO: Control-flow Desugarings](pipeline/control-flow.md)
    - [TODO: Loop Desugaring](pipeline/loop-desugaring.md)
    - [TODO: Try Desugaring](pipeline/try-desugaring.md)
    - [TODO: Lazy Boolean Operators and Let chains](pipeline/let-chains.md)
  - [TODO: Invisible Expression Transformations](pipeline/expr-transforms.md)
    - [TODO: Autoderef](pipeline/autoderef.md)
    - [TODO: Coercions](pipeline/coercions.md)
    - [TODO: Method Resolution & Operator Overload](pipeline/method-resolution.md)
    - [TODO: Match Ergonomics](pipeline/match-ergonomics.md)
      <!-- TODO: indexing -->
      <!-- TODO: desugar aggregates to assignments -->
      <!-- TODO: temporaries needs `ref` pats?? -->
      <!-- TODO: make copies explicit, any other place-to-val is a move -->
      <!-- TODO: explicit binding scopes: forward-declare all bindings? storage_dead them? -->
    - [TODO: Temporaries and Intermediate Subexpressions](pipeline/temporaries.md)
  - [Pattern Desugarings](pipeline/patterns.md)
    - [TODO: Desugaring Patterns to Matches](pipeline/everything-is-match.md)
    - [TODO: Or-patterns](pipeline/or-patterns.md)
    - [TODO: Match Bindings](pipeline/match-bindings.md)
    - [TODO: Match Unnesting](pipeline/match-desugaring.md)
      <!-- TODO: desugar match-continue -->
  - [Closure Desugarings](pipeline/closures.md)
    - [Closure Capture](pipeline/closure-capture.md)
    - [Closure To Struct Desugaring](pipeline/closure-adt.md)
  - [Ownership Desugarings](pipeline/explicit-ownership.md)
    - [Explicit Copies/Moves](pipeline/copy-move.md)
    - [Drop Elaboration](pipeline/drop-elaboration.md)
      <!-- TODO: wait shit, need unwind blocks for that -->
    - [Borrow Checking?](pipeline/borrow-checking.md)
      <!-- [Coroutine Transformation](pipeline/coroutine.md) -->
  - [TODO: The Final Language](pipeline/final-language.md)
- [Extra Language Features](language-features.md)
  - [TODO: Enum Projections](features/enum-projections.md)
  - [TODO: Enum Discriminant Access](features/enum-discriminant.md)
  - [Explicit Hygiene Markers](features/hygiene-markers.md)
  - [TODO: Explicit Return Place](features/return-place.md)
  - [Explicit Copy/Move](features/explicit-copy-move.md)
  - [TODO: `let place`?](features/let-place.md)
  - [TODO: Moving Out Of `&mut`](features/moving-out-of-mut.md)
  - [Move Expressions for Closure Captures](features/move-expressions.md)
  - [TODO: Non-Dropping Assignment](features/non-dropping-assignment.md)
  - [TODO: Phased Initialization](features/phased-initialization.md)
  - [Unique-Immutable Borrow](features/uniq-borrow.md)
