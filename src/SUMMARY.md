# Summary

- [Introduction](introduction.md)
- [Desugaring Steps](pipeline/overview.md)
  - [Name Resolution & Macro Expansion](pipeline/name-resolution-macro-expansion.md)
  - [Loop Desugaring](pipeline/loop-desugaring.md)
  <!-- `?` desugaring -->
  <!-- - [Autoderef and deref coercion?], needed for computing the type of places -->
  <!-- Coercions: coercion sites, reborrow, unsizing, autoderef -->
  - [Method Resolution & Operator Overload](pipeline/method-resolution.md)
  - [Temporaries and Intermediate Subexpressions](pipeline/temporaries.md)
  <!-- Desugaring patterns into matches (if let, let else, etc) -->
  <!-- Match ergonomics -->
  <!-- Match lowering -->
  <!-- Closure capture desugaring with move expressions? Or capture lists  -->
  <!-- Closure desugaring into ADTs  -->
  <!-- Async bloc capture? -->
  <!-- Explicit copy vs move -->
  - [Drop Elaboration](pipeline/drop-elaboration.md)
  - [Borrow Checking?](pipeline/borrow-checking.md)
  <!-- Coroutine transformation -->
  - [MiniRust](pipeline/minirust.md)
