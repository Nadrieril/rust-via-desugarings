//@ known-failure
// An enclosing function's parameter hides a crate-level function from a nested function.
// rustc: error[E0434]: can't capture dynamic environment in a fn item
fn f1() {}

fn f2(f1: bool) {
    fn f3() {
        f1();
    }
}

fn main() {}
