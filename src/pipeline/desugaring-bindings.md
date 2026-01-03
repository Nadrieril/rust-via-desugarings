# Desugaring Bindings

All the `let` expressions left are now bindings.
We desugar them all into by-value bindings:

- By-ref bindings:

    ```rust
    let ref x = $place;
    // becomes
    let x = &$place;
    ```

- By-ref-mut bindings:

    ```rust
    let ref mut x = $place;
    // becomes
    let x = &mut $place;
    ```

- Place aliases:

    For place aliases, we now the RHS is already a side-effect-free place expression.
    We can therefore simply substitute `$place` for `p` syntactically.
    For example:
    ```rust
    let place p = x.field;
    something(&p);
    something_else(p);

    // becomes:
    something(&x.field);
    something_else(x.field);
    ```

At the end of this step, the only bindings left are by-value.
