# Explicit Binding Scopes

This step makes the end of variable scopes explicit using the [Explicit End Of
Scope](../features/scope-end.md) feature.

At the end of each scope, for each variable `x` declared in that scope in reverse order of
declaration, we add a `scope_end!(x)` statement.

Before every `break;`/`continue;` statement, we similarly `scope_end` all the in-scope variables  that will no
longer be in scope at the target of the `break;`/`continue;`.

Finally before a `return $local;` statement we end the scopes of all locals except `$local`.
