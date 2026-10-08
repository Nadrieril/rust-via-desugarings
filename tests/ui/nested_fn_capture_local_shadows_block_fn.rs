//@ known-failure
// An enclosing function's local hides a block-level function from a nested function.
// rustc: error[E0434]: can't capture dynamic environment in a fn item
fn main() {
    fn f1() {}
    let f1: bool = true;
    fn f2() {
        f1();
    }
}
