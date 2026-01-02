# Enum Projections

To desugar enum patterns we need a way to talk about the places inside of an enum variant.

I propose that if `$place` is of an enum type, `$place.$variant_name` is a place expression that
refers to the contents of the `$variant_name` variant.
That place then has one field for each field of the variant.

For example:
```rust
let opt: &mut Option<u32> = ...;
if let Some(ref mut x) = *opt {
    // x: &mut u32 here
}

// desugars to:
if let Some(_) = *opt {
    let x = unsafe { &mut (*opt).Some.0 };
}
```

This operation is UB if the enum value didn't have the correct discriminant, and thus this operation
requires an `unsafe` block.

The place `$place.$variant_name` can't be used by itself because we don't have a type for it; it
must be directly followed by a field projection.
