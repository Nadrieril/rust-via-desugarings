# Worked Examples

## Example 1: A pattern match

Source:
```rust
fn is_north(cmd: &(Option<&str>, i32)) -> bool {
    match cmd {
        (Some("north" | "n"), dist) if *dist > 0 => true,
        (Some(_), _) | (None, _) => false,
    }
}
```

After match ergonomics:
```rust
fn is_north(cmd: &(Option<&str>, i32)) -> bool {
    match cmd {
        &(Some("north" | "n"), ref dist) if *dist > 0 => true,
        &(Some(_), _) | (None, _) => false,
    }
}
```

After or-pattern desugaring and match desugaring:
```rust
fn is_north(cmd: &(Option<&str>, i32)) -> bool {
    'match_end: {
        let dist_;
        'arm: {
            break 'match_end (if let &(Some("north"), ref dist) = cmd && *dist > 0 {
                dist_ = dist;
                break 'arm;
            } else if let &(Some("n"), ref dist) = cmd && *dist > 0 {
                dist_ = dist;
                break 'arm;
            } else if let &(Some(_), _) = cmd {
                false
            } else if let &(None, _) = cmd {
                false
            } else {
                unsafe { core::hint::unreachable_unchecked() }
            });
        }
        true
    }
}
```

After pattern unnesting:
```rust
fn is_north(cmd: &(Option<&str>, i32)) -> bool {
    'match_end: {
        let dist_;
        'arm: {
            break 'match_end (
                if (*cmd).0.enum#discriminant == discriminant_of!(Option, Some)
                    && (*cmd).0.Some.0 == "north"
                    && let ref dist = (*cmd).1
                    && *dist > 0
                {
                    dist_ = dist;
                    break 'arm;
                } else if (*cmd).0.enum#discriminant == discriminant_of!(Option, Some)
                    && (*cmd).0.Some.0 == "n"
                    && let ref dist = (*cmd).1
                    && *dist > 0
                {
                    dist_ = dist;
                    break 'arm;
                } else if (*cmd).0.enum#discriminant == discriminant_of!(Option, Some) {
                    false
                } else if (*cmd).0.enum#discriminant == discriminant_of!(Option, None) {
                    false
                } else {
                    unsafe { core::hint::unreachable_unchecked() }
                }
            );
        }
        true
    }
}
```

After if-let-chain desugaring (and simplifying blocks a bit):
```rust
fn is_north(cmd: &(Option<&str>, i32)) -> bool {
    'match_end: {
        let dist_;
        'arm: {
            if (*cmd).0.enum#discriminant == discriminant_of!(Option, Some) {
                if (*cmd).0.Some.0 == "north" {
                    if let ref dist = (*cmd).1 {
                        if *dist > 0 {
                            dist_ = dist;
                            break 'arm;
                        }
                    }
                }
            }
            if (*cmd).0.enum#discriminant == discriminant_of!(Option, Some) {
                if (*cmd).0.Some.0 == "n" {
                    if let ref dist = (*cmd).1 {
                        if *dist > 0 {
                            dist_ = dist;
                            break 'arm;
                        }
                    }
                }
            }
            break 'match_end {
                if (*cmd).0.enum#discriminant == discriminant_of!(Option, Some) {
                    false
                } else if (*cmd).0.enum#discriminant == discriminant_of!(Option, None) {
                    false
                } else {
                    unsafe { core::hint::unreachable_unchecked() }
                }
            };
        }
        true
    }
}
```

## Example 2: closure capture and bounds checks

Source:
```rust
fn bump_first(xs: &mut [i32]) -> i32 {
    let mut total = 0;
    let mut bump = |delta: i32| {
        total += delta;
        xs[0] += total;
        total
    };
    bump(1);
    bump(2)
}
```

After expression unnesting:
```rust
fn bump_first(xs: &mut [i32]) -> i32 {
    let mut total = 0;
    let mut bump = |delta: i32| {
        total = copy!(total) + copy!(delta);
        let index = 0;
        let len = core::slice::length(&raw const *xs);
        assert!(copy!(index) < copy!(len), "index out of bounds");
        unchecked_index!(*xs, copy!(index)) += copy!(total);
        total
    };
    bump(1);
    bump(2)
}
```

After closure capture desugarings:
```rust
fn bump_first(xs: &mut [i32]) -> i32 {
    let mut total = 0;
    let mut bump = |delta: i32| {
        let place total = *move(&mut total);
        let place xs = *move(&uniq xs);
        total = copy!(total) + copy!(delta);
        let index = 0;
        let len = core::slice::length(&raw const *xs);
        assert!(copy!(index) < copy!(len), "index out of bounds");
        unchecked_index!(*xs, copy!(index)) += copy!(total);
        total
    };
    bump(1);
    bump(2)
}
```
