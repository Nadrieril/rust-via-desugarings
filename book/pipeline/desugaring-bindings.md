# Desugaring Bindings

All the `let` expressions left are now bindings.
We desugar them all into binding declarations and assignments:

- By-value bindings:

    ```rust
    let x = $place;

    // becomes
    let x: $ty;
    x = &$place;
    ```

- By-ref bindings:

    ```rust
    let ref x = $place;

    // becomes
    let x: $ty;
    x = &$place;
    ```

- By-ref-mut bindings:

    ```rust
    let ref mut x = $place;

    // becomes
    let x: $ty;
    x = &mut $place;
    ```

- Place aliases:

    For place aliases, the RHS is already a side-effect-free place expression.
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

At the end of this step, all the bindings are declared uninitialized (`let x;`/`let mut x;`).
