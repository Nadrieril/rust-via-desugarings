# Or-patterns

Or-patterns is the name we give to patterns that look like `$pat | $pat`. In this step we desugar
them into explicit control-flow.

The first step is to move any nested `|` to the outside of a pattern, e.g. `(0 | 1, 2 | 3)` becomes
`(0, 2) | (0, 3) | (1, 2) | (1, 3)`[^1].

Or-patterns inherently give rise to non-tree-like control-flow. To handle them in their full
generality, we will resort to not-very-pretty control-flow constructs:
```rust
match $expr {
    $pat1 | $pat2 if $guard => $arm,
    $remaining_arms
}

// becomes:
'match_end: {
    'arm: {
        match $expr {
            $pat1 if $guard => break 'arm,
            $pat2 if $guard => break 'arm,
            $remaining_arms
        }
        break 'match_end
    }
    $arm
}
```

Note an interesting property that this desugaring makes clear: a single match guard may run several
times. This can be observed, e.g.:
```rust
let mut guard_count = 0;
match (false, false) {
    (a, _) | (_, a) if { guard_count += 1; a } => {}
    _ => {}
}
assert_eq!(guard_count, 2);

// desugars to (omitting the breaks because they do nothing):
let mut guard_count = 0;
match (false, false) {
    (a, _) if { guard_count += 1; a } => {}
    (_, a) if { guard_count += 1; a } => {}
    _ => {}
}
assert_eq!(guard_count, 2);
```

After this step, patterns don't involve `|`.

[^1]: The drawback of this desugaring is that the resulting pattern may be exponentially bigger than the original one. If a pattern doesn't have bindings nor a match guard, then we don't really need to desugar it now. If it has either however, desugaring is necessary to get the right control-flow. This is in fact pretty much what we do when lowering to MIR today so I don't know if we can do any better.
