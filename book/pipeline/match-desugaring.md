# Match Desugaring

We can now simply transform:
```rust
match $place {
    $pat1 if $guard1 => $arm1,
    $pat2 if $guard2 => $arm2,
    $pat3 => $arm3,
}
```
into:
```rust
if let $pat1 = $place && $guard1 {
    $arm1
} else if let $pat2 = $place && $guard2 {
    $arm2
} else if let $pat3 = $place {
    $arm3
} else {
    unsafe { core::hint::unreachable_unchecked() }
}
```

This is valid because 1. the scrutinee of the match has been turned into a side-effect-less place
expression, and 2. we've dealt with any trickiness around guards, either related to or-patterns
or to bindings.

If there are no arms, we emit:
```rust
let _ = $place;
unsafe { core::hint::unreachable_unchecked() }
```

At the end of this step the only remaining branching and pattern construct is `if let else`.
