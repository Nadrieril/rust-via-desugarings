# Match Ergonomics

In their fundamental operation, patterns must have the same type as the place
they're matching on, e.g. `Some(_)` applies to places of type `Option<..>`.
"Match ergonomics" is the feature that allows some mismatches here, specifically
this allows patterns to transparently match through references.

The exact details are given in [RFC 2005 "Match
ergonomics"](https://rust-lang.github.io/rfcs/2005-match-ergonomics.html) and
the [edition guide](https://doc.rust-lang.org/edition-guide/rust-2024/match-ergonomics.html).
In terms of desugaring, this step transforms patterns that involve match ergonomics
into patterns that don't, i.e. that have exact types.

```rust
let opt: &&Option<u32> = ..;
if let Some(x) = opt {
    ..
}

// becomes:
if let &&Some(ref x) = opt {
    ..
}
```

After this step, patterns have exact types and explicit binding modes (i.e. `x` vs `ref x` vs
`ref mut x`).
