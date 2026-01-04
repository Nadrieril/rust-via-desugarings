# Deref/DerefMut Desugarings

The expression `*x` is allowed if `x` is a built-in reference type (`&T`, `&mut T`, `*const T`,
`*mut T` and `Box<T>`), as well as if `x: T` and `T: core::ops::Deref`.
In this step we desugar the `Deref` case.

Every place context has an associated mutability: `&mut $expr` and `if let Some(ref mut x) = $expr`
are two examples of mutable contexts. Uses that don't require mutability are considered immutable contexts.
Examples of immutable contexts are `&$expr` and `function_call($expr)`.

Let `T: Deref` be a type and `$expr` be an expression of that type.
Let `context!(..)` be a context in which one may find the expression `*$expr`.
We desugar as follows:

```rust
context!(*$expr)
// becomes, if the context is considered mutable:
context!(*<T as DerefMut>::deref_mut(&mut $expr))
// otherwise, if the context is considered immutable:
context!(*<T as Deref>::deref(&$expr))
```
