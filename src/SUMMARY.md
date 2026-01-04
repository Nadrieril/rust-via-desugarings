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
    - [Match Ergonomics](pipeline/match-ergonomics.md)
  - [Expression Unnesting](pipeline/expr-unnesting.md)
    - [Temporaries and Lifetime Extension](pipeline/value-to-place.md)
    - [TODO: Intermediate Subexpression Elimination](pipeline/subexpr-elim.md)
    - [Bound Checks](pipeline/bound-checks.md)
    - [Overflow Checks](pipeline/overflow-checks.md)
    - [Functional Record Update](pipeline/fru.md)
      <!-- TODO: somewhere here desugar `$place += $expr` for the builtin case -->
    - [Explicit Copies/Moves](pipeline/copy-move.md)
  - [Pattern Desugarings](pipeline/patterns.md)
    - [Desugaring Pattern Expressions](pipeline/everything-is-match.md)
    - [Or-patterns](pipeline/or-patterns.md)
    - [By-Value Bindings](pipeline/by-value-bindings.md)
    - [Match Guard Mutable Bindings](pipeline/guard-bindings.md)
    - [Desugaring Matches](pipeline/match-desugaring.md)
    - [Pattern Unnesting](pipeline/pattern-unnesting.md)
    - [Let Chains](pipeline/let-chains.md)
    - [Desugaring Bindings](pipeline/desugaring-bindings.md)
  - [Closure Desugarings](pipeline/closures.md)
    - [Closure Capture](pipeline/closure-capture.md)
      <!-- TODO: didn't clarify what happens with multiple references to the same place!
            e.g. `x = x + 1`. let place is an insanely cool solution -->
    - [Closure To Struct Desugaring](pipeline/closure-adt.md)
  - [Desugaring Nested Scopes]()
    - [Removing Tail Expressions]()
    - [Explicit Binding Scopes]()
    - [Explicit Unwind Paths]()
    - [Pre-Declaring All Bindings]()
    - [Scope Flattening]()
      <!-- - [Temporaries and Subexpressions](pipeline/temporaries.md) -->
      <!-- TODO: 
        - replace tail expressions with assignments, includes `break val`
          e.g. `let b = { ..; $expr }` -> `let b; { ..; b = expr; };`
        - add `scope_end`s everywhere (don't forget before assignments)
        - also add conditional_drops, what's the diff??? need to do that before unwind paths for sure
        - add explicit unwind paths with more `scope_end`s
            around calls and scope_end!()
        - forward-declare all bindings at start of function, with type annotations
        - remove all non-control-flow scopes
        - don't forget to update drop elab once we have explicit `scope_end`s
      -->
  - [Ownership Desugarings](pipeline/explicit-ownership.md)
    - [Drop Elaboration](pipeline/drop-elaboration.md)
    - [Phased Initialization](pipeline/phased-initialization.md)
    - [Borrow Checking?](pipeline/borrow-checking.md)
      <!-- [Coroutine Transformation](pipeline/coroutine.md) -->
  - [TODO: The Final Language](pipeline/final-language.md)
- [Extra Language Features](language-features.md)
  - [Cleanup On Unwinding](features/on-unwind.md)
  - [Enum Discriminant Access](features/enum-discriminant.md)
  - [Enum Projections](features/enum-projections.md)
  - [Explicit Copy/Move](features/explicit-copy-move.md)
  - [Explicit End Of Scope](features/scope-end.md)
  - [Explicit Hygiene Markers](features/hygiene-markers.md)
  - [Extended Let Chains](features/extended-let-chains.md)
  - [If Let Guards](features/if-let-guards.md)
  - [Move Expressions for Closure Captures](features/move-expressions.md)
  - [Moving Out Of `&mut`](features/moving-out-of-mut.md)
  - [Phased Initialization](features/phased-initialization.md)
  - [Place Aliases](features/let-place.md)
  - [Unchecked Indexing](features/unchecked-indexing.md)
  - [Unique-Immutable Borrow](features/uniq-borrow.md)
- [Worked Examples](worked-examples.md)
