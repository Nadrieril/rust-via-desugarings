# `let place`

`let place p = $expr;` evaluates `$expr` to a place expression and then works as an alias to that
place. `place p` is allowed anywhere a binding is, e.g. in patterns `let Some(place p) = ...`,
except inside or-patterns.

It expresses the idea of "compute a place once and use it many times". In practice, if we apply our
initial desugaring steps to `let place p = $expr;`, we end up with `$expr` being a side-effect-free
place expression, which we can then syntactically substitute wherever `p` is used.

For example:
```rust
let place p = x.field; // this does not try to move out of the place
something(&p);
something_else(p); // now this moves out

// would be desugared to:
something(&x.field);
something_else(x.field); // now this moves out
```

```rust
let place p = x.method().field;
something(&p);

// would be desugared to:
let tmp = x.method();
something(&tmp.field);
```

The one point where this feature is a bit tricky is autoderef:
```rust
let mut x: std::cell::RefMut<Struct> = ...;
let place p = x.field; // should this use `deref` or `deref_mut`?
something(&p);
something_else(&mut p); // causes `deref_mut` to be called above

// becomes:
let mut x: std::cell::RefMut<Struct> = ...;
let tmp = <_ as DerefMut>::deref_mut(&mut x)
let place p = (*tmp).field;
something(&p);
something_else(&mut p);

// and then:
let mut x: std::cell::RefMut<Struct> = ...;
let mut x: std::cell::RefMut<Struct> = ...;
let tmp = <_ as DerefMut>::deref_mut(&mut x)
something(&(*tmp).field);
something_else(&mut (*tmp).field);
```

For that to work, we first infer for each place alias whether it is used by-ref, by-ref-mut or by-move
(like closure captures I think).
We then use that information when desugaring autoderefs to know which `Deref` variant to call.
