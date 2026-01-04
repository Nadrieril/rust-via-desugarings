# Explicit Binding Scopes

This step makes the end of variable scopes explicit using the [Explicit End Of
Scope](../features/scope-end.md) feature.

At the end of each scope, for each variable `x` declared in that scope in reverse order of
declaration, we add a `scope_end!(x)` statement.
