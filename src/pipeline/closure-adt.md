# Closure Desugaring

Once captures are explicit, desugaring closures into ADTs becomes straightforward.

A closure becomes a struct, with one field per `move($expr)` expression, and that field is
initialized with `$expr`. That struct then implements the appropriate `Fn*` traits. In the new
function body, what was a `move(..)` expression is replaced with the appropriate field expression.

Let's take our previous examples again:
```rust
let mut increment = || *move(&mut x) += 1;

// desugars to
struct Closure<'a> {
    capture1: &'a mut u32,
}
impl FnOnce<()> for Closure<'_> {
    type Output = ();
    fn call_once(mut self, args: ())  {
        self.call_mut(args)
    }
}
impl FnMut<()> for Closure<'_> {
    fn call_mut(&mut self, _args: ()) {
        *self.capture1 += 1
    }
}
let mut increment = Closure { capture1: &mut x };
```
and
```rust
let mut replace = |new: u32| Option::replace(&mut move(x), new);

// desugars to
struct Closure {
    capture1: u32,
}
impl FnOnce<(u32,)> for Closure {
    type Output = Option<u32>;
    fn call_once(mut self, args: (u32,)) -> Option<u32>  {
        self.call_mut(args)
    }
}
impl FnMut<(u32,)> for Closure {
    fn call_mut(&mut self, (new,): (u32,)) -> Option<u32> {
        Option::replace(&mut self.capture1, new)
    }
}
let mut replace = Closure { capture1: x };
```

To clean up the newly generated closure expressions, we run the [Intermediate Subexpression
Elimination](subexpr-elim.md) and [Explicit
Copies/Moves](copy-move.md) steps again.

After this step, there are no closure expressions left.
