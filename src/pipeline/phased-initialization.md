# Phased Initialization

At this stage, some compound value expressions remain,
namely struct, enum and union constructor.
In this step we desugar those into individual assignments,
using [Phased Initialization](../features/phased-initialization.md).

```rust
x = Struct { a: $expr_a, b: $expr_b };

// becomes:
x.a = $expr_a;
x.b = $expr_b;
```

```rust
x = Enum::Variant { a: $expr_a, b: $expr_b };

// becomes:
x.Variant.a = $expr_a;
x.Variant.b = $expr_b;
x.enum#discriminant = discriminant_of!(Enum, Variant));
```

Note that we don't desugar tuple struct/enum constructors since these are semantically function
calls.

The one aggregate we keep is array repeat expressions `[$expr; $const]`, which would be wasteful to
turn into a loop.
