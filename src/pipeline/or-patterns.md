# Or-patterns

"Or-patterns" are the patterns that look like `$pat | $pat`.
They are tricky when they have bindings and when they're under match guards.
In this step we desugar them into explicit control-flow.

The first step is to move any nested `|` to the outside of a pattern, e.g. `(0 | 1, 2 | 3)` becomes
`(0, 2) | (0, 3) | (1, 2) | (1, 3)` (see Discussion below about the combinatorial explosion). This
expansion is done left-to-right.

Inside let chains, we simply turn `let $pat1 | $pat2 = $expr` into `let $pat1 = $expr || let $pat2
= $expr` using [Extended Let Chains](../features/extended-let-chains.md).

Inside matches, we encode the non-tree-like control-flow directly:
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

### Drop order

The let-chain desugaring is actually incorrect wrt drop order: or-patterns declare their bindings in
the order given by the first subpattern
([Reference](https://doc.rust-lang.org/reference/destructors.html#r-destructors.scope.bindings.or-patterns)),
but our desugaring will drop them in the order of the alternative that succeeds.

### Combinatorial explosion

This desugaring has the benefit of simplicity but two big drawbacks: it duplicates user code (the
match guards), and more importantly causes combinatorial explosion.
For example, `(true|false, true|false, true|false, true|false) if $guard` desugars to 16 patterns
and 16 copies of the guard code.

A more robust approach could be to give an index to each sub-pattern
and branch/loop on these indices to know the right bindings to use/number of times to run a guard.
For example, using [guard patterns](https://rust-lang.github.io/rfcs//3637-guard-patterns.html):

```rust
match $place {
    ($a | $b, Some($c | $d)) if $guard => $arm,
    _ => {}
}

// could become something like:
'success: for i in 0..=1 {
    for j in 0..=1 {
        if let (
            place p if (i == 0 && let $a = p || i == 1 && let $b = p),
            Some(place q if (j == 0 && let $c = q || j == 1 && let $d = q))
        ) && guard {
            $arm
            break 'success
        }
    }
}
```
