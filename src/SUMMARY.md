# Summary

- [Introduction](introduction.md)
- [Desugaring Steps](pipeline/overview.md)
  - [Name Resolution & Macro Expansion](pipeline/name-resolution-macro-expansion.md)
  - [Control-flow Desugarings](pipeline/control-flow.md)
    - [Loop Desugaring](pipeline/loop-desugaring.md)
    - [Try Desugaring](pipeline/try-desugaring.md)
    - [Lazy Boolean Operators](pipeline/boolean-operators.md)
  - [Type-Directed Expression Transformations](pipeline/expr-transforms.md)
      <!-- important: we need the `use Trait;` statements for method res! -->
    - [Method Resolution & Operator Overload](pipeline/method-resolution.md)
      <!-- TODO: two-phase borrows -->
    - [Autoderef](pipeline/autoderef.md)
    - [Coercions](pipeline/coercions.md)
    - [`Deref`/`DerefMut` Desugarings](pipeline/smart-ptr-deref.md)
      <!-- TODO: Index/IndexMut works the same as this -->
    - [Match Ergonomics](pipeline/match-ergonomics.md)
  - [Expression Unnesting](pipeline/expr-unnesting.md)
    - [Temporaries and Lifetime Extension](pipeline/value-to-place.md)
    - [Intermediate Subexpression Elimination](pipeline/subexpr-elim.md)
    - [Bound Checks](pipeline/bound-checks.md)
    - [Overflow Checks](pipeline/overflow-checks.md)
    - [Functional Record Update](pipeline/fru.md)
      <!-- TODO: somewhere here desugar `$place += $expr` for the built-in case -->
    - [Explicit Copies/Moves](pipeline/copy-move.md)
  - [Pattern Desugarings](pipeline/patterns.md)
    - [Desugaring Pattern Expressions](pipeline/unify-pattern-exprs.md)
    - [Or-patterns](pipeline/or-patterns.md)
    - [By-Value Bindings](pipeline/by-value-bindings.md)
    - [Match Guard Mutable Bindings](pipeline/guard-bindings.md)
    - [Desugaring Matches](pipeline/match-desugaring.md)
    - [Pattern Unnesting](pipeline/pattern-unnesting.md)
    - [Let Chains](pipeline/let-chains.md)
    - [Desugaring Bindings](pipeline/desugaring-bindings.md)
      <!-- explicit types on all bindings -->
      <!-- explicit types on generic calls -->
  - [Closure Desugarings](pipeline/closures.md)
    - [Closure Capture](pipeline/closure-capture.md)
    - [Closure To Struct Desugaring](pipeline/closure-adt.md)
  - [Desugaring Nested Scopes](pipeline/desugar-scopes.md)
    - [Removing Tail Expressions](pipeline/remove-tail-exprs.md)
    - [Explicit Binding Scopes](pipeline/scope-end.md)
    - [Explicit Drop Locations](pipeline/explicit-drop.md)
    - [Explicit Unwind Cleanup](pipeline/explicit-unwind.md)
    - [Scope Flattening](pipeline/scope-flattening.md)
  - [Final Desugarings](pipeline/final-desugarings.md)
    - [Phased Initialization](pipeline/phased-initialization.md)
    - [Drop Elaboration](pipeline/drop-elaboration.md)
    - [Borrow Checking?](pipeline/borrow-checking.md)
      <!-- [Coroutine Transformation](pipeline/coroutine.md) -->
      <!-- Extra MIR steps suggested by dianne: -->
      <!-- - box deref elaboration -->
      <!--   -> would need `mark_uninitialized!(*b); let x = unsafe { ptr::read(b as ...) }` type things -->
      <!--   -> could use that for drop_in_place actually -->
      <!-- - moves for packed drops -->
      <!-- - derefer -->
      <!--   -> could keep `let place p = *$place` in the final language. would help with FP. -->
  - [The Final Language](pipeline/final-language.md)
- [Extra Language Features](language-features.md)
  - [Automatic Drop](features/auto-drop.md)
  - [Cleanup On Unwinding](features/on-unwind.md)
  - [Enum Discriminant Access](features/enum-discriminant.md)
  - [Enum Projections](features/enum-projections.md)
  - [Explicit Copy/Move](features/explicit-copy-move.md)
  - [Explicit End Of Scope](features/scope-end.md)
  - [Explicit Hygiene Markers](features/hygiene-markers.md)
  - [Extended Let Chains](features/extended-let-chains.md)
  - [If Let Guards](features/if-let-guards.md)
  - [In-Place Drop](features/in-place-drop.md)
  - [Move Expressions for Closure Captures](features/move-expressions.md)
  - [Moving Out Of `&mut`](features/moving-out-of-mut.md)
  - [Phased Initialization](features/phased-initialization.md)
  - [Place Aliases](features/let-place.md)
  - [Unchecked Indexing](features/unchecked-indexing.md)
  - [Unique-Immutable Borrow](features/uniq-borrow.md)
- [Worked Examples](worked-examples.md)
  - [Example 1: A pattern match](worked-examples/pattern-match.md)
  - [Example 2: Closure capture and bounds checks](worked-examples/closure-capture-and-bounds.md)
