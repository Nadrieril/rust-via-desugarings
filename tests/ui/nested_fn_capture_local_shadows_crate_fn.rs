//@ known-failure
// An enclosing function's local hides a crate-level function from a nested function.
// rustc: error[E0434]: can't capture dynamic environment in a fn item
fn f1() {}

fn main() {
    let f1: bool = true;
    fn f2() {
        f1();
    }
}
