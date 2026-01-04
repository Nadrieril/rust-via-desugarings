# Explicit Unwind Cleanup

In this step we make explicit the cleanup that happens on unwinding, using the [Cleanup On
Unwinding](../features/on-unwind.md) feature.

We surround every function call and use of `ensure_dropped` with a `on_unwind` block.
In the cleanup part of this block, we add `ensure_dropped!($local); scope_end!($local);` statements
for each in-scope local, in reverse order of declaration.
For the block around `ensure_dropped!($local)`, we skip adding a duplicate `ensure_dropped!($local)`
in its cleanup path.

```rust
let n = 42;
let x = String::new();

// becomes, before this stage:
let n;
n = 42;
let x;
x = String::new();
ensure_dropped!(x);
scope_end!(x);
ensure_dropped!(n);
scope_end!(n);

// becomes, after this stage:
let n;
n = 42;
let x;
x = on_unwind String::new() {
    ensure_dropped!(x);
    scope_end!(x);
    ensure_dropped!(n);
    scope_end!(n);
};
on_unwind ensure_dropped!(x) {
    scope_end!(x);
    ensure_dropped!(n);
    scope_end!(n);
};
scope_end!(x);
on_unwind ensure_dropped!(n) {
    scope_end!(n);
};
scope_end!(n);
```

After this step, unwinding no longer causes any code to run implicitly; it has all been made
explicit.
