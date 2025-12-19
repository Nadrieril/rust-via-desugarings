# Loop Desugaring

`for` and `while` loops are desugared into a conditionless `loop`:
```rust
for $pat in $iter {
    $loop_body
}
// becomes
{
    let mut iter = IntoIterator::into_iter($iter);
    while let Some($pat) = iter.next() {
        $loop_body
    }
}
```

And then:
```rust
while $condition {
    $loop_body
}
// becomes
loop {
    if $condition {
        $loop_body
    } else {
        break;
    }
}
```

TODO: is this correct wrt temporaries?
