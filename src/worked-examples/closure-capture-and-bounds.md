# Example 2: Closure capture and bounds checks

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
