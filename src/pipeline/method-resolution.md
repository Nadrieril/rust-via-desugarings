# Method Resolution & Operator Overload

Method calls are the expressions that look like `$receiver.method($args..)`. Method calls in Rust
involve a fair bit of implicit magic: the receiver expression may be referenced and/or dereferenced
to get to the right type, and figuring out which methods are available will likely involve trait
solving.

Explaining how that works is out of scope for this guide (see the [Reference
section](https://doc.rust-lang.org/reference/expressions/method-call-expr.html#method-call-expressions));
whatever the exact process, the result is that we replace each method call with a full-unambiguous
function call expression:

```rust
let opt = Some(42);
let x: &i32 = opt.as_ref().clone().unwrap();
// desugars to:
let x: &i32 = Option::unwrap(<Option<&i32> as Clone>::clone(&Option::as_ref(&opt)));
```

The `<Type as Trait>::method(self, args..)` syntax is called UFCS (Uniform Function Call Syntax) and
allows specifying exactly what trait method is getting called. Note also how `opt` got borrowed into
`&opt` in order to match the `self` type required for `Option::as_ref`. This may even insert calls
to `deref`/`deref_mut` (like in the previous section).

On top of postfix method calls, there are also a number of operations that stand for trait method
calls:
- `a + b -> Add::add(a, b)`
- `a += b -> AddAssign::add_assign(&mut a, b)`
- `a - b -> Sub::sub(a, b)`
- `-a -> Neg::neg(a)`
- `a[b] -> Index::index(&a, b)/IndexMut::index_mut(&mut a, b)`
- `f(args...) -> Fn::call/FnMut::call_mut/FnOnce::call_once(f, (args...))`

and a few others. All the non-builtin uses of these operations get desugared to the appropriate
trait method call. "Built-in" here means the most basic version of that operation, e.g. addition on
integers, indexing on arrays/slices, function call on function pointers. Those don't get desugared,
the rest do.

```rust
let x = 1 + 2 + &3;
// becomes
let x = 1 + <i32 as Add<&i32>>::add(2, &3);
```

At the end of this step every method call and overridden operator use has been turned into a plain
function call.
