# Summary

- [Introduction](introduction.md)
- [Desugaring Steps](pipeline/overview.md)
  - [Name Resolution & Macro Expansion](pipeline/name-resolution-macro-expansion.md)
  - [Control-flow Desugarings](pipeline/control-flow.md)
    - [Loop Desugaring](pipeline/loop-desugaring.md)
    - [Try Desugaring](pipeline/try-desugaring.md)
    - [Lazy Boolean Operators](pipeline/boolean-operators.md)
  - [Type-Directed Expression Transformations](pipeline/expr-transforms.md)
    - [Method Resolution & Operator Overload](pipeline/method-resolution.md)
    - [TODO: Autoderef]()
      <!-- - [TODO: Autoderef](pipeline/autoderef.md) -->
    - [Coercions](pipeline/coercions.md)
    - [TODO: Match Ergonomics](pipeline/match-ergonomics.md)
      <!-- TODO: indexing -->
      <!-- - [TODO: Overflow and Bounds Checks]() -->
  - [Temporaries and Subexpressions](pipeline/temporaries.md)
    - [Temporaries and Lifetime Extension](pipeline/value-to-place.md)
    - [Intermediate Subexpression Elimination]()
      <!-- TODO: remove tail expressions; explicit return -->
      <!-- - [Explicit Binding Scopes]() -->
      <!-- TODO: explicit unwind paths around calls and scope_end!(). when adding an unwind path,
      <!-- TODO: block unnesting, e.g. `if { ...; cond } {}`. requires explicit scopes, requires
      unwind paths -->
      <!-- TODO: add types to bindings -->
      <!-- TODO: replace `break val` and `return val` -->
      <!-- TODO: explicit binding scopes: forward-declare all bindings? storage_dead them? -->
      <!-- TODO: how about `scope_end!($place)` before drop elab? don't forget to update drop elab
      once we have explicit `scope_end`s -->
      need to add a lot of scope_end!()s -->
      <!-- TODO: where do I turn `let ref x = $expr` into `let x = &$expr`? -->
  - [Pattern Desugarings](pipeline/patterns.md)
    - [Desugaring Pattern Expressions](pipeline/everything-is-match.md)
    - [Or-patterns](pipeline/or-patterns.md)
    - [By-Value Bindings](pipeline/by-value-bindings.md)
      <!-- TODO: guard mutability restrictions with fake reads -->
    - [Match Guard Mutable Bindings](pipeline/guard-bindings.md)
    - [Desugaring Matches](pipeline/match-desugaring.md)
    - [Pattern Unnesting](pipeline/pattern-unnesting.md)
    - [Let Chains](pipeline/let-chains.md)
      <!-- TODO: handle plain bindings somewhere -->
    - [Desugaring Bindings](pipeline/desugaring-bindings.md)
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
    <!-- TODO: on_unwind $expr { block }  -->
  - [Extended Let Chains](features/extended-let-chains.md)
  - [If Let Guards](features/if-let-guards.md)
  - [Move Expressions for Closure Captures](features/move-expressions.md)
  - [Moving Out Of `&mut`](features/moving-out-of-mut.md)
  - [Phased Initialization](features/phased-initialization.md)
  - [Place Aliases](features/let-place.md)
  - [Unique-Immutable Borrow](features/uniq-borrow.md)
