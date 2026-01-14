# Method Resolution & Operator Overload

Method calls are the expressions that look like `$receiver.method($args..)`. Method calls in Rust
involve a fair bit of implicit magic: the receiver expression may be referenced and/or dereferenced
to get to the right type, and figuring out which methods are available requires trait solving.

Explaining how that works is out of scope for this guide (see the [Reference
section](https://doc.rust-lang.org/reference/expressions/method-call-expr.html#method-call-expressions));
whatever the exact process, the result is that we replace each method call with a full-unambiguous
function call expression and some expression adjustments:

```rust
let opt = Some(42);
let x: &i32 = opt.as_ref().clone().unwrap();
// desugars to:
let x: &i32 = Option::unwrap(<Option<&i32> as Clone>::clone(&Option::as_ref(&opt)));
```

The `<Type as Trait>::method(self, args..)` syntax is called [UFCS (Uniform Function Call
Syntax)](https://doc.rust-lang.org/reference/expressions/call-expr.html#disambiguating-function-calls)
and allows specifying exactly what trait method is getting called. Note also how `opt` got borrowed
into `&opt` in order to match the type required for `Option::as_ref`.

Aside postfix method calls, a number of operations can be overridden
using traits. We desugar such overridden operations into the appropriate method call:
- `a + b -> Add::add(a, b)`
- `a += b -> AddAssign::add_assign(&mut a, b)`
- `a - b -> Sub::sub(a, b)`
- `-a -> Neg::neg(a)`
- `a[b] -> Index::index(&a, b)/IndexMut::index_mut(&mut a, b)`
- `f(args...) -> Fn::call/FnMut::call_mut/FnOnce::call_once(f, (args...))`
- etc

The non-overriden versions of these operations stay unchanged.
For example `+` on integers is built-in, but on integer references is defined by a trait:
```rust
let x = 1 + 2 + &3;
// becomes
let x = 1 + <i32 as Add<&i32>>::add(2, &3);
```

Note that we don't handle the `*` operator (overrideable with `Deref`/`DerefMut`) here, we'll do it
in [`Deref`/`DerefMut` Desugarings](smart-ptr-deref.md).

At the end of this step every method call and overridden operator use has been turned into a plain
function call.
