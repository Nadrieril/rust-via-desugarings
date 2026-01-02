# Phased Initialization

I propose that the following should be allowed[^1] :
```rust
let x: (u32, u32);
x.0 = 42;
x.1 = 43;
// use `x` as normal here
```

Until the value is fully initialized, panic/going out of scope would simply drop the
already-initialized fields. The moment the last field is initialized, the value is treated as a full
value, and dropping it would use its `Drop` impl, if any.

Note that by that token, `let x: ();` is sufficient to initialize a ZST. This may or may not be
desirable.

For enums, I propose to use the semantics of [RFC 3727](https://github.com/rust-lang/rfcs/pull/3727)
along with the syntax of [RFC 3607](https://github.com/rust-lang/rfcs/pull/3607) that we saw in
[Enum Discriminant Access](enum-discriminant.md):

```rust
let x: Option<u32>;
unsafe {
    x.Some.0 = 42;
    x.enum#discriminant = discriminant_of!(Option<u32>, Some));
}
```

Per the RFC, the discriminant is only allowed to be set once the rest of the fields are initialized.
The value is therefore considered fully initialized the moment we write to its discriminant.

[^1]: I swear I recall seeing an RFC proposing exactly that but I can't find it.
