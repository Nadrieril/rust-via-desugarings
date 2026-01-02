# Or-patterns

"Or-patterns" are the patterns that look like `$pat | $pat`.
They are tricky when they have bindings and when they're under match guards.
In this step we desugar them into explicit control-flow.

The first step is to move any nested `|` to the outside of a pattern, e.g. `(0 | 1, 2 | 3)` becomes
`(0, 2) | (0, 3) | (1, 2) | (1, 3)` (see Discussion below about the combinatorial explosion). This
expansion is done left-to-right.

Or-patterns inherently give rise to non-tree-like control-flow. Just like for let-chains, we express
that using named block breaks:
```rust
match $place {
    $pat1 | $pat2 if $guard => $arm,
    $remaining_arms
}

// becomes:
'match_end: {
    let x1_; // declare the bindings bound in the patterns, renamed to avoid shadowing.
    ..
    let xn_;
    'arm: {
        match $place {
            $pat1 if $guard => {
                x1_ = x1;
                ..
                xn_ = xn;
                break 'arm;
            },
            $pat2 if $guard => {
                x1_ = x1;
                ..
                xn_ = xn;
                break 'arm
            },
            $remaining_arms
        }
        break 'match_end
    }
    $arm_ // modified to use `x1_` instead of `x1` etc
}
```

Note an interesting property that this desugaring makes clear: a single match guard may run several
times. This can be observed, e.g. (see also [this
test](https://github.com/rust-lang/rust/blob/267cae5bdbd602dd13f3851b9c96ce93697e59a0/tests/ui/or-patterns/search-via-bindings.rs)):
```rust
let mut guard_count = 0;
match (false, false) {
    (a, _) | (_, a) if { guard_count += 1; a } => {}
    _ => {}
}
assert_eq!(guard_count, 2);

// is equivalent to:
let mut guard_count = 0;
match (false, false) {
    (a, _) if { guard_count += 1; a } => {}
    (_, a) if { guard_count += 1; a } => {}
    _ => {}
}
assert_eq!(guard_count, 2);
```

After this step, patterns don't involve `|`.

---

## Discussion

This desugaring has the benefit of simplicity but two big drawbacks: it duplicates user code (the
match guards), which is unfortunate, and more importantly causes combinatorial explosion.
For example, `(true|false, true|false, true|false, true|false)` desugars to 16 patterns.

Today's rustc limits this explosion by only expanding or-patterns when there's a guard or they have
bindings.
This can still very much cause combinatorial explosion.

A more robust approach could be to to give an index to each or-alternative
(e.g. using [guard patterns](https://rust-lang.github.io/rfcs//3637-guard-patterns.html) +
[`if let` guards](https://rust-lang.github.io/rfcs/2294-if-let-guard.html)),
and branch on these indices to know the right bindings to use/number of times to run a guard.

This might look like `(true if let i=0 | false if let i=1, true if let j=0 | false if let j=1, ..)`.
This would unfortunately mean we couldn't desugar patterns to simple `matches!` anymore;
I have not found a satisfactory solution to this conundrum.
