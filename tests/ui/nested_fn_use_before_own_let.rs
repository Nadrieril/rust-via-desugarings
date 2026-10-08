//@ known-failure
// A use before a nested function's own let refers to the enclosing function's local.
// rustc: error[E0434]: can't capture dynamic environment in a fn item
fn main() {
    let x: bool = true;
    fn f1() {
        print(x);
        let x: bool = false;
    }
}
