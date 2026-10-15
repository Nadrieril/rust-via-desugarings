//@ known-failure
// An enclosing function's local hides a nested function from its own body.
// rustc: error[E0434]: can't capture dynamic environment in a fn item
fn main() {
    let f1: bool = true;
    fn f1() {
        f1();
    }
}
