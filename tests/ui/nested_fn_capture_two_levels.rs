//@ known-failure
// A local two function levels out hides a crate-level function.
// rustc: error[E0434]: can't capture dynamic environment in a fn item
fn f1() {}

fn main() {
    let f1: bool = true;
    fn f2() {
        fn f3() {
            f1();
        }
    }
}
